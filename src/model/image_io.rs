use crate::model::PixelType;
use color_eyre::eyre::{eyre, Result};
use exr::prelude::{read, MetaData, ReadChannels, ReadLayers, ReadSpecificChannel, Text, Vec2};
use image::{DynamicImage, ImageFormat, ImageReader};
use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom},
    path::Path,
    sync::Once,
};
use tiff::{
    decoder::{Decoder as TiffDecoder, DecodingResult, Limits as TiffLimits},
    ColorType as TiffColorType,
};

const EXR_MAGIC: [u8; 4] = [0x76, 0x2f, 0x31, 0x01];
const JP2_MAGIC: [u8; 8] = [0x00, 0x00, 0x00, 0x0c, 0x6a, 0x50, 0x20, 0x20];
const J2C_MAGIC: [u8; 4] = [0xff, 0x4f, 0xff, 0x51];
const TIFF_LE_MAGIC: [u8; 4] = [b'I', b'I', 42, 0];
const TIFF_BE_MAGIC: [u8; 4] = [b'M', b'M', 0, 42];
const BIG_TIFF_LE_MAGIC: [u8; 4] = [b'I', b'I', 43, 0];
const BIG_TIFF_BE_MAGIC: [u8; 4] = [b'M', b'M', 0, 43];

static DECODER_HOOKS: Once = Once::new();

fn ensure_decoder_hooks() {
    DECODER_HOOKS.call_once(|| {
        hayro_jpeg2000::integration::register_decoding_hook();
    });
}

pub(crate) enum DecodedPixels {
    /// An encoded-format payload whose scalar layout is described by
    /// `DecodedTransform`. This avoids allocating a second full-image buffer
    /// for simple uncompressed formats such as PFM and FLO.
    Raw(Vec<u8>),
    U8(Vec<u8>),
    I8(Vec<i8>),
    U16(Vec<u16>),
    I16(Vec<i16>),
    I32(Vec<i32>),
    U32(Vec<u32>),
    F16(Vec<u16>),
    F32(Vec<f32>),
}

impl DecodedPixels {
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Raw(values) => values.len(),
            Self::U8(values) => values.len(),
            Self::I8(values) => values.len(),
            Self::U16(values) => values.len(),
            Self::I16(values) => values.len(),
            Self::I32(values) => values.len(),
            Self::U32(values) => values.len(),
            Self::F16(values) => values.len(),
            Self::F32(values) => values.len(),
        }
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        match self {
            Self::Raw(values) => values,
            Self::U8(values) => values,
            Self::I8(values) => bytemuck::cast_slice(values),
            Self::U16(values) => bytemuck::cast_slice(values),
            Self::I16(values) => bytemuck::cast_slice(values),
            Self::I32(values) => bytemuck::cast_slice(values),
            Self::U32(values) => bytemuck::cast_slice(values),
            Self::F16(values) => bytemuck::cast_slice(values),
            Self::F32(values) => bytemuck::cast_slice(values),
        }
    }

    pub(crate) fn f32_slice(&self) -> Option<&[f32]> {
        match self {
            Self::F32(values) => Some(values),
            _ => None,
        }
    }

    pub(crate) fn sample_bytes(&self) -> usize {
        match self {
            Self::Raw(_) | Self::I32(_) | Self::U32(_) | Self::F32(_) => 4,
            Self::U8(_) | Self::I8(_) => 1,
            Self::U16(_) | Self::I16(_) | Self::F16(_) => 2,
        }
    }

    pub(crate) fn shader_kind(&self) -> Option<u32> {
        match self {
            Self::Raw(_) => Some(6),
            Self::U8(_) => Some(0),
            Self::I8(_) => Some(1),
            Self::U16(_) => Some(2),
            Self::I16(_) => Some(3),
            Self::I32(_) => Some(4),
            Self::F16(_) => Some(5),
            Self::F32(_) => Some(6),
            Self::U32(_) => Some(7),
        }
    }
}

pub(crate) struct DecodedLayout {
    pub row_stride_bytes: usize,
    pub plane_stride_bytes: usize,
    pub planes: u32,
    pub bit_depth: u8,
    pub input_channels: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct DecodedTransform {
    pub data_offset_bytes: usize,
    pub flip_y: bool,
    pub swap_bytes: bool,
    pub scale: f32,
}

impl Default for DecodedTransform {
    fn default() -> Self {
        Self {
            data_offset_bytes: 0,
            flip_y: false,
            swap_bytes: false,
            scale: 1.0,
        }
    }
}

pub(crate) enum DecodedColor {
    Direct,
    Palette(Vec<u16>),
    Cmyk,
    Cmyka,
    YCbCr,
    Lab,
}

impl DecodedColor {
    pub(crate) fn shader_kind(&self) -> u32 {
        match self {
            Self::Direct => 0,
            Self::Palette(_) => 1,
            Self::Cmyk => 2,
            Self::Cmyka => 3,
            Self::YCbCr => 4,
            Self::Lab => 5,
        }
    }

    pub(crate) fn palette(&self) -> Option<&[u16]> {
        match self {
            Self::Palette(values) => Some(values),
            _ => None,
        }
    }
}

pub struct DecodedImage {
    pub width: i32,
    pub height: i32,
    pub channels: i32,
    pub pixel_type: PixelType,
    pub(crate) pixels: DecodedPixels,
    pub(crate) layout: DecodedLayout,
    pub(crate) color: DecodedColor,
    pub(crate) transform: DecodedTransform,
}

impl DecodedImage {
    pub(crate) fn new(
        width: u32,
        height: u32,
        channels: i32,
        pixel_type: PixelType,
        pixels: DecodedPixels,
    ) -> Result<Self> {
        let input_channels = channels as usize;
        let sample_bytes = pixels.sample_bytes();
        let bit_depth = (sample_bytes * 8) as u8;
        Self::new_with_layout(
            width,
            height,
            channels,
            pixel_type,
            pixels,
            DecodedLayout {
                row_stride_bytes: width as usize * input_channels * sample_bytes,
                plane_stride_bytes: width as usize * height as usize * input_channels * sample_bytes,
                planes: 1,
                bit_depth,
                input_channels,
            },
            DecodedColor::Direct,
        )
    }

    pub(crate) fn new_with_layout(
        width: u32,
        height: u32,
        channels: i32,
        pixel_type: PixelType,
        pixels: DecodedPixels,
        layout: DecodedLayout,
        color: DecodedColor,
    ) -> Result<Self> {
        Self::new_with_transform(
            width,
            height,
            channels,
            pixel_type,
            pixels,
            layout,
            color,
            DecodedTransform::default(),
        )
    }

    #[allow(clippy::too_many_arguments)] // Decoding metadata is assembled independently by each format reader.
    pub(crate) fn new_with_transform(
        width: u32,
        height: u32,
        channels: i32,
        pixel_type: PixelType,
        pixels: DecodedPixels,
        layout: DecodedLayout,
        color: DecodedColor,
        transform: DecodedTransform,
    ) -> Result<Self> {
        let width = i32::try_from(width).map_err(|_| eyre!("Image width exceeds i32: {width}"))?;
        let height = i32::try_from(height).map_err(|_| eyre!("Image height exceeds i32: {height}"))?;
        if width <= 0 || height <= 0 || !(1..=4).contains(&channels) {
            return Err(eyre!("Invalid decoded image dimensions or channels"));
        }
        if !(1..=5).contains(&layout.input_channels) || layout.planes == 0 || layout.planes > 5 {
            return Err(eyre!("Invalid decoded image channel or plane layout"));
        }
        let samples_per_row = if layout.planes > 1 {
            width as usize
        } else {
            width as usize * layout.input_channels
        };
        let minimum_row_bytes = if layout.bit_depth < 8 {
            (samples_per_row * layout.bit_depth as usize).div_ceil(8)
        } else {
            samples_per_row * pixels.sample_bytes()
        };
        if layout.row_stride_bytes < minimum_row_bytes {
            return Err(eyre!(
                "Decoded image row stride is too small: {} < {minimum_row_bytes}",
                layout.row_stride_bytes
            ));
        }
        let plane_bytes = layout
            .row_stride_bytes
            .checked_mul(height as usize)
            .ok_or_else(|| eyre!("Decoded image plane size overflow"))?;
        if layout.planes > 1 && layout.plane_stride_bytes < plane_bytes {
            return Err(eyre!("Decoded image plane stride is too small"));
        }
        let required_bytes = if layout.planes > 1 {
            (layout.planes as usize - 1)
                .checked_mul(layout.plane_stride_bytes)
                .and_then(|offset| offset.checked_add(plane_bytes))
        } else {
            Some(plane_bytes)
        }
        .ok_or_else(|| eyre!("Decoded image buffer size overflow"))?;
        let required_bytes = transform
            .data_offset_bytes
            .checked_add(required_bytes)
            .ok_or_else(|| eyre!("Decoded image buffer offset overflow"))?;
        if pixels.bytes().len() < required_bytes {
            return Err(eyre!(
                "Decoded image buffer is truncated: {} < {required_bytes}",
                pixels.bytes().len()
            ));
        }
        if let DecodedColor::Palette(values) = &color {
            let required = 3_usize
                .checked_mul(1_usize << layout.bit_depth)
                .ok_or_else(|| eyre!("Decoded palette size overflow"))?;
            if values.len() < required {
                return Err(eyre!("Decoded palette is truncated: {} < {required}", values.len()));
            }
        }
        Ok(Self {
            width,
            height,
            channels,
            pixel_type,
            pixels,
            layout,
            color,
            transform,
        })
    }

