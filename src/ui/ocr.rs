use std::sync::{mpsc, Arc};

use color_eyre::eyre::{eyre, Result};
use eframe::egui;
use windows::{
    core::HSTRING,
    Globalization::Language,
    Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap},
    Media::Ocr::OcrEngine,
    Security::Cryptography::CryptographicBuffer,
    Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED},
};

use crate::{
    model::Recti,
    res::{KeyboardShortcutExt, COPY_SC, CUT_SC, NOTICE_ERROR_TEXT, PASTE_SC, SELECT_ALL_SC},
    ui::gpu::ExportCompletion,
};

#[derive(Clone, Copy)]
enum TextAction {
    Cut,
    Copy,
    Paste,
    SelectAll,
}

pub(super) struct OcrExport {
    pub source: String,
    pub rect: Recti,
    pub scale: f32,
    pub completion: ExportCompletion,
}

struct OcrLanguage {
    tag: String,
    label: String,
}

struct OcrJob {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    language: String,
}

enum WorkerEvent {
    Ready(Result<(Vec<OcrLanguage>, String, u32), String>),
    Finished(Result<String, String>),
}

pub(super) struct OcrUi {
    job_tx: mpsc::Sender<OcrJob>,
    event_tx: mpsc::Sender<WorkerEvent>,
    event_rx: mpsc::Receiver<WorkerEvent>,
    started: bool,
    languages: Vec<OcrLanguage>,
    language: String,
    max_dimension: u32,
    initialization_error: Option<String>,
    pending: Option<OcrExport>,
    busy: bool,
    open: bool,
    text: String,
    has_text_result: bool,
    error: Option<String>,
}

impl Default for OcrUi {
    fn default() -> Self {
        let (job_tx, _) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        Self {
            job_tx,
            event_tx,
            event_rx,
            started: false,
            languages: Vec::new(),
            language: String::new(),
            max_dimension: 0,
            initialization_error: None,
            pending: None,
            busy: false,
            open: false,
            text: String::new(),
            has_text_result: false,
            error: None,
        }
    }
}

