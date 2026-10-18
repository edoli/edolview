use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};

use color_eyre::eyre::{eyre, Result};

use crate::model::{gpu_compute, Image, ImageData, Recti};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MeanDim {
    All,
    Column,
    Row,
}

/// CPU summed-area table using the same normalized f32 values as `ImageData`.
/// Accumulation is f64 to preserve the precision of the former integral table.
pub(crate) struct IntegralImage {
    width: usize,
    height: usize,
    channels: usize,
    stride: usize,
    values: Vec<f64>,
}

#[inline(always)]
fn build_cpu_integral<const CHANNELS: usize>(
    pixels: &[f32],
    width: usize,
    height: usize,
    stride: usize,
    values: &mut [f64],
    active_image_id: &AtomicU64,
    image_id: u64,
) -> bool {
    let input = pixels.as_ptr();
    let output = values.as_mut_ptr();
    for y in 0..height {
        if active_image_id.load(Ordering::Acquire) != image_id {
            return false;
        }
        let mut row_sum = [0.0f64; CHANNELS];
        for x in 0..width {
            let pixel = (y * width + x) * CHANNELS;
            let dst = ((y + 1) * stride + x + 1) * CHANNELS;
            let above = (y * stride + x + 1) * CHANNELS;
            for (channel, row_sum) in row_sum.iter_mut().enumerate() {
                // ImageData validates the pixel count, and the integral allocation
                // includes the one-pixel top/left border used by these offsets.
                unsafe {
                    *row_sum += *input.add(pixel + channel) as f64;
                    *output.add(dst + channel) = *output.add(above + channel) + *row_sum;
                }
            }
        }
    }
    true
}

fn build_cpu_integral_rgba(
    pixels: &[f32],
    width: usize,
    height: usize,
    stride: usize,
    values: &mut [f64],
    active_image_id: &AtomicU64,
    image_id: u64,
) -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if std::arch::is_x86_feature_detected!("avx2") {
        // The runtime feature check satisfies the target_feature contract.
        return unsafe {
            build_cpu_integral_rgba_avx2(pixels, width, height, stride, values, active_image_id, image_id)
        };
    }
    build_cpu_integral::<4>(pixels, width, height, stride, values, active_image_id, image_id)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn build_cpu_integral_rgba_avx2(
    pixels: &[f32],
    width: usize,
    height: usize,
    stride: usize,
    values: &mut [f64],
    active_image_id: &AtomicU64,
    image_id: u64,
) -> bool {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let input = pixels.as_ptr();
    let output = values.as_mut_ptr();
    for y in 0..height {
        if active_image_id.load(Ordering::Acquire) != image_id {
            return false;
        }
        let mut row_sum = _mm256_setzero_pd();
        for x in 0..width {
            let pixel = (y * width + x) * 4;
            let dst = ((y + 1) * stride + x + 1) * 4;
            let above = (y * stride + x + 1) * 4;
            row_sum = _mm256_add_pd(row_sum, _mm256_cvtps_pd(_mm_loadu_ps(input.add(pixel))));
            let total = _mm256_add_pd(_mm256_loadu_pd(output.add(above)), row_sum);
            _mm256_storeu_pd(output.add(dst), total);
        }
    }
    true
}

#[inline(always)]
fn build_derived_integral<const CHANNELS: usize>(
    image: &ImageData,
    width: usize,
    height: usize,
    stride: usize,
    values: &mut [f64],
    active_image_id: &AtomicU64,
    image_id: u64,
) -> Result<bool> {
    for y in 0..height {
        if active_image_id.load(Ordering::Acquire) != image_id {
            return Ok(false);
        }
        let mut row_sum = [0.0f64; CHANNELS];
        for x in 0..width {
            let pixel = y * width + x;
            let (pixel_values, channels) = image
                .normalized_pixel_at(pixel)
                .ok_or_else(|| eyre!("Image pixels are unavailable for integral precompute"))?;
            debug_assert_eq!(channels, CHANNELS);
            let dst = ((y + 1) * stride + x + 1) * CHANNELS;
            let above = (y * stride + x + 1) * CHANNELS;
            for channel in 0..CHANNELS {
                row_sum[channel] += pixel_values[channel] as f64;
                values[dst + channel] = values[above + channel] + row_sum[channel];
            }
        }
    }
    Ok(true)
}