    pub(crate) fn f32_pixels(&self) -> Option<&[f32]> {
        if self.is_canonical_direct() {
            self.pixels.f32_slice()
        } else {
            None
        }
    }

    pub(crate) fn is_canonical_direct(&self) -> bool {
        matches!(self.color, DecodedColor::Direct)
            && matches!(
                self.transform,
                DecodedTransform {
                    data_offset_bytes: 0,
                    flip_y: false,
                    swap_bytes: false,
                    scale: 1.0
                }
            )
            && !matches!(self.pixels, DecodedPixels::Raw(_))
            && self.layout.planes == 1
            && self.layout.bit_depth >= 8
            && self.layout.input_channels == self.channels as usize
            && self.layout.row_stride_bytes
                == self.width as usize * self.layout.input_channels * self.pixels.sample_bytes()
    }

    pub(crate) fn normalized_pixel(&self, x: usize, y: usize) -> Result<([f32; 4], usize)> {
        if x >= self.width as usize || y >= self.height as usize {
            return Err(eyre!("Pixel coordinates are outside the decoded image"));
        }
        let mut source = [0.0_f32; 5];
        for (channel, value) in source.iter_mut().enumerate().take(self.layout.input_channels) {
            *value = self.normalized_sample(x, y, channel)?;
        }
        let mut output = [0.0_f32; 4];
        let channels = self.channels as usize;
        match &self.color {
            DecodedColor::Direct => output[..channels].copy_from_slice(&source[..channels]),
            DecodedColor::Palette(palette) => {
                let entries = 1_usize << self.layout.bit_depth;
                let index = (source[0] * (entries - 1) as f32).round() as usize;
                output[..3].copy_from_slice(&[
                    palette[index] as f32 / u16::MAX as f32,
                    palette[entries + index] as f32 / u16::MAX as f32,
                    palette[2 * entries + index] as f32 / u16::MAX as f32,
                ]);
            }
            DecodedColor::Cmyk | DecodedColor::Cmyka => {
                let k = 1.0 - source[3];
                output[..3].copy_from_slice(&[(1.0 - source[0]) * k, (1.0 - source[1]) * k, (1.0 - source[2]) * k]);
                if matches!(self.color, DecodedColor::Cmyka) {
                    output[3] = source[4];
                }
            }
            DecodedColor::YCbCr => {
                let cb = source[1] - 0.5;
                let cr = source[2] - 0.5;
                output[..3].copy_from_slice(&[
                    (source[0] + 1.402 * cr).clamp(0.0, 1.0),
                    (source[0] - 0.344_136 * cb - 0.714_136 * cr).clamp(0.0, 1.0),
                    (source[0] + 1.772 * cb).clamp(0.0, 1.0),
                ]);
            }
            DecodedColor::Lab => output[..3].copy_from_slice(&tiff_lab_to_rgb(source[0], source[1], source[2])),
        }
        Ok((output, channels))
    }

    pub(crate) fn normalized_scalar(&self, pixel_index: usize, channel: usize) -> Option<f32> {
        if channel >= self.channels as usize {
            return None;
        }
        let x = pixel_index % self.width as usize;
        let y = pixel_index / self.width as usize;
        self.normalized_pixel(x, y).ok().map(|(values, _)| values[channel])
    }

    fn normalized_sample(&self, x: usize, y: usize, channel: usize) -> Result<f32> {
        let y = if self.transform.flip_y {
            self.height as usize - y - 1
        } else {
            y
        };
        let bit_depth = self.layout.bit_depth as usize;
        if bit_depth < 8 {
            let (base, sample) = if self.layout.planes > 1 {
                (channel * self.layout.plane_stride_bytes + y * self.layout.row_stride_bytes, x)
            } else {
                (y * self.layout.row_stride_bytes, x * self.layout.input_channels + channel)
            };
            let bit = sample * bit_depth;
            let byte = *self
                .pixels
                .bytes()
                .get(base + bit / 8)
                .ok_or_else(|| eyre!("Packed image row is truncated"))?;
            let shift = 8 - bit_depth - bit % 8;
            let raw = (byte >> shift) & ((1_u8 << bit_depth) - 1);
            return Ok(raw as f32 / ((1_u16 << bit_depth) - 1) as f32);
        }

        let sample_bytes = self.pixels.sample_bytes();
        let byte_offset = self.transform.data_offset_bytes
            + if self.layout.planes > 1 {
                channel * self.layout.plane_stride_bytes + y * self.layout.row_stride_bytes + x * sample_bytes
            } else {
                y * self.layout.row_stride_bytes + (x * self.layout.input_channels + channel) * sample_bytes
            };
        let index = byte_offset / sample_bytes;
        let unsigned_max = if self.layout.bit_depth >= 32 {
            u32::MAX as f64
        } else {
            ((1_u32 << self.layout.bit_depth) - 1) as f64
        };
        let signed_max = if self.layout.bit_depth >= 32 {
            i32::MAX as f64
        } else {
            ((1_i32 << (self.layout.bit_depth - 1)) - 1).max(1) as f64
        };
        macro_rules! sample {
            ($values:expr) => {
                *$values
                    .get(index)
                    .ok_or_else(|| eyre!("Decoded image sample buffer is truncated"))?
            };
        }
        let value = match &self.pixels {
            DecodedPixels::Raw(values) => {
                let bytes: [u8; 4] = values
                    .get(byte_offset..byte_offset + 4)
                    .ok_or_else(|| eyre!("Decoded raw sample buffer is truncated"))?
                    .try_into()
                    .expect("exact raw f32 sample");
                let bits = u32::from_le_bytes(bytes);
                let bits = if self.transform.swap_bytes {
                    bits.swap_bytes()
                } else {
                    bits
                };
                f32::from_bits(bits) as f64
            }
            DecodedPixels::U8(values) => sample!(values) as f64 / unsigned_max,
            DecodedPixels::I8(values) => sample!(values) as f64 / signed_max,
            DecodedPixels::U16(values) => sample!(values) as f64 / unsigned_max,
            DecodedPixels::I16(values) => sample!(values) as f64 / signed_max,
            DecodedPixels::I32(values) => sample!(values) as f64 / signed_max,
            DecodedPixels::U32(values) => sample!(values) as f64 / unsigned_max,
            DecodedPixels::F16(values) => half::f16::from_bits(sample!(values)).to_f32() as f64,
            DecodedPixels::F32(values) => sample!(values) as f64,
        };
        Ok(value as f32 * self.transform.scale)
    }
}

pub fn decode_path(path: &Path) -> Result<DecodedImage> {
    if crate::supported_image::is_raw_path(path) {
        return super::raw::decode_path(path);
    }
    ensure_decoder_hooks();
    let mut reader = BufReader::new(File::open(path)?);
    let magic = reader.fill_buf()?;
    if super::raw::has_signature(magic) {
        return super::raw::decode_path(path);
    }
    if magic.starts_with(&EXR_MAGIC) {
        return decode_exr_reader(reader);
    }
    if is_jpeg2000(magic) {
        return decode_jpeg2000_reader(reader);
    }
    if is_tiff(magic) {
        return decode_tiff_reader(reader);
    }
    let mut image_reader = ImageReader::new(reader).with_guessed_format()?;
    if image_reader.format().is_none() {
        if let Ok(format) = ImageFormat::from_path(path) {
            image_reader.set_format(format);
        }
    }
    let image = image_reader.decode()?;
    decoded_dynamic_image(image)
}

pub fn decode_bytes(bytes: &[u8]) -> Result<DecodedImage> {
    if super::raw::has_signature(bytes) {
        return super::raw::decode_bytes(bytes);
    }
    ensure_decoder_hooks();
    if bytes.starts_with(&EXR_MAGIC) {
        return decode_exr_reader(Cursor::new(bytes));
    }
    if is_jpeg2000(bytes) {
        return decode_jpeg2000_reader(Cursor::new(bytes));
    }
    if is_tiff(bytes) {
        return decode_tiff_reader(Cursor::new(bytes));
    }
    let image_reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let image = if image_reader.format().is_some() {
        image_reader.decode()?
    } else {
        // TGA deliberately has no fixed magic signature. Try its strict header
        // parser only after every signature-based format has been ruled out.
        ImageReader::with_format(Cursor::new(bytes), ImageFormat::Tga).decode()?
    };
    decoded_dynamic_image(image)
}

fn is_jpeg2000(bytes: &[u8]) -> bool {
    bytes.starts_with(&JP2_MAGIC) || bytes.starts_with(&J2C_MAGIC)
}

fn is_tiff(bytes: &[u8]) -> bool {
    [TIFF_LE_MAGIC, TIFF_BE_MAGIC, BIG_TIFF_LE_MAGIC, BIG_TIFF_BE_MAGIC]
        .iter()
        .any(|magic| bytes.starts_with(magic))
}

#[derive(Clone)]
enum ExrLayout {
    Rgb,
    Rgba,
    Mono(Text),
}

#[derive(Clone)]
struct ExrSelection {
    layout: ExrLayout,
    width: usize,
    height: usize,
    data_offset_x: i32,
    data_offset_y: i32,
}

impl ExrSelection {
    fn channels(&self) -> usize {
        match self.layout {
            ExrLayout::Rgb => 3,
            ExrLayout::Rgba => 4,
            ExrLayout::Mono(_) => 1,
        }
    }

