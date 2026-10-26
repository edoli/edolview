#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SupportedImageFormat {
    pub ext: &'static str,
    pub mime: &'static str,
}

pub const BASE_SUPPORTED_IMAGE_FORMATS: &[SupportedImageFormat] = &[
    SupportedImageFormat {
        ext: "png",
        mime: "image/png",
    },
    SupportedImageFormat {
        ext: "apng",
        mime: "image/apng",
    },
    SupportedImageFormat {
        ext: "jpeg",
        mime: "image/jpeg",
    },
    SupportedImageFormat {
        ext: "jpg",
        mime: "image/jpeg",
    },
    SupportedImageFormat {
        ext: "jpe",
        mime: "image/jpeg",
    },
    SupportedImageFormat {
        ext: "jfif",
        mime: "image/jpeg",
    },
    SupportedImageFormat {
        ext: "jp2",
        mime: "image/jp2",
    },
    SupportedImageFormat {
        ext: "j2k",
        mime: "image/jp2",
    },
    SupportedImageFormat {
        ext: "j2c",
        mime: "image/j2c",
    },
    SupportedImageFormat {
        ext: "jpc",
        mime: "image/j2c",
    },
    SupportedImageFormat {
        ext: "jpf",
        mime: "image/jpx",
    },
    SupportedImageFormat {
        ext: "bmp",
        mime: "image/bmp",
    },
    SupportedImageFormat {
        ext: "dib",
        mime: "image/bmp",
    },
    SupportedImageFormat {
        ext: "dds",
        mime: "image/vnd-ms.dds",
    },
    SupportedImageFormat {
        ext: "exr",
        mime: "image/x-exr",
    },
    SupportedImageFormat {
        ext: "tif",
        mime: "image/tiff",
    },
    SupportedImageFormat {
        ext: "tiff",
        mime: "image/tiff",
    },
    SupportedImageFormat {
        ext: "hdr",
        mime: "image/vnd.radiance",
    },
    SupportedImageFormat {
        ext: "pic",
        mime: "image/x-pictor",
    },
    SupportedImageFormat {
        ext: "webp",
        mime: "image/webp",
    },
    SupportedImageFormat {
        ext: "gif",
        mime: "image/gif",
    },
    SupportedImageFormat {
        ext: "tga",
        mime: "image/x-tga",
    },
    SupportedImageFormat {
        ext: "ico",
        mime: "image/x-icon",
    },
    SupportedImageFormat {
        ext: "ff",
        mime: "image/x-farbfeld",
    },
    SupportedImageFormat {
        ext: "qoi",
        mime: "image/qoi",
    },
    SupportedImageFormat {
        ext: "pfm",
        mime: "image/x-portable-floatmap",
    },
    SupportedImageFormat {
        ext: "pgm",
        mime: "image/x-portable-graymap",
    },
    SupportedImageFormat {
        ext: "ppm",
        mime: "image/x-portable-pixmap",
    },
    SupportedImageFormat {
        ext: "pbm",
        mime: "image/x-portable-bitmap",
    },
    SupportedImageFormat {
        ext: "pxm",
        mime: "image/x-portable-anymap",
    },
    SupportedImageFormat {
        ext: "pnm",
        mime: "image/x-portable-anymap",
    },
    SupportedImageFormat {
        ext: "pam",
        mime: "image/x-portable-arbitrarymap",
    },
    SupportedImageFormat {
        ext: "flo",
        mime: "application/x-middlebury-flow",
    },
];

#[cfg(feature = "avif")]
pub const AVIF_SUPPORTED_IMAGE_FORMATS: &[SupportedImageFormat] = &[SupportedImageFormat {
    ext: "avif",
    mime: "image/avif",
}];

#[cfg(not(feature = "avif"))]
pub const AVIF_SUPPORTED_IMAGE_FORMATS: &[SupportedImageFormat] = &[];

#[cfg(feature = "heif")]
pub const HEIF_SUPPORTED_IMAGE_FORMATS: &[SupportedImageFormat] = &[
    SupportedImageFormat {
        ext: "heic",
        mime: "image/heic",
    },
    SupportedImageFormat {
        ext: "heif",
        mime: "image/heif",
    },
];

#[cfg(not(feature = "heif"))]
pub const HEIF_SUPPORTED_IMAGE_FORMATS: &[SupportedImageFormat] = &[];