impl OcrUi {
    /// Lazily initialize WinRT on its own thread; never activate OCR on the UI thread.
    fn start(&mut self, ctx: &egui::Context) {
        if self.started {
            return;
        }
        self.started = true;
        let (job_tx, job_rx) = mpsc::channel();
        self.job_tx = job_tx;
        let event_tx = self.event_tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let apartment = WinRtApartment::new();
            let ready = apartment
                .as_ref()
                .map_err(|error| eyre!("{error:#}"))
                .and_then(|_| available_languages());
            let usable = ready.is_ok();
            let _ = event_tx.send(WorkerEvent::Ready(
                ready.map_err(|error| format!("Windows OCR unavailable: {error:#}")),
            ));
            ctx.request_repaint();
            if !usable {
                return;
            }
            // Engines are created and retained only on this initialized worker thread.
            let mut engine: Option<(String, OcrEngine)> = None;
            while let Ok(job) = job_rx.recv() {
                let result = recognize(job, &mut engine).map_err(|error| format!("OCR failed: {error:#}"));
                let _ = event_tx.send(WorkerEvent::Finished(result));
                ctx.request_repaint();
            }
        });
    }

    pub fn context_menu(&mut self, ui: &mut egui::Ui, source: String, rect: Recti) {
        self.start(ui.ctx());
        let rect = rect.validate();
        let ready = self.max_dimension > 0 && !self.languages.is_empty();
        let tooltip = self.initialization_error.as_deref().unwrap_or(if !ready {
            "Loading installed Windows OCR languages..."
        } else if rect.empty() {
            "Select an image region first."
        } else {
            "Recognize the selected region using Windows OCR and the current display settings."
        });
        let row_height = ui.spacing().interact_size.y;
        // A popup's sizing pass must measure content, not reserve the window's
        // remaining width. Later passes can fill the menu's measured width.
        let row_width = if ui.is_sizing_pass() {
            ui.min_rect().width()
        } else {
            ui.available_width()
        };
        let scan_clicked = ui
            .allocate_ui_with_layout(
                egui::vec2(row_width, row_height),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    // Reserve the right edge first; the rest of the row is the scan button.
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.allocate_ui_with_layout(
                        egui::vec2(110.0, row_height),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            // Context menus remove widget fills/strokes. Use the same
                            // app style as the channel and colormap dropdowns instead.
                            ui.set_style(ui.ctx().global_style());
                            ui.add_enabled_ui(ready && !self.busy, |ui| {
                                let selected_language = self
                                    .languages
                                    .iter()
                                    .find(|language| language.tag == self.language)
                                    .map(|language| language.label.as_str())
                                    .unwrap_or("Loading...");
                                egui::ComboBox::from_id_salt("ocr_language")
                                    .width(110.0)
                                    .truncate()
                                    .selected_text(selected_language)
                                    .show_ui(ui, |ui| {
                                        for language in &self.languages {
                                            ui.selectable_value(
                                                &mut self.language,
                                                language.tag.clone(),
                                                &language.label,
                                            );
                                        }
                                    })
                                    .response
                                    .on_hover_text("OCR language");
                            });
                        },
                    );
                    let scan_size = egui::vec2(ui.available_width(), row_height);
                    ui.add_enabled(
                        ready && !self.busy && !rect.empty(),
                        egui::Button::new(()).left_text("Text Scan (OCR)").min_size(scan_size),
                    )
                    .on_disabled_hover_text(tooltip)
                    .on_hover_text(tooltip)
                    .clicked()
                },
            )
            .inner;
        if scan_clicked {
            self.request(ui.ctx(), source, rect);
            ui.close();
        }
    }

    fn request(&mut self, ctx: &egui::Context, source: String, rect: Recti) {
        let scale = (self.max_dimension as f32 / rect.width().max(rect.height()) as f32).min(1.0);
        let width = (rect.width() as f32 * scale).round().max(1.0) as u32;
        let height = (rect.height() as f32 * scale).round().max(1.0) as u32;
        let job_tx = self.job_tx.clone();
        let event_tx = self.event_tx.clone();
        let language = self.language.clone();
        let ctx = ctx.clone();
        self.pending = Some(OcrExport {
            source,
            rect,
            scale,
            completion: Arc::new(move |pixels| {
                let error = match pixels {
                    Ok(rgba) => job_tx
                        .send(OcrJob {
                            width,
                            height,
                            rgba,
                            language: language.clone(),
                        })
                        .err()
                        .map(|_| "Windows OCR worker disconnected".to_string()),
                    Err(error) => Some(format!("Could not capture OCR region: {error}")),
                };
                if let Some(error) = error {
                    let _ = event_tx.send(WorkerEvent::Finished(Err(error)));
                }
                ctx.request_repaint();
            }),
        });
        self.text.clear();
        self.has_text_result = false;
        self.error = None;
        self.busy = true;
        self.open = true;
    }

    pub fn take_export(&mut self) -> Option<OcrExport> {
        self.pending.take()
    }

    /// Returns user-visible failures for the app's existing toast and stderr flow.
    pub fn show(&mut self, ctx: &egui::Context) -> Vec<String> {
        let mut errors = Vec::new();
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                WorkerEvent::Ready(Ok((languages, language, max_dimension))) => {
                    self.languages = languages;
                    self.language = language;
                    self.max_dimension = max_dimension;
                }
                WorkerEvent::Ready(Err(error)) => {
                    self.initialization_error = Some(error.clone());
                    errors.push(error);
                }
                WorkerEvent::Finished(result) => {
                    self.busy = false;
                    match result {
                        Ok(text) => {
                            self.has_text_result = !text.trim().is_empty();
                            self.text = text;
                        }
                        Err(error) => {
                            self.error = Some(error.clone());
                            errors.push(error);
                        }
                    }
                }
            }
        }
        egui::Window::new("Recognized Text (OCR)")
            .open(&mut self.open)
            .default_size(egui::vec2(560.0, 300.0))
            .show(ctx, |ui| {
                if self.busy {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Recognizing selected text...");
                    });
                } else if let Some(error) = &self.error {
                    ui.colored_label(NOTICE_ERROR_TEXT, error);
                } else if !self.has_text_result {
                    ui.label("No text found. Try another OCR language or adjust the selection / display settings.");
                } else {
                    if ui.button("Copy Text").clicked() {
                        ctx.copy_text(self.text.clone());
                    }
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        show_text_editor(ui, &mut self.text, &mut errors);
                    });
                }
            });
        errors
    }
}

