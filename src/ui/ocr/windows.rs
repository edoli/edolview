use color_eyre::eyre::{eyre, Result};
use windows::{
    core::HSTRING,
    Globalization::Language,
    Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap},
    Media::Ocr::OcrEngine,
    Security::Cryptography::CryptographicBuffer,
    Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED},
};

use super::{OcrJob, OcrLanguage};

struct WinRtApartment(std::marker::PhantomData<std::rc::Rc<()>>);

impl WinRtApartment {
    fn new() -> Result<Self> {
        // SAFETY: called once on a dedicated worker thread with no existing COM
        // apartment; the guard balances this successful initialization on that thread.
        unsafe { RoInitialize(RO_INIT_MULTITHREADED)? };
        Ok(Self(Default::default()))
    }
}

impl Drop for WinRtApartment {
    fn drop(&mut self) {
        // SAFETY: the guard never leaves its worker and outlives all WinRT objects.
        unsafe { RoUninitialize() };
    }
}

pub(super) struct Backend {
    engine: Option<(String, OcrEngine)>,
    _apartment: WinRtApartment,
}

impl Backend {
    pub(super) fn new() -> Result<Self> {
        Ok(Self {
            engine: None,
            _apartment: WinRtApartment::new()?,
        })
    }

    pub(super) fn available_languages(&self) -> Result<(Vec<OcrLanguage>, String, u32)> {
        let mut languages = Vec::new();
        for language in OcrEngine::AvailableRecognizerLanguages()? {
            languages.push(OcrLanguage {
                tag: language.LanguageTag()?.to_string(),
                label: format!("{} ({})", language.DisplayName()?, language.LanguageTag()?),
            });
        }
        let first = languages.first().ok_or_else(|| {
            eyre!("No OCR language packs installed. Add an OCR language in Windows language settings.")
        })?;
        let language = OcrEngine::TryCreateFromUserProfileLanguages()
            .and_then(|engine| engine.RecognizerLanguage()?.LanguageTag())
            .map(|tag| tag.to_string())
            .unwrap_or_else(|_| first.tag.clone());
        let limit = OcrEngine::MaxImageDimension()?.min(4096);
        if limit == 0 {
            return Err(eyre!("Windows OCR reported an invalid image size limit"));
        }
        Ok((languages, language, limit))
    }

    pub(super) fn recognize(&mut self, mut job: OcrJob) -> Result<String> {
        if self.engine.as_ref().is_none_or(|(language, _)| language != &job.language) {
            let language = Language::CreateLanguage(&HSTRING::from(&job.language))?;
            self.engine = Some((job.language, OcrEngine::TryCreateFromLanguage(&language)?));
        }
        let engine = &self.engine.as_ref().ok_or_else(|| eyre!("Could not initialize OCR engine"))?.1;
        // GPU exports are RGBA; Windows OCR accepts BGRA8 (or Gray8).
        for pixel in job.rgba.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        let buffer = CryptographicBuffer::CreateFromByteArray(&job.rgba)?;
        let bitmap = SoftwareBitmap::CreateCopyFromBuffer(
            &buffer,
            BitmapPixelFormat::Bgra8,
            job.width as i32,
            job.height as i32,
        )?;
        let result = engine.RecognizeAsync(&bitmap)?.join()?;
        let mut lines = Vec::new();
        for line in result.Lines()? {
            lines.push(line.Text()?.to_string());
        }
        bitmap.Close()?;
        Ok(lines.join("\n"))
    }
}
