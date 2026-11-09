//! Tests and opt-in measurements of the production 128-thread statistics path.
use super::*;
use crate::model::{Image, ImageData, PixelType};
use std::{hint::black_box, time::Instant};

fn context() -> Arc<GpuComputeContext> {
    super::tests::context()
}

fn generated(width: i32, height: i32, channels: i32, value: impl Fn(i32, i32) -> f32) -> ImageData {
    let pixels = (0..width * height)
        .flat_map(|i| (0..channels).map(move |c| (i, c)))
        .map(|(i, c)| value(i, c))
        .collect();
    ImageData::from_f32(ImageSpec::new(width, height, channels, PixelType::F32), pixels).unwrap()
}

fn mixed_value(i: i32, c: i32) -> f32 {
    let mut bits = (i as u32).wrapping_mul(747796405).wrapping_add(2891336453 + c as u32 * 12345);
    bits ^= bits >> 16;
    bits = bits.wrapping_mul(2246822519);
    bits ^= bits >> 13;
    (bits >> 8) as f32 / 16777215.0
}

fn compensated_sum(values: impl Iterator<Item = f64>) -> f64 {
    let (mut sum, mut correction) = (0.0, 0.0);
    for value in values {
        let adjusted = value - correction;
        let next = sum + adjusted;
        correction = (next - sum) - adjusted;
        sum = next;
    }
    sum
}

fn check_ordinary_accuracy(image: &ImageData, rect: Recti) {
    let spec = image.spec();
    let roi = normalized_rect(&spec, rect);
    let actual = context().statistics(&image.gpu_texture().unwrap(), rect).unwrap();
    for c in 0..spec.channels as usize {
        let values: Vec<f64> = (roi.min.y..roi.max.y)
            .flat_map(|y| {
                (roi.min.x..roi.max.x).filter_map(move |x| {
                    let (pixel, _) = image.normalized_pixel_at((y * spec.width + x) as usize).unwrap();
                    pixel[c].is_finite().then_some(f64::from(pixel[c]))
                })
            })
            .collect();
        assert_eq!(actual.finite_count[c], values.len() as u64);
        if values.is_empty() {
            assert!(actual.min[c].is_nan() && actual.max[c].is_nan());
            assert!(actual.variance[c].is_nan() && actual.std_dev[c].is_nan());
            continue;
        }
        assert_eq!(
            actual.min[c].to_bits(),
            values.iter().copied().min_by(f64::total_cmp).unwrap().to_bits()
        );
        assert_eq!(
            actual.max[c].to_bits(),
            values.iter().copied().max_by(f64::total_cmp).unwrap().to_bits()
        );
        // Reference arithmetic is centered f64 with compensated sums. The GPU
        // deliberately uses ordinary f32 moments; bounded data uses a relative tolerance.
        let anchor = values[0];
        let mean = compensated_sum(values.iter().map(|v| v - anchor)) / values.len() as f64;
        let variance = compensated_sum(values.iter().map(|v| ((v - anchor) - mean).powi(2))) / values.len() as f64;
        assert!(
            (actual.variance[c] - variance).abs() <= 2e-5 * variance.max(1e-12),
            "c{c}: {actual:?} vs {variance}"
        );
        assert!((actual.std_dev[c] - variance.sqrt()).abs() <= 2e-5 * variance.sqrt().max(1e-12));
    }
}

#[test]
fn statistics_match_f64_on_ordinary_images_and_rois() {
    for channels in 1..=4 {
        let image = generated(1025, 257, channels, |i, c| mixed_value(i, c) * 8.0 - 4.0);
        for rect in [
            Recti::ZERO,
            Recti::from_min_size(Vec2i::new(13, 7), Vec2i::new(503, 111)),
            Recti::from_min_max(Vec2i::new(516, 118), Vec2i::new(13, 7)),
            Recti::from_min_size(Vec2i::new(-5, -7), Vec2i::new(20, 22)),
            Recti::from_min_size(Vec2i::new(2000, 2000), Vec2i::new(10, 10)),
            Recti::from_min_size(Vec2i::new(3, 2), Vec2i::new(1, 1)),
        ] {
            check_ordinary_accuracy(&image, rect);
        }
    }
    let bytes: Vec<u8> = (0..257 * 131 * 4).map(|i| (i * 37) as u8).collect();
    check_ordinary_accuracy(
        &ImageData::from_raw_bytes(&bytes, 257, 131, 4, PixelType::U8).unwrap(),
        Recti::ZERO,
    );
    let values: Vec<u16> = (0..257 * 131 * 4).map(|i| (i * 37) as u16).collect();
    check_ordinary_accuracy(
        &ImageData::from_raw_bytes(bytemuck::cast_slice(&values), 257, 131, 4, PixelType::U16).unwrap(),
        Recti::ZERO,
    );
}