impl IntegralImage {
    pub(crate) fn build(image: &ImageData, active_image_id: &AtomicU64) -> Result<Option<Self>> {
        let spec = image.spec();
        if spec.width <= 0 || spec.height <= 0 || !(1..=4).contains(&spec.channels) {
            return Err(eyre!("Invalid image dimensions for integral table"));
        }
        let width = spec.width as usize;
        let height = spec.height as usize;
        let channels = spec.channels as usize;
        let stride = width + 1;
        let element_count = stride
            .checked_mul(height + 1)
            .and_then(|value| value.checked_mul(channels))
            .ok_or_else(|| eyre!("Integral table size overflow"))?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(element_count)
            .map_err(|error| eyre!("Failed to allocate integral table: {error}"))?;
        values.resize(element_count, 0.0);

        let image_id = image.id();
        let complete = match (image.pixels(), channels) {
            (Some(pixels), 1) => {
                build_cpu_integral::<1>(pixels, width, height, stride, &mut values, active_image_id, image_id)
            }
            (Some(pixels), 2) => {
                build_cpu_integral::<2>(pixels, width, height, stride, &mut values, active_image_id, image_id)
            }
            (Some(pixels), 3) => {
                build_cpu_integral::<3>(pixels, width, height, stride, &mut values, active_image_id, image_id)
            }
            (Some(pixels), 4) => {
                build_cpu_integral_rgba(pixels, width, height, stride, &mut values, active_image_id, image_id)
            }
            (None, 1) => {
                build_derived_integral::<1>(image, width, height, stride, &mut values, active_image_id, image_id)?
            }
            (None, 2) => {
                build_derived_integral::<2>(image, width, height, stride, &mut values, active_image_id, image_id)?
            }
            (None, 3) => {
                build_derived_integral::<3>(image, width, height, stride, &mut values, active_image_id, image_id)?
            }
            (None, 4) => {
                build_derived_integral::<4>(image, width, height, stride, &mut values, active_image_id, image_id)?
            }
            _ => unreachable!("channel count was validated above"),
        };
        if !complete {
            return Ok(None);
        }

        Ok(Some(Self {
            width,
            height,
            channels,
            stride,
            values,
        }))
    }

    pub(crate) fn mean(&self, rect: Recti, dim: MeanDim) -> Result<Vec<f64>> {
        let rect = rect.validate();
        if rect.empty() {
            return Ok(Vec::new());
        }
        let (x, y, width, height) = rect.xywh();
        if x < 0
            || y < 0
            || width <= 0
            || height <= 0
            || x + width > self.width as i32
            || y + height > self.height as i32
        {
            return Err(eyre!("Mean rectangle is outside the image"));
        }
        let x = x as usize;
        let y = y as usize;
        let width = width as usize;
        let height = height as usize;

        match dim {
            MeanDim::All => {
                let mut result = vec![0.0; self.channels];
                let divisor = (width * height) as f64;
                for (channel, value) in result.iter_mut().enumerate() {
                    *value = self.rect_sum(x, y, width, height, channel) / divisor;
                }
                Ok(result)
            }
            MeanDim::Column => {
                let mut result = vec![0.0; width * self.channels];
                let top_left = (y * self.stride + x) * self.channels;
                let bottom_left = ((y + height) * self.stride + x) * self.channels;
                let divisor = height as f64;
                for (offset, value) in result.iter_mut().enumerate() {
                    *value = (self.values[bottom_left + self.channels + offset]
                        - self.values[bottom_left + offset]
                        - self.values[top_left + self.channels + offset]
                        + self.values[top_left + offset])
                        / divisor;
                }
                Ok(result)
            }
            MeanDim::Row => {
                let mut result = vec![0.0; height * self.channels];
                let row_stride = self.stride * self.channels;
                let right_offset = width * self.channels;
                let divisor = width as f64;
                let values = self.values.as_ptr();
                let output = result.as_mut_ptr();
                for row in 0..height {
                    let top_left = ((y + row) * self.stride + x) * self.channels;
                    let bottom_left = top_left + row_stride;
                    for channel in 0..self.channels {
                        // Bounds were established by the rectangle validation above, and all
                        // offsets address the allocated (width + 1) x (height + 1) table.
                        unsafe {
                            *output.add(row * self.channels + channel) = (*values
                                .add(bottom_left + right_offset + channel)
                                - *values.add(bottom_left + channel)
                                - *values.add(top_left + right_offset + channel)
                                + *values.add(top_left + channel))
                                / divisor;
                        }
                    }
                }
                Ok(result)
            }
        }
    }

    #[inline]
    fn rect_sum(&self, x: usize, y: usize, width: usize, height: usize, channel: usize) -> f64 {
        let top_left = (y * self.stride + x) * self.channels + channel;
        let top_right = (y * self.stride + x + width) * self.channels + channel;
        let bottom_left = ((y + height) * self.stride + x) * self.channels + channel;
        let bottom_right = ((y + height) * self.stride + x + width) * self.channels + channel;
        self.values[bottom_right] - self.values[bottom_left] - self.values[top_right] + self.values[top_left]
    }