    fn value_count(&self) -> Result<usize> {
        self.width
            .checked_mul(self.height)
            .and_then(|pixels| pixels.checked_mul(self.channels()))
            .ok_or_else(|| eyre!("EXR image size overflow"))
    }
}

struct ExrPixels {
    values: Vec<f32>,
    selection: ExrSelection,
}

impl ExrPixels {
    fn new(selection: ExrSelection) -> Self {
        Self {
            values: vec![0.0; selection.value_count().expect("validated EXR image size")],
            selection,
        }
    }

    fn set<const N: usize>(&mut self, position: Vec2<usize>, values: [f32; N]) {
        debug_assert_eq!(N, self.selection.channels());
        let x = position.x() as i64 + self.selection.data_offset_x as i64;
        let y = position.y() as i64 + self.selection.data_offset_y as i64;
        if x < 0 || y < 0 || x >= self.selection.width as i64 || y >= self.selection.height as i64 {
            return;
        }
        let start = (y as usize * self.selection.width + x as usize) * N;
        self.values[start..start + N].copy_from_slice(&values);
    }
}

fn decode_exr_reader<R: BufRead + Seek>(reader: R) -> Result<DecodedImage> {
    let exr_reader = exr::block::read(reader, false)?;
    let selection = select_exr_layout(exr_reader.meta_data())?;
    selection.value_count()?;

    let pixels = match selection.layout.clone() {
        ExrLayout::Rgb => {
            let pixel_selection = selection.clone();
            let image = read()
                .no_deep_data()
                .largest_resolution_level()
                .rgb_channels(
                    move |_resolution, _channels| ExrPixels::new(pixel_selection.clone()),
                    |pixels, position, (r, g, b): (f32, f32, f32)| pixels.set(position, [r, g, b]),
                )
                .first_valid_layer()
                .all_attributes()
                .from_chunks(exr_reader)?;
            image.layer_data.channel_data.pixels.values
        }
        ExrLayout::Rgba => {
            let pixel_selection = selection.clone();
            let image = read()
                .no_deep_data()
                .largest_resolution_level()
                .rgba_channels(
                    move |_resolution, _channels| ExrPixels::new(pixel_selection.clone()),
                    |pixels, position, (r, g, b, a): (f32, f32, f32, f32)| {
                        pixels.set(position, [r, g, b, a]);
                    },
                )
                .first_valid_layer()
                .all_attributes()
                .from_chunks(exr_reader)?;
            image.layer_data.channel_data.pixels.values
        }
        ExrLayout::Mono(channel_name) => {
            let pixel_selection = selection.clone();
            let image = read()
                .no_deep_data()
                .largest_resolution_level()
                .specific_channels()
                .required::<f32>(channel_name)
                .collect_pixels(
                    move |_resolution, _channels| ExrPixels::new(pixel_selection.clone()),
                    |pixels, position, (value,): (f32,)| pixels.set(position, [value]),
                )
                .first_valid_layer()
                .all_attributes()
                .from_chunks(exr_reader)?;
            image.layer_data.channel_data.pixels.values
        }
    };

    DecodedImage::new(
        u32::try_from(selection.width).map_err(|_| eyre!("EXR width exceeds u32"))?,
        u32::try_from(selection.height).map_err(|_| eyre!("EXR height exceeds u32"))?,
        selection.channels() as i32,
        PixelType::F32,
        DecodedPixels::F32(pixels),
    )
}

fn select_exr_layout(meta: &MetaData) -> Result<ExrSelection> {
    let has_channel = |header: &exr::meta::header::Header, name: &'static str| {
        header.channels.find_index_of_channel(&Text::from(name)).is_some()
    };

    // Preserve image-rs's existing preference for the first usable RGB layer.
    if let Some(header) = meta
        .headers
        .iter()
        .find(|header| !header.deep && ["R", "G", "B"].into_iter().all(|name| has_channel(header, name)))
    {
        let layout = if has_channel(header, "A") {
            ExrLayout::Rgba
        } else {
            ExrLayout::Rgb
        };
        return exr_selection(header, layout);
    }

    // A float mono image has no representation in image-rs DynamicImage.
    // Custom EXR passes frequently use semantic names such as `depth`,
    // `mask`, or `normal.x`. When a non-deep layer has exactly one channel,
    // its name is unambiguous and it can use the same direct mono path without
    // expanding the image to RGB.
    if let Some(header) = meta
        .headers
        .iter()
        .find(|header| !header.deep && header.channels.list.len() == 1)
    {
        let channel_name = header.channels.list[0].name.clone();
        return exr_selection(header, ExrLayout::Mono(channel_name));
    }

    Err(eyre!("EXR has no non-deep RGB/RGBA layer or exactly one-channel mono layer"))
}

fn exr_selection(header: &exr::meta::header::Header, layout: ExrLayout) -> Result<ExrSelection> {
    let display_window = header.shared_attributes.display_window;
    let data_offset = header.own_attributes.layer_position - display_window.position;
    let selection = ExrSelection {
        layout,
        width: display_window.size.width(),
        height: display_window.size.height(),
        data_offset_x: data_offset.x(),
        data_offset_y: data_offset.y(),
    };
    if selection.width == 0 || selection.height == 0 {
        return Err(eyre!("EXR has invalid zero-sized display window"));
    }
    selection.value_count()?;
    Ok(selection)
}

