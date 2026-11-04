use color_eyre::eyre::{eyre, Result};

use super::image_io::DecodedImage;

#[cfg(not(windows))]
pub(super) fn read_image() -> Result<DecodedImage> {
    use super::{image_io::DecodedPixels, PixelType};

    let mut clipboard = arboard::Clipboard::new().map_err(|error| eyre!("Failed to open clipboard: {error}"))?;
    let image = clipboard
        .get_image()
        .map_err(|error| eyre!("Failed to read clipboard image: {error}"))?;
    let width = u32::try_from(image.width)?;
    let height = u32::try_from(image.height)?;
    DecodedImage::new(width, height, 4, PixelType::U8, DecodedPixels::U8(image.bytes.into_owned()))
}

#[cfg(windows)]
pub(super) fn read_image() -> Result<DecodedImage> {
    use clipboard_win::{formats, is_format_avail, raw, register_format};

    let mut failures = Vec::new();
    // Applications and remote clipboard tools use both names for encoded PNG.
    // Decode after releasing the clipboard so large images never hold it open.
    for name in ["PNG", "image/png"] {
        let Some(format) = register_format(name).filter(|format| is_format_avail(format.get())) else {
            continue;
        };
        let result = (|| {
            let bytes = {
                let _clipboard = open_windows_clipboard()?;
                let mut bytes = Vec::new();
                raw::get_vec(format.get(), &mut bytes).map_err(|error| eyre!("Clipboard read failed: {error}"))?;
                bytes
            };
            super::image_io::decode_bytes(&bytes)
        })();
        match result {
            Ok(image) => return Ok(image),
            Err(error) => failures.push(format!("{name}: {error}")),
        }
    }
    for (format, name) in [(formats::CF_DIBV5, "CF_DIBV5"), (formats::CF_DIB, "CF_DIB")] {
        if !is_format_avail(format) {
            continue;
        }
        let result = (|| {
            let bytes = {
                let _clipboard = open_windows_clipboard()?;
                // get_vec appends the clipboard bytes. Reserve the BMP file
                // header up front to avoid copying or shifting the full DIB.
                let mut bytes = vec![0; BMP_FILE_HEADER_SIZE];
                raw::get_vec(format, &mut bytes).map_err(|error| eyre!("Clipboard read failed: {error}"))?;
                bytes
            };
            decode_windows_dib(bytes)
        })();
        match result {
            Ok(image) => return Ok(image),
            Err(error) => failures.push(format!("{name}: {error}")),
        }
    }
    if failures.is_empty() {
        Err(eyre!("Clipboard does not contain supported image data"))
    } else {
        Err(eyre!("Failed to decode clipboard image: {}", failures.join("; ")))
    }
}

#[cfg(windows)]
fn open_windows_clipboard() -> Result<clipboard_win::Clipboard> {
    // Clipboard owners can temporarily keep it locked during a copy operation.
    for attempt in 0..=5 {
        match clipboard_win::Clipboard::new() {
            Ok(clipboard) => return Ok(clipboard),
            Err(error) if attempt == 5 => return Err(eyre!("Failed to open clipboard: {error}")),
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(5)),
        }
    }
    unreachable!()
}

#[cfg(any(windows, test))]
const BMP_FILE_HEADER_SIZE: usize = 14;