    pub(crate) fn bytes(&self) -> usize {
        self.values.len() * std::mem::size_of::<f64>()
    }
}

struct IntegralImageCache {
    image_id: u64,
    building: bool,
    integral: Option<IntegralImage>,
}

struct CachedMean {
    image_id: u64,
    rect: Recti,
    values: Vec<f64>,
}

impl Default for IntegralImageCache {
    fn default() -> Self {
        Self {
            image_id: u64::MAX,
            building: false,
            integral: None,
        }
    }
}

pub struct MeanProcessor {
    integral_cache: Arc<Mutex<IntegralImageCache>>,
    mean_results: Mutex<[Option<CachedMean>; 3]>,
    active_image_id: Arc<AtomicU64>,
    precompute_enabled: AtomicBool,
}

impl MeanProcessor {
    pub fn new() -> Self {
        Self {
            integral_cache: Arc::new(Mutex::new(IntegralImageCache::default())),
            mean_results: Mutex::new(Default::default()),
            active_image_id: Arc::new(AtomicU64::new(u64::MAX)),
            precompute_enabled: AtomicBool::new(true),
        }
    }

    pub fn set_precompute_enabled(&self, enabled: bool) {
        let was_enabled = self.precompute_enabled.swap(enabled, Ordering::AcqRel);
        if !enabled && was_enabled {
            self.active_image_id.store(u64::MAX, Ordering::Release);
            *self.integral_cache.lock().unwrap() = IntegralImageCache::default();
        }
    }

    pub fn precompute_enabled(&self) -> bool {
        self.precompute_enabled.load(Ordering::Acquire)
    }

    /// Uses the O(1) summed-area query whenever precompute is ready. The GPU
    /// reduction remains a no-copy fallback during the short build window.
    pub fn compute(&self, image: &ImageData, rect: Recti, dim: MeanDim) -> Result<Vec<f64>> {
        let rect = rect.validate();
        if rect.empty() {
            return Ok(Vec::new());
        }
        let cache_result = !self.precompute_enabled();
        let slot = match dim {
            MeanDim::All => 0,
            MeanDim::Column => 1,
            MeanDim::Row => 2,
        };
        if cache_result {
            let mean_results = self.mean_results.lock().unwrap();
            if let Some(result) = &mean_results[slot] {
                if result.image_id == image.id() && result.rect == rect {
                    return Ok(result.values.clone());
                }
            }
        }
        {
            let integral_cache = self.integral_cache.lock().unwrap();
            if integral_cache.image_id == image.id() {
                if let Some(integral) = integral_cache.integral.as_ref() {
                    return integral.mean(rect, dim);
                }
            }
        }

        if self.precompute_enabled() {
            self.precompute_async(image);
        }
        let texture = image.gpu_texture()?;
        let values = gpu_compute()?.mean(&texture, rect, dim)?;
        if cache_result {
            // Keep one result per reduction dimension so the status bar and
            // row/column plot do not evict each other's results.
            self.mean_results.lock().unwrap()[slot] = Some(CachedMean {
                image_id: image.id(),
                rect,
                values: values.clone(),
            });
        }
        Ok(values)
    }

    pub fn precompute_async(&self, image: &ImageData) {
        if !self.precompute_enabled() {
            return;
        }
        let image_id = image.id();
        {
            let mut integral_cache = self.integral_cache.lock().unwrap();
            if integral_cache.image_id == image_id && (integral_cache.building || integral_cache.integral.is_some()) {
                return;
            }
            integral_cache.image_id = image_id;
            integral_cache.building = true;
            integral_cache.integral = None;
        }
        self.active_image_id.store(image_id, Ordering::Release);

        let image = image.clone();
        let integral_cache = Arc::clone(&self.integral_cache);
        let active_image_id = Arc::clone(&self.active_image_id);
        std::thread::spawn(move || {
            let result = IntegralImage::build(&image, &active_image_id);
            let mut integral_cache = integral_cache.lock().unwrap();
            if integral_cache.image_id != image_id {
                return;
            }
            integral_cache.building = false;
            match result {
                Ok(Some(integral)) => integral_cache.integral = Some(integral),
                Ok(None) => {}
                Err(error) => eprintln!("Mean integral precompute failed: {error}"),
            }
        });
    }

    #[cfg(test)]
    fn install_integral(&self, image: &ImageData, integral: IntegralImage) {
        self.active_image_id.store(image.id(), Ordering::Release);
        *self.integral_cache.lock().unwrap() = IntegralImageCache {
            image_id: image.id(),
            building: false,
            integral: Some(integral),
        };
    }