fn decode_jpeg2000_reader<R: Read + Seek>(mut reader: R) -> Result<DecodedImage> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    let image = hayro_jpeg2000::Image::new(&bytes, &hayro_jpeg2000::DecodeSettings::default())?;

    // Hayro's image-rs hook always calls `store_u8_into`, even for a 16-bit
    // codestream. Read simple display color spaces from the decoded component
    // planes instead so EdolView never performs the lossy U16 -> U8 -> F32
    // round trip. CMYK and ICC images retain the hook's color-management path.
    if !matches!(
        image.color_space(),
        hayro_jpeg2000::ColorSpace::Gray | hayro_jpeg2000::ColorSpace::RGB
    ) {
        let dynamic = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?.decode()?;
        return decoded_dynamic_image(dynamic);
    }

    let width = image.width();
    let height = image.height();
    let channels = image.color_space().num_channels() as usize + usize::from(image.has_alpha());
    let pixel_count = width as usize * height as usize;
    pixel_count
        .checked_mul(channels)
        .ok_or_else(|| eyre!("JPEG 2000 image size overflow"))?;

    let mut context = hayro_jpeg2000::DecoderContext::default();
    let decoded = image.decode(&mut context)?;
    let components = decoded.components();
    if components.len() != channels || components.iter().any(|component| component.samples().len() != pixel_count) {
        return Err(eyre!(
            "Unexpected JPEG 2000 component layout: {} components for {channels} channels",
            components.len()
        ));
    }

    let max_depth = components.iter().map(|component| component.bit_depth()).max().unwrap_or(8);
    let uniform_depth = components.iter().all(|component| component.bit_depth() == max_depth);
    let pixel_type = match max_depth {
        0..=8 => PixelType::U8,
        9..=16 => PixelType::U16,
        _ => PixelType::F32,
    };
    if uniform_depth && max_depth <= 16 {
        let pixels = if max_depth <= 8 {
            DecodedPixels::U8(
                components
                    .iter()
                    .flat_map(|component| {
                        component
                            .samples()
                            .iter()
                            .map(|value| value.round().clamp(0.0, u8::MAX as f32) as u8)
                    })
                    .collect(),
            )
        } else {
            DecodedPixels::U16(
                components
                    .iter()
                    .flat_map(|component| {
                        component
                            .samples()
                            .iter()
                            .map(|value| value.round().clamp(0.0, u16::MAX as f32) as u16)
                    })
                    .collect(),
            )
        };
        let sample_bytes = pixels.sample_bytes();
        return DecodedImage::new_with_layout(
            width,
            height,
            channels as i32,
            pixel_type,
            pixels,
            DecodedLayout {
                row_stride_bytes: width as usize * sample_bytes,
                plane_stride_bytes: pixel_count * sample_bytes,
                planes: channels as u32,
                bit_depth: max_depth,
                input_channels: channels,
            },
            DecodedColor::Direct,
        );
    }

    let mut pixels = Vec::with_capacity(pixel_count * channels);
    for pixel in 0..pixel_count {
        for component in components {
            let bit_depth = component.bit_depth();
            let max_value = if bit_depth >= 32 {
                u32::MAX as f32
            } else {
                ((1_u32 << bit_depth) - 1) as f32
            };
            pixels.push(component.samples()[pixel] / max_value);
        }
    }

    DecodedImage::new(width, height, channels as i32, pixel_type, DecodedPixels::F32(pixels))
}

#[derive(Clone, Copy)]
enum TiffColorLayout {
    Direct { channels: usize },
    Palette,
    Cmyk { alpha: bool },
    YCbCr,
    Lab,
}

impl TiffColorLayout {
    fn input_channels(self) -> usize {
        match self {
            Self::Direct { channels } => channels,
            Self::Palette => 1,
            Self::Cmyk { alpha: false } => 4,
            Self::Cmyk { alpha: true } => 5,
            Self::YCbCr | Self::Lab => 3,
        }
    }

    fn output_channels(self) -> usize {
        match self {
            Self::Direct { channels } => channels,
            Self::Palette | Self::Cmyk { alpha: false } | Self::YCbCr | Self::Lab => 3,
            Self::Cmyk { alpha: true } => 4,
        }
    }
}

fn decode_tiff_reader<R: Read + Seek>(mut reader: R) -> Result<DecodedImage> {
    // tiff 0.11 rejects palette photometric images while constructing the
    // decoder, before callers can retrieve ColorMap. Inspect that small piece
    // of metadata first and only copy/patch the compressed stream for this
    // otherwise unsupported layout.
    let prepatch = inspect_tiff_prepatch(&mut reader)?;
    reader.seek(SeekFrom::Start(0))?;
    if let Some(prepatch) = prepatch {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        let replacement = match prepatch {
            TiffPrepatch::Palette(_) => 1, // BlackIsZero
            TiffPrepatch::Lab { .. } => 2, // RGB
        };
        patch_tiff_photometric(&mut bytes, replacement)?;
        let mut decoder = TiffDecoder::new(Cursor::new(bytes))?;
        let (width, height) = decoder.dimensions()?;
        let (color_type, color_layout, palette) = match prepatch {
            TiffPrepatch::Palette(palette) => (
                TiffColorType::Palette(palette.bit_depth),
                TiffColorLayout::Palette,
                Some(palette.colors),
            ),
            TiffPrepatch::Lab { bit_depth } => (TiffColorType::Lab(bit_depth), TiffColorLayout::Lab, None),
        };
        return decode_tiff_image(decoder, width, height, color_type, color_layout, palette);
    }

    let mut decoder = TiffDecoder::new(reader)?;
    let (width, height) = decoder.dimensions()?;
    let color_type = decoder.colortype()?;
    let color_layout = tiff_color_layout(color_type)?;

    decode_tiff_image(decoder, width, height, color_type, color_layout, None)
}

fn tiff_color_layout(color_type: TiffColorType) -> Result<TiffColorLayout> {
    let layout = match color_type {
        TiffColorType::Gray(_) => TiffColorLayout::Direct { channels: 1 },
        TiffColorType::GrayA(_) => TiffColorLayout::Direct { channels: 2 },
        TiffColorType::RGB(_) => TiffColorLayout::Direct { channels: 3 },
        TiffColorType::RGBA(_) => TiffColorLayout::Direct { channels: 4 },
        TiffColorType::Palette(_) => TiffColorLayout::Palette,
        TiffColorType::CMYK(_) => TiffColorLayout::Cmyk { alpha: false },
        TiffColorType::CMYKA(_) => TiffColorLayout::Cmyk { alpha: true },
        TiffColorType::YCbCr(_) => TiffColorLayout::YCbCr,
        TiffColorType::Lab(_) => TiffColorLayout::Lab,
        TiffColorType::Multiband { num_samples, .. } if num_samples <= 4 => TiffColorLayout::Direct {
            channels: num_samples as usize,
        },
        TiffColorType::Multiband { num_samples, .. } => {
            return Err(eyre!("TIFF has {num_samples} bands; EdolView supports at most four"));
        }
        _ => return Err(eyre!("Unsupported TIFF color layout: {color_type:?}")),
    };
    Ok(layout)
}

#[derive(Clone, Copy)]
enum TiffByteOrder {
    Little,
    Big,
}

impl TiffByteOrder {
    fn u16(self, bytes: &[u8]) -> u16 {
        let bytes = bytes.try_into().expect("two TIFF bytes");
        match self {
            Self::Little => u16::from_le_bytes(bytes),
            Self::Big => u16::from_be_bytes(bytes),
        }
    }

    fn u32(self, bytes: &[u8]) -> u32 {
        let bytes = bytes.try_into().expect("four TIFF bytes");
        match self {
            Self::Little => u32::from_le_bytes(bytes),
            Self::Big => u32::from_be_bytes(bytes),
        }
    }

    fn u64(self, bytes: &[u8]) -> u64 {
        let bytes = bytes.try_into().expect("eight TIFF bytes");
        match self {
            Self::Little => u64::from_le_bytes(bytes),
            Self::Big => u64::from_be_bytes(bytes),
        }
    }
}

struct TiffIfdEntry {
    field_type: u16,
    count: u64,
    value_offset: u64,
    inline: [u8; 8],
    inline_capacity: usize,
}

struct TiffPalette {
    bit_depth: u8,
    colors: Vec<u16>,
}

enum TiffPrepatch {
    Palette(TiffPalette),
    Lab { bit_depth: u8 },
}