/// `bytes` starts with a reserved BMP file header, followed by a packed DIB.
/// Supply an explicit pixel offset: image 0.25.10's headerless BMP decoder skips
/// an extra 12 bytes for V4/V5 BI_BITFIELDS even though their masks are in-header.
#[cfg(any(windows, test))]
fn decode_windows_dib(mut bytes: Vec<u8>) -> Result<DecodedImage> {
    let dib = bytes
        .get(BMP_FILE_HEADER_SIZE..)
        .ok_or_else(|| eyre!("Missing clipboard DIB"))?;
    let u16_at = |offset| -> Result<u16> {
        let data = dib
            .get(offset..offset + 2)
            .ok_or_else(|| eyre!("Truncated clipboard DIB header"))?;
        Ok(u16::from_le_bytes(data.try_into()?))
    };
    let u32_at = |offset| -> Result<u32> {
        let data = dib
            .get(offset..offset + 4)
            .ok_or_else(|| eyre!("Truncated clipboard DIB header"))?;
        Ok(u32::from_le_bytes(data.try_into()?))
    };
    let header_size = usize::try_from(u32_at(0)?)?;
    let mut preserve_alpha = false;
    let mut default_rgb_masks = false;
    let (colors, entry_size, masks) = match header_size {
        12 => {
            let bits = u16_at(10)?;
            (if bits <= 8 { 1usize << bits } else { 0 }, 3, 0)
        }
        40 | 52 | 56 | 108 | 124 => {
            let bits = u16_at(14)?;
            let compression = u32_at(16)?;
            // Match arboard's handling of browsers that label meaningful
            // 32-bit alpha as BI_RGB rather than BI_BITFIELDS.
            preserve_alpha = header_size >= 56 && bits == 32 && compression == 0 && u32_at(52)? == 0xff000000;
            default_rgb_masks = preserve_alpha && u32_at(40)? == 0 && u32_at(44)? == 0 && u32_at(48)? == 0;
            let used = usize::try_from(u32_at(32)?)?;
            let colors = if used == 0 && bits <= 8 { 1usize << bits } else { used };
            let masks = if header_size == 40 {
                match compression {
                    3 => 12,
                    6 => 16,
                    _ => 0,
                }
            } else {
                0
            };
            (colors, 4, masks)
        }
        _ => return Err(eyre!("Unsupported clipboard DIB header size: {header_size}")),
    };
    let pixel_offset = colors
        .checked_mul(entry_size)
        .and_then(|palette| header_size.checked_add(masks)?.checked_add(palette))
        .filter(|offset| *offset <= dib.len())
        .and_then(|offset| offset.checked_add(BMP_FILE_HEADER_SIZE))
        .ok_or_else(|| eyre!("Clipboard DIB metadata extends beyond its buffer"))?;
    let file_size = u32::try_from(bytes.len())?;
    if preserve_alpha {
        bytes[BMP_FILE_HEADER_SIZE + 16..BMP_FILE_HEADER_SIZE + 20].copy_from_slice(&3u32.to_le_bytes());
        if default_rgb_masks {
            for (index, mask) in [0x00ff0000u32, 0x0000ff00, 0x000000ff].into_iter().enumerate() {
                let offset = BMP_FILE_HEADER_SIZE + 40 + index * 4;
                bytes[offset..offset + 4].copy_from_slice(&mask.to_le_bytes());
            }
        }
    }
    bytes[0..2].copy_from_slice(b"BM");
    bytes[2..6].copy_from_slice(&file_size.to_le_bytes());
    bytes[10..14].copy_from_slice(&u32::try_from(pixel_offset)?.to_le_bytes());
    super::image_io::decode_bytes(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dib(header_size: usize, bits: u16, compression: u32, width: i32, height: i32) -> Vec<u8> {
        let mut bytes = vec![0; BMP_FILE_HEADER_SIZE + header_size];
        let header = &mut bytes[BMP_FILE_HEADER_SIZE..];
        header[0..4].copy_from_slice(&(header_size as u32).to_le_bytes());
        header[4..8].copy_from_slice(&width.to_le_bytes());
        header[8..12].copy_from_slice(&height.to_le_bytes());
        header[12..14].copy_from_slice(&1u16.to_le_bytes());
        header[14..16].copy_from_slice(&bits.to_le_bytes());
        header[16..20].copy_from_slice(&compression.to_le_bytes());
        bytes
    }

    #[test]
    fn bitfield_dibs_preserve_first_pixels_row_order_and_alpha() {
        let expected = [255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 64, 255, 255, 255, 0];
        let top_row = [0, 0, 255, 255, 0, 255, 0, 128];
        let bottom_row = [255, 0, 0, 64, 255, 255, 255, 0];
        for header_size in [108, 124] {
            for height in [2, -2] {
                let mut bytes = dib(header_size, 32, 3, 2, height);
                for (index, mask) in [0x00ff0000u32, 0x0000ff00, 0x000000ff, 0xff000000].into_iter().enumerate() {
                    let offset = BMP_FILE_HEADER_SIZE + 40 + index * 4;
                    bytes[offset..offset + 4].copy_from_slice(&mask.to_le_bytes());
                }
                if height > 0 {
                    bytes.extend_from_slice(&bottom_row);
                    bytes.extend_from_slice(&top_row);
                } else {
                    bytes.extend_from_slice(&top_row);
                    bytes.extend_from_slice(&bottom_row);
                }
                let image = decode_windows_dib(bytes).unwrap();
                assert_eq!((image.width, image.height, image.channels), (2, 2, 4));
                assert_eq!(image.pixels.bytes(), expected);
            }
        }
    }

    #[test]
    fn dib_info_bitfields_and_palette_use_the_correct_pixel_offset() {
        let mut bytes = dib(40, 32, 3, 1, 1);
        for mask in [0x00ff0000u32, 0x0000ff00, 0x000000ff] {
            bytes.extend_from_slice(&mask.to_le_bytes());
        }
        bytes.extend_from_slice(&[7, 8, 9, 0]);
        let image = decode_windows_dib(bytes).unwrap();
        assert_eq!(image.pixels.bytes(), [9, 8, 7]);

        let mut bytes = dib(40, 8, 0, 2, 1);
        bytes[BMP_FILE_HEADER_SIZE + 32..BMP_FILE_HEADER_SIZE + 36].copy_from_slice(&2u32.to_le_bytes());
        bytes.extend_from_slice(&[0, 0, 255, 0, 0, 255, 0, 0]);
        bytes.extend_from_slice(&[0, 1, 0, 0]);
        let image = decode_windows_dib(bytes).unwrap();
        assert_eq!(image.pixels.bytes(), [255, 0, 0, 0, 255, 0]);
    }

    #[test]
    fn browser_rgb_dib_preserves_meaningful_alpha() {
        let mut bytes = dib(124, 32, 0, 1, 1);
        bytes[BMP_FILE_HEADER_SIZE + 52..BMP_FILE_HEADER_SIZE + 56].copy_from_slice(&0xff000000u32.to_le_bytes());
        bytes.extend_from_slice(&[0, 0, 255, 128]);
        let image = decode_windows_dib(bytes).unwrap();
        assert_eq!(image.pixels.bytes(), [255, 0, 0, 128]);
    }

    #[test]
    fn malformed_dibs_return_errors() {
        for length in [0, 14, 18, 30] {
            assert!(decode_windows_dib(vec![0; length]).is_err());
        }
        let mut bytes = dib(40, 8, 0, 1, 1);
        bytes[BMP_FILE_HEADER_SIZE + 32..BMP_FILE_HEADER_SIZE + 36].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode_windows_dib(bytes).is_err());
        let mut bytes = dib(124, 32, 3, 1, 1);
        bytes.extend_from_slice(&[0; 3]);
        assert!(decode_windows_dib(bytes).is_err());
    }
}