fn show_text_editor(ui: &mut egui::Ui, text: &mut String, errors: &mut Vec<String>) -> egui::Id {
    let id = ui.make_persistent_id("ocr_result_text");
    let action_id = id.with("pending_action");
    let pending_action = ui.ctx().data_mut(|data| {
        let action = data.get_temp::<TextAction>(action_id);
        data.remove::<TextAction>(action_id);
        action
    });
    if let Some(action) = pending_action {
        ui.memory_mut(|memory| memory.request_focus(id));
        let event = match action {
            TextAction::Cut => Some(egui::Event::Cut),
            TextAction::Copy => Some(egui::Event::Copy),
            TextAction::Paste => match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.get_text()) {
                Ok(text) => Some(egui::Event::Paste(text)),
                Err(error) => {
                    errors.push(format!("Could not paste clipboard text: {error}"));
                    None
                }
            },
            TextAction::SelectAll => {
                let mut state = egui::TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::select_all_str(text)));
                state.store(ui.ctx(), id);
                None
            }
        };
        if let Some(event) = event {
            ui.input_mut(|input| input.events.push(event));
        }
    }
    let previous_selection = if ui.input(|input| input.pointer.secondary_pressed()) {
        egui::TextEdit::load_state(ui.ctx(), id)
            .and_then(|state| state.cursor.char_range())
            .filter(|range| !range.is_empty())
    } else {
        None
    };
    let mut editor = egui::TextEdit::multiline(text)
        .id(id)
        .desired_width(f32::INFINITY)
        .desired_rows(10)
        .show(ui);
    // egui starts a text selection on any mouse press. A right-click that
    // opens editing commands must preserve the selection those commands use.
    if editor.response.response.contains_pointer() {
        if let Some(selection) = previous_selection {
            editor.state.cursor.set_char_range(Some(selection));
            editor.state.clone().store(ui.ctx(), id);
        }
    }
    let menu_selection_id = id.with("menu_selection");
    if editor.response.response.secondary_clicked() {
        ui.ctx()
            .data_mut(|data| data.insert_temp(menu_selection_id, editor.state.cursor.char_range()));
    }
    if editor.response.response.context_menu_opened() {
        if let Some(Some(selection)) = ui
            .ctx()
            .data(|data| data.get_temp::<Option<egui::text::CCursorRange>>(menu_selection_id))
        {
            // Menu pointer presses can overlap the editor. Keep its selection
            // stable until the chosen action is handed back to TextEdit.
            editor.state.cursor.set_char_range(Some(selection));
            editor.state.clone().store(ui.ctx(), id);
        }
    }
    let has_selection = editor.state.cursor.char_range().is_some_and(|range| !range.is_empty());
    let mut action = None;
    editor.response.response.context_menu(|ui| {
        for (label, shortcut, enabled, text_action) in [
            ("Cut", CUT_SC, has_selection, TextAction::Cut),
            ("Copy", COPY_SC, has_selection, TextAction::Copy),
            ("Paste", PASTE_SC, true, TextAction::Paste),
            ("Select All", SELECT_ALL_SC, !text.is_empty(), TextAction::SelectAll),
        ] {
            let command = ui.add_enabled(enabled, egui::Button::new(label).shortcut_text(shortcut.format_sys()));
            if command.clicked() {
                action = Some(text_action);
                ui.close();
            }
        }
    });
    if let Some(action) = action {
        // Restore focus after the popup has closed, then let TextEdit handle
        // the command next frame with its usual selection and undo behavior.
        ui.ctx().data_mut(|data| data.insert_temp(action_id, action));
        ui.ctx().request_repaint();
    }
    editor.response.id
}

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

