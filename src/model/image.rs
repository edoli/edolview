use super::histogram::ImageHistogramCache;
mod gpu_cache;
use crate::model::{gpu_compute, GpuImageTexture, MeanDim, MeanProcessor, Recti};
use color_eyre::eyre::{eyre, Result};
use gpu_cache::{GpuCache, GPU_CACHE_BUDGET};
use std::f64;
use std::sync::{Arc, LazyLock, Mutex, Weak};
use std::{
    fs, mem,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    sync::OnceLock,
};

/// Source component type. Numeric values intentionally match the existing
/// socket protocol's legacy depth codes so external clients remain compatible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PixelType {
    U8 = 0,
    I8 = 1,
    U16 = 2,
    I16 = 3,
    I32 = 4,
    F32 = 5,
    F64 = 6,
    F16 = 7,
}

impl PixelType {
    pub fn from_protocol_code(code: u32) -> Result<Self> {
        match code {
            0 => Ok(Self::U8),
            1 => Ok(Self::I8),
            2 => Ok(Self::U16),
            3 => Ok(Self::I16),
            4 => Ok(Self::I32),
            5 => Ok(Self::F32),
            6 => Ok(Self::F64),
            7 => Ok(Self::F16),
            _ => Err(eyre!("Unsupported pixel type code: {code}")),
        }
    }

    pub fn bytes(self) -> usize {
        match self {
            Self::U8 | Self::I8 => 1,
            Self::U16 | Self::I16 | Self::F16 => 2,
            Self::I32 | Self::F32 => 4,
            Self::F64 => 8,
        }
    }

    pub fn is_floating(self) -> bool {
        matches!(self, Self::F16 | Self::F32 | Self::F64)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::U8 => "uint8",
            Self::I8 => "int8",
            Self::U16 => "uint16",
            Self::I16 => "int16",
            Self::I32 => "int32",
            Self::F16 => "float16",
            Self::F32 => "float32",
            Self::F64 => "float64",
        }
    }

    pub fn alpha(self) -> f64 {
        match self {
            Self::U8 => u8::MAX as f64,
            Self::I8 => i8::MAX as f64,
            Self::U16 => u16::MAX as f64,
            Self::I16 => i16::MAX as f64,
            Self::I32 => i32::MAX as f64,
            Self::F16 | Self::F32 | Self::F64 => 1.0,
        }
    }
}

#[derive(Clone)]
pub struct ImageSpec {
    pub width: i32,
    pub height: i32,
    pub channels: i32,
    pub dtype: PixelType,
}

// data of ImageSpec should be always f32
impl ImageSpec {
    pub fn new(width: i32, height: i32, channels: i32, dtype: PixelType) -> Self {
        Self {
            width,
            height,
            channels,
            dtype,
        }
    }

    pub fn total_bytes(&self) -> usize {
        (self.width as usize) * (self.height as usize) * (self.channels as usize) * mem::size_of::<f32>()
    }

    pub fn bytes_per_pixel(&self) -> usize {
        (self.channels as usize) * mem::size_of::<f32>()
    }

    pub fn pixel_values_to_string<T: Into<f64> + Copy>(&self, vals: &[T]) -> String {
        let alpha = self.dtype.alpha();
        let is_float = self.dtype.is_floating();
        let mut parts: Vec<String> = Vec::with_capacity(vals.len());
        for &v in vals.iter() {
            let vf: f64 = v.into();
            if is_float {
                parts.push(format!("{:.4}", vf * alpha));
            } else {
                parts.push(format!("{:.0}", vf * alpha));
            }
        }
        parts.join(", ")
    }
}

pub trait Image {
    fn id(&self) -> u64;
    fn spec(&self) -> ImageSpec;
    fn data(&self) -> Option<&[f32]>;
    fn gpu_texture(&self) -> Result<Arc<GpuImageTexture>>;
    fn get_pixel_at(&self, x: i32, y: i32) -> Result<PixelValues<'_>> {
        let spec = self.spec();
        if x < 0 || x >= spec.width || y < 0 || y >= spec.height {
            return Err(color_eyre::eyre::eyre!("Coordinates out of bounds"));
        }
        let bytes_per_pixel = spec.bytes_per_pixel();
        let row_bytes = (spec.width as usize) * bytes_per_pixel;
        let start = (y as usize) * row_bytes + (x as usize) * bytes_per_pixel;
        let end = start + bytes_per_pixel;
        self.data()
            .map(|data| PixelValues::Borrowed(&data[start / 4..end / 4]))
            .ok_or_else(|| eyre!("CPU pixel data is unavailable for this GPU-derived image"))
    }
}

