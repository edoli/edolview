use color_eyre::eyre::{eyre, Result};
use objc2::{
    rc::{autoreleasepool, Retained},
    AnyThread,
};
use objc2_core_graphics::{
    CGBitmapInfo, CGColorRenderingIntent, CGColorSpace, CGDataProvider, CGImage, CGImageAlphaInfo, CGImageByteOrderInfo,
};
use objc2_foundation::{NSArray, NSData, NSDictionary, NSLocale, NSString};
use objc2_vision::{VNImageRequestHandler, VNRecognizeTextRequest, VNRequest, VNRequestTextRecognitionLevel};

use super::{OcrJob, OcrLanguage};

pub(super) struct Backend {
    // Created, configured and reused exclusively on the OCR worker thread.
    request: Retained<VNRecognizeTextRequest>,
}

impl Backend {
    pub(super) fn new() -> Result<Self> {
        autoreleasepool(|_| {
            // Matches the app bundle's minimum macOS version. The instance
            // language-query API is available starting with macOS 11.
            if !objc2::available!(macos = 11.0) {
                return Err(eyre!("Vision OCR requires macOS 11 or later"));
            }
            let request = VNRecognizeTextRequest::new();
            request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
            request.setUsesLanguageCorrection(true);
            Ok(Self { request })
        })
    }

    pub(super) fn available_languages(&self) -> Result<(Vec<OcrLanguage>, String, u32)> {
        autoreleasepool(|_| {
            // SAFETY: the request is configured for Accurate recognition and
            // is accessed only on its owning worker; macOS availability was checked.
            let supported = unsafe { self.request.supportedRecognitionLanguagesAndReturnError() }
                .map_err(|error| eyre!("Could not query Vision OCR languages: {error}"))?;
            let locale = NSLocale::currentLocale();
            let languages: Vec<_> = supported
                .iter()
                .map(|tag| OcrLanguage {
                    tag: tag.to_string(),
                    label: format!(
                        "{} ({tag})",
                        locale.localizedStringForLanguageCode(&tag).unwrap_or_else(|| tag.clone())
                    ),
                })
                .collect();
            if languages.is_empty() {
                return Err(eyre!("Vision reported no supported OCR languages"));
            }
            let preferences: Vec<_> = NSLocale::preferredLanguages().iter().map(|tag| tag.to_string()).collect();
            let language = preferred_language(&languages, &preferences).to_string();
            // Bound GPU readback and recognition memory, as on Windows. This
            // is an application limit, not a maximum reported by Vision.
            Ok((languages, language, 4096))
        })
    }

    pub(super) fn recognize(&mut self, job: OcrJob) -> Result<String> {
        autoreleasepool(|_| {
            let language = NSString::from_str(&job.language);
            self.request.setRecognitionLanguages(&NSArray::from_slice(&[&*language]));

            // Transfer the GPU's owned RGBA allocation to NSData. Its deallocator
            // keeps the pixels alive for every CoreGraphics/Vision retain without
            // another pixel copy, PNG encoding, or a temporary file.
            let data = NSData::from_vec(job.rgba);
            let provider = CGDataProvider::with_cf_data(Some((*data).as_ref()))
                .ok_or_else(|| eyre!("Could not create OCR image data provider"))?;
            let color_space =
                CGColorSpace::new_device_rgb().ok_or_else(|| eyre!("Could not create OCR image color space"))?;
            // SAFETY: the worker validated tightly packed RGBA8 dimensions before
            // dispatch. The provider owns immutable pixel storage; decode is null.
            let image = unsafe {
                CGImage::new(
                    job.width as usize,
                    job.height as usize,
                    8,
                    32,
                    job.width as usize * 4,
                    Some(&color_space),
                    CGBitmapInfo(CGImageAlphaInfo::Last.0 | CGImageByteOrderInfo::Order32Big.0),
                    Some(&provider),
                    std::ptr::null(),
                    false,
                    CGColorRenderingIntent::RenderingIntentDefault,
                )
            }
            .ok_or_else(|| eyre!("Could not create OCR image"))?;
            // SAFETY: the options dictionary is empty and the retained image is
            // immutable for the handler's entire lifetime. No EXIF transform is
            // needed: GPU exports are already in displayed, top-to-bottom order.
            let handler = unsafe {
                VNImageRequestHandler::initWithCGImage_options(
                    VNImageRequestHandler::alloc(),
                    &image,
                    &NSDictionary::new(),
                )
            };
            let request: &VNRequest = self.request.as_ref();
            handler
                .performRequests_error(&NSArray::from_slice(&[request]))
                .map_err(|error| eyre!("Vision text recognition failed: {error}"))?;
            let observations = self.request.results().ok_or_else(|| eyre!("Vision returned no OCR results"))?;
            let lines: Vec<_> = observations
                .iter()
                .filter_map(|observation| observation.topCandidates(1).firstObject())
                .map(|candidate| candidate.string().to_string())
                .collect();
            Ok(lines.join("\n"))
        })
    }
}

fn preferred_language<'a>(languages: &'a [OcrLanguage], preferences: &[String]) -> &'a str {
    for preference in preferences {
        let mut prefix = preference.as_str();
        loop {
            if let Some(language) = languages.iter().find(|language| language.tag.eq_ignore_ascii_case(prefix)) {
                return &language.tag;
            }
            let subtag_prefix = format!("{}-", prefix.to_ascii_lowercase());
            if let Some(language) = languages
                .iter()
                .find(|language| language.tag.to_ascii_lowercase().starts_with(&subtag_prefix))
            {
                return &language.tag;
            }
            let Some((parent, _)) = prefix.rsplit_once('-') else {
                break;
            };
            prefix = parent;
        }
    }
    languages
        .iter()
        .find(|language| language.tag == "en-US")
        .unwrap_or(&languages[0])
        .tag
        .as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_preferences_preserve_scripts_and_fall_back_to_available_languages() {
        let languages: Vec<_> = ["en-US", "zh-Hans", "zh-Hant", "ko-KR"]
            .into_iter()
            .map(|tag| OcrLanguage {
                tag: tag.into(),
                label: tag.into(),
            })
            .collect();
        assert_eq!(preferred_language(&languages, &["zh-Hant-TW".into()]), "zh-Hant");
        assert_eq!(preferred_language(&languages, &["ko".into()]), "ko-KR");
        assert_eq!(preferred_language(&languages, &["en-GB".into()]), "en-US");
        assert_eq!(preferred_language(&languages, &["unsupported".into(), "ko-KR".into()]), "ko-KR");
    }

    #[test]
    fn vision_recognizes_the_rgba_fixture() {
        let mut backend = Backend::new().unwrap();
        let (languages, default, _) = backend.available_languages().unwrap();
        assert!(languages.iter().any(|language| language.tag == default));
        let language = languages.iter().find(|language| language.tag == "en-US").unwrap();
        let image = image::load_from_memory(include_bytes!("testdata/english.png"))
            .unwrap()
            .to_rgba8();
        let job = OcrJob {
            width: image.width(),
            height: image.height(),
            rgba: image.into_raw(),
            language: language.tag.clone(),
        };
        job.validate().unwrap();
        let text = backend.recognize(job).unwrap();
        assert!(text.contains("MAC OCR 123"), "Unexpected Vision OCR result: {text:?}");
        let blank = OcrJob {
            width: 200,
            height: 100,
            rgba: vec![255; 200 * 100 * 4],
            language: language.tag.clone(),
        };
        assert!(
            backend.recognize(blank).unwrap().trim().is_empty(),
            "The previous request's results must not leak into the next image"
        );
    }
}