fn inspect_tiff_prepatch<R: Read + Seek>(reader: &mut R) -> Result<Option<TiffPrepatch>> {
    reader.seek(SeekFrom::Start(0))?;
    let mut header = [0_u8; 16];
    reader.read_exact(&mut header[..8])?;
    let order = match &header[..2] {
        b"II" => TiffByteOrder::Little,
        b"MM" => TiffByteOrder::Big,
        _ => return Err(eyre!("Invalid TIFF byte order")),
    };
    let big_tiff = match order.u16(&header[2..4]) {
        42 => false,
        43 => true,
        magic => return Err(eyre!("Invalid TIFF magic: {magic}")),
    };
    if big_tiff {
        reader.read_exact(&mut header[8..16])?;
        if order.u16(&header[4..6]) != 8 || order.u16(&header[6..8]) != 0 {
            return Err(eyre!("Unsupported BigTIFF offset layout"));
        }
    }

    let ifd_offset = if big_tiff {
        order.u64(&header[8..16])
    } else {
        order.u32(&header[4..8]) as u64
    };
    reader.seek(SeekFrom::Start(ifd_offset))?;
    let entry_count = if big_tiff {
        let mut count = [0_u8; 8];
        reader.read_exact(&mut count)?;
        order.u64(&count)
    } else {
        let mut count = [0_u8; 2];
        reader.read_exact(&mut count)?;
        order.u16(&count) as u64
    };
    if entry_count > 65_535 {
        return Err(eyre!("TIFF IFD has an unreasonable number of entries: {entry_count}"));
    }

    let mut photometric = None;
    let mut bits = None;
    let mut color_map = None;
    for _ in 0..entry_count {
        let mut raw = [0_u8; 20];
        let entry_len = if big_tiff { 20 } else { 12 };
        reader.read_exact(&mut raw[..entry_len])?;
        let tag = order.u16(&raw[..2]);
        if !matches!(tag, 258 | 262 | 320) {
            continue;
        }
        let field_type = order.u16(&raw[2..4]);
        let (count, value_start, inline_capacity) = if big_tiff {
            (order.u64(&raw[4..12]), 12, 8)
        } else {
            (order.u32(&raw[4..8]) as u64, 8, 4)
        };
        let value_offset = if big_tiff {
            order.u64(&raw[value_start..value_start + 8])
        } else {
            order.u32(&raw[value_start..value_start + 4]) as u64
        };
        let mut inline = [0_u8; 8];
        inline[..inline_capacity].copy_from_slice(&raw[value_start..value_start + inline_capacity]);
        let entry = TiffIfdEntry {
            field_type,
            count,
            value_offset,
            inline,
            inline_capacity,
        };
        match tag {
            258 => bits = Some(entry),
            262 => photometric = Some(entry),
            320 => color_map = Some(entry),
            _ => unreachable!(),
        }
    }

    let Some(photometric) = photometric else {
        return Ok(None);
    };
    let photometric = read_tiff_short_values(reader, order, &photometric, 1)?;
    let photometric = photometric.first().copied();
    if !matches!(photometric, Some(3) | Some(8)) {
        return Ok(None);
    }
    let bits = bits.ok_or_else(|| eyre!("TIFF has no BitsPerSample tag"))?;
    let bit_depth = *read_tiff_short_values(reader, order, &bits, 4)?
        .first()
        .ok_or_else(|| eyre!("TIFF has an empty BitsPerSample tag"))?;
    let bit_depth = u8::try_from(bit_depth).map_err(|_| eyre!("TIFF bit depth is too large"))?;
    if photometric == Some(8) {
        return Ok(Some(TiffPrepatch::Lab { bit_depth }));
    }
    if bit_depth == 0 || bit_depth > 16 {
        return Err(eyre!("Unsupported palette TIFF bit depth: {bit_depth}"));
    }
    let color_map = color_map.ok_or_else(|| eyre!("Palette TIFF has no ColorMap tag"))?;
    let expected_colors = 3_usize
        .checked_mul(1_usize << bit_depth)
        .ok_or_else(|| eyre!("Palette TIFF ColorMap size overflow"))?;
    let colors = read_tiff_short_values(reader, order, &color_map, expected_colors)?;
    if colors.len() < expected_colors {
        return Err(eyre!(
            "Palette TIFF ColorMap is truncated: {} < {expected_colors}",
            colors.len()
        ));
    }
    Ok(Some(TiffPrepatch::Palette(TiffPalette { bit_depth, colors })))
}

fn read_tiff_short_values<R: Read + Seek>(
    reader: &mut R,
    order: TiffByteOrder,
    entry: &TiffIfdEntry,
    maximum: usize,
) -> Result<Vec<u16>> {
    if entry.field_type != 3 {
        return Err(eyre!("Expected TIFF SHORT field, found type {}", entry.field_type));
    }
    let count = usize::try_from(entry.count).map_err(|_| eyre!("TIFF SHORT count overflow"))?;
    if count > maximum {
        return Err(eyre!("TIFF SHORT field is too large: {count} > {maximum}"));
    }
    let byte_count = count.checked_mul(2).ok_or_else(|| eyre!("TIFF SHORT size overflow"))?;
    let mut bytes = vec![0_u8; byte_count];
    if byte_count <= entry.inline_capacity {
        bytes.copy_from_slice(&entry.inline[..byte_count]);
    } else {
        reader.seek(SeekFrom::Start(entry.value_offset))?;
        reader.read_exact(&mut bytes)?;
    }
    Ok(bytes.chunks_exact(2).map(|bytes| order.u16(bytes)).collect())
}

fn decode_tiff_image<R: Read + Seek>(
    mut decoder: TiffDecoder<R>,
    width: u32,
    height: u32,
    color_type: TiffColorType,
    color_layout: TiffColorLayout,
    palette: Option<Vec<u16>>,
) -> Result<DecodedImage> {
    let bit_depth = color_type.bit_depth();

    let pixel_count = width as usize * height as usize;
    let packed_bytes = pixel_count
        .checked_mul(color_layout.input_channels())
        .and_then(|samples| samples.checked_mul(bit_depth as usize))
        .map(|bits| bits.div_ceil(8))
        .ok_or_else(|| eyre!("TIFF image size overflow"))?;
    let mut limits = TiffLimits::default();
    limits.decoding_buffer_size = limits.decoding_buffer_size.max(packed_bytes);
    limits.intermediate_buffer_size = limits.intermediate_buffer_size.max(packed_bytes.min(512 * 1024 * 1024));
    decoder = decoder.with_limits(limits);

    let mut result = DecodingResult::U8(Vec::new());
    let buffer_layout = decoder.read_image_to_buffer(&mut result)?;
    let source_sample_bytes = match &result {
        DecodingResult::U8(_) | DecodingResult::I8(_) => 1,
        DecodingResult::U16(_) | DecodingResult::I16(_) | DecodingResult::F16(_) => 2,
        DecodingResult::U32(_) | DecodingResult::I32(_) | DecodingResult::F32(_) => 4,
        DecodingResult::U64(_) | DecodingResult::I64(_) | DecodingResult::F64(_) => 8,
    };
    let (pixels, mut pixel_type) = decoded_tiff_pixels(result, bit_depth);
    let sample_bytes = pixels.sample_bytes();
    let input_channels = color_layout.input_channels();
    let mut row_stride_bytes = buffer_layout.row_stride.map_or_else(
        || {
            if bit_depth < 8 {
                (width as usize * input_channels * bit_depth as usize).div_ceil(8)
            } else {
                width as usize * input_channels * sample_bytes
            }
        },
        |stride| stride.get(),
    );
    let mut plane_stride_bytes = buffer_layout
        .plane_stride
        .map_or(row_stride_bytes * height as usize, |stride| stride.get());
    if bit_depth >= 8 && source_sample_bytes != sample_bytes {
        row_stride_bytes = row_stride_bytes / source_sample_bytes * sample_bytes;
        plane_stride_bytes = plane_stride_bytes / source_sample_bytes * sample_bytes;
    }
    let color = match color_layout {
        TiffColorLayout::Direct { .. } => DecodedColor::Direct,
        TiffColorLayout::Palette => {
            // The TIFF ColorMap itself is U16 even when its indices are packed
            // U1/U2/U4/U8 samples. Keep the displayed value type consistent
            // with the pre-GPU implementation.
            pixel_type = PixelType::U16;
            DecodedColor::Palette(palette.ok_or_else(|| eyre!("Palette TIFF has no color map"))?)
        }
        TiffColorLayout::Cmyk { alpha: false } => DecodedColor::Cmyk,
        TiffColorLayout::Cmyk { alpha: true } => DecodedColor::Cmyka,
        TiffColorLayout::YCbCr => DecodedColor::YCbCr,
        TiffColorLayout::Lab => {
            // Lab conversion produces nonlinear RGB floats.
            pixel_type = PixelType::F32;
            DecodedColor::Lab
        }
    };
    DecodedImage::new_with_layout(
        width,
        height,
        color_layout.output_channels() as i32,
        pixel_type,
        pixels,
        DecodedLayout {
            row_stride_bytes,
            plane_stride_bytes,
            planes: buffer_layout.planes as u32,
            bit_depth,
            input_channels,
        },
        color,
    )
}

fn decoded_tiff_pixels(result: DecodingResult, bit_depth: u8) -> (DecodedPixels, PixelType) {
    let unsigned_max = if bit_depth >= 64 {
        u64::MAX as f64
    } else {
        ((1_u64 << bit_depth) - 1) as f64
    };
    let signed_max = if bit_depth >= 64 {
        i64::MAX as f64
    } else {
        ((1_i64 << bit_depth.saturating_sub(1)) - 1).max(1) as f64
    };
    match result {
        DecodingResult::U8(values) => (DecodedPixels::U8(values), PixelType::U8),
        DecodingResult::U16(values) => (DecodedPixels::U16(values), PixelType::U16),
        DecodingResult::U32(values) => (DecodedPixels::U32(values), PixelType::F32),
        DecodingResult::U64(values) => (
            DecodedPixels::F32(values.into_iter().map(|value| (value as f64 / unsigned_max) as f32).collect()),
            PixelType::F32,
        ),
        DecodingResult::I8(values) => (DecodedPixels::I8(values), PixelType::I8),
        DecodingResult::I16(values) => (DecodedPixels::I16(values), PixelType::I16),
        DecodingResult::I32(values) => (DecodedPixels::I32(values), PixelType::I32),
        DecodingResult::I64(values) => (
            DecodedPixels::F32(values.into_iter().map(|value| (value as f64 / signed_max) as f32).collect()),
            PixelType::F32,
        ),
        DecodingResult::F16(values) => (
            DecodedPixels::F16(values.into_iter().map(half::f16::to_bits).collect()),
            PixelType::F16,
        ),
        DecodingResult::F32(values) => (DecodedPixels::F32(values), PixelType::F32),
        DecodingResult::F64(values) => (
            DecodedPixels::F32(values.into_iter().map(|value| value as f32).collect()),
            PixelType::F64,
        ),
    }
}