pub static MEAN_PROCESSOR: LazyLock<MeanProcessor> = LazyLock::new(MeanProcessor::new);

pub enum PixelValues<'a> {
    Borrowed(&'a [f32]),
    Owned { values: [f32; 4], len: usize },
}

impl std::ops::Deref for PixelValues<'_> {
    type Target = [f32];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(values) => values,
            Self::Owned { values, len } => &values[..*len],
        }
    }
}

#[derive(Clone)]
pub struct MinMaxTotal {
    mins: Vec<f32>,
    maxs: Vec<f32>,

    total_min: OnceLock<f32>,
    total_max: OnceLock<f32>,
}

pub fn empty_minmax() -> MinMaxTotal {
    MinMaxTotal::new(Vec::new(), Vec::new())
}

impl MinMaxTotal {
    pub fn new(mins: Vec<f32>, maxs: Vec<f32>) -> Self {
        Self {
            mins,
            maxs,
            total_min: OnceLock::new(),
            total_max: OnceLock::new(),
        }
    }

    pub fn min(&self, channel: usize) -> f32 {
        if channel >= self.mins.len() {
            0.0
        } else {
            self.mins[channel]
        }
    }

    pub fn max(&self, channel: usize) -> f32 {
        if channel >= self.maxs.len() {
            0.0
        } else {
            self.maxs[channel]
        }
    }

    #[inline]
    pub fn min_abs(&self, channel: usize) -> f32 {
        let channel_min = self.min(channel);
        let channel_max = self.max(channel);

        if f32::signum(channel_min) != f32::signum(channel_max) {
            0.0
        } else {
            f32::min(channel_min.abs(), channel_max.abs())
        }
    }

    #[inline]
    pub fn max_abs(&self, channel: usize) -> f32 {
        if channel >= self.maxs.len() {
            0.0
        } else {
            f32::max(self.mins[channel].abs(), self.maxs[channel].abs())
        }
    }

    pub fn total_min(&self) -> f32 {
        self.total_min
            .get_or_init(|| {
                if self.mins.is_empty() {
                    0.0
                } else {
                    self.mins.iter().copied().fold(f32::INFINITY, f32::min)
                }
            })
            .to_owned()
    }

    pub fn total_max(&self) -> f32 {
        self.total_max
            .get_or_init(|| {
                if self.maxs.is_empty() {
                    0.0
                } else {
                    self.maxs.iter().copied().fold(f32::NEG_INFINITY, f32::max)
                }
            })
            .to_owned()
    }

    #[inline]
    pub fn total_min_abs(&self) -> f32 {
        let total_min = self.total_min();
        let total_max = self.total_max();
        if f32::signum(total_min) != f32::signum(total_max) {
            0.0
        } else {
            f32::min(total_min.abs(), total_max.abs())
        }
    }

    #[inline]
    pub fn total_max_abs(&self) -> f32 {
        f32::max(self.total_min().abs(), self.total_max().abs())
    }
}

#[derive(Clone)]
pub struct ImageData(Arc<ImageDataInner>);

static GPU_TEXTURE_CACHE: LazyLock<Mutex<GpuCache<Arc<GpuImageTexture>>>> =
    LazyLock::new(|| Mutex::new(GpuCache::new(GPU_CACHE_BUDGET)));

struct ImageDataInner {
    id: u64,
    spec: ImageSpec,
    storage: ImageStorage,
    // CPU assets do not own GPU residency. The bounded LRU, renderer and active
    // operations share textures through this non-owning lookup.
    gpu: Mutex<Weak<GpuImageTexture>>,
    hist: OnceLock<Vec<Vec<f32>>>,
    histograms: ImageHistogramCache,
    minmax: OnceLock<MinMaxTotal>,
}