    #[cfg(test)]
    fn clear_integral(&self) {
        self.active_image_id.store(u64::MAX, Ordering::Release);
        *self.integral_cache.lock().unwrap() = IntegralImageCache::default();
    }

    #[cfg(test)]
    fn cached_integral_bytes(&self) -> usize {
        self.integral_cache
            .lock()
            .unwrap()
            .integral
            .as_ref()
            .map_or(0, IntegralImage::bytes)
    }
}

impl Default for MeanProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ImageSpec, PixelType};

    #[test]
    fn integral_means_match_direct_values() {
        let spec = ImageSpec::new(3, 2, 2, PixelType::F32);
        let image =
            ImageData::from_f32(spec, vec![1.0, 10.0, 2.0, 20.0, 3.0, 30.0, 4.0, 40.0, 5.0, 50.0, 6.0, 60.0]).unwrap();
        let active = AtomicU64::new(image.id());
        let integral = IntegralImage::build(&image, &active).unwrap().unwrap();
        let full = Recti::from_min_size(crate::util::math_ext::vec2i(0, 0), crate::util::math_ext::vec2i(3, 2));
        assert_eq!(integral.mean(full, MeanDim::All).unwrap(), vec![3.5, 35.0]);
        assert_eq!(
            integral.mean(full, MeanDim::Column).unwrap(),
            vec![2.5, 25.0, 3.5, 35.0, 4.5, 45.0]
        );
        assert_eq!(integral.mean(full, MeanDim::Row).unwrap(), vec![2.0, 20.0, 5.0, 50.0]);
    }

    #[test]
    fn specialized_rgb_and_rgba_integrals_match_direct_values() {
        for channels in [3, 4] {
            let width = 7;
            let height = 5;
            let pixels: Vec<f32> = (0..width * height * channels)
                .map(|index| (index as f32 - 17.0) / 13.0)
                .collect();
            let image =
                ImageData::from_f32(ImageSpec::new(width, height, channels, PixelType::F32), pixels.clone()).unwrap();
            let active = AtomicU64::new(image.id());
            let integral = IntegralImage::build(&image, &active).unwrap().unwrap();
            let full =
                Recti::from_min_size(crate::util::math_ext::vec2i(0, 0), crate::util::math_ext::vec2i(width, height));
            let actual = integral.mean(full, MeanDim::All).unwrap();
            for channel in 0..channels as usize {
                let expected = pixels
                    .chunks_exact(channels as usize)
                    .map(|pixel| pixel[channel] as f64)
                    .sum::<f64>()
                    / (width * height) as f64;
                assert!((actual[channel] - expected).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn disabling_precompute_discards_the_cached_integral() {
        let image = ImageData::from_f32(ImageSpec::new(2, 2, 1, PixelType::F32), vec![1.0; 4]).unwrap();
        let active = AtomicU64::new(image.id());
        let integral = IntegralImage::build(&image, &active).unwrap().unwrap();
        let processor = MeanProcessor::new();
        processor.install_integral(&image, integral);
        assert!(processor.cached_integral_bytes() > 0);

        processor.set_precompute_enabled(false);
        assert!(!processor.precompute_enabled());
        assert_eq!(processor.cached_integral_bytes(), 0);
    }

    #[test]
    #[ignore = "manual 8K EXR integral-table memory benchmark"]
    fn benchmark_8k_exr_integral_table_memory() {
        let path =
            std::path::PathBuf::from(std::env::var("EDOLVIEW_8K_EXR").expect("set EDOLVIEW_8K_EXR to an 8K EXR path"));
        let decode_started = std::time::Instant::now();
        let image = ImageData::load_from_path(&path).unwrap();
        let decode_elapsed = decode_started.elapsed();

        let disabled_processor = MeanProcessor::new();
        disabled_processor.set_precompute_enabled(false);
        disabled_processor.precompute_async(&image);
        assert_eq!(disabled_processor.cached_integral_bytes(), 0);

        let active = AtomicU64::new(image.id());
        let table_started = std::time::Instant::now();
        let integral = IntegralImage::build(&image, &active).unwrap().unwrap();
        eprintln!(
            "8K EXR: {}x{}x{}, decode={:.1?}, decoded pixels={:.1} MiB, disabled integral=0 MiB, integral={:.1?}, enabled integral={:.1} MiB",
            image.spec().width,
            image.spec().height,
            image.spec().channels,
            decode_elapsed,
            image.pixels().map_or(0, std::mem::size_of_val) as f64 / (1024.0 * 1024.0),
            table_started.elapsed(),
            integral.bytes() as f64 / (1024.0 * 1024.0),
        );
    }
}
