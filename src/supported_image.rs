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

pub fn supported_image_formats() -> Vec<SupportedImageFormat> {
    let mut formats = Vec::with_capacity(
        BASE_SUPPORTED_IMAGE_FORMATS.len() + AVIF_SUPPORTED_IMAGE_FORMATS.len() + HEIF_SUPPORTED_IMAGE_FORMATS.len(),
    );
    formats.extend_from_slice(BASE_SUPPORTED_IMAGE_FORMATS);
    formats.extend_from_slice(AVIF_SUPPORTED_IMAGE_FORMATS);
    formats.extend_from_slice(HEIF_SUPPORTED_IMAGE_FORMATS);
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
}

pub fn is_heif_extension(ext: &str) -> bool {
    HEIF_SUPPORTED_IMAGE_FORMATS.iter().any(|format| format.ext == ext)
}