fn patch_tiff_photometric(bytes: &mut [u8], replacement: u16) -> Result<()> {
    if bytes.len() < 8 {
        return Err(eyre!("Palette TIFF header is truncated"));
    }
    let little = match &bytes[..2] {
        b"II" => true,
        b"MM" => false,
        _ => return Err(eyre!("Invalid TIFF byte order")),
    };
    let read_u16 = |slice: &[u8]| {
        if little {
            u16::from_le_bytes(slice.try_into().expect("two TIFF bytes"))
        } else {
            u16::from_be_bytes(slice.try_into().expect("two TIFF bytes"))
        }
    };
    let read_u32 = |slice: &[u8]| {
        if little {
            u32::from_le_bytes(slice.try_into().expect("four TIFF bytes"))
        } else {
            u32::from_be_bytes(slice.try_into().expect("four TIFF bytes"))
        }
    };
    let read_u64 = |slice: &[u8]| {
        if little {
            u64::from_le_bytes(slice.try_into().expect("eight TIFF bytes"))
        } else {
            u64::from_be_bytes(slice.try_into().expect("eight TIFF bytes"))
        }
    };

    let big_tiff = read_u16(&bytes[2..4]) == 43;
    let (entry_count, entries_start, entry_size) = if big_tiff {
        if bytes.len() < 16 {
            return Err(eyre!("BigTIFF header is truncated"));
        }
        let offset = usize::try_from(read_u64(&bytes[8..16])).map_err(|_| eyre!("BigTIFF IFD offset overflow"))?;
        let count_end = offset.checked_add(8).ok_or_else(|| eyre!("BigTIFF IFD overflow"))?;
        let count = usize::try_from(read_u64(
            bytes.get(offset..count_end).ok_or_else(|| eyre!("BigTIFF IFD is truncated"))?,
        ))
        .map_err(|_| eyre!("BigTIFF entry count overflow"))?;
        (count, count_end, 20)
    } else {
        let offset = read_u32(&bytes[4..8]) as usize;
        let count_end = offset.checked_add(2).ok_or_else(|| eyre!("TIFF IFD overflow"))?;
        let count = read_u16(bytes.get(offset..count_end).ok_or_else(|| eyre!("TIFF IFD is truncated"))?) as usize;
        (count, count_end, 12)
    };

    for index in 0..entry_count {
        let start = entries_start
            .checked_add(index.checked_mul(entry_size).ok_or_else(|| eyre!("TIFF entry overflow"))?)
            .ok_or_else(|| eyre!("TIFF entry overflow"))?;
        let end = start.checked_add(entry_size).ok_or_else(|| eyre!("TIFF entry overflow"))?;
        let entry = bytes.get_mut(start..end).ok_or_else(|| eyre!("TIFF entry is truncated"))?;
        if read_u16(&entry[..2]) == 262 {
            let value_offset = if big_tiff { 12 } else { 8 };
            if little {
                entry[value_offset..value_offset + 2].copy_from_slice(&replacement.to_le_bytes());
            } else {
                entry[value_offset..value_offset + 2].copy_from_slice(&replacement.to_be_bytes());
            }
            return Ok(());
        }
    }
    Err(eyre!("TIFF has no PhotometricInterpretation tag"))
}

fn tiff_lab_to_rgb(l: f32, a: f32, b: f32) -> [f32; 3] {
    let l = l * 100.0;
    let a = a * 255.0 - 128.0;
    let b = b * 255.0 - 128.0;
    let fy = (l + 16.0) / 116.0;
    let fx = fy + a / 500.0;
    let fz = fy - b / 200.0;
    let pivot = |value: f32| {
        let cube = value * value * value;
        if cube > 0.008_856 {
            cube
        } else {
            (116.0 * value - 16.0) / 903.3
        }
    };
    let x = 0.950_47 * pivot(fx);
    let y = pivot(fy);
    let z = 1.088_83 * pivot(fz);
    let linear = [
        3.240_454 * x - 1.537_139 * y - 0.498_531 * z,
        -0.969_266 * x + 1.876_011 * y + 0.041_556 * z,
        0.055_643 * x - 0.204_026 * y + 1.057_225 * z,
    ];
    linear.map(|value| {
        let value = value.max(0.0);
        if value <= 0.003_130_8 {
            12.92 * value
        } else {
            1.055 * value.powf(1.0 / 2.4) - 0.055
        }
        .clamp(0.0, 1.0)
    })
}

fn decoded_dynamic_image(image: DynamicImage) -> Result<DecodedImage> {
    match image {
        DynamicImage::ImageLuma8(image) => {
            let (width, height) = image.dimensions();
            DecodedImage::new(width, height, 1, PixelType::U8, DecodedPixels::U8(image.into_raw()))
        }
        DynamicImage::ImageLumaA8(image) => {
            let (width, height) = image.dimensions();
            DecodedImage::new(width, height, 2, PixelType::U8, DecodedPixels::U8(image.into_raw()))
        }
        DynamicImage::ImageRgb8(image) => {
            let (width, height) = image.dimensions();
            DecodedImage::new(width, height, 3, PixelType::U8, DecodedPixels::U8(image.into_raw()))
        }
        DynamicImage::ImageRgba8(image) => {
            let (width, height) = image.dimensions();
            DecodedImage::new(width, height, 4, PixelType::U8, DecodedPixels::U8(image.into_raw()))
        }
        DynamicImage::ImageLuma16(image) => {
            let (width, height) = image.dimensions();
            DecodedImage::new(width, height, 1, PixelType::U16, DecodedPixels::U16(image.into_raw()))
        }
        DynamicImage::ImageLumaA16(image) => {
            let (width, height) = image.dimensions();
            DecodedImage::new(width, height, 2, PixelType::U16, DecodedPixels::U16(image.into_raw()))
        }
        DynamicImage::ImageRgb16(image) => {
            let (width, height) = image.dimensions();
            DecodedImage::new(width, height, 3, PixelType::U16, DecodedPixels::U16(image.into_raw()))
        }
        DynamicImage::ImageRgba16(image) => {
            let (width, height) = image.dimensions();
            DecodedImage::new(width, height, 4, PixelType::U16, DecodedPixels::U16(image.into_raw()))
        }
        DynamicImage::ImageRgb32F(image) => {
            let (width, height) = image.dimensions();
            DecodedImage::new(width, height, 3, PixelType::F32, DecodedPixels::F32(image.into_raw()))
        }
        DynamicImage::ImageRgba32F(image) => {
            let (width, height) = image.dimensions();
            DecodedImage::new(width, height, 4, PixelType::F32, DecodedPixels::F32(image.into_raw()))
        }
        other => {
            let image = other.into_rgba32f();
            let (width, height) = image.dimensions();
            DecodedImage::new(width, height, 4, PixelType::F32, DecodedPixels::F32(image.into_raw()))
        }
    }
}

#[derive(Clone, Copy)]
struct PfmHeader {
    width: u32,
    height: u32,
    channels: i32,
    data_offset: usize,
    row_values: usize,
    value_count: usize,
    byte_count: usize,
    little_endian: bool,
    scale: f32,
}

pub fn decode_pfm(bytes: &[u8]) -> Result<DecodedImage> {
    let header = parse_pfm_header(bytes)?;
    if header.channels == 1 {
        decode_pfm_gray(bytes, header)
    } else {
        decode_pfm_rgb_owned(bytes.to_vec(), header)
    }
}

pub fn decode_pfm_owned(bytes: Vec<u8>) -> Result<DecodedImage> {
    let header = parse_pfm_header(&bytes)?;
    if header.channels == 1 {
        decode_pfm_gray(&bytes, header)
    } else {
        decode_pfm_rgb_owned(bytes, header)
    }
}