enum ImageStorage {
    // `ImageDataInner` is already reference counted, so an additional `Arc`
    // around the pixels only adds a second allocation and a full Vec-to-slice
    // copy during construction.
    Cpu(crate::model::image_io::DecodedImage),
    Derived(DerivedImage),
    Empty,
}

struct DerivedImage {
    primary: ImageData,
    secondary: ImageData,
    mode: crate::model::ComparisonMode,
    blend_alpha: f32,
    channel_strategy: u32,
}

impl Drop for ImageDataInner {
    fn drop(&mut self) {
        // The cache owns only the GPU texture, so deleting the last CPU asset
        // removes its entry without keeping the image/derived source graph alive.
        if let Ok(mut cache) = GPU_TEXTURE_CACHE.lock() {
            cache.remove(self.id);
        }
    }
}

impl ImageData {
    pub(super) fn histogram_cache(&self) -> &ImageHistogramCache {
        &self.0.histograms
    }

    pub fn from_f32(spec: ImageSpec, pixels: Vec<f32>) -> Result<Self> {
        let expected = spec.width.max(0) as usize * spec.height.max(0) as usize * spec.channels.max(0) as usize;
        if pixels.len() != expected {
            return Err(eyre!("Unexpected image data length: {} != {expected}", pixels.len()));
        }
        let decoded = crate::model::image_io::DecodedImage::new(
            spec.width as u32,
            spec.height as u32,
            spec.channels,
            spec.dtype,
            crate::model::image_io::DecodedPixels::F32(pixels),
        )?;
        Ok(Self(Arc::new(ImageDataInner {
            id: new_id(),
            spec,
            storage: ImageStorage::Cpu(decoded),
            gpu: Mutex::new(Weak::new()),
            hist: OnceLock::new(),
            histograms: ImageHistogramCache::default(),
            minmax: OnceLock::new(),
        })))
    }

    pub fn empty(dtype: PixelType) -> Self {
        Self(Arc::new(ImageDataInner {
            id: new_id(),
            spec: ImageSpec::new(0, 0, 0, dtype),
            storage: ImageStorage::Empty,
            gpu: Mutex::new(Weak::new()),
            hist: OnceLock::new(),
            histograms: ImageHistogramCache::default(),
            minmax: OnceLock::new(),
        }))
    }

    pub fn derived_comparison(
        primary: ImageData,
        secondary: ImageData,
        spec: ImageSpec,
        mode: crate::model::ComparisonMode,
        blend_alpha: f32,
        channel_strategy: u32,
    ) -> Self {
        Self(Arc::new(ImageDataInner {
            id: new_id(),
            spec,
            storage: ImageStorage::Derived(DerivedImage {
                primary,
                secondary,
                mode,
                blend_alpha,
                channel_strategy,
            }),
            gpu: Mutex::new(Weak::new()),
            hist: OnceLock::new(),
            histograms: ImageHistogramCache::default(),
            minmax: OnceLock::new(),
        }))
    }

    pub fn pixels(&self) -> Option<&[f32]> {
        match &self.0.storage {
            ImageStorage::Cpu(image) => image.f32_pixels(),
            ImageStorage::Derived(_) | ImageStorage::Empty => None,
        }
    }

    pub fn mean_value_in_rect(&self, rect: Recti, dim: MeanDim) -> Result<Vec<f64>> {
        #[cfg(debug_assertions)]
        let _timer = match dim {
            MeanDim::All => crate::util::timer::ScopedTimer::new("Compute mean all"),
            MeanDim::Column => crate::util::timer::ScopedTimer::new("Compute mean column"),
            MeanDim::Row => crate::util::timer::ScopedTimer::new("Compute mean row"),
        };

        MEAN_PROCESSOR.compute(self, rect, dim)
    }

    pub fn compute_hist(&self) -> Vec<Vec<f32>> {
        #[cfg(debug_assertions)]
        let _timer = crate::util::timer::ScopedTimer::new("Compute hist");

        let spec = self.spec();
        if spec.width == 0 || spec.height == 0 || spec.channels <= 0 {
            return vec![];
        }
        self.gpu_texture()
            .and_then(|texture| gpu_compute()?.histogram(&texture))
            .unwrap_or_else(|error| {
                eprintln!("GPU histogram failed: {error}");
                Vec::new()
            })
    }

