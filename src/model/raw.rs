//! Camera RAW samples in decoder order, without photographic development.
use color_eyre::eyre::{eyre, Result, WrapErr};
use rawler::{decoders::RawDecodeParams, rawimage::RawImageData, rawsource::RawSource};
use std::path::Path;

use super::{
    image_io::{DecodedImage, DecodedPixels},
    PixelType,
};

pub(super) fn decode_path(path: &Path) -> Result<DecodedImage> {
    let source = RawSource::new(path).wrap_err_with(|| format!("Failed to read RAW file '{}'", path.display()))?;
    decode_source(&source).wrap_err_with(|| format!("Failed to decode RAW file '{}'", path.display()))
}

pub(super) fn decode_bytes(bytes: &[u8]) -> Result<DecodedImage> {
    decode_source(&RawSource::new_from_slice(bytes)).wrap_err("Failed to decode RAW image bytes")
}

fn decode_source(source: &RawSource) -> Result<DecodedImage> {
    // rawler's lazy camera database must stay on this decoding path: ordinary
    // startup, PNG/JPEG loading and file-list filtering do not initialize it.
    let raw = rawler::decode(source, &RawDecodeParams::default()).wrap_err("RAW decoding failed")?;
    decoded_samples(raw.width, raw.height, raw.cpp, raw.data)
}

fn decoded_samples(width: usize, height: usize, channels: usize, data: RawImageData) -> Result<DecodedImage> {
    if !matches!(channels, 1 | 3 | 4) {
        return Err(eyre!("Unsupported RAW channel layout: {channels}"));
    }
    // Move the original decoder allocation directly into the image. Keep the
    // full sensor grid, including masked margins, and its original coordinates.
    // Do not rescale black/white levels, demosaic, crop or apply orientation.
    let (pixel_type, pixels) = match data {
        RawImageData::Integer(values) => (PixelType::U16, DecodedPixels::U16(values)),
        RawImageData::Float(values) => (PixelType::F32, DecodedPixels::F32(values)),
    };
    DecodedImage::new(
        u32::try_from(width).wrap_err("RAW width exceeds u32")?,
        u32::try_from(height).wrap_err("RAW height exceeds u32")?,
        channels as i32,
        pixel_type,
        pixels,
    )
}

/// RAW signatures that can be recognized without initializing the camera DB.
/// TIFF-based camera formats without a distinguishing signature use their file
/// extension; DNG is identified by its root IFD's mandatory DNGVersion tag.
pub(super) fn has_signature(bytes: &[u8]) -> bool {
    bytes.starts_with(b"FUJIFILMCCD-RAW")
        || bytes.starts_with(b"\0MRM")
        || bytes.starts_with(b"IIRO")
        || bytes.starts_with(b"IIRS")
        || bytes.starts_with(b"IIU\0")
        || bytes.starts_with(b"II*\0\x10\0\0\0CR\x02\0")
        || bytes.get(0..14) == Some(b"II\x1a\0\0\0HEAPCCDR")
        || bytes.get(4..12) == Some(b"ftypcrx ")
        || is_dng(bytes)
}