fn available_languages() -> Result<(Vec<OcrLanguage>, String, u32)> {
    let mut languages = Vec::new();
    for language in OcrEngine::AvailableRecognizerLanguages()? {
        languages.push(OcrLanguage {
            tag: language.LanguageTag()?.to_string(),
            label: format!("{} ({})", language.DisplayName()?, language.LanguageTag()?),
        });
    }
    let first = languages
        .first()
        .ok_or_else(|| eyre!("No OCR language packs installed. Add an OCR language in Windows language settings."))?;
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

fn recognize(mut job: OcrJob, cached: &mut Option<(String, OcrEngine)>) -> Result<String> {
    if job.width == 0 || job.height == 0 || job.rgba.len() != job.width as usize * job.height as usize * 4 {
        return Err(eyre!("Invalid OCR image dimensions or pixel data"));
    }
    if cached.as_ref().is_none_or(|(language, _)| language != &job.language) {
        let language = Language::CreateLanguage(&HSTRING::from(&job.language))?;
        *cached = Some((job.language, OcrEngine::TryCreateFromLanguage(&language)?));
    }
    let engine = &cached.as_ref().ok_or_else(|| eyre!("Could not initialize OCR engine"))?.1;
    // GPU exports are RGBA; Windows OCR accepts BGRA8 (or Gray8).
    for pixel in job.rgba.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    let buffer = CryptographicBuffer::CreateFromByteArray(&job.rgba)?;
    let bitmap =
        SoftwareBitmap::CreateCopyFromBuffer(&buffer, BitmapPixelFormat::Bgra8, job.width as i32, job.height as i32)?;
    let result = engine.RecognizeAsync(&bitmap)?.join()?;
    let mut lines = Vec::new();
    for line in result.Lines()? {
        lines.push(line.Text()?.to_string());
    }
    bitmap.Close()?;
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::math_ext::vec2i;

    #[test]
    fn text_editor_cut_and_paste_preserve_unicode_and_work_when_empty() {
        fn frame(ctx: &egui::Context, text: &mut String, events: Vec<egui::Event>) -> (egui::Id, egui::PlatformOutput) {
            let mut id = None;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    id = Some(
                        egui::CentralPanel::default()
                            .show(ui, |ui| {
                                let id = show_text_editor(ui, text, &mut Vec::new());
                                ui.memory_mut(|memory| memory.request_focus(id));
                                id
                            })
                            .inner,
                    );
                },
            );
            output.textures_delta.clear();
            (id.unwrap(), output.platform_output)
        }

        let ctx = egui::Context::default();
        let original = "한글 OCR 🌱\nsecond line";
        let mut text = original.to_string();
        let (id, _) = frame(&ctx, &mut text, vec![]);
        let mut state = egui::TextEdit::load_state(&ctx, id).unwrap();
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::select_all_str(&text)));
        state.store(&ctx, id);
        let position = ctx.read_response(id).unwrap().rect.left_top() + egui::vec2(3.0, 3.0);
        for pressed in [true, false] {
            frame(
                &ctx,
                &mut text,
                vec![
                    egui::Event::PointerMoved(position),
                    egui::Event::PointerButton {
                        pos: position,
                        button: egui::PointerButton::Secondary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        let state = egui::TextEdit::load_state(&ctx, id).unwrap();
        assert_eq!(state.cursor.char_range(), Some(egui::text::CCursorRange::select_all_str(&text)));
        frame(&ctx, &mut text, vec![]);
        let position = ctx.read_response(id.with("popup")).unwrap().rect.left_top() + egui::vec2(10.0, 10.0);
        for pressed in [true, false] {
            frame(
                &ctx,
                &mut text,
                vec![
                    egui::Event::PointerMoved(position),
                    egui::Event::PointerButton {
                        pos: position,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        let (_, output) = frame(&ctx, &mut text, vec![]);
        assert!(text.is_empty());
        assert!(output
            .commands
            .iter()
            .any(|command| matches!(command, egui::OutputCommand::CopyText(value) if value == original)));
        frame(&ctx, &mut text, vec![egui::Event::Paste(original.to_string())]);
        assert_eq!(text, original);
        let state = egui::TextEdit::load_state(&ctx, id).unwrap();
        assert_eq!(state.cursor.char_range().unwrap().primary.index.0, original.chars().count());
    }

    #[test]
    fn capture_failure_releases_busy_and_reports_the_failure() {
        let ctx = egui::Context::default();
        let mut ocr = OcrUi {
            max_dimension: 100,
            ..Default::default()
        };
        ocr.request(&ctx, "primary".into(), Recti::from_min_size(vec2i(0, 0), vec2i(40, 20)));
        assert!(ocr.busy);
        (ocr.take_export().unwrap().completion)(Err("readback failed".into()));
        ctx.begin_pass(egui::RawInput::default());
        let errors = ocr.show(&ctx);
        ctx.end_pass().textures_delta.clear();
        assert!(!ocr.busy);
        assert_eq!(errors, ["Could not capture OCR region: readback failed"]);
    }

    #[test]
    fn capture_preserves_its_language_and_caps_large_regions_before_readback() {
        let ctx = egui::Context::default();
        let (job_tx, job_rx) = mpsc::channel();
        let mut ocr = OcrUi {
            job_tx,
            max_dimension: 100,
            language: "en-US".into(),
            ..Default::default()
        };
        let rect = Recti::from_min_size(vec2i(50, 80), vec2i(20_000, 10_000));
        ocr.request(&ctx, "secondary".into(), rect);
        ocr.language = "ko".into();
        let export = ocr.take_export().unwrap();
        assert_eq!(export.source, "secondary");
        assert_eq!(export.rect, rect);
        assert_eq!(export.scale, 0.005);
        (export.completion)(Ok(vec![255; 100 * 50 * 4]));
        let job = job_rx.try_recv().unwrap();
        assert_eq!((job.width, job.height), (100, 50));
        assert_eq!(job.language, "en-US");
    }

    #[test]
    fn malformed_pixels_are_rejected_before_calling_windows() {
        let job = OcrJob {
            width: 40,
            height: 20,
            rgba: vec![0; 10],
            language: "en-US".into(),
        };
        assert!(recognize(job, &mut None).unwrap_err().to_string().contains("Invalid OCR image"));
    }
}