fn parse_pfm_header(bytes: &[u8]) -> Result<PfmHeader> {
    let mut offset = 0;
    let magic = next_pfm_token(bytes, &mut offset)?;
    let channels = match magic {
        b"PF" => 3,
        b"Pf" => 1,
        _ => return Err(eyre!("Invalid PFM magic")),
    };
    let width = parse_pfm_token::<u32>(next_pfm_token(bytes, &mut offset)?, "width")?;
    let height = parse_pfm_token::<u32>(next_pfm_token(bytes, &mut offset)?, "height")?;
    let scale = parse_pfm_token::<f32>(next_pfm_token(bytes, &mut offset)?, "scale")?;
    if width == 0 || height == 0 || scale == 0.0 || !scale.is_finite() {
        return Err(eyre!("Invalid PFM dimensions or scale"));
    }

    // The binary payload starts after the line terminator following scale.
    while matches!(bytes.get(offset), Some(b' ' | b'\t')) {
        offset += 1;
    }
    if bytes.get(offset) == Some(&b'\r') {
        offset += 1;
    }
    if bytes.get(offset) == Some(&b'\n') {
        offset += 1;
    } else {
        return Err(eyre!("PFM scale line has no line terminator"));
    }

    let row_values = width as usize * channels as usize;
    let value_count = row_values
        .checked_mul(height as usize)
        .ok_or_else(|| eyre!("PFM image size overflow"))?;
    let byte_count = value_count.checked_mul(4).ok_or_else(|| eyre!("PFM byte size overflow"))?;
    bytes
        .get(offset..offset + byte_count)
        .ok_or_else(|| eyre!("PFM pixel payload is truncated"))?;
    Ok(PfmHeader {
        width,
        height,
        channels,
        data_offset: offset,
        row_values,
        value_count,
        byte_count,
        little_endian: scale.is_sign_negative(),
        scale: scale.abs(),
    })
}

fn decode_pfm_gray(bytes: &[u8], header: PfmHeader) -> Result<DecodedImage> {
    let payload = &bytes[header.data_offset..header.data_offset + header.byte_count];
    let mut pixels = vec![0.0_f32; header.value_count];
    let row_bytes = header.row_values * 4;
    for source_y in 0..header.height as usize {
        let target_y = header.height as usize - source_y - 1;
        let source = &payload[source_y * row_bytes..(source_y + 1) * row_bytes];
        let target = &mut pixels[target_y * header.row_values..(target_y + 1) * header.row_values];
        bytemuck::cast_slice_mut(target).copy_from_slice(source);
    }

    let swap_bytes = cfg!(target_endian = "little") != header.little_endian;
    if swap_bytes {
        for value in &mut pixels {
            *value = f32::from_bits(value.to_bits().swap_bytes()) * header.scale;
        }
    } else if header.scale != 1.0 {
        for value in &mut pixels {
            *value *= header.scale;
        }
    }
    DecodedImage::new(
        header.width,
        header.height,
        header.channels,
        PixelType::F32,
        DecodedPixels::F32(pixels),
    )
}

fn decode_pfm_rgb_owned(bytes: Vec<u8>, header: PfmHeader) -> Result<DecodedImage> {
    DecodedImage::new_with_transform(
        header.width,
        header.height,
        header.channels,
        PixelType::F32,
        DecodedPixels::Raw(bytes),
        DecodedLayout {
            row_stride_bytes: header.row_values * 4,
            plane_stride_bytes: header.byte_count,
            planes: 1,
            bit_depth: 32,
            input_channels: header.channels as usize,
        },
        DecodedColor::Direct,
        DecodedTransform {
            data_offset_bytes: header.data_offset,
            flip_y: true,
            swap_bytes: !header.little_endian,
            scale: header.scale,
        },
    )
}

fn next_pfm_token<'a>(bytes: &'a [u8], offset: &mut usize) -> Result<&'a [u8]> {
    loop {
        while bytes.get(*offset).is_some_and(u8::is_ascii_whitespace) {
            *offset += 1;
        }
        if bytes.get(*offset) != Some(&b'#') {
            break;
        }
        while bytes.get(*offset).is_some_and(|byte| *byte != b'\n') {
            *offset += 1;
        }
    }
    let start = *offset;
    while bytes.get(*offset).is_some_and(|byte| !byte.is_ascii_whitespace()) {
        *offset += 1;
    }
    if start == *offset {
        return Err(eyre!("Unexpected end of PFM header"));
    }
    Ok(&bytes[start..*offset])
}

fn parse_pfm_token<T: std::str::FromStr>(token: &[u8], name: &str) -> Result<T> {
    std::str::from_utf8(token)?.parse().map_err(|_| eyre!("Invalid PFM {name}"))
}

pub fn decode_flo(bytes: &[u8]) -> Result<DecodedImage> {
    decode_flo_owned(bytes.to_vec())
}

pub fn decode_flo_owned(bytes: Vec<u8>) -> Result<DecodedImage> {
    if bytes.len() < 12 {
        return Err(eyre!(".flo: file too small: {} bytes", bytes.len()));
    }
    let magic = f32::from_le_bytes(bytes[0..4].try_into().expect("flo magic"));
    if magic != 202021.25 {
        return Err(eyre!(".flo: invalid magic: {magic}"));
    }
    let width = i32::from_le_bytes(bytes[4..8].try_into().expect("flo width"));
    let height = i32::from_le_bytes(bytes[8..12].try_into().expect("flo height"));
    if width <= 0 || height <= 0 {
        return Err(eyre!(".flo: invalid dimensions: {width}x{height}"));
    }
    let value_count = width as usize * height as usize * 2;
    let data_bytes = value_count.checked_mul(4).ok_or_else(|| eyre!(".flo data size overflow"))?;
    bytes
        .get(12..12 + data_bytes)
        .ok_or_else(|| eyre!(".flo pixel payload is truncated"))?;
    DecodedImage::new_with_transform(
        width as u32,
        height as u32,
        2,
        PixelType::F32,
        DecodedPixels::Raw(bytes),
        DecodedLayout {
            row_stride_bytes: width as usize * 2 * 4,
            plane_stride_bytes: data_bytes,
            planes: 1,
            bit_depth: 32,
            input_channels: 2,
        },
        DecodedColor::Direct,
        DecodedTransform {
            data_offset_bytes: 12,
            ..DecodedTransform::default()
        },
    )
}