// Keep this independent of rawler's camera database. X3F is intentionally
// excluded: rawler 0.8.0 recognizes the container but cannot decode its pixels.
// SR2/SRF use its Sony decoder even though its extension helper omits them.
pub const RAW_SUPPORTED_IMAGE_FORMATS: &[SupportedImageFormat] = &[
    SupportedImageFormat {
        ext: "3fr",
        mime: "image/x-hasselblad-3fr",
    },
    SupportedImageFormat {
        ext: "ari",
        mime: "image/x-raw",
    },
    SupportedImageFormat {
        ext: "arw",
        mime: "image/x-sony-arw",
    },
    SupportedImageFormat {
        ext: "cr2",
        mime: "image/x-canon-cr2",
    },
    SupportedImageFormat {
        ext: "cr3",
        mime: "image/x-canon-cr3",
    },
    SupportedImageFormat {
        ext: "crm",
        mime: "image/x-raw",
    },
    SupportedImageFormat {
        ext: "crw",
        mime: "image/x-canon-crw",
    },
    SupportedImageFormat {
        ext: "dcr",
        mime: "image/x-kodak-dcr",
    },
    SupportedImageFormat {
        ext: "dcs",
        mime: "image/x-kodak-dcs",
    },
    SupportedImageFormat {
        ext: "dng",
        mime: "image/x-adobe-dng",
    },
    SupportedImageFormat {
        ext: "erf",
        mime: "image/x-epson-erf",
    },
    SupportedImageFormat {
        ext: "fff",
        mime: "image/x-hasselblad-fff",
    },
    SupportedImageFormat {
        ext: "iiq",
        mime: "image/x-phaseone-iiq",
    },
    SupportedImageFormat {
        ext: "kdc",
        mime: "image/x-kodak-kdc",
    },
    SupportedImageFormat {
        ext: "mef",
        mime: "image/x-mamiya-mef",
    },
    SupportedImageFormat {
        ext: "mos",
        mime: "image/x-leaf-mos",
    },
    SupportedImageFormat {
        ext: "mrw",
        mime: "image/x-minolta-mrw",
    },
    SupportedImageFormat {
        ext: "nef",
        mime: "image/x-nikon-nef",
    },
    SupportedImageFormat {
        ext: "nrw",
        mime: "image/x-nikon-nrw",
    },
    SupportedImageFormat {
        ext: "orf",
        mime: "image/x-olympus-orf",
    },
    SupportedImageFormat {
        ext: "ori",
        mime: "image/x-olympus-orf",
    },
    SupportedImageFormat {
        ext: "pef",
        mime: "image/x-pentax-pef",
    },
    SupportedImageFormat {
        ext: "qtk",
        mime: "image/x-raw",
    },
    SupportedImageFormat {
        ext: "raf",
        mime: "image/x-fuji-raf",
    },
    SupportedImageFormat {
        ext: "raw",
        mime: "image/x-raw",
    },
    SupportedImageFormat {
        ext: "rw2",
        mime: "image/x-panasonic-rw2",
    },
    SupportedImageFormat {
        ext: "rwl",
        mime: "image/x-leica-rwl",
    },
    SupportedImageFormat {
        ext: "sr2",
        mime: "image/x-sony-sr2",
    },
    SupportedImageFormat {
        ext: "srf",
        mime: "image/x-sony-srf",
    },
    SupportedImageFormat {
        ext: "srw",
        mime: "image/x-samsung-srw",
    },
];

pub fn is_raw_path(path: &std::path::Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
        RAW_SUPPORTED_IMAGE_FORMATS
            .iter()
            .any(|format| ext.eq_ignore_ascii_case(format.ext))
    })
}

pub fn supported_image_formats() -> Vec<SupportedImageFormat> {
    let mut formats = Vec::with_capacity(
        BASE_SUPPORTED_IMAGE_FORMATS.len()
            + AVIF_SUPPORTED_IMAGE_FORMATS.len()
            + HEIF_SUPPORTED_IMAGE_FORMATS.len()
            + RAW_SUPPORTED_IMAGE_FORMATS.len(),
    );
    formats.extend_from_slice(BASE_SUPPORTED_IMAGE_FORMATS);
    formats.extend_from_slice(AVIF_SUPPORTED_IMAGE_FORMATS);
    formats.extend_from_slice(HEIF_SUPPORTED_IMAGE_FORMATS);
    formats.extend_from_slice(RAW_SUPPORTED_IMAGE_FORMATS);
    formats
}

pub fn supported_image_extensions() -> Vec<&'static str> {
    supported_image_formats().into_iter().map(|format| format.ext).collect()
}

pub fn supported_image_mime_types() -> Vec<&'static str> {
    let mut mime_types = Vec::new();
    for format in supported_image_formats() {
        if !mime_types.contains(&format.mime) {
            mime_types.push(format.mime);
        }
    }
    mime_types
}

pub fn is_supported_image_extension(ext: &str) -> bool {
    BASE_SUPPORTED_IMAGE_FORMATS.iter().any(|format| format.ext == ext)
        || AVIF_SUPPORTED_IMAGE_FORMATS.iter().any(|format| format.ext == ext)
        || HEIF_SUPPORTED_IMAGE_FORMATS.iter().any(|format| format.ext == ext)
        || RAW_SUPPORTED_IMAGE_FORMATS.iter().any(|format| format.ext == ext)
}

pub fn is_heif_extension(ext: &str) -> bool {
    HEIF_SUPPORTED_IMAGE_FORMATS.iter().any(|format| format.ext == ext)
}
