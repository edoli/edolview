use super::*;

const STATISTICS_WORKGROUP_SIZE: u32 = 128;
pub const STATISTICS_WORKGROUP_STORAGE_SIZE: u32 = STATISTICS_WORKGROUP_SIZE * 4 * std::mem::size_of::<Moment>() as u32;

/// Values are in the texture's normalized units. Only finite samples contribute.
/// An empty channel has NaN statistics; one finite sample has zero population variance.
/// GPU moments use ordinary f32 Welford/Chan arithmetic. Very small variations
/// may lose precision, and overflowing moments are reported as unavailable.
#[derive(Debug)]
pub struct ImageStatistics {
    pub min: Vec<f64>,
    pub max: Vec<f64>,
    pub variance: Vec<f64>,
    pub std_dev: Vec<f64>,
    pub finite_count: Vec<u64>,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Moment {
    lo: u32,
    hi: u32,
    count: u32,
    mean: f32,
    m2: f32,
}

impl GpuComputeContext {
    pub fn statistics(&self, image: &GpuImageTexture, rect: Recti) -> Result<ImageStatistics> {
        let pipeline = self.statistics_pipeline.get_or_init(|| {
            let module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("edolview 128-thread image statistics"),
                source: wgpu::ShaderSource::Wgsl(include_str!("gpu_statistics.wgsl").into()),
            });
            self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("edolview 128-thread image statistics"),
                layout: Some(&self.compute_pipeline_layout),
                module: &module,
                entry_point: Some("statistics"),
                compilation_options: Default::default(),
                cache: None,
            })
        });
        let params = ComputeParams::new(&image.spec, rect);
        let groups = params.roi[2]
            .saturating_mul(params.roi[3])
            .div_ceil(STATISTICS_WORKGROUP_SIZE)
            .clamp(1, REDUCTION_GROUPS);
        let size = u64::from(groups) * 4 * std::mem::size_of::<Moment>() as u64;
        let output = self.empty_storage_buffer(size, "edolview image statistics partials");
        self.run_readback(pipeline, image, None, &output, &params, groups, 1, size, |bytes| {
            decode_statistics(bytemuck::cast_slice(bytes), image.spec.channels as usize)
        })
    }
}

fn decode_statistics(partials: &[Moment], channels: usize) -> ImageStatistics {
    let mut result = ImageStatistics {
        min: vec![f64::NAN; channels],
        max: vec![f64::NAN; channels],
        variance: vec![f64::NAN; channels],
        std_dev: vec![f64::NAN; channels],
        finite_count: vec![0; channels],
    };
    for c in 0..channels {
        let mut lo = u32::MAX;
        let mut hi = 0;
        // Merge the small GPU partials in f64 without copying image pixels.
        let mut mean = 0.0;
        let mut m2 = 0.0;
        let mut count = 0u64;
        for group in partials.chunks_exact(4) {
            let b = group[c];
            if b.count == 0 {
                continue;
            }
            lo = lo.min(b.lo);
            hi = hi.max(b.hi);
            let b_mean = f64::from(b.mean);
            let b_m2 = f64::from(b.m2);
            if count == 0 {
                mean = b_mean;
                m2 = b_m2;
                count = u64::from(b.count);
                continue;
            }
            let total = count + u64::from(b.count);
            let delta = b_mean - mean;
            let weight = f64::from(b.count) / total as f64;
            mean += delta * weight;
            m2 += b_m2 + delta * delta * (count as f64 * weight);
            count = total;
        }
        if count > 0 {
            result.min[c] = ordered_to_float(lo) as f64;
            result.max[c] = ordered_to_float(hi) as f64;
            // Do not turn a non-finite intermediate into a plausible zero.
            let variance = if m2.is_finite() {
                (m2 / count as f64).max(0.0)
            } else {
                f64::NAN
            };
            result.variance[c] = variance;
            result.std_dev[c] = variance.sqrt();
            result.finite_count[c] = count;
        }
    }
    result
}