#[cfg(feature = "heif")]
pub fn decode_heif(path: &Path) -> Result<DecodedImage> {
    use std::ffi::{c_void, CStr};

    fn heif_error(error: libheif_sys::heif_error, context: &str) -> color_eyre::Report {
        let message = if error.message.is_null() {
            "unknown libheif error".into()
        } else {
            // SAFETY: libheif returns a valid NUL-terminated message for the
            // lifetime of the error value.
            unsafe { CStr::from_ptr(error.message) }.to_string_lossy()
        };
        eyre!("{context}: {message}")
    }

    let bytes = std::fs::read(path).map_err(|error| eyre!("Failed to read HEIF bytes: {error}"))?;

    // SAFETY: Every libheif object allocated below is released on each exit
    // path, and `bytes` remains alive while the context reads and decodes it.
    unsafe {
        libheif_sys::heif_init(std::ptr::null_mut());
        let context = libheif_sys::heif_context_alloc();
        if context.is_null() {
            return Err(eyre!("Failed to allocate HEIF context"));
        }
        let error = libheif_sys::heif_context_read_from_memory(
            context,
            bytes.as_ptr() as *mut c_void,
            bytes.len(),
            std::ptr::null(),
        );
        if error.code != libheif_sys::heif_error_code_heif_error_Ok {
            libheif_sys::heif_context_free(context);
            return Err(heif_error(error, "Failed to read HEIF image"));
        }

        let mut handle = std::ptr::null_mut();
        let error = libheif_sys::heif_context_get_primary_image_handle(context, &mut handle);
        if error.code != libheif_sys::heif_error_code_heif_error_Ok {
            libheif_sys::heif_context_free(context);
            return Err(heif_error(error, "Failed to get HEIF image handle"));
        }
        let width = libheif_sys::heif_image_handle_get_width(handle);
        let height = libheif_sys::heif_image_handle_get_height(handle);
        if width <= 0 || height <= 0 {
            libheif_sys::heif_image_handle_release(handle);
            libheif_sys::heif_context_free(context);
            return Err(eyre!("Invalid HEIF dimensions: {width}x{height}"));
        }
        let channels = if libheif_sys::heif_image_handle_has_alpha_channel(handle) != 0 {
            4
        } else {
            3
        };
        let chroma = if channels == 4 {
            libheif_sys::heif_chroma_heif_chroma_interleaved_RGBA
        } else {
            libheif_sys::heif_chroma_heif_chroma_interleaved_RGB
        };

        let mut image = std::ptr::null_mut();
        let options = libheif_sys::heif_decoding_options_alloc();
        let error = libheif_sys::heif_decode_image(
            handle,
            &mut image,
            libheif_sys::heif_colorspace_heif_colorspace_RGB,
            chroma,
            options,
        );
        libheif_sys::heif_decoding_options_free(options);
        if error.code != libheif_sys::heif_error_code_heif_error_Ok {
            libheif_sys::heif_image_handle_release(handle);
            libheif_sys::heif_context_free(context);
            return Err(heif_error(error, "Failed to decode HEIF image"));
        }

        let mut stride = 0;
        let source = libheif_sys::heif_image_get_plane_readonly(
            image,
            libheif_sys::heif_channel_heif_channel_interleaved,
            &mut stride,
        );
        let row_bytes = width as usize * channels as usize;
        if source.is_null() || stride < 0 || (stride as usize) < row_bytes {
            libheif_sys::heif_image_release(image);
            libheif_sys::heif_image_handle_release(handle);
            libheif_sys::heif_context_free(context);
            return Err(eyre!("Invalid HEIF image plane or stride"));
        }

        let mut pixels = Vec::with_capacity(row_bytes * height as usize);
        for y in 0..height as usize {
            // SAFETY: stride and visible row length were validated above and
            // libheif owns this plane until `heif_image_release` below.
            let row = std::slice::from_raw_parts(source.add(y * stride as usize), row_bytes);
            pixels.extend_from_slice(row);
        }
        libheif_sys::heif_image_release(image);
        libheif_sys::heif_image_handle_release(handle);
        libheif_sys::heif_context_free(context);
        DecodedImage::new(width as u32, height as u32, channels, PixelType::U8, DecodedPixels::U8(pixels))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exr::prelude::{Image as ExrImage, SpecificChannels, WritableImage};
    use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};

    fn normalized_values(image: &DecodedImage) -> Vec<f32> {
        (0..image.width as usize * image.height as usize)
            .flat_map(|index| {
                let (values, channels) = image
                    .normalized_pixel(index % image.width as usize, index / image.width as usize)
                    .unwrap();
                values[..channels].to_vec()
            })
            .collect()
    }

    fn encode_mono_exr(channel_name: &'static str) -> Vec<u8> {
        let channels = SpecificChannels::build()
            .with_channel::<f32>(channel_name)
            .with_pixel_fn(|position| ([0.25_f32, 0.75][position.x()],));
        let image = ExrImage::from_channels((2, 1), channels);
        let mut bytes = Cursor::new(Vec::new());
        image.write().to_buffered(&mut bytes).unwrap();
        bytes.into_inner()
    }

    fn encode_rgb_exr(with_alpha: bool) -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());
        if with_alpha {
            let channels = SpecificChannels::rgba(|position: Vec2<usize>| {
                (position.x() as f32, position.y() as f32, 0.5_f32, 0.25_f32)
            });
            ExrImage::from_channels((2, 2), channels)
                .write()
                .to_buffered(&mut bytes)
                .unwrap();
        } else {
            let channels =
                SpecificChannels::rgb(|position: Vec2<usize>| (position.x() as f32, position.y() as f32, 0.5_f32));
            ExrImage::from_channels((2, 2), channels)
                .write()
                .to_buffered(&mut bytes)
                .unwrap();
        }
        bytes.into_inner()
    }

    #[test]
    fn decodes_little_endian_pfm_and_flips_rows() {
        let mut bytes = b"Pf\n2 2\n-2.0  \n".to_vec();
        for value in [3.0_f32, 4.0, 1.0, 2.0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let image = decode_pfm(&bytes).unwrap();
        assert_eq!((image.width, image.height, image.channels), (2, 2, 1));
        assert_eq!(normalized_values(&image), vec![2.0, 4.0, 6.0, 8.0]);
    }

    #[test]
    fn decodes_middlebury_flo() {
        let mut bytes = 202021.25_f32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&2_i32.to_le_bytes());
        bytes.extend_from_slice(&1_i32.to_le_bytes());
        for value in [1.0_f32, -1.0, 2.0, -2.0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let image = decode_flo(&bytes).unwrap();
        assert_eq!((image.width, image.height, image.channels), (2, 1, 2));
        assert_eq!(normalized_values(&image), vec![1.0, -1.0, 2.0, -2.0]);
    }

    #[test]
    fn decodes_rgb_and_rgba_exr_through_the_dedicated_path() {
        for (with_alpha, expected_channels) in [(false, 3), (true, 4)] {
            let decoded = decode_bytes(&encode_rgb_exr(with_alpha)).unwrap();
            assert_eq!((decoded.width, decoded.height), (2, 2));
            assert_eq!(decoded.channels, expected_channels);
            assert_eq!(decoded.pixel_type, PixelType::F32);
            assert_eq!(decoded.pixels.len(), 2 * 2 * expected_channels as usize);
        }
    }

    #[test]
    fn decodes_named_and_custom_mono_exr_as_one_float_channel() {
        for channel_name in ["R", "L", "Y", "Z", "depth", "custom.mask"] {
            let decoded = decode_bytes(&encode_mono_exr(channel_name)).unwrap();
            assert_eq!((decoded.width, decoded.height, decoded.channels), (2, 1, 1), "{channel_name}");
            assert_eq!(decoded.pixel_type, PixelType::F32, "{channel_name}");
            assert_eq!(normalized_values(&decoded), vec![0.25, 0.75], "{channel_name}");
        }
    }

    #[test]
    fn dedicated_tiff_path_decodes_signed_and_float_gray_samples() {
        use tiff::encoder::{colortype, TiffEncoder};

        let mut signed = Cursor::new(Vec::new());
        TiffEncoder::new(&mut signed)
            .unwrap()
            .write_image::<colortype::GrayI32>(3, 1, &[-i32::MAX, 0, i32::MAX])
            .unwrap();
        let decoded = decode_bytes(signed.get_ref()).unwrap();
        assert_eq!((decoded.width, decoded.height, decoded.channels), (3, 1, 1));
        assert_eq!(decoded.pixel_type, PixelType::I32);
        assert_eq!(normalized_values(&decoded), vec![-1.0, 0.0, 1.0]);

        let mut float = Cursor::new(Vec::new());
        TiffEncoder::new(&mut float)
            .unwrap()
            .write_image::<colortype::Gray32Float>(3, 1, &[-0.25, 0.5, 2.0])
            .unwrap();
        let decoded = decode_bytes(float.get_ref()).unwrap();
        assert_eq!((decoded.width, decoded.height, decoded.channels), (3, 1, 1));
        assert_eq!(decoded.pixel_type, PixelType::F32);
        assert_eq!(normalized_values(&decoded), vec![-0.25, 0.5, 2.0]);
    }

    #[test]
    fn image_rs_round_trips_enabled_ldr_formats() {
        let source = DynamicImage::ImageRgb8(ImageBuffer::from_fn(3, 2, |x, y| {
            Rgb([(x * 70) as u8, (y * 110) as u8, ((x + y) * 40) as u8])
        }));
        for format in [
            ImageFormat::Png,
            ImageFormat::Jpeg,
            ImageFormat::Bmp,
            ImageFormat::Tiff,
            ImageFormat::WebP,
            ImageFormat::Pnm,
            ImageFormat::Gif,
            ImageFormat::Tga,
            ImageFormat::Qoi,
        ] {
            let mut encoded = Cursor::new(Vec::new());
            source.write_to(&mut encoded, format).unwrap();
            let decoded = decode_bytes(encoded.get_ref()).unwrap();
            assert_eq!((decoded.width, decoded.height), (3, 2), "{format:?}");
            assert!(matches!(decoded.channels, 3 | 4), "{format:?}");
        }

        let rgba = DynamicImage::ImageRgba8(source.to_rgba8());
        let mut encoded = Cursor::new(Vec::new());
        rgba.write_to(&mut encoded, ImageFormat::Ico).unwrap();
        let decoded = decode_bytes(encoded.get_ref()).unwrap();
        assert_eq!((decoded.width, decoded.height, decoded.channels), (3, 2, 4));

        let rgba16 = DynamicImage::ImageRgba16(image::ImageBuffer::from_fn(3, 2, |x, y| {
            image::Rgba([(x * 10_000) as u16, (y * 20_000) as u16, 30_000, u16::MAX])
        }));
        let mut encoded = Cursor::new(Vec::new());
        rgba16.write_to(&mut encoded, ImageFormat::Farbfeld).unwrap();
        let decoded = decode_bytes(encoded.get_ref()).unwrap();
        assert_eq!((decoded.width, decoded.height, decoded.channels), (3, 2, 4));
        assert_eq!(decoded.pixel_type, PixelType::U16);
    }
}