#[test]
fn statistics_exclude_nonfinite_samples_per_channel() {
    let image = generated(257, 131, 4, |i, c| match c {
        0 => f32::NAN,
        1 => {
            if i % 3 == 0 {
                f32::INFINITY
            } else {
                (i % 11) as f32
            }
        }
        2 => {
            if i == 17 {
                8.0
            } else {
                f32::NEG_INFINITY
            }
        }
        _ => (i % 13) as f32 - 6.0,
    });
    check_ordinary_accuracy(&image, Recti::ZERO);
    let result = context().statistics(&image.gpu_texture().unwrap(), Recti::ZERO).unwrap();
    assert_eq!(result.finite_count[0], 0);
    assert_eq!(result.finite_count[2], 1);
    assert_eq!(result.variance[2], 0.0);
    assert_eq!(result.std_dev[2], 0.0);
}

#[test]
fn statistics_preserve_minmax_when_moments_overflow() {
    let image = ImageData::from_f32(ImageSpec::new(3, 1, 1, PixelType::F32), vec![-f32::MAX, 0.0, f32::MAX]).unwrap();
    let result = context().statistics(&image.gpu_texture().unwrap(), Recti::ZERO).unwrap();
    assert_eq!(result.min, vec![-f64::from(f32::MAX)]);
    assert_eq!(result.max, vec![f64::from(f32::MAX)]);
    assert_eq!(result.finite_count, vec![3]);
    // Overflow must be unavailable, never silently clamped to zero variance.
    assert!(result.variance[0].is_nan() && result.std_dev[0].is_nan());
}

#[test]
fn statistics_handle_constant_channels_and_signed_zero() {
    for value in [0.0, -0.0, 1.0, f32::MAX, -f32::MAX, f32::from_bits(1)] {
        let image = generated(1025, 257, 4, |_, _| value);
        let result = context().statistics(&image.gpu_texture().unwrap(), Recti::ZERO).unwrap();
        assert!(result.min.iter().all(|v| v.to_bits() == f64::from(value).to_bits()));
        assert!(result.max.iter().all(|v| v.to_bits() == f64::from(value).to_bits()));
        assert_eq!(result.variance, vec![0.0; 4]);
        assert_eq!(result.std_dev, vec![0.0; 4]);
    }
    let image = ImageData::from_f32(ImageSpec::new(2, 1, 1, PixelType::F32), vec![-0.0, 0.0]).unwrap();
    let result = context().statistics(&image.gpu_texture().unwrap(), Recti::ZERO).unwrap();
    assert_eq!(result.min[0].to_bits(), (-0.0f64).to_bits());
    assert_eq!(result.max[0].to_bits(), 0.0f64.to_bits());
}

#[test]
fn statistics_fit_16kib_device_limit() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
    assert_eq!(STATISTICS_WORKGROUP_STORAGE_SIZE, 10 * 1024);
    assert!(STATISTICS_WORKGROUP_STORAGE_SIZE <= device.limits().max_compute_workgroup_storage_size);
    let compute = GpuComputeContext::new(device, queue, adapter.get_info().backend);
    let texture = compute.upload(&ImageSpec::new(2, 1, 1, PixelType::F32), &[2.0, 4.0]).unwrap();
    let result = compute.statistics(&texture, Recti::ZERO).unwrap();
    assert_eq!(result.variance, vec![1.0]);
    assert_eq!(result.std_dev, vec![1.0]);
}

#[test]
#[ignore = "manual latency measurement of the production 128-thread statistics path"]
fn benchmark_selection_statistics() {
    let compute = context();
    let image = generated(7680, 4320, 4, mixed_value);
    let texture = image.gpu_texture().unwrap();
    compute.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    for (w, h) in [(64, 64), (512, 512), (1920, 1080), (3840, 2160), (7680, 4320)] {
        let rect = Recti::from_min_size(Vec2i::ZERO, Vec2i::new(w, h));
        let mut times = Vec::new();
        for iteration in 0..116 {
            let start = Instant::now();
            black_box(compute.statistics(&texture, rect).unwrap());
            if iteration >= 15 {
                times.push(start.elapsed().as_secs_f64() * 1000.0);
            }
        }
        times.sort_by(f64::total_cmp);
        eprintln!("STATISTICS_PERF_CSV,{w}x{h},{:.6},{:.6},101", times[50], times[95]);
    }
}