fn is_dng(bytes: &[u8]) -> bool {
    let little = match bytes.get(..4) {
        Some(b"II*\0") => true,
        Some(b"MM\0*") => false,
        _ => return false,
    };
    let read_u16 = |offset: usize| -> Option<u16> {
        let data = bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?;
        Some(if little {
            u16::from_le_bytes(data)
        } else {
            u16::from_be_bytes(data)
        })
    };
    let Some(offset) = bytes.get(4..8).and_then(|data| <[u8; 4]>::try_from(data).ok()) else {
        return false;
    };
    let offset = if little {
        u32::from_le_bytes(offset)
    } else {
        u32::from_be_bytes(offset)
    } as usize;
    let Some(count) = read_u16(offset) else { return false };
    let Some(start) = offset.checked_add(2) else {
        return false;
    };
    for index in 0..count as usize {
        let Some(entry) = index.checked_mul(12).and_then(|relative| start.checked_add(relative)) else {
            return false;
        };
        match read_u16(entry) {
            Some(50706) => return true,
            Some(_) => {}
            None => return false,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dng_preserves_samples_without_black_level_crop_or_orientation_changes() {
        use std::io::Cursor;
        use tiff::{
            encoder::{colortype::Gray16, TiffEncoder},
            tags::Tag,
        };

        let mut bytes = Cursor::new(Vec::new());
        let values: Vec<u16> = (0..24).flat_map(|_| (0..32).map(|x| (x * 65535 / 31) as u16)).collect();
        {
            let mut encoder = TiffEncoder::new(&mut bytes).unwrap();
            let mut image = encoder.new_image::<Gray16>(32, 24).unwrap();
            image.encoder().write_tag(Tag::Unknown(50706), &[1_u8, 4, 0, 0][..]).unwrap();
            image.encoder().write_tag(Tag::Unknown(50714), 512_u16).unwrap();
            image.encoder().write_tag(Tag::Unknown(50717), 4095_u32).unwrap();
            image.encoder().write_tag(Tag::Unknown(50719), &[2_u32, 4][..]).unwrap();
            image.encoder().write_tag(Tag::Unknown(50720), &[24_u32, 16][..]).unwrap();
            image.encoder().write_tag(Tag::Orientation, 6_u16).unwrap();
            image.encoder().write_tag(Tag::Unknown(33421), &[2_u16, 2][..]).unwrap();
            image.encoder().write_tag(Tag::Unknown(33422), &[0_u8, 1, 1, 2][..]).unwrap();
            image.write_data(&values).unwrap();
        }
        let bytes = bytes.into_inner();
        assert!(has_signature(&bytes));
        let decoded = super::super::image_io::decode_bytes(&bytes).unwrap();
        assert_eq!((decoded.width, decoded.height, decoded.channels), (32, 24, 1));
        assert_eq!(decoded.pixel_type, PixelType::U16);
        let DecodedPixels::U16(samples) = &decoded.pixels else {
            panic!("RAW integer samples must stay u16")
        };
        assert_eq!(samples, &values);
        assert_eq!(decoded.normalized_pixel(0, 0).unwrap().0[0], 0.0);
        assert_eq!(decoded.normalized_pixel(31, 0).unwrap().0[0], 1.0);
    }

    #[test]
    #[ignore = "manual public camera RAW sample preservation matrix; set EDOLVIEW_RAW_SAMPLES"]
    fn public_raw_sample_matrix() {
        let root =
            std::path::PathBuf::from(std::env::var_os("EDOLVIEW_RAW_SAMPLES").expect("set EDOLVIEW_RAW_SAMPLES"));
        let mut count = 0;
        let mut failures = Vec::new();
        for entry in std::fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            if !crate::supported_image::is_raw_path(&path) {
                continue;
            }
            count += 1;
            let start = std::time::Instant::now();
            let result = super::super::image_io::decode_path(&path);
            if path.file_name().unwrap().to_string_lossy().starts_with("pixls_6616_") {
                assert!(result.is_err(), "Nikon HE must return an unsupported-format error");
                eprintln!("RAW_MATRIX {}", serde_json::json!({"path":path,"expected_unsupported":true}));
                continue;
            }
            match result {
                Ok(image) => {
                    let source = RawSource::new(&path).unwrap();
                    let expected = rawler::decode(&source, &RawDecodeParams::default()).unwrap();
                    assert_eq!(
                        (image.width as usize, image.height as usize, image.channels as usize),
                        (expected.width, expected.height, expected.cpp)
                    );
                    assert_eq!(
                        image.pixels.len(),
                        image.width as usize * image.height as usize * image.channels as usize
                    );
                    let expected_bytes: &[u8] = match &expected.data {
                        RawImageData::Integer(values) => {
                            assert_eq!(image.pixel_type, PixelType::U16);
                            bytemuck::cast_slice(values)
                        }
                        RawImageData::Float(values) => {
                            assert_eq!(image.pixel_type, PixelType::F32);
                            bytemuck::cast_slice(values)
                        }
                    };
                    assert_eq!(
                        image.pixels.bytes(),
                        expected_bytes,
                        "{} changed decoder samples",
                        path.display()
                    );
                    eprintln!(
                        "RAW_MATRIX {}",
                        serde_json::json!({"path":path,"width":image.width,"height":image.height,"channels":image.channels,"ms":start.elapsed().as_secs_f64()*1000.0})
                    );
                }
                Err(error) => {
                    eprintln!("RAW_MATRIX {}", serde_json::json!({"path":path,"error":format!("{error:#}")}));
                    failures.push((path, error));
                }
            }
        }
        assert!(count > 0, "No RAW fixtures were tested");
        assert!(failures.is_empty(), "RAW development failures: {failures:?}");
    }

    #[test]
    fn integer_samples_keep_values_and_allocation() {
        let values = vec![0_u16, 512, 1024, 4095, 16383, 65535];
        let pointer = values.as_ptr();
        let decoded = decoded_samples(3, 2, 1, RawImageData::Integer(values.clone())).unwrap();
        let DecodedPixels::U16(samples) = &decoded.pixels else {
            panic!("Expected u16")
        };
        assert_eq!(samples, &values);
        let moved = decoded_samples(3, 2, 1, RawImageData::Integer(values)).unwrap();
        let DecodedPixels::U16(samples) = moved.pixels else {
            panic!("Expected u16")
        };
        assert_eq!(samples.as_ptr(), pointer);
    }

    #[test]
    fn float_and_multichannel_samples_keep_their_exact_bits() {
        let values = vec![-0.25_f32, 0.125, 1.5, f32::from_bits(0x7fc0_1234), 2.0, 0.0];
        let bits: Vec<_> = values.iter().map(|value| value.to_bits()).collect();
        let pointer = values.as_ptr();
        let decoded = decoded_samples(2, 1, 3, RawImageData::Float(values)).unwrap();
        assert_eq!(decoded.channels, 3);
        assert_eq!(decoded.pixel_type, PixelType::F32);
        let samples = decoded.f32_pixels().unwrap();
        assert_eq!(samples.as_ptr(), pointer);
        assert_eq!(samples.iter().map(|value| value.to_bits()).collect::<Vec<_>>(), bits);
    }

    #[test]
    fn signature_detection_distinguishes_dng_from_regular_tiff_and_truncation() {
        for little in [true, false] {
            let mut bytes = if little {
                b"II*\0\x08\0\0\0".to_vec()
            } else {
                b"MM\0*\0\0\0\x08".to_vec()
            };
            bytes.extend(if little { [1, 0] } else { [0, 1] });
            bytes.extend(if little {
                50706_u16.to_le_bytes()
            } else {
                50706_u16.to_be_bytes()
            });
            assert!(has_signature(&bytes));
            bytes.truncate(11);
            assert!(!has_signature(&bytes));
        }
        assert!(!has_signature(b"II*\0\x08\0\0\0\0\0\0\0\0\0"));
        assert!(!has_signature(b"\x89PNG\r\n\x1a\n"));
    }
}