    pub fn hist(&self) -> &Vec<Vec<f32>> {
        self.0.hist.get_or_init(|| self.compute_hist())
    }

    fn compute_minmax(&self) -> MinMaxTotal {
        #[cfg(debug_assertions)]
        let _timer = crate::util::timer::ScopedTimer::new("Compute minmax");

        let spec = self.spec();
        if spec.width == 0 || spec.height == 0 || spec.channels <= 0 {
            return MinMaxTotal::new(vec![], vec![]);
        }

        match self
            .gpu_texture()
            .and_then(|texture| gpu_compute()?.minmax(&texture, Recti::ZERO))
        {
            Ok((mins, maxs)) => MinMaxTotal::new(mins, maxs),
            Err(error) => {
                eprintln!("GPU min/max failed: {error}");
                MinMaxTotal::new(vec![0.0; spec.channels as usize], vec![0.0; spec.channels as usize])
            }
        }
    }

    pub fn minmax(&self) -> &MinMaxTotal {
        self.0.minmax.get_or_init(|| self.compute_minmax())
    }

    pub fn from_raw_bytes(
        bytes: &[u8],
        width: i32,
        height: i32,
        channels: i32,
        pixel_type: PixelType,
    ) -> Result<ImageData> {
        if width <= 0 || height <= 0 || !(1..=4).contains(&channels) {
            return Err(eyre!("Invalid raw image dimensions or channels"));
        }
        let element_count = width as usize * height as usize * channels as usize;
        let expected_bytes = element_count * pixel_type.bytes();
        if bytes.len() != expected_bytes {
            return Err(eyre!("Unexpected raw image size: {} != {expected_bytes}", bytes.len()));
        }
        let pixels = decoded_pixels_from_bytes(bytes, element_count, pixel_type)?;
        let decoded =
            crate::model::image_io::DecodedImage::new(width as u32, height as u32, channels, pixel_type, pixels)?;
        Self::from_decoded(decoded)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<ImageData> {
        Self::from_decoded(crate::model::image_io::decode_bytes(bytes)?)
    }

    pub fn load_from_path(path: &PathBuf) -> Result<ImageData> {
        if !path.exists() {
            return Err(eyre!("Image does not exist: {:?}", path));
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        let decoded = {
            #[cfg(debug_assertions)]
            let _timer = crate::util::timer::ScopedTimer::new("Image read");
            if ext == "pfm" {
                crate::model::image_io::decode_pfm_owned(fs::read(path)?)?
            } else if ext == "flo" {
                crate::model::image_io::decode_flo_owned(fs::read(path)?)?
            } else if crate::supported_image::is_heif_extension(ext.as_str()) {
                #[cfg(not(feature = "heif"))]
                return Err(eyre!("HEIF support is not enabled"));

                #[cfg(feature = "heif")]
                {
                    crate::model::image_io::decode_heif(path)?
                }
            } else {
                crate::model::image_io::decode_path(path)?
            }
        };
        Self::from_decoded(decoded)
    }

    pub fn load_from_clipboard() -> Result<ImageData> {
        #[cfg(debug_assertions)]
        let _timer = crate::util::timer::ScopedTimer::new("Clipboard read");

        Self::from_decoded(super::clipboard_image::read_image()?)
    }

    pub fn load_from_url(url: &str) -> Result<ImageData> {
        #[cfg(debug_assertions)]
        let _timer = crate::util::timer::ScopedTimer::new("Image download");

        // A stalled server must not block subsequent queued image loads indefinitely.
        let bytes = ureq::get(url)
            .config()
            .timeout_global(Some(std::time::Duration::from_secs(30)))
            .build()
            .call()?
            .body_mut()
            .read_to_vec()?;

        Self::from_bytes(&bytes)
    }

    pub(crate) fn from_decoded(decoded: crate::model::image_io::DecodedImage) -> Result<ImageData> {
        #[cfg(debug_assertions)]
        let _timer = crate::util::timer::ScopedTimer::new("Image read postprocess");
        let spec = ImageSpec::new(decoded.width, decoded.height, decoded.channels, decoded.pixel_type);
        Ok(Self(Arc::new(ImageDataInner {
            id: new_id(),
            spec,
            storage: ImageStorage::Cpu(decoded),
            gpu: Mutex::new(Weak::new()),
            hist: OnceLock::new(),
            histograms: ImageHistogramCache::default(),
            minmax: OnceLock::new(),
        })))
    }

    pub fn gpu_texture(&self) -> Result<Arc<GpuImageTexture>> {
        let mut resident = self.0.gpu.lock().unwrap();
        if let Some(texture) = resident.upgrade() {
            self.retain_gpu_texture(&texture);
            return Ok(texture);
        }
        let compute = gpu_compute()?;
        let texture = match &self.0.storage {
            ImageStorage::Cpu(image) => compute.upload_decoded(image)?,
            ImageStorage::Derived(derived) => {
                let primary = derived.primary.gpu_texture()?;
                let secondary = derived.secondary.gpu_texture()?;
                compute.compare(
                    &primary,
                    &secondary,
                    &self.0.spec,
                    derived.mode,
                    derived.blend_alpha,
                    derived.channel_strategy,
                )?
            }
            ImageStorage::Empty => return Err(eyre!("Image is empty")),
        };
        *resident = Arc::downgrade(&texture);
        self.retain_gpu_texture(&texture);
        Ok(texture)
    }

    fn retain_gpu_texture(&self, image: &Arc<GpuImageTexture>) {
        let texture = &image.texture;
        // Image textures currently have one mip and sample. Bypass unfamiliar
        // layouts rather than undercounting future multisample/mipmap resources.
        if texture.mip_level_count() != 1 || texture.sample_count() != 1 {
            return;
        }
        let size = texture.size();
        if let Some(bytes) =
            gpu_cache::payload_bytes(texture.format(), size.width, size.height, size.depth_or_array_layers)
        {
            GPU_TEXTURE_CACHE.lock().unwrap().retain(self.id(), bytes, Arc::clone(image));
        }
    }

    #[cfg(test)]
    pub(crate) fn evict_gpu_cache(&self) {
        GPU_TEXTURE_CACHE.lock().unwrap().remove(self.id());
    }

    fn pixel_values(&self, x: i32, y: i32) -> Result<Vec<f32>> {
        let spec = self.spec();
        if x < 0 || x >= spec.width || y < 0 || y >= spec.height {
            return Err(eyre!("Coordinates out of bounds"));
        }
        match &self.0.storage {
            ImageStorage::Cpu(image) => {
                let (values, channels) = image.normalized_pixel(x as usize, y as usize)?;
                Ok(values[..channels].to_vec())
            }
            ImageStorage::Derived(derived) => {
                let mut lhs = derived.primary.pixel_values(x, y)?;
                let mut rhs = derived.secondary.pixel_values(x, y)?;
                normalize_pixel_channels(&mut lhs, &mut rhs, derived.channel_strategy, spec.channels as usize);
                Ok(lhs
                    .iter()
                    .zip(rhs.iter())
                    .take(spec.channels as usize)
                    .map(|(&a, &b)| match derived.mode {
                        crate::model::ComparisonMode::Diff => a - b,
                        crate::model::ComparisonMode::Blend => {
                            a * (1.0 - derived.blend_alpha) + b * derived.blend_alpha
                        }
                        crate::model::ComparisonMode::Split => a,
                    })
                    .collect())
            }
            ImageStorage::Empty => Err(eyre!("Image is empty")),
        }
    }

    pub(crate) fn scalar_at(&self, pixel_index: usize, channel: usize) -> Option<f32> {
        let spec = self.spec();
        if pixel_index >= spec.width as usize * spec.height as usize || channel >= spec.channels as usize {
            return None;
        }
        match &self.0.storage {
            ImageStorage::Cpu(image) => image.normalized_scalar(pixel_index, channel),
            ImageStorage::Derived(derived) => {
                let primary_channel = if derived.channel_strategy == 1 { 0 } else { channel };
                let secondary_channel = if derived.channel_strategy == 2 { 0 } else { channel };
                let lhs = derived.primary.scalar_at(pixel_index, primary_channel)?;
                let rhs = derived.secondary.scalar_at(pixel_index, secondary_channel)?;
                Some(match derived.mode {
                    crate::model::ComparisonMode::Diff => lhs - rhs,
                    crate::model::ComparisonMode::Blend => {
                        lhs * (1.0 - derived.blend_alpha) + rhs * derived.blend_alpha
                    }
                    crate::model::ComparisonMode::Split => lhs,
                })
            }
            ImageStorage::Empty => None,
        }
    }

    pub(crate) fn normalized_pixel_at(&self, pixel_index: usize) -> Option<([f32; 4], usize)> {
        let spec = self.spec();
        if pixel_index >= spec.width as usize * spec.height as usize {
            return None;
        }
        match &self.0.storage {
            ImageStorage::Cpu(image) => image
                .normalized_pixel(pixel_index % spec.width as usize, pixel_index / spec.width as usize)
                .ok(),
            ImageStorage::Derived(_) => {
                let mut values = [0.0; 4];
                for (channel, value) in values.iter_mut().enumerate().take(spec.channels as usize) {
                    *value = self.scalar_at(pixel_index, channel)?;
                }
                Some((values, spec.channels as usize))
            }
            ImageStorage::Empty => None,
        }
    }
}

impl Image for ImageData {
    fn id(&self) -> u64 {
        self.0.id
    }

    fn spec(&self) -> ImageSpec {
        self.0.spec.clone()
    }

    fn data(&self) -> Option<&[f32]> {
        self.pixels()
    }

    fn gpu_texture(&self) -> Result<Arc<GpuImageTexture>> {
        ImageData::gpu_texture(self)
    }

    fn get_pixel_at(&self, x: i32, y: i32) -> Result<PixelValues<'_>> {
        if let Some(pixels) = self.pixels() {
            let spec = self.spec();
            if x < 0 || x >= spec.width || y < 0 || y >= spec.height {
                return Err(eyre!("Coordinates out of bounds"));
            }
            let start = ((y * spec.width + x) * spec.channels) as usize;
            return Ok(PixelValues::Borrowed(&pixels[start..start + spec.channels as usize]));
        }
        let spec = self.spec();
        let pixel = self.pixel_values(x, y)?;
        let mut values = [0.0; 4];
        values[..pixel.len()].copy_from_slice(&pixel);
        Ok(PixelValues::Owned {
            values,
            len: spec.channels as usize,
        })
    }
}

fn decoded_pixels_from_bytes(
    bytes: &[u8],
    element_count: usize,
    pixel_type: PixelType,
) -> Result<crate::model::image_io::DecodedPixels> {
    let required_bytes = element_count
        .checked_mul(pixel_type.bytes())
        .ok_or_else(|| eyre!("Raw image byte count overflow"))?;
    let bytes = bytes
        .get(..required_bytes)
        .ok_or_else(|| eyre!("Unexpected raw image size: {} < {required_bytes}", bytes.len()))?;

    macro_rules! read_values {
        ($ty:ty) => {{
            if let Ok(values) = bytemuck::try_cast_slice::<u8, $ty>(bytes) {
                values.to_vec()
            } else {
                // Socket payloads can begin at arbitrary byte alignment.
                bytes
                    .chunks_exact(mem::size_of::<$ty>())
                    .map(|chunk| <$ty>::from_ne_bytes(chunk.try_into().expect("exact scalar chunk")))
                    .collect()
            }
        }};
    }

    Ok(match pixel_type {
        PixelType::U8 => crate::model::image_io::DecodedPixels::U8(bytes.to_vec()),
        PixelType::I8 => crate::model::image_io::DecodedPixels::I8(bytes.iter().map(|&value| value as i8).collect()),
        PixelType::U16 => crate::model::image_io::DecodedPixels::U16(read_values!(u16)),
        PixelType::I16 => crate::model::image_io::DecodedPixels::I16(read_values!(i16)),
        PixelType::I32 => crate::model::image_io::DecodedPixels::I32(read_values!(i32)),
        PixelType::F32 => crate::model::image_io::DecodedPixels::F32(read_values!(f32)),
        PixelType::F64 => crate::model::image_io::DecodedPixels::F32(
            read_values!(f64).into_iter().map(|value| value as f32).collect(),
        ),
        PixelType::F16 => crate::model::image_io::DecodedPixels::F16(read_values!(u16)),
    })
}

fn normalize_pixel_channels(lhs: &mut Vec<f32>, rhs: &mut Vec<f32>, strategy: u32, output_channels: usize) {
    match strategy {
        1 => lhs.resize(output_channels, lhs[0]),
        2 => rhs.resize(output_channels, rhs[0]),
        _ => {
            lhs.truncate(output_channels);
            rhs.truncate(output_channels);
        }
    }
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn new_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn from_f32_keeps_the_input_allocation() {
        let pixels = vec![0.0, 0.25, 0.5, 1.0];
        let input_ptr = pixels.as_ptr();
        let image = ImageData::from_f32(ImageSpec::new(1, 1, 4, PixelType::F32), pixels).unwrap();

        assert_eq!(image.pixels().unwrap().as_ptr(), input_ptr);
    }

    #[test]
    fn unaligned_typed_bytes_still_decode() {
        let values = [1_u16, 32768, u16::MAX];
        let mut bytes = vec![0_u8];
        bytes.extend_from_slice(bytemuck::cast_slice(&values));

        let image = ImageData::from_raw_bytes(&bytes[1..], values.len() as i32, 1, 1, PixelType::U16).unwrap();

        let expected: Vec<f32> = values.iter().map(|&value| value as f32 / u16::MAX as f32).collect();
        let actual: Vec<f32> = (0..values.len()).map(|x| image.get_pixel_at(x as i32, 0).unwrap()[0]).collect();
        assert_eq!(actual, expected);

        let float_values = [0.125_f32, 0.5, 1.0];
        let mut float_bytes = vec![0_u8];
        float_bytes.extend_from_slice(bytemuck::cast_slice(&float_values));
        let float_image =
            ImageData::from_raw_bytes(&float_bytes[1..], float_values.len() as i32, 1, 1, PixelType::F32).unwrap();
        assert_eq!(float_image.pixels().unwrap(), float_values);
    }

    #[test]
    fn image_rs_decodes_the_embedded_png() {
        let image = ImageData::from_bytes(include_bytes!("../../icons/icon.png")).unwrap();
        let spec = image.spec();
        assert!(spec.width > 0 && spec.height > 0);
        assert_eq!(spec.channels, 4);
        assert_eq!(spec.dtype, PixelType::U8);
    }

    #[test]
    #[ignore = "manual reference image loading benchmark"]
    fn loads_reference_images() {
        let root = PathBuf::from(std::env::var("EDOLVIEW_BENCH_DATA").expect("EDOLVIEW_BENCH_DATA"));
        for (format, stem) in [("exr", "metro_noord"), ("hdr", "voortrekker_interior")] {
            for size in ["1k", "2k", "4k", "8k"] {
                let path = root.join(format!("{stem}_{size}.{format}"));
                let start = Instant::now();
                let image = ImageData::load_from_path(&path).unwrap();
                let spec = image.spec();
                println!(
                    "{format},{size},{},{},{},{},{:.3}",
                    spec.width,
                    spec.height,
                    spec.channels,
                    spec.dtype.name(),
                    start.elapsed().as_secs_f64() * 1000.0
                );
                assert_eq!(
                    image.pixels().unwrap().len(),
                    spec.width as usize * spec.height as usize * spec.channels as usize
                );
            }
        }
        if let Some(path) = std::env::var_os("EDOLVIEW_JP2_TEST").map(PathBuf::from) {
            let image = ImageData::load_from_path(&path).unwrap();
            let spec = image.spec();
            println!(
                "jp2,test,{},{},{},{}",
                spec.width,
                spec.height,
                spec.channels,
                spec.dtype.name()
            );
            assert!(spec.width > 0 && spec.height > 0 && (1..=4).contains(&spec.channels));
        }
    }
}
