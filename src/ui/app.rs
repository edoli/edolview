use color_eyre::eyre::Report;
use core::f32;
use eframe::egui::{self, pos2, vec2, Color32, Rangef, Visuals};
use rfd::FileDialog;
use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    sync::{atomic::Ordering, mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
    vec,
};

#[cfg(debug_assertions)]
use crate::util::timer::ScopedTimer;
use crate::{
    model::{
        start_server_with_retry, AppState, AssetType, ComparisonMode, FileAsset, Image, ImageData, MeanDim, Recti,
        SocketAsset, StatisticsScope, StatisticsType, StatisticsUpdate, StatisticsWorker,
    },
    res::{
        icons::Icons, KeyboardShortcutExt, ASSET_SECONDARY_SELECTION_FILL, CONTROL_LISTENER_UNAVAILABLE_TEXT,
        DANGER_TEXT, NOTICE_ERROR_TEXT, NOTICE_WARNING_TEXT, STATISTICS_MAX_TOGGLE_FILL, STATISTICS_MIN_TOGGLE_FILL,
        UPDATE_ACCENT_FILL, UPDATE_ACCENT_TEXT,
    },
    ui::{
        component::{
            channel_toggle_ui, display_controls_ui, display_profile_slider, draw_histogram, draw_multi_line_plot,
            egui_ext::{ComboBoxExt, ResponseExt, Size, UiExt},
            show_bookmark_window, BookmarkJumpMode, CopyExport, ExportAction, SaveExport, Toast, ToastUi, ToastsExt,
        },
        fonts::{apply_fallback_fonts, spawn_fallback_font_loader, LoadedFallbackFonts},
        ImageViewer,
    },
    util::{concurrency::mpsc_with_notify, math_ext::vec2i, series::SeriesRef},
};

#[derive(PartialEq, Clone)]
enum PlotDim {
    Column,
    Row,
    Auto,
}

#[derive(Clone, PartialEq, Eq)]
struct Bookmark {
    rect: Recti,
}

#[derive(Clone, PartialEq, Eq)]
enum UpdateStatus {
    Idle,
    Checking,
    Available(crate::update::AvailableUpdate),
    Applying(String),
    UpToDate,
    Error(String),
}

struct PendingImageSaveDialog {
    rx: mpsc::Receiver<Option<(PathBuf, String)>>,
}

enum StartupPathLoadResult {
    Loaded {
        path: PathBuf,
        hash: String,
        image: ImageData,
    },
    Reused {
        path: PathBuf,
        hash: String,
    },
    Failed {
        path: PathBuf,
        error: Report,
    },
}

#[derive(Clone, Copy)]
enum MarqueeAngleDisplay {
    Degrees(f32),
    Radians(f32),
}

fn statistics_overlay_toggle(
    ui: &mut egui::Ui,
    selected: &mut bool,
    value: f64,
    active_fill: Color32,
    hover_text: String,
) -> egui::Response {
    let response = ui
        .scope(|ui| {
            if *selected {
                let visuals = ui.visuals_mut();
                visuals.selection.bg_fill = active_fill;
                visuals.selection.stroke.color = active_fill;
                visuals.override_text_color = Some(Color32::WHITE);
            }

            ui.selectable_label(*selected, format!("{:.4}", value))
                .on_hover_text(hover_text)
        })
        .inner;
    response.context_menu(|ui| {
        if ui.button("Copy Value").clicked() {
            ui.ctx().copy_text(value.to_string());
            ui.close();
        }
    });
    if response.clicked() {
        *selected = !*selected;
    }
    response
}

pub struct ViewerApp {
    state: AppState,
    viewer: ImageViewer,
    last_path: Option<PathBuf>,
    startup_paths: Vec<PathBuf>,
    startup_path_rx: Option<mpsc::Receiver<StartupPathLoadResult>>,
    tmp_marquee_rect: Recti,
    marquee_rect_text: String,
    is_start_background_event_handlers_called: bool,

    // Marquee change callbacks
    last_marquee_rect_for_cb: Recti,
    last_marquee_asset_hash: Option<String>,
    bookmarks: Vec<Bookmark>,
    active_bookmark_index: Option<usize>,
    bookmark_jump_mode: BookmarkJumpMode,

    show_plot_channels: [bool; 4],

    show_histogram: bool,
    show_histogram_channels: [bool; 4],

    show_statistics: bool,
    show_statistics_min_overlay_channels: [bool; 4],
    show_statistics_max_overlay_channels: [bool; 4],

    plot_dim: PlotDim,

    icons: Icons,

    toasts: Vec<Toast>,

    socket_rx: mpsc::Receiver<SocketAsset>,
    socket_nx: Option<mpsc::Receiver<()>>,
    socket_server: Option<crate::model::SocketServer>,
    control_nx: Option<mpsc::Receiver<()>>,

    statistics_worker: Arc<Mutex<StatisticsWorker>>,
    statistics_tx: mpsc::Sender<Vec<StatisticsUpdate>>,
    statistics_rx: mpsc::Receiver<Vec<StatisticsUpdate>>,
    next_statistics_request_id: u64,

    // Async font loading: receiver for optional user/system fallback fonts.
    font_rx: Option<mpsc::Receiver<LoadedFallbackFonts>>,
    fallback_font_applied: bool,

    update_rx: Option<mpsc::Receiver<crate::update::UpdateCheck>>,
    update_apply_rx: Option<mpsc::Receiver<Result<crate::update::PreparedUpdate, String>>>,
    update_status: UpdateStatus,
    update_toast_shown: bool,
    close_for_update: bool,
    pending_update_confirmation: Option<crate::update::AvailableUpdate>,
    app_settings: crate::settings::AppSettings,
    show_settings_modal: bool,
    show_bookmarks_modal: bool,
    control_rx: mpsc::Receiver<Vec<PathBuf>>,
    control_instance: Option<crate::control::ControlInstance>,
    last_control_touch: Instant,
    was_focused_last_frame: bool,
    last_image_save_dir: Option<PathBuf>,
    pending_image_save_dialog: Option<PendingImageSaveDialog>,
}

impl Drop for ViewerApp {
    fn drop(&mut self) {
        if let Some(socket_server) = self.socket_server.as_mut() {
            if let Err(err) = socket_server.shutdown() {
                eprintln!("Failed to stop socket listener: {err}");
            }
        }

        if let Some(mut control_instance) = self.control_instance.take() {
            if let Err(err) = control_instance.remove() {
                eprintln!("Failed to remove active window registration: {err}");
            }
            control_instance.shutdown();
        }
    }
}

fn marquee_angle_radian(rect: Recti) -> Option<f32> {
    let rect = rect.validate();
    if rect.empty() {
        return None;
    }

    Some((rect.height() as f32).atan2(rect.width() as f32))
}

fn marquee_angle(rect: Recti, unit: crate::settings::AngleDisplayUnit) -> Option<MarqueeAngleDisplay> {
    let angle_radians = marquee_angle_radian(rect)?;
    Some(match unit {
        crate::settings::AngleDisplayUnit::Degrees => MarqueeAngleDisplay::Degrees(angle_radians.to_degrees()),
        crate::settings::AngleDisplayUnit::Radians => MarqueeAngleDisplay::Radians(angle_radians),
    })
}

fn marquee_angle_tooltip(unit: crate::settings::AngleDisplayUnit) -> &'static str {
    match unit {
        crate::settings::AngleDisplayUnit::Degrees => {
            "Arc tan of marquee aspect ratio (height / width), shown in degrees."
        }
        crate::settings::AngleDisplayUnit::Radians => {
            "Arc tan of marquee aspect ratio (height / width), shown in radians."
        }
    }
}

fn format_marquee_angle(angle: Option<MarqueeAngleDisplay>) -> String {
    match angle {
        Some(MarqueeAngleDisplay::Degrees(value)) => format!("{value:.2}°"),
        Some(MarqueeAngleDisplay::Radians(value)) => format!("{value:.3}"),
        None => "-".to_string(),
    }
}

impl ViewerApp {
    fn request_root_repaint(ctx: &egui::Context) {
        ctx.request_repaint_of(egui::ViewportId::ROOT);
    }

    fn egui_has_label_text_selection(ctx: &egui::Context) -> bool {
        ctx.plugin_or_default::<egui::text_selection::LabelSelectionState>()
            .lock()
            .has_selection()
    }

    pub fn new() -> Self {
        let mut state = AppState::empty();
        let marquee_rect = state.marquee_rect;
        let app_settings = crate::settings::AppSettings::load().unwrap_or_else(|err| {
            eprintln!("Failed to load settings: {err}");
            crate::settings::AppSettings::default()
        });
        let persisted_ui_state = app_settings.ui_state.clone();
        crate::model::MEAN_PROCESSOR.set_precompute_enabled(app_settings.integral_table_precompute);
        state.background_style = persisted_ui_state.background_style;
        state.is_show_pixel_value = persisted_ui_state.is_show_pixel_value;
        state.is_show_crosshair = persisted_ui_state.is_show_crosshair;
        state.is_show_sidebar = persisted_ui_state.is_show_sidebar;
        state.is_show_statusbar = persisted_ui_state.is_show_statusbar;
        state.copy_use_original_size = persisted_ui_state.copy_use_original_size;

        // Start socket server for receiving images
        let host = "127.0.0.1";
        let port = 21734;
        let (socket_tx, socket_rx, socket_nx) = mpsc_with_notify::<SocketAsset>();
        let socket_state = state.socket_state.clone();
        let socket_info = state.socket_info.clone();

        let (statistics_tx, statistics_rx) = mpsc::channel::<Vec<StatisticsUpdate>>();
        let (control_tx, control_rx, control_nx) = mpsc_with_notify::<Vec<PathBuf>>();

        let mut toasts = Vec::new();
        let socket_server = match start_server_with_retry(host, port, socket_tx, socket_state, socket_info) {
            Ok(server) => Some(server),
            Err(err) => {
                state.socket_state.is_socket_active.store(false, Ordering::Relaxed);
                eprintln!("Failed to start socket server: {err}");
                toasts.add_error(format!("Failed to start socket server: {err}"));
                None
            }
        };
        let control_instance = match crate::control::start_control_listener(control_tx) {
            Ok(instance) => Some(instance),
            Err(err) => {
                toasts.add_error(format!("Failed to start file-open control listener: {err}"));
                None
            }
        };

        // Load optional user override and platform CJK fallback fonts off the UI thread.
        let font_rx = spawn_fallback_font_loader();

        Self {
            state,
            viewer: ImageViewer::new(),

            last_path: None,
            startup_paths: Vec::new(),
            startup_path_rx: None,

            tmp_marquee_rect: marquee_rect,
            marquee_rect_text: marquee_rect.to_string(),
            is_start_background_event_handlers_called: false,

            last_marquee_rect_for_cb: marquee_rect,
            last_marquee_asset_hash: None,
            bookmarks: Vec::new(),
            active_bookmark_index: None,
            bookmark_jump_mode: BookmarkJumpMode::Center,

            show_plot_channels: [true, true, true, false],

            show_histogram: false,
            show_histogram_channels: [true, true, true, false],

            show_statistics: false,
            show_statistics_min_overlay_channels: [false; 4],
            show_statistics_max_overlay_channels: [false; 4],

            plot_dim: PlotDim::Auto,

            toasts,

            icons: Icons::new(),

            socket_rx,
            socket_nx: Some(socket_nx),
            socket_server,
            control_nx: Some(control_nx),

            statistics_tx,
            statistics_rx,

            statistics_worker: Arc::new(Mutex::new(StatisticsWorker::new())),
            next_statistics_request_id: 1,

            font_rx,
            fallback_font_applied: false,

            update_rx: None,
            update_apply_rx: None,
            update_status: UpdateStatus::Idle,
            update_toast_shown: false,
            close_for_update: false,
            pending_update_confirmation: None,
            app_settings,
            show_settings_modal: false,
            show_bookmarks_modal: false,
            control_rx,
            control_instance,
            last_control_touch: Instant::now(),
            was_focused_last_frame: false,
            last_image_save_dir: None,
            pending_image_save_dialog: None,
        }
    }

    #[inline]
    pub fn with_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.startup_paths = paths;
        self
    }

    fn start_startup_path_loading(&mut self, ctx: &egui::Context) {
        if self.startup_path_rx.is_some() || self.startup_paths.is_empty() {
            return;
        }

        let paths = std::mem::take(&mut self.startup_paths);
        let (tx, rx) = mpsc::channel();
        self.startup_path_rx = Some(rx);

        let load_ctx = ctx.clone();
        thread::spawn(move || {
            let mut seen_hashes = HashSet::new();

            for path in paths {
                let result = match FileAsset::hash_from_path(&path) {
                    Ok(hash) => {
                        if seen_hashes.insert(hash.clone()) {
                            match ImageData::load_from_path(&path) {
                                Ok(image) => StartupPathLoadResult::Loaded { path, hash, image },
                                Err(err) => StartupPathLoadResult::Failed { path, error: err },
                            }
                        } else {
                            StartupPathLoadResult::Reused { path, hash }
                        }
                    }
                    Err(err) => StartupPathLoadResult::Failed { path, error: err },
                };

                if tx.send(result).is_err() {
                    break;
                }

                Self::request_root_repaint(&load_ctx);
            }
        });
    }

    fn load_fail(toasts: &mut Vec<Toast>, message: &str, path: Option<&PathBuf>, e: &Report) {
        eprintln!("{message}: {e}");
        let path_str = path.map_or("<invalid>", |p| p.to_str().unwrap_or("<invalid>"));
        toasts.add_error(format!("{message}: {path_str}"));
    }

    fn open_from_clipboard(&mut self) {
        self.state.load_from_clipboard().unwrap_or_else(|e| {
            Self::load_fail(&mut self.toasts, "Failed to load image from clipboard", None, &e);
        });
    }

    // egui-winit turns Ctrl+V key-down into Paste before app shortcuts see it.
    // The key-up still arrives, so use it as a fallback for the more specific Ctrl+Shift+V command.
    fn open_from_clipboard_shortcut_released(input: &egui::InputState) -> bool {
        input.events.iter().any(|event| {
            matches!(
                event,
                egui::Event::Key {
                    key: egui::Key::V,
                    pressed: false,
                    modifiers,
                    ..
                } if modifiers.matches_logically(crate::res::OPEN_FROM_CLIPBOARD_SC.modifiers)
            )
        })
    }

    fn handle_export_action(&mut self, export: ExportAction) {
        match export {
            ExportAction::Copy(payload) => self.copy_export_text(payload),
            ExportAction::Save(payload) => self.save_export_text(payload),
        }
    }

    fn copy_export_text(&mut self, payload: CopyExport) {
        match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(payload.text)) {
            Ok(()) => self.toasts.add_success(format!("Copied {} to clipboard", payload.title)),
            Err(err) => {
                eprintln!("Failed to copy {} to clipboard: {err}", payload.title);
                self.toasts.add_error(format!("Failed to copy {} to clipboard", payload.title));
            }
        }
    }

    fn save_export_text(&mut self, payload: SaveExport) {
        let Some(path) = FileDialog::new()
            .add_filter("CSV file", &["csv"])
            .set_file_name(payload.suggested_file_name)
            .save_file()
        else {
            return;
        };

        match fs::write(&path, payload.text) {
            Ok(()) => self
                .toasts
                .add_success(format!("Saved {} to {}", payload.title, path.display())),
            Err(err) => {
                eprintln!("Failed to save {} to {}: {err}", payload.title, path.display());
                self.toasts
                    .add_error(format!("Failed to save {} to {}", payload.title, path.display()));
            }
        }
    }

    fn active_display_source_label(&self) -> &'static str {
        if self.state.is_comparison() && self.state.comparison_mode == ComparisonMode::Split {
            if self.state.cursor_on_secondary {
                "secondary"
            } else {
                "primary"
            }
        } else {
            "image"
        }
    }

    fn active_display_file_path(&self) -> Option<PathBuf> {
        self.active_display_asset()
            .filter(|asset| asset.asset_type() == AssetType::File)
            .map(|asset| PathBuf::from(asset.name()))
            .or_else(|| self.state.path.clone())
    }

    fn default_image_export_file_name(&self) -> String {
        let active_path = self.active_display_file_path();
        let base_name = active_path
            .as_ref()
            .and_then(|path| path.file_stem())
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .unwrap_or("image");

        let rect = self.state.marquee_rect.validate();
        if rect.empty() {
            format!("{base_name}.png")
        } else {
            let (x, y, width, height) = rect.xywh();
            format!("{base_name}-{x}_{y}_{width}_{height}.png")
        }
    }

    fn build_image_save_dialog(&self) -> FileDialog {
        let has_selection = !self.state.marquee_rect.validate().empty();
        let title = if has_selection {
            "Save Selected Image As"
        } else {
            "Save Image As"
        };

        let mut dialog = FileDialog::new()
            .add_filter("PNG image", &["png"])
            .add_filter("JPEG image", &["jpg", "jpeg"])
            .set_title(title)
            .set_file_name(self.default_image_export_file_name());

        if let Some(directory) = self.last_image_save_dir.clone().or_else(|| {
            self.active_display_file_path()
                .and_then(|path| path.parent().map(PathBuf::from))
        }) {
            dialog = dialog.set_directory(directory);
        }

        dialog
    }

    fn request_viewer_image_save(&mut self, ctx: &egui::Context) {
        if self.pending_image_save_dialog.is_some() {
            return;
        }

        let dialog = self.build_image_save_dialog();
        let source_label = self.active_display_source_label().to_string();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(dialog.save_file().map(|path| (path, source_label)));
        });
        self.pending_image_save_dialog = Some(PendingImageSaveDialog { rx });
        ctx.request_repaint();
    }

    fn poll_pending_image_save_dialog(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.pending_image_save_dialog.as_ref() else {
            return;
        };

        let result = match pending.rx.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => {
                ctx.request_repaint_after(Duration::from_millis(16));
                return;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.pending_image_save_dialog = None;
                return;
            }
        };

        self.pending_image_save_dialog = None;
        if let Some((path, source_label)) = result {
            if let Some(parent) = path.parent() {
                self.last_image_save_dir = Some(parent.to_path_buf());
            }
            self.viewer.request_save(path, source_label);
            ctx.request_repaint();
        }
    }

    fn save_view_preset(&mut self, slot: usize) {
        let preset = crate::settings::ViewPreset {
            colormap_rgb: self.state.colormap_rgb.clone(),
            colormap_mono: self.state.colormap_mono.clone(),
            shader_params: self.state.shader_params.clone(),
        };

        self.app_settings.set_view_preset(slot, preset);
        match self.app_settings.save() {
            Ok(()) => self.toasts.add_success(format!("Saved view preset {}", slot + 1)),
            Err(err) => {
                eprintln!("Failed to save view preset {}: {err}", slot + 1);
                self.toasts.add_error(format!("Failed to save view preset {}", slot + 1));
            }
        }
    }

    fn apply_view_preset(&mut self, slot: usize, ctx: &egui::Context) {
        let Some(preset) = self.app_settings.view_preset(slot).cloned() else {
            self.toasts.add_info(format!("View preset {} is empty", slot + 1));
            return;
        };

        self.state.shader_params = preset.shader_params;

        if self.state.colormap_rgb_list.contains(&preset.colormap_rgb) {
            self.state.colormap_rgb = preset.colormap_rgb;
        }
        if self.state.colormap_mono_list.contains(&preset.colormap_mono) {
            self.state.colormap_mono = preset.colormap_mono;
        }

        ctx.request_repaint();
        self.toasts.add_success(format!("Applied view preset {}", slot + 1));
    }

    fn current_persistent_ui_state(&self) -> crate::settings::PersistentUiState {
        let mut ui_state = crate::settings::PersistentUiState::default();
        ui_state.background_style = self.state.background_style;
        ui_state.is_show_pixel_value = self.state.is_show_pixel_value;
        ui_state.is_show_crosshair = self.state.is_show_crosshair;
        ui_state.is_show_sidebar = self.state.is_show_sidebar;
        ui_state.is_show_statusbar = self.state.is_show_statusbar;
        ui_state.copy_use_original_size = self.state.copy_use_original_size;
        ui_state.angle_display_unit = self.app_settings.ui_state.angle_display_unit;
        ui_state
    }

    fn save_persistent_ui_state_if_needed(&mut self) {
        let ui_state = self.current_persistent_ui_state();
        if self.app_settings.ui_state == ui_state {
            return;
        }

        self.app_settings.ui_state = ui_state;
        if let Err(err) = self.app_settings.save() {
            eprintln!("Failed to save UI settings: {err}");
            self.toasts.add_error("Failed to save UI settings".to_string());
        }
    }

    fn current_bookmark_rect(&self) -> Option<Recti> {
        let rect = self.state.marquee_rect.validate();
        if !rect.empty() {
            return Some(rect);
        }
        None
    }

    fn toggle_bookmark(&mut self) {
        let Some(rect) = self.current_bookmark_rect() else {
            self.toasts.add_info("No selection to bookmark".to_string());
            return;
        };

        let bookmark = Bookmark { rect };
        if let Some(index) = self.bookmarks.iter().position(|entry| entry == &bookmark) {
            self.bookmarks.remove(index);
            self.active_bookmark_index = match self.active_bookmark_index {
                Some(active) if active == index => None,
                Some(active) if active > index => Some(active - 1),
                active => active,
            };
            self.toasts.add_success("Removed bookmark".to_string());
            return;
        }

        self.bookmarks.push(bookmark);
        self.active_bookmark_index = Some(self.bookmarks.len() - 1);
        self.toasts.add_success(format!("Added bookmark {}", self.bookmarks.len()));
    }

    fn remove_bookmark(&mut self, index: usize) {
        if index >= self.bookmarks.len() {
            return;
        }

        self.bookmarks.remove(index);
        self.active_bookmark_index = match self.active_bookmark_index {
            Some(active) if active == index => None,
            Some(active) if active > index => Some(active - 1),
            active => active,
        };
    }

    fn move_bookmark(&mut self, index: usize, direction: i32) {
        if index >= self.bookmarks.len() {
            return;
        }

        let target = if direction < 0 {
            index.saturating_sub(1)
        } else {
            (index + 1).min(self.bookmarks.len().saturating_sub(1))
        };

        if target == index {
            return;
        }

        self.bookmarks.swap(index, target);
        self.active_bookmark_index = self.active_bookmark_index.map(|active| {
            if active == index {
                target
            } else if active == target {
                index
            } else {
                active
            }
        });
    }

    fn clear_bookmarks(&mut self) {
        self.bookmarks.clear();
        self.active_bookmark_index = None;
    }

    fn navigate_bookmark(&mut self, direction: i32, ctx: &egui::Context) {
        if self.bookmarks.is_empty() {
            self.toasts.add_info("No bookmarks".to_string());
            return;
        }

        let len = self.bookmarks.len();
        let current = match self.active_bookmark_index {
            Some(index) => index,
            None if direction >= 0 => len - 1,
            None => 0,
        };
        let next = if direction >= 0 {
            (current + 1) % len
        } else if current == 0 {
            len - 1
        } else {
            current - 1
        };

        self.apply_bookmark(next, ctx);
    }

    fn apply_bookmark(&mut self, index: usize, ctx: &egui::Context) {
        let Some(bookmark) = self.bookmarks.get(index).cloned() else {
            return;
        };
        let Some(asset) = &self.state.asset else {
            self.toasts.add_info("No image loaded".to_string());
            return;
        };
        let spec = asset.image().spec();
        let image_rect = Recti::from_min_size(vec2i(0, 0), vec2i(spec.width, spec.height));
        let requested_rect = bookmark.rect.validate();
        let clipped_rect = requested_rect.intersect(image_rect);

        self.state.set_marquee_rect(requested_rect);
        self.tmp_marquee_rect = self.state.marquee_rect;
        self.marquee_rect_text = self.state.marquee_rect.to_string();
        if clipped_rect.empty() {
        } else {
            match self.bookmark_jump_mode {
                BookmarkJumpMode::None => {}
                BookmarkJumpMode::Center => self.viewer.center_rect(self.state.marquee_rect),
                BookmarkJumpMode::Fit => self.viewer.fit_rect(self.state.marquee_rect),
            }
        }

        self.active_bookmark_index = Some(index);
        ctx.request_repaint();
        if clipped_rect.empty() {
            self.toasts
                .add_info("The bookmarked area is outside the current image bounds".to_string());
        } else if clipped_rect != requested_rect {
            self.toasts
                .add_info("The bookmarked area is clipped to the current image bounds".to_string());
        }
    }

    fn begin_update(&mut self, ctx: &egui::Context, update: crate::update::AvailableUpdate) {
        let (tx, rx) = mpsc::channel();
        self.update_apply_rx = Some(rx);
        self.update_status = UpdateStatus::Applying(format!(
            "Downloading {} and preparing the {}. Edolview will close when ready.",
            update.version,
            update.target.label()
        ));

        let update_ctx = ctx.clone();
        thread::spawn(move || {
            let result = crate::update::start_update(&update);
            let _ = tx.send(result);
            Self::request_root_repaint(&update_ctx);
        });
    }

    fn show_update_confirmation_dialog(&mut self, ctx: &egui::Context) {
        let Some(update) = self.pending_update_confirmation.clone() else {
            return;
        };

        let screen_rect = ctx.input(|i| i.viewport_rect());
        let overlay_layer = egui::LayerId::new(egui::Order::Middle, egui::Id::new("confirm_update_overlay"));
        ctx.layer_painter(overlay_layer)
            .rect_filled(screen_rect, 0.0, Color32::from_black_alpha(180));

        egui::Area::new(egui::Id::new("confirm_update_modal"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .interactable(true)
            .show(ctx, |ui| {
                egui::Frame::window(ui.style())
                    .inner_margin(egui::Margin::same(14))
                    .show(ui, |ui| {
                        ui.set_min_width(440.0);
                        ui.set_max_width(440.0);

                        ui.heading("Confirm Update");

                        ui.add_space(8.0);
                        ui.label(format!(
                            "Install {} using the {} update package?",
                            update.version,
                            update.target.label()
                        ));
                        ui.add_space(8.0);
                        ui.label(
                            if cfg!(target_os = "windows") {
                                "Edolview will download the update, close briefly, install silently, and reopen automatically."
                            } else {
                                "Edolview will download the update first. When the install step is ready, the app will close and a separate updater window will stay visible until the update finishes."
                            },
                        );
                        ui.add_space(14.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let update_clicked = ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("Update").color(UPDATE_ACCENT_TEXT),
                                    )
                                    .fill(UPDATE_ACCENT_FILL),
                                )
                                .clicked();
                            let cancel_clicked = ui.button("Cancel").clicked();

                            if update_clicked {
                                self.pending_update_confirmation = None;
                                self.begin_update(ctx, update.clone());
                            } else if cancel_clicked {
                                self.pending_update_confirmation = None;
                            }
                        });
                    });
            });
    }

    fn show_update_progress_dialog(&self, ctx: &egui::Context) {
        if cfg!(target_os = "windows") {
            return;
        }
        let UpdateStatus::Applying(message) = &self.update_status else {
            return;
        };

        egui::Window::new("Updating Edolview")
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .title_bar(true)
            .show(ctx, |ui| {
                ui.set_max_width(420.0);
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new());
                    ui.label(message);
                });
            });
    }

    fn show_settings_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_settings_modal {
            return;
        }

        let mut keep_open = self.show_settings_modal;
        egui::Window::new("Settings")
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                ui.set_min_width(420.0);
                ui.heading("Application");
                ui.add_space(8.0);

                if let UpdateStatus::Applying(message) = &self.update_status {
                    ui.add_enabled(false, egui::Button::new("Updating..."))
                        .on_hover_text(message);
                } else if let UpdateStatus::Available(update) = &self.update_status {
                    let update = update.clone();
                    let update_button = ui
                        .button(egui::RichText::new("Update Available").color(UPDATE_ACCENT_TEXT))
                        .on_hover_text(format!(
                            "Download {} and apply it automatically using {}. The app will close to finish the update. (current: v{})",
                            update.version,
                            update.target.label(),
                            crate::update::current_version()
                        ));
                    if update_button.clicked() {
                        if cfg!(target_os = "windows") {
                            self.begin_update(ctx, update);
                        } else {
                            self.pending_update_confirmation = Some(update);
                        }
                    }
                }

                ui.label(
                    egui::RichText::new(format!("Current version: v{}", crate::update::current_version()))
                        .color(ui.visuals().weak_text_color()),
                )
                .on_hover_text("Current version");

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);
                ui.heading("External file open behavior");
                ui.add_space(8.0);

                let mut mode = self.app_settings.external_open_mode;
                let changed_new = ui
                    .radio_value(
                        &mut mode,
                        crate::settings::ExternalOpenMode::NewWindow,
                        crate::settings::ExternalOpenMode::NewWindow.label(),
                    )
                    .on_hover_text("Always open external files in a new Edolview window.")
                    .changed();
                let changed_existing = ui
                    .radio_value(
                        &mut mode,
                        crate::settings::ExternalOpenMode::ExistingWindow,
                        crate::settings::ExternalOpenMode::ExistingWindow.label(),
                    )
                    .on_hover_text("Send external files to the last active Edolview window and add them to its image list.")
                    .changed();

                if changed_new || changed_existing {
                    self.app_settings.external_open_mode = mode;
                    if let Err(err) = self.app_settings.save() {
                        self.toasts.add_error(err);
                    }
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);
                ui.heading("Configuration");
                ui.add_space(8.0);

                ui.checkbox(&mut self.state.copy_use_original_size, "Copy at original size")
                    .on_hover_text(format!(
                        "When enabled, {} copies marquee at image pixel size (ignores zoom).",
                        crate::res::COPY_SC.format_sys()
                    ));

                let precompute_changed = ui
                    .checkbox(
                        &mut self.app_settings.integral_table_precompute,
                        "Precompute integral table for mean calculation",
                    )
                    .on_hover_text(
                        "Keeps marquee mean and plot queries fast after loading, but uses 8 bytes per channel for every image pixel. Disable it to minimize CPU memory; mean queries then run on the GPU.",
                    )
                    .changed();
                if precompute_changed {
                    let enabled = self.app_settings.integral_table_precompute;
                    crate::model::MEAN_PROCESSOR.set_precompute_enabled(enabled);
                    if enabled {
                        if let Some(asset) = &self.state.asset {
                            crate::model::MEAN_PROCESSOR.precompute_async(asset.image());
                        }
                    }
                    if let Err(err) = self.app_settings.save() {
                        self.toasts.add_error(err);
                    }
                }

                ui.add_space(10.0);
                if let Some(control_instance) = &self.control_instance {
                    ui.label(format!("Local control address: {}", control_instance.address()));
                } else {
                    ui.colored_label(
                        CONTROL_LISTENER_UNAVAILABLE_TEXT,
                        "Local control listener is unavailable, so send-to-existing-window mode cannot work in this session.",
                    );
                }
            });

        self.show_settings_modal = keep_open;
    }

    fn show_bookmarks_dialog(&mut self, ctx: &egui::Context) {
        let bookmark_rects: Vec<_> = self.bookmarks.iter().map(|bookmark| bookmark.rect).collect();
        let actions = show_bookmark_window(
            ctx,
            &self.icons,
            &mut self.show_bookmarks_modal,
            &bookmark_rects,
            self.active_bookmark_index,
            &mut self.bookmark_jump_mode,
            &crate::res::BOOKMARK_ADD.format_sys(),
        );

        if let Some(index) = actions.move_up {
            self.move_bookmark(index, -1);
        } else if let Some(index) = actions.move_down {
            self.move_bookmark(index, 1);
        }

        if let Some(index) = actions.remove_index {
            self.remove_bookmark(index);
        }

        if actions.clear_all {
            self.clear_bookmarks();
        }

        if actions.add_current {
            self.toggle_bookmark();
        }

        if let Some(index) = actions.jump_to {
            self.apply_bookmark(index, ctx);
        }
    }

    fn refresh_control_registration(&mut self, ctx: &egui::Context) {
        let Some(control_instance) = &self.control_instance else {
            return;
        };

        let is_focused = ctx.input(|i| i.viewport().focused.unwrap_or(false));
        if is_focused && (!self.was_focused_last_frame || self.last_control_touch.elapsed() >= Duration::from_secs(1)) {
            if let Err(err) = control_instance.touch_active() {
                eprintln!("Failed to update active window registry: {err}");
            } else {
                self.last_control_touch = Instant::now();
            }
        }
        self.was_focused_last_frame = is_focused;
    }

    fn update_statistics(&mut self) {
        if !self.show_statistics {
            return;
        }
        let rect = self.state.marquee_rect.validate();
        let scope = StatisticsScope {
            request_id: self.next_statistics_request_id,
            rect,
            asset_hash: self.state.asset.as_ref().map(|asset| asset.hash().to_string()),
        };
        self.next_statistics_request_id = self.next_statistics_request_id.saturating_add(1);

        if let Some(asset) = &self.state.asset {
            let img = asset.image();

            // single image statistics
            self.statistics_worker
                .lock()
                .unwrap()
                .run_minmax(img.clone(), img.spec().dtype.alpha(), scope.clone());
        }

        if let Some(a1) = &self.state.asset_primary {
            let img1 = a1.image();

            if let Some(a2) = &self.state.asset_secondary {
                let img2 = a2.image();

                // two image statistics
                self.statistics_worker.lock().unwrap().run_psnr(
                    img1.clone(),
                    img2.clone(),
                    1.0,
                    img1.spec().dtype.alpha(),
                    scope.clone(),
                );

                // Temporarily disable SSIM due to performance issue
                // self.statistics_worker.run_ssim(img1.clone(), img2.clone(), scope.clone());
            }
        }
    }

    fn on_marquee_changed(&mut self) {
        self.update_statistics();
    }

    fn active_display_asset(&self) -> Option<&crate::model::SharedAsset> {
        if self.state.is_comparison() && self.state.comparison_mode == ComparisonMode::Split {
            if self.state.cursor_on_secondary {
                return self.state.asset_secondary.as_ref().or(self.state.asset_primary.as_ref());
            }
            return self.state.asset_primary.as_ref().or(self.state.asset_secondary.as_ref());
        }

        self.state.asset.as_ref()
    }

    #[cfg(debug_assertions)]
    fn statistics_rect_mismatch_message(&self) -> Option<String> {
        if !self.show_statistics {
            return None;
        }

        let statistics_scope = self.state.statistics.min_max.scope.as_ref()?;
        let current_asset_hash = self.state.asset.as_ref().map(|asset| asset.hash().to_string());
        if statistics_scope.asset_hash == current_asset_hash {
            return None;
        }

        Some(format!(
            "Statistics asset {:?} differs from current asset {:?}.",
            statistics_scope.asset_hash, current_asset_hash
        ))
    }

    fn start_background_event_handlers(&mut self, ctx: &egui::Context) {
        let state = &self.state;
        let _ctx = ctx.clone();

        let socket_nx = self.socket_nx.take().unwrap();
        let control_nx = self.control_nx.take().unwrap();
        let socket_state = state.socket_state.clone();
        let statistics_worker = Arc::clone(&self.statistics_worker);
        let statistics_tx = self.statistics_tx.clone();

        thread::spawn(move || {
            let mut tmp_is_receiving = false;

            loop {
                let current_is_receiving = socket_state.is_socket_receiving.load(Ordering::Relaxed);
                if tmp_is_receiving != current_is_receiving {
                    Self::request_root_repaint(&_ctx);
                    tmp_is_receiving = current_is_receiving;
                }

                match socket_nx.try_recv() {
                    Ok(()) => {
                        Self::request_root_repaint(&_ctx);
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                    Err(mpsc::TryRecvError::Disconnected) => {}
                }

                match control_nx.try_recv() {
                    Ok(()) => {
                        Self::request_root_repaint(&_ctx);
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                    Err(mpsc::TryRecvError::Disconnected) => {}
                }

                let updates = statistics_worker.lock().unwrap().invalidate();
                if !updates.is_empty() {
                    statistics_tx.send(updates).unwrap();
                    Self::request_root_repaint(&_ctx);
                }

                thread::sleep(Duration::from_millis(100));
            }
        });

        let (update_tx, update_rx) = mpsc::channel();
        self.update_rx = Some(update_rx);
        self.update_status = UpdateStatus::Checking;

        let update_ctx = ctx.clone();
        thread::spawn(move || {
            let previous_failure = crate::update::take_update_failure().unwrap_or_else(|error| Some(error));
            let result = crate::update::check_for_update();
            let _ = update_tx.send(crate::update::UpdateCheck {
                available: result,
                previous_failure,
            });
            Self::request_root_repaint(&update_ctx);
        });
    }

    fn handle_event(&mut self, ctx: &egui::Context) {
        match self.socket_rx.try_recv() {
            Ok(asset) => {
                self.state.set_primary_asset(Arc::new(asset));
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {}
        }

        match self.statistics_rx.try_recv() {
            Ok(updates) => {
                let mut is_pending_update = false;
                for result in updates {
                    match result.stat_type {
                        StatisticsType::PSNRRMSE => {
                            is_pending_update |= result.is_pending;
                            self.state.statistics.psnr_rmse.value.psnr = result.value[0];
                            self.state.statistics.psnr_rmse.value.rmse = result.value[1];
                            self.state.statistics.psnr_rmse.scope = Some(result.scope.clone());
                        }
                        StatisticsType::SSIM => {
                            is_pending_update |= result.is_pending;
                            self.state.statistics.ssim.value = result.value[0];
                            self.state.statistics.ssim.scope = Some(result.scope.clone());
                        }
                        StatisticsType::MinMax => {
                            is_pending_update |= result.is_pending;
                            self.state.statistics.min_max.value.min = result.value.iter().step_by(2).cloned().collect();
                            self.state.statistics.min_max.value.max =
                                result.value.iter().skip(1).step_by(2).cloned().collect();
                            self.state.statistics.min_max.scope = Some(result.scope.clone());
                        }
                        _ => {}
                    }
                }

                if is_pending_update {
                    self.update_statistics();
                }
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {}
        }

        loop {
            match self.control_rx.try_recv() {
                Ok(paths) => {
                    for path in paths {
                        if let Err(err) = self.state.load_from_path(path.clone()) {
                            Self::load_fail(
                                &mut self.toasts,
                                "Failed to open externally requested file",
                                Some(&path),
                                &err,
                            );
                        }
                    }
                    if let Some(control_instance) = &self.control_instance {
                        if let Err(err) = control_instance.touch_active() {
                            eprintln!("Failed to update active window registry after external open: {err}");
                        } else {
                            self.last_control_touch = Instant::now();
                        }
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }

        // Non-blocking check for fallback fonts and apply once.
        if !self.fallback_font_applied {
            if let Some(rx) = &self.font_rx {
                match rx.try_recv() {
                    Ok(fallback_fonts) => {
                        apply_fallback_fonts(ctx, fallback_fonts);
                        self.fallback_font_applied = true;
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                    Err(mpsc::TryRecvError::Disconnected) => {
                        // Channel failed; avoid retry spam.
                        self.fallback_font_applied = true;
                    }
                }
            } else {
                self.fallback_font_applied = true; // No font to load.
            }
        }

        if let Some(rx) = &self.update_rx {
            match rx.try_recv() {
                Ok(check) => {
                    if let Some(error) = check.previous_failure {
                        eprintln!("{error}");
                        self.toasts.add_error(error);
                    }
                    match check.available {
                        Ok(Some(update)) => {
                            if !self.update_toast_shown {
                                self.toasts.add_info(format!("Update available: {}", update.version));
                                self.update_toast_shown = true;
                            }
                            self.update_status = UpdateStatus::Available(update);
                        }
                        Ok(None) => {
                            self.update_status = UpdateStatus::UpToDate;
                        }
                        Err(err) => {
                            eprintln!("Failed to check for updates: {err}");
                            self.toasts.add_error(err.clone());
                            self.update_status = UpdateStatus::Error(err);
                        }
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    if self.update_status == UpdateStatus::Checking {
                        self.update_status = UpdateStatus::Error("Update check disconnected".to_string());
                    }
                }
            }
        }

        if let Some(rx) = &self.startup_path_rx {
            let mut should_clear_rx = false;

            loop {
                match rx.try_recv() {
                    Ok(StartupPathLoadResult::Loaded { path, hash, image }) => {
                        self.state.apply_loaded_file_asset(path, hash, image);
                    }
                    Ok(StartupPathLoadResult::Reused { path, hash }) => {
                        self.state.set_file_asset_primary_by_hash_and_path(&hash, &path);
                    }
                    Ok(StartupPathLoadResult::Failed { path, error }) => {
                        Self::load_fail(&mut self.toasts, "Failed to load image", Some(&path), &error);
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        should_clear_rx = true;
                        break;
                    }
                }
            }

            if should_clear_rx {
                self.startup_path_rx = None;
            }
        }
    }

    fn handle_window_commands(&mut self, ctx: &egui::Context) {
        // Commit only when the UI can immediately request closure and return
        // from this frame, rather than while a native dialog is blocking it.
        if let Some(rx) = &self.update_apply_rx {
            match rx.try_recv() {
                Ok(result) => match result.and_then(crate::update::PreparedUpdate::commit) {
                    Ok(message) => {
                        self.update_status = UpdateStatus::Applying(message);
                        self.close_for_update = true;
                    }
                    Err(err) => {
                        eprintln!("Failed to apply update: {err}");
                        self.toasts.add_error(err.clone());
                        self.update_status = UpdateStatus::Error(err);
                    }
                },
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    if matches!(self.update_status, UpdateStatus::Applying(_)) {
                        self.update_status = UpdateStatus::Error("Update worker disconnected".to_string());
                    }
                }
            }
        }

        if self.close_for_update {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        if self.state.path != self.last_path {
            self.last_path = self.state.path.clone();
            if let Some(p) = &self.state.path {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("{} - edolview", p.display()).to_owned()));
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title("edolview".to_owned()));
            }
        }

        if ctx.input_mut(|i| i.consume_shortcut(&crate::res::FULLSCREEN_TOGGLE)) {
            let cur_full = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!cur_full));
        }
    }

    fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }

        let mut request_save = false;
        let mut apply_view_preset = None;
        let mut save_view_preset = None;
        let mut toggle_bookmark_panel = false;
        let mut add_bookmark = false;
        let mut navigate_prev_bookmark = false;
        let mut navigate_next_bookmark = false;
        let mut open_from_clipboard = false;
        ctx.input_mut(|i| {
            for slot in 0..crate::settings::VIEW_PRESET_COUNT {
                if i.consume_shortcut(&crate::res::PRESET_SAVE_SHORTCUTS[slot]) {
                    save_view_preset = Some(slot);
                    break;
                }
                if i.consume_shortcut(&crate::res::PRESET_APPLY_SHORTCUTS[slot]) {
                    apply_view_preset = Some(slot);
                    break;
                }
            }
            if i.consume_shortcut(&crate::res::SELECT_ALL_SC) {
                if let Some(asset) = &self.state.asset {
                    let spec = asset.image().spec();
                    let img_rect = Recti::from_min_size(vec2i(0, 0), vec2i(spec.width, spec.height));
                    self.state.marquee_rect = img_rect;
                    self.tmp_marquee_rect = img_rect;
                    self.marquee_rect_text = img_rect.to_string();
                }
            }
            if i.consume_shortcut(&crate::res::SELECT_NONE_SC) {
                self.state.reset_marquee_rect();
            }
            request_save |= i.consume_shortcut(&crate::res::SAVE_IMAGE_SC);
            toggle_bookmark_panel |= i.consume_shortcut(&crate::res::BOOKMARK_PANEL_TOGGLE);
            add_bookmark |= i.consume_shortcut(&crate::res::BOOKMARK_ADD);
            navigate_prev_bookmark |= i.consume_shortcut(&crate::res::BOOKMARK_PREV);
            navigate_next_bookmark |= i.consume_shortcut(&crate::res::BOOKMARK_NEXT);
            open_from_clipboard |= i.consume_shortcut(&crate::res::OPEN_FROM_CLIPBOARD_SC)
                || Self::open_from_clipboard_shortcut_released(i);
            if i.consume_shortcut(&crate::res::NAVIGATE_PREV) {
                if let Err(e) = self.state.navigate_prev() {
                    let path = self.state.file_nav.navigate_prev();
                    Self::load_fail(&mut self.toasts, "Failed to load navigated file", path.as_ref(), &e);
                }
            }
            if i.consume_shortcut(&crate::res::NAVIGATE_NEXT) {
                if let Err(e) = self.state.navigate_next() {
                    let path = self.state.file_nav.navigate_next();
                    Self::load_fail(&mut self.toasts, "Failed to load navigated file", path.as_ref(), &e);
                }
            }
            if i.consume_shortcut(&crate::res::NAVIGATE_ASSET_PREV) {
                self.state.navigate_asset_prev();
            }
            if i.consume_shortcut(&crate::res::NAVIGATE_ASSET_NEXT) {
                self.state.navigate_asset_next();
            }
            if i.consume_shortcut(&crate::res::RESET_VIEW) {
                self.viewer.reset_view();
            }
            if i.consume_shortcut(&crate::res::ZOOM_IN) {
                self.viewer.zoom_in(1.0, None);
            }
            if i.consume_shortcut(&crate::res::ZOOM_OUT) {
                self.viewer.zoom_in(-1.0, None);
            }
        });
        if request_save && self.state.asset.is_some() {
            self.request_viewer_image_save(ctx);
        }
        if let Some(slot) = save_view_preset {
            self.save_view_preset(slot);
            ctx.request_repaint();
        } else if let Some(slot) = apply_view_preset {
            self.apply_view_preset(slot, ctx);
        }
        if toggle_bookmark_panel {
            self.show_bookmarks_modal = !self.show_bookmarks_modal;
            ctx.request_repaint();
        } else if add_bookmark {
            self.toggle_bookmark();
            ctx.request_repaint();
        } else if navigate_prev_bookmark {
            self.navigate_bookmark(-1, ctx);
        } else if navigate_next_bookmark {
            self.navigate_bookmark(1, ctx);
        }
        if open_from_clipboard {
            self.open_from_clipboard();
        }
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        if dropped_files.is_empty() {
            return;
        }

        for f in dropped_files {
            if let Some(path) = f.path {
                match self.state.load_from_path(path.clone()) {
                    Ok(_) => {}
                    Err(e) => {
                        Self::load_fail(&mut self.toasts, "Failed to load dropped file", Some(&path), &e);
                    }
                }
            }
        }
    }

    fn run_logic(&mut self, ctx: &egui::Context) {
        #[cfg(debug_assertions)]
        let _timer = ScopedTimer::new("ui.app.logic");

        self.poll_pending_image_save_dialog(ctx);

        ctx.set_visuals(Visuals::dark());

        if !self.is_start_background_event_handlers_called {
            self.start_background_event_handlers(ctx);
            self.is_start_background_event_handlers_called = true;
        }

        self.start_startup_path_loading(ctx);
        self.handle_event(ctx);
        self.refresh_control_registration(ctx);
        self.handle_window_commands(ctx);

        if self.close_for_update {
            return;
        }

        self.handle_global_shortcuts(ctx);
        self.handle_dropped_files(ctx);

        self.state.validate_marquee_rect();
        self.state.process_watcher_events();
    }

    fn paint_asset_drag_preview(&self, ctx: &egui::Context) {
        let Some(pointer_pos) = ctx.pointer_interact_pos() else {
            return;
        };
        let Some(payload) = egui::DragAndDrop::payload::<String>(ctx) else {
            return;
        };
        let Some(asset) = self.state.assets.get(payload.as_str()) else {
            return;
        };

        egui::Area::new(egui::Id::new("asset_drag_preview"))
            .order(egui::Order::Tooltip)
            .interactable(false)
            .fixed_pos(pointer_pos + vec2(16.0, 16.0))
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.label(asset.name());
                });
            });
    }
}

impl eframe::App for ViewerApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.run_logic(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        #[cfg(debug_assertions)]
        let _timer = ScopedTimer::new("ui.app.ui");

        let ctx = ui.ctx().clone();

        if let Some(render_state) = frame.wgpu_render_state() {
            crate::model::install_gpu_compute(
                &render_state.device,
                &render_state.queue,
                render_state.adapter.get_info().backend,
            );
        }

        if self.close_for_update {
            return;
        }

        // Handle drag & drop events (files) at the start of frame
        // Show a visual hint while hovering
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            egui::Area::new(egui::Id::new("file_hover_overlay"))
                .fixed_pos(egui::pos2(16.0, 16.0))
                .show(&ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.add(egui::Label::new("Release to open image...").wrap_mode(egui::TextWrapMode::Extend));
                        for f in ctx.input(|i| i.raw.hovered_files.clone()) {
                            if let Some(path) = f.path {
                                ui.add(
                                    egui::Label::new(path.display().to_string()).wrap_mode(egui::TextWrapMode::Extend),
                                );
                            }
                        }
                    });
                });
        }

        egui::Panel::top("top").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open...").clicked() {
                        ui.close();
                        let supported_extensions = crate::supported_image::supported_image_extensions();
                        if let Some(path) = FileDialog::new()
                            .add_filter("Images", supported_extensions.as_slice())
                            .pick_file()
                        {
                            match self.state.load_from_path(path.clone()) {
                                Ok(_) => self.viewer.reset_view(),
                                Err(e) => Self::load_fail(&mut self.toasts, "Failed to open file", Some(&path), &e),
                            }
                        }
                    }

                    if ui
                        .button(format!(
                            "Open from clipboard ({})",
                            crate::res::OPEN_FROM_CLIPBOARD_SC.format_sys()
                        ))
                        .on_hover_text("Load image from clipboard")
                        .clicked()
                    {
                        self.open_from_clipboard();
                    }

                    if ui.button("Exit").clicked() {
                        ui.close();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.separator();
                if ui.button("Reset").on_hover_text("Reset zoom and pan to original").clicked() {
                    self.viewer.reset_view();
                }

                if ui.button("Fit").on_hover_text("Fit marquee to view").clicked() {
                    let rect = self.state.marquee_rect.validate();
                    if rect.empty() {
                        if let Some(asset) = &self.state.asset {
                            let spec = asset.image().spec();
                            let img_rect = Recti::from_min_size(vec2i(0, 0), vec2i(spec.width, spec.height));
                            self.viewer.fit_rect(img_rect);
                        }
                    } else {
                        self.viewer.fit_rect(rect);
                    }
                }
                if ui.button("Center").on_hover_text("Center marquee in view").clicked() {
                    let rect = self.state.marquee_rect.validate();
                    if rect.empty() {
                        if let Some(asset) = &self.state.asset {
                            let spec = asset.image().spec();
                            let img_rect = Recti::from_min_size(vec2i(0, 0), vec2i(spec.width, spec.height));
                            self.viewer.center_rect(img_rect);
                        }
                    } else {
                        self.viewer.center_rect(rect);
                    }
                }

                ui.separator();
                ui.toggle_icon(
                    &mut self.state.is_show_pixel_value,
                    self.icons.get_show_pixel_value(&ctx),
                    "Show Pixel Value (Zoom in to see values)",
                );
                ui.toggle_icon(
                    &mut self.state.is_show_crosshair,
                    self.icons.get_show_crosshair(&ctx),
                    "Show Crosshair",
                );
                let background_button = ui
                    .add(egui::Button::image(self.icons.get_background(
                        &ctx,
                        self.state.background_style,
                        ui.visuals(),
                    )))
                    .on_hover_text(format!(
                        "Background: {}\nClick to choose a background; scroll to change it.",
                        self.state.background_style.label()
                    ));
                background_button.hover_scroll(
                    ui,
                    &crate::settings::BackgroundStyle::ALL,
                    &mut self.state.background_style,
                    false,
                );
                egui::Popup::menu(&background_button).show(|ui| {
                    for style in crate::settings::BackgroundStyle::ALL {
                        if ui
                            .add(
                                egui::Button::image_and_text(
                                    self.icons.get_background(&ctx, style, ui.visuals()),
                                    style.label(),
                                )
                                .selected(self.state.background_style == style),
                            )
                            .clicked()
                        {
                            self.state.background_style = style;
                            ui.close();
                        }
                    }
                });

                ui.visuals_mut().override_text_color = Some(ui.visuals().weak_text_color());
                let socket_address = self.state.socket_info.lock().unwrap().address.clone();
                ui.label(socket_address.clone())
                    .on_hover_text("Socket Listener Address")
                    .context_menu(|ui| {
                        if ui.button("Copy Address").clicked() {
                            arboard::Clipboard::new()
                                .and_then(|mut cb| cb.set_text(socket_address))
                                .unwrap_or_else(|e| {
                                    eprintln!("Failed to copy socket address to clipboard: {e}");
                                });
                            ui.close();
                        }
                    });
                ui.visuals_mut().override_text_color = None;
                ui.indicator_icon(
                    self.state.socket_state.is_socket_receiving.load(Ordering::Relaxed),
                    self.icons.get_downloading(&ctx),
                    "Image Receiving",
                );

                // Right end: panel visibility toggles
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let settings_button = ui.button("⚙").on_hover_text("Settings");
                    if settings_button.clicked() {
                        self.show_settings_modal = true;
                    }
                    if matches!(self.update_status, UpdateStatus::Available(_)) {
                        let indicator_center = settings_button.rect.right_bottom() + egui::vec2(-4.0, -4.0);
                        ui.painter().circle_filled(indicator_center, 2.0, UPDATE_ACCENT_TEXT);
                    }

                    ui.toggle_value(&mut self.state.is_show_statusbar, "Status Bar");
                    ui.toggle_value(&mut self.state.is_show_sidebar, "Sidebar");
                    ui.toggle_value(&mut self.show_bookmarks_modal, "Bookmarks")
                        .on_hover_text(format!(
                            "Show bookmark panel ({})",
                            crate::res::BOOKMARK_PANEL_TOGGLE.format_sys()
                        ));
                });
            });
        });

        self.show_update_confirmation_dialog(&ctx);
        self.show_update_progress_dialog(&ctx);
        self.show_settings_dialog(&ctx);
        self.show_bookmarks_dialog(&ctx);

        if self.state.is_show_statusbar {
            egui::Panel::bottom("bottom").show(ui, |ui| {
                ui.columns_sized(
                    [
                        Size::exact(400.0),
                        Size::exact(160.0),
                        Size::exact(40.0),
                        Size::remainder(1.0),
                        Size::exact(108.0),
                    ],
                    |columns| {
                        columns[0].vertical(|ui| {
                            if let Some(asset) = self.active_display_asset() {
                                let cursor_image = asset.image();
                                let spec = cursor_image.spec();
                                let dtype = cursor_image.spec().dtype;

                                let cursor_color = if let Ok(pixel) = cursor_image.get_pixel_at(
                                    self.state.cursor_pos.map_or(-1, |p| p.x),
                                    self.state.cursor_pos.map_or(-1, |p| p.y),
                                ) {
                                    pixel.iter().cloned().collect()
                                } else {
                                    vec![0.0; spec.channels as usize]
                                };
                                ui.label_with_colored_rect(cursor_color, dtype);

                                let rect = self.state.marquee_rect;
                                let rect_image = asset.image();
                                let mean_color = if let Ok(mean) = rect_image.mean_value_in_rect(rect, MeanDim::All) {
                                    mean.iter().map(|&v| v as f32).collect()
                                } else {
                                    vec![0.0; rect_image.spec().channels as usize]
                                };
                                ui.label_with_colored_rect(mean_color, dtype);
                            }
                        });

                        columns[1].vertical(|ui| {
                            if let Some(cursor_pos) = self.state.cursor_pos {
                                ui.label(format!("Cursor: {}, {}", cursor_pos.x, cursor_pos.y));
                            } else {
                                ui.label("Cursor: -, -");
                            }

                            ui.text_edit_value_capture(
                                &mut self.marquee_rect_text,
                                &mut self.state.marquee_rect,
                                &mut self.tmp_marquee_rect,
                            )
                            .on_hover_text("Selected marquee rectangle bounds (x, y, width, height)")
                            .context_menu(|ui| {
                                if ui.button("Copy Numpy Indexing").clicked() {
                                    let rect = self.state.marquee_rect.validate();
                                    let np_indexing =
                                        format!("{}:{}, {}:{}", rect.min.y, rect.max.y, rect.min.x, rect.max.x,);
                                    arboard::Clipboard::new()
                                        .and_then(|mut cb| cb.set_text(np_indexing))
                                        .unwrap_or_else(|e| {
                                            eprintln!("Failed to copy numpy indexing to clipboard: {e}");
                                        });
                                    ui.close();
                                }
                                if ui.button("Copy x1, y1, x2, y2").clicked() {
                                    let rect = self.state.marquee_rect.validate();
                                    let rect_str =
                                        format!("{}, {}, {}, {}", rect.min.x, rect.min.y, rect.max.x, rect.max.y);
                                    arboard::Clipboard::new()
                                        .and_then(|mut cb| cb.set_text(rect_str))
                                        .unwrap_or_else(|e| {
                                            eprintln!("Failed to copy rect to clipboard: {e}");
                                        });
                                    ui.close();
                                }
                            });
                        });

                        columns[2].vertical(|ui| {
                            ui.label("");
                            let unit = self.app_settings.ui_state.angle_display_unit;
                            let angle = marquee_angle(self.state.marquee_rect, unit);
                            let angle_label = format_marquee_angle(angle);
                            let mut show_radians = unit == crate::settings::AngleDisplayUnit::Radians;
                            ui.label(angle_label)
                                .on_hover_text(marquee_angle_tooltip(unit))
                                .context_menu(|ui| {
                                    if ui.checkbox(&mut show_radians, "Show in radians").changed() {
                                        self.app_settings.ui_state.angle_display_unit = if show_radians {
                                            crate::settings::AngleDisplayUnit::Radians
                                        } else {
                                            crate::settings::AngleDisplayUnit::Degrees
                                        };
                                        ui.close();
                                    }
                                });
                        });

                        columns[4].with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                            if let Some(asset) = &self.state.asset {
                                let spec = asset.image().spec();
                                ui.add(
                                    egui::Label::new(format!("{}×{} | {}", spec.width, spec.height, spec.dtype.name()))
                                        .extend(),
                                );
                            } else {
                                ui.label("No image loaded");
                            }
                            ui.label(format!("Zoom: {:.2}x", self.viewer.zoom()));
                        });
                    },
                );
            });
        }

        if self.state.is_show_sidebar {
            egui::Panel::right("right")
                .resizable(true)
                .size_range(Rangef::new(240.0, f32::INFINITY))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("View Settings");
                    });
                    ui.style_mut().spacing.slider_rail_height = 4.0;

                    let channels = self.state.asset.as_ref().map(|a| a.image().spec().channels).unwrap_or(0);
                    let is_mono = self.state.channel_index != -1 || channels == 1;

                    ui.horizontal(|ui| {
                        let sizes = if is_mono {
                            ui.calc_sizes([Size::exact(58.0), Size::remainder(1.0), Size::exact(0.0)])
                        } else {
                            ui.calc_sizes([Size::exact(58.0), Size::remainder(1.0), Size::exact(54.0)])
                        };
                        ui.spacing_mut().combo_width = sizes[0];
                        let channel_values: Vec<i32> = if channels > 1 {
                            (-1..channels).collect()
                        } else {
                            vec![-1]
                        };
                        ui.add_enabled_ui(channels > 1, |ui| {
                            egui::ComboBox::from_id_salt("channel_index")
                                .combo_i32_with(ui, &mut self.state.channel_index, &channel_values, |v| match v {
                                    -1 => {
                                        if channels > 1 {
                                            "Color".to_string()
                                        } else {
                                            "Mono".to_string()
                                        }
                                    }
                                    0 => "Red".to_string(),
                                    1 => "Green".to_string(),
                                    2 => "Blue".to_string(),
                                    3 => "Alpha".to_string(),
                                    _ => format!("C{}", v),
                                })
                                .response
                                .on_hover_text("Channel to display");
                        });

                        ui.spacing_mut().combo_width = sizes[1];
                        if is_mono {
                            egui::ComboBox::from_id_salt("colormap_mono").combo(
                                ui,
                                &mut self.state.colormap_mono,
                                &self.state.colormap_mono_list,
                            )
                        } else {
                            egui::ComboBox::from_id_salt("colormap_rgb").combo(
                                ui,
                                &mut self.state.colormap_rgb,
                                &self.state.colormap_rgb_list,
                            )
                        }
                        .response
                        .on_hover_text("Colormap");

                        if !is_mono {
                            ui.allocate_ui(vec2(sizes[2], ui.spacing().interact_size.y), |ui| {
                                ui.checkbox(&mut self.state.shader_params.use_alpha, "Alpha");
                            });
                        }
                    });

                    ui.separator();
                    ui.checkbox(&mut self.state.shader_params.use_per_channel, "Per-channel controls");

                    if let Some(asset) = &self.state.asset {
                        let image = asset.image();
                        ui.vertical(|ui| {
                            let mut style: egui::Style = ui.style().as_ref().clone();
                            egui::containers::menu::menu_style(&mut style);
                            ui.set_style(std::sync::Arc::new(style));

                            if self.state.shader_params.use_per_channel {
                                for i in 0..channels {
                                    ui.horizontal(|ui| {
                                        let label = match (channels, i) {
                                            (1, 0) => "C".to_string(),
                                            (_, 0) => "R".to_string(),
                                            (_, 1) => "G".to_string(),
                                            (_, 2) => "B".to_string(),
                                            _ => "A".to_string(),
                                        };
                                        ui.label(label.to_string());

                                        display_controls_ui(
                                            ui,
                                            &self.icons,
                                            image,
                                            i,
                                            &mut self.state.shader_params.scale_mode_channels[i as usize],
                                            &mut self.state.shader_params.auto_minmax_channels[i as usize],
                                            &mut self.state.shader_params.min_v_channels[i as usize],
                                            &mut self.state.shader_params.max_v_channels[i as usize],
                                        );
                                    });
                                }
                            } else {
                                ui.horizontal(|ui| {
                                    display_controls_ui(
                                        ui,
                                        &self.icons,
                                        image,
                                        -1,
                                        &mut self.state.shader_params.scale_mode,
                                        &mut self.state.shader_params.auto_minmax,
                                        &mut self.state.shader_params.min_v,
                                        &mut self.state.shader_params.max_v,
                                    );
                                });
                            }
                        });
                    }

                    ui.separator();

                    display_profile_slider(ui, &mut self.state.shader_params.offset, -5.0, 5.0, 0.0, "Offset")
                        .on_hover_text("Add a constant offset to the displayed values.");
                    display_profile_slider(ui, &mut self.state.shader_params.exposure, -5.0, 5.0, 0.0, "Exposure")
                        .on_hover_text("Adjust brightness in exposure stops.");
                    display_profile_slider(ui, &mut self.state.shader_params.gamma, 0.1, 5.0, 1.0, "Gamma")
                        .on_hover_text("Apply gamma correction to the display.");

                    let desired_size_plot = egui::vec2(ui.available_width(), 100.0);
                    if let Some(asset) = self.active_display_asset() {
                        let rect = self.state.marquee_rect;
                        let mean_dim = match self.plot_dim {
                            PlotDim::Auto => {
                                if rect.width() >= rect.height() {
                                    MeanDim::Column
                                } else {
                                    MeanDim::Row
                                }
                            }
                            PlotDim::Column => MeanDim::Column,
                            PlotDim::Row => MeanDim::Row,
                        };
                        let reduced_value = asset
                            .image()
                            .mean_value_in_rect(rect, mean_dim)
                            .expect("Failed to compute mean in rect");
                        let (position_label, position_offset) = match mean_dim {
                            MeanDim::Column => ("x", rect.min.x),
                            MeanDim::Row => ("y", rect.min.y),
                            MeanDim::All => ("x", 0),
                        };

                        let channels = asset.image().spec().channels as usize;
                        let mut plot_data = Vec::with_capacity(channels);

                        for i in 0..channels {
                            let mut channel_data = Vec::with_capacity(reduced_value.len() / channels);
                            for j in 0..(reduced_value.len() / channels) {
                                channel_data.push(reduced_value[j * channels + i]);
                            }
                            plot_data.push(channel_data);
                        }

                        if channels > 1 {
                            let plot_refs = (0..channels).map(|i| plot_data[i].as_slice()).collect::<Vec<_>>();
                            if let Some(export) = draw_multi_line_plot(
                                ui,
                                desired_size_plot,
                                SeriesRef::new(&plot_refs),
                                &self.show_plot_channels,
                                asset.image().spec().dtype.alpha(),
                                position_label,
                                position_offset,
                            ) {
                                self.handle_export_action(export);
                            }
                        } else {
                            let plot_refs = vec![plot_data[0].as_slice()];
                            if let Some(export) = draw_multi_line_plot(
                                ui,
                                desired_size_plot,
                                SeriesRef::new(&plot_refs),
                                &[true],
                                asset.image().spec().dtype.alpha(),
                                position_label,
                                position_offset,
                            ) {
                                self.handle_export_action(export);
                            }
                        }

                        ui.horizontal(|ui| {
                            let mut style: egui::Style = ui.style().as_ref().clone();
                            egui::containers::menu::menu_style(&mut style);
                            ui.set_style(std::sync::Arc::new(style));

                            ui.radio_icon(
                                &mut self.plot_dim,
                                PlotDim::Auto,
                                self.icons.get_reduce_auto(&ctx),
                                "Auto (Mean Shorter Side)",
                            );
                            ui.radio_icon(
                                &mut self.plot_dim,
                                PlotDim::Column,
                                self.icons.get_reduce_column(&ctx),
                                "Mean Column",
                            );
                            ui.radio_icon(
                                &mut self.plot_dim,
                                PlotDim::Row,
                                self.icons.get_reduce_row(&ctx),
                                "Mean Row",
                            );

                            if channels > 1 {
                                channel_toggle_ui(ui, &mut self.show_plot_channels, channels);
                            }
                        });
                    } else {
                        ui.allocate_ui_with_layout(
                            desired_size_plot,
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.label("No data to display.");
                            },
                        );
                    }

                    ui.separator();

                    ui.checkbox(&mut self.show_histogram, "Show Histogram")
                        .on_hover_text("Show the histogram of the current image.");

                    if self.show_histogram {
                        let desired_size = egui::vec2(ui.available_width(), 100.0);
                        if let Some(asset) = &self.state.asset {
                            let hist = asset.image().hist();
                            let max = hist.iter().flatten().copied().fold(f32::NAN, f32::max);

                            if !hist.is_empty() {
                                let display_hist: Vec<&[f32]> = hist.iter().map(Vec::as_slice).collect();

                                if channels > 1 {
                                    if let Some(export) = draw_histogram(
                                        ui,
                                        desired_size,
                                        SeriesRef::new(&display_hist),
                                        &self.show_histogram_channels,
                                        max,
                                    ) {
                                        self.handle_export_action(export);
                                    }
                                    channel_toggle_ui(ui, &mut self.show_histogram_channels, channels as usize);
                                } else {
                                    if let Some(export) =
                                        draw_histogram(ui, desired_size, SeriesRef::new(&display_hist), &[true], max)
                                    {
                                        self.handle_export_action(export);
                                    }
                                }
                            }
                        } else {
                            ui.allocate_ui_with_layout(
                                desired_size,
                                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                |ui| {
                                    ui.label("No data to display.");
                                },
                            );
                        }
                    }

                    ui.separator();

                    if ui
                        .checkbox(&mut self.show_statistics, "Show Statistics")
                        .on_hover_text("Show min/max and comparison metrics for the current selection.")
                        .clicked()
                        && self.show_statistics {
                        self.update_statistics();
                    }

                    if self.show_statistics {
                        let min_values = &self.state.statistics.min_max.value.min;
                        let max_values = &self.state.statistics.min_max.value.max;
                        let num_channels = min_values.len().min(max_values.len()).min(4);

                        if num_channels > 0 {
                            egui::Grid::new("statistics_min_max_grid")
                                .num_columns(num_channels + 1)
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label("Min:").on_hover_text("Minimum value per channel.");
                                    for (i, &min_value) in min_values.iter().take(num_channels).enumerate() {
                                        statistics_overlay_toggle(
                                            ui,
                                            &mut self.show_statistics_min_overlay_channels[i],
                                            min_value,
                                            STATISTICS_MIN_TOGGLE_FILL,
                                            "Highlight pixels matching this minimum value.".to_owned(),
                                        );
                                    }
                                    ui.end_row();

                                    ui.label("Max:").on_hover_text("Maximum value per channel.");
                                    for (i, &max_value) in max_values.iter().take(num_channels).enumerate() {
                                        statistics_overlay_toggle(
                                            ui,
                                            &mut self.show_statistics_max_overlay_channels[i],
                                            max_value,
                                            STATISTICS_MAX_TOGGLE_FILL,
                                            "Highlight pixels matching this maximum value.".to_owned(),
                                        );
                                    }
                                    ui.end_row();
                                });
                        }

                        if self.state.is_comparison() {
                            egui::Grid::new("statistics_metrics_grid").num_columns(2).striped(true).show(ui, |ui| {
                                ui.label("RMSE:");
                                ui.label(format!("{:.4}", self.state.statistics.psnr_rmse.value.rmse));
                                ui.end_row();

                                ui.label("PSNR:");
                                ui.label(format!("{:.4}", self.state.statistics.psnr_rmse.value.psnr));
                                ui.end_row();

                                // ui.label("SSIM:");
                                // ui.label(format!("{:.4}", self.state.statistics.ssim));
                                // ui.end_row();
                            });
                        }
                    }

                    ui.separator();

                    if self.state.is_comparison() {
                        ui.heading("Comparison");
                        let previous_comparison_mode = self.state.comparison_mode;
                        let mut comparison_mode_changed = false;
                        let mut comparison_changed = false;
                        ui.horizontal(|ui| {
                            comparison_mode_changed |= ui
                                .radio_value(&mut self.state.comparison_mode, ComparisonMode::Diff, "Diff")
                                .on_hover_text("Show the pixel-wise difference: primary - secondary.")
                                .changed();
                            comparison_mode_changed |= ui
                                .radio_value(&mut self.state.comparison_mode, ComparisonMode::Blend, "Blend")
                                .on_hover_text("Blend the primary and secondary images.")
                                .changed();
                            comparison_mode_changed |= ui
                                .radio_value(&mut self.state.comparison_mode, ComparisonMode::Split, "Split")
                                .on_hover_text("Show the primary and secondary images side by side with synchronized pan and zoom.")
                                .changed();
                        });
                        comparison_changed |= comparison_mode_changed;
                        if self.state.comparison_mode == ComparisonMode::Blend {
                            comparison_changed |= ui
                                .add(egui::Slider::new(&mut self.state.comparison_blend, 0.0..=1.0).text("Blend"))
                                .changed();
                        }
                        if comparison_changed {
                            self.state.update_asset();
                            if comparison_mode_changed
                                && previous_comparison_mode != ComparisonMode::Split
                                && self.state.comparison_mode == ComparisonMode::Split
                            {
                                if let Some(asset) = self.state.asset_primary.as_ref() {
                                    let spec = asset.image().spec();
                                    self.viewer.fit_split(spec.width, spec.height);
                                }
                            }
                        }
                        ui.separator();
                    }

                    ui.horizontal(|ui| {
                        ui.heading("Image List");
                        ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                            ui.button("Clear").clicked().then(|| {
                                self.state.assets.clear();
                                self.state.clear_asset();
                            });
                        });
                    });
                    let asset_primary_hash = self.state.asset_primary.as_ref().map(|asset| asset.hash().to_owned());
                    let asset_secondary_hash = self.state.asset_secondary.as_ref().map(|asset| asset.hash().to_owned());

                    egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
                        let list_clip_rect = ui.clip_rect();
                        let mut to_set_primary: Option<_> = None;
                        let mut to_set_secondary: Option<_> = None;
                        let mut deselect_secondary = false;
                        let mut to_remove: HashSet<_> = HashSet::new();
                        let mut to_retain: HashSet<_> = HashSet::new();
                        let mut reorder_request: Option<(String, usize)> = None;
                        let mut first_row_rect: Option<egui::Rect> = None;
                        let mut last_row_rect: Option<egui::Rect> = None;
                        let asset_rows: Vec<_> = self
                            .state
                            .assets
                            .iter()
                            .enumerate()
                            .map(|(index, (hash, asset))| (index, hash.clone(), asset.clone()))
                            .collect();

                        asset_rows.into_iter().for_each(|(asset_index, hash, asset)| {
                            let name = asset.name();
                            let available_width = ui.available_width();

                            let style = ui.style();
                            let font_id = style.text_styles.get(&egui::TextStyle::Button).cloned().unwrap_or_default();
                            let padding = style.spacing.button_padding.x * 2.0;
                            let target_width = (available_width - padding - 5.0).max(0.0);

                            // Truncate name with ellipsis if too long. Example: "very_long_filename.png" -> "...lename.png"
                            let display_name = ui.fonts_mut(|fonts| {
                                let name_width = fonts
                                    .layout_no_wrap(name.to_string(), font_id.clone(), Color32::WHITE)
                                    .rect
                                    .width();

                                if name_width <= target_width {
                                    name.to_string()
                                } else {
                                    let dots_width = fonts
                                        .layout_no_wrap("...".to_string(), font_id.clone(), Color32::WHITE)
                                        .rect
                                        .width();
                                    let content_width = target_width - dots_width;

                                    if content_width <= 0.0 {
                                        return "...".to_string();
                                    }

                                    let mut current_width = 0.0;
                                    let mut start_index = name.len();

                                    for (i, c) in name.char_indices().rev() {
                                        let char_width = fonts.glyph_width(&font_id, c) + 0.075;
                                        if current_width + char_width > content_width {
                                            break;
                                        }
                                        current_width += char_width;
                                        start_index = i;
                                    }
                                    format!("...{}", &name[start_index..])
                                }
                            });

                            let row = ui
                                .with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                                    let btn = if Some(hash.as_str()) == asset_primary_hash.as_deref() {
                                        ui.add(
                                            egui::Button::selectable(true, &display_name)
                                                .sense(egui::Sense::click_and_drag()),
                                        )
                                    } else if Some(hash.as_str()) == asset_secondary_hash.as_deref() {
                                        ui.style_mut().visuals.selection.bg_fill = ASSET_SECONDARY_SELECTION_FILL;
                                        ui.add(
                                            egui::Button::selectable(true, &display_name)
                                                .sense(egui::Sense::click_and_drag()),
                                        )
                                    } else {
                                        ui.add(
                                            egui::Button::selectable(false, &display_name)
                                                .sense(egui::Sense::click_and_drag()),
                                        )
                                    };
                                    btn.context_menu(|ui| {
                                        ui.visuals_mut().override_text_color = Some(DANGER_TEXT);
                                        if ui.button("Delete").clicked() {
                                            to_remove.insert(hash.clone());
                                            ui.close();
                                        }
                                        if ui.button("Delete Others").clicked() {
                                            to_retain.insert(hash.clone());
                                            ui.close();
                                        }
                                        ui.visuals_mut().override_text_color = None;

                                        ui.separator();
                                        let is_secondary = self
                                            .state
                                            .asset_secondary
                                            .as_ref()
                                            .is_some_and(|a| Arc::ptr_eq(a, &asset));
                                        let secondary_label = if is_secondary {
                                            "Clear Secondary Asset"
                                        } else {
                                            "Set as Secondary Asset (Ctrl + Click)"
                                        };
                                        if ui.button(secondary_label).clicked() {
                                            if is_secondary {
                                                deselect_secondary = true;
                                            } else {
                                                to_set_secondary = Some(asset.clone());
                                            }
                                            ui.close();
                                        }
                                        ui.separator();

                                        match asset.asset_type() {
                                            crate::model::AssetType::File => {
                                                if ui.button("Copy Path").clicked() {
                                                    let path = asset.name();
                                                    arboard::Clipboard::new()
                                                        .and_then(|mut cb| cb.set_text(path.to_string()))
                                                        .unwrap_or_else(|e| {
                                                            eprintln!("Failed to copy path to clipboard: {e}");
                                                        });
                                                    ui.close();
                                                }
                                                if ui.button("Reveal in File Explorer").clicked() {
                                                    let path = asset.name();
                                                    let path_buf = PathBuf::from(path);
                                                    if let Err(e) = opener::open(
                                                        path_buf.parent().unwrap_or_else(|| std::path::Path::new(".")),
                                                    ) {
                                                        eprintln!("Failed to open file explorer: {e}");
                                                    }
                                                    ui.close();
                                                }
                                            }
                                            crate::model::AssetType::Clipboard => {}
                                            _ => {}
                                        }
                                    });

                                    btn.dnd_set_drag_payload(hash.clone());

                                    if btn.clicked() {
                                        if ui.input(|i| i.modifiers.command) {
                                            // Ctrl/Cmd + Click: set secondary
                                            if self
                                                .state
                                                .asset_secondary
                                                .as_ref()
                                                .is_some_and(|a| Arc::ptr_eq(a, &asset))
                                            {
                                                deselect_secondary = true;
                                            } else {
                                                to_set_secondary = Some(asset.clone());
                                            }
                                        } else {
                                            // Normal click: set primary
                                            to_set_primary = Some(asset.clone());
                                        }
                                    }

                                    btn
                                })
                                .inner;
                            first_row_rect.get_or_insert(row.rect);
                            last_row_rect = Some(row.rect);

                            let pointer_pos = ui.ctx().pointer_interact_pos();
                            let insertion_index = pointer_pos
                                .map(|pos| {
                                    if pos.y < row.rect.center().y {
                                        asset_index
                                    } else {
                                        asset_index + 1
                                    }
                                })
                                .unwrap_or(asset_index + 1);

                            if row
                                .dnd_hover_payload::<String>()
                                .is_some_and(|payload| payload.as_ref().as_str() != hash.as_str())
                            {
                                let line_y = if insertion_index == asset_index {
                                    row.rect.top() - 1.0
                                } else {
                                    row.rect.bottom() + 2.0
                                };
                                let stroke = egui::Stroke::new(2.0, ui.visuals().selection.stroke.color);
                                ui.painter().line_segment(
                                    [
                                        pos2(row.rect.left() + 3.0, line_y),
                                        pos2(row.rect.right() - 3.0, line_y),
                                    ],
                                    stroke,
                                );
                            }

                            if let Some(payload) = row.dnd_release_payload::<String>() {
                                let dragged_hash = payload.as_ref().clone();
                                if dragged_hash != hash {
                                    reorder_request = Some((dragged_hash, insertion_index));
                                }
                            }
                        });

                        let pointer_pos = ui.ctx().pointer_interact_pos();
                        let pointer_in_list_x = pointer_pos
                            .is_some_and(|pos| pos.x >= list_clip_rect.left() && pos.x <= list_clip_rect.right());
                        let dragging_asset = egui::DragAndDrop::payload::<String>(ui.ctx());
                        let top_drop_active = dragging_asset.is_some()
                            && pointer_in_list_x
                            && first_row_rect.zip(pointer_pos).is_some_and(|(rect, pos)| pos.y < rect.top());
                        let bottom_drop_active = dragging_asset.is_some()
                            && pointer_in_list_x
                            && last_row_rect.zip(pointer_pos).is_some_and(|(rect, pos)| pos.y > rect.bottom());

                        if top_drop_active {
                            if let Some(rect) = first_row_rect {
                                let stroke = egui::Stroke::new(2.0, ui.visuals().selection.stroke.color);
                                ui.painter().line_segment(
                                    [
                                        pos2(rect.left() + 3.0, rect.top() - 1.0),
                                        pos2(rect.right() - 3.0, rect.top() - 1.0),
                                    ],
                                    stroke,
                                );
                            }
                        } else if bottom_drop_active {
                            if let Some(rect) = last_row_rect {
                                let stroke = egui::Stroke::new(2.0, ui.visuals().selection.stroke.color);
                                ui.painter().line_segment(
                                    [
                                        pos2(rect.left() + 3.0, rect.bottom() + 2.0),
                                        pos2(rect.right() - 3.0, rect.bottom() + 2.0),
                                    ],
                                    stroke,
                                );
                            }
                        }

                        if reorder_request.is_none() && ui.input(|i| i.pointer.any_released()) {
                            if top_drop_active {
                                if let Some(payload) = egui::DragAndDrop::take_payload::<String>(ui.ctx()) {
                                    reorder_request = Some((payload.as_ref().clone(), 0));
                                }
                            } else if bottom_drop_active {
                                if let Some(payload) = egui::DragAndDrop::take_payload::<String>(ui.ctx()) {
                                    reorder_request = Some((payload.as_ref().clone(), self.state.assets.len()));
                                }
                            }
                        }

                        if let Some(to_set_primary) = to_set_primary {
                            if self
                                .state
                                .asset_secondary
                                .as_ref()
                                .is_some_and(|a| Arc::ptr_eq(a, &to_set_primary))
                            {
                                deselect_secondary = true;
                            }

                            self.state.set_primary_asset(to_set_primary);
                        }
                        if let Some(to_set_secondary) = to_set_secondary {
                            self.state.set_secondary_asset(Some(to_set_secondary));
                        } else if deselect_secondary {
                            self.state.set_secondary_asset(None);
                        }

                        if to_retain.is_empty() {
                            self.state.assets.retain(|hash, _| !to_remove.contains(hash));
                        } else {
                            self.state.assets.retain(|hash, _| to_retain.contains(hash));
                        }

                        if let Some((hash, insertion_index)) = reorder_request {
                            self.state.reorder_asset_by_hash(&hash, insertion_index);
                        }

                        if let Some(asset) = &self.state.asset {
                            if asset.asset_type() != AssetType::Comparison
                                && !self.state.assets.contains_key(asset.hash())
                            {
                                self.state.clear_asset();
                            }
                        }
                    });
                });
        }

        self.paint_asset_drag_preview(&ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().inner_margin(0))
            .show(ui, |ui| {
                self.viewer.show_image(
                    ui,
                    frame,
                    &mut self.state,
                    self.show_statistics,
                    self.show_statistics_min_overlay_channels,
                    self.show_statistics_max_overlay_channels,
                );

                if self.viewer.take_save_dialog_request() {
                    self.request_viewer_image_save(&ctx);
                }

                for (is_success, message) in self.viewer.take_export_toasts() {
                    if is_success {
                        self.toasts.add_success(message);
                    } else {
                        self.toasts.add_error(message);
                    }
                }

                // Comparison notice overlay
                if let Some(message) = &self.state.comparison_notice {
                    egui::Area::new(egui::Id::new("comparison_notice_overlay"))
                        .order(egui::Order::Foreground)
                        .anchor(egui::Align2::LEFT_TOP, egui::vec2(6.0, 30.0))
                        .interactable(false)
                        .show(&ctx, |ui| {
                            let notice_color = if message.starts_with("Error:") {
                                NOTICE_ERROR_TEXT
                            } else {
                                NOTICE_WARNING_TEXT
                            };
                            ui.add(
                                egui::Label::new(egui::RichText::new(message).color(notice_color))
                                    .wrap_mode(egui::TextWrapMode::Extend),
                            );
                        });
                }

                #[cfg(debug_assertions)]
                if let Some(message) = self.statistics_rect_mismatch_message() {
                    egui::Area::new(egui::Id::new("statistics_rect_mismatch_overlay"))
                        .order(egui::Order::Foreground)
                        .anchor(egui::Align2::LEFT_TOP, egui::vec2(6.0, 54.0))
                        .interactable(false)
                        .show(&ctx, |ui| {
                            ui.add(
                                egui::Label::new(egui::RichText::new(message).color(NOTICE_WARNING_TEXT))
                                    .wrap_mode(egui::TextWrapMode::Extend),
                            );
                        });
                }

                if let Some(err) = self.viewer.take_shader_error() {
                    self.toasts.add_error(format!("Shader error: {err}"));
                }

                self.toasts.retain_active();
                ui.add(ToastUi::new(&mut self.toasts));
            });

        let request_copy = !ctx.egui_wants_keyboard_input()
            && !Self::egui_has_label_text_selection(&ctx)
            && (ctx.input(|i| i.events.iter().any(|event| matches!(event, egui::Event::Copy)))
                || ctx.input_mut(|i| i.consume_shortcut(&crate::res::COPY_SC)));
        if request_copy {
            self.viewer.request_copy(self.active_display_source_label().to_string());
            ctx.request_repaint();
        }

        // Marquee change detection & callbacks (run once per frame after updates)
        let current_rect = self.state.marquee_rect;
        let current_asset_hash = self.state.asset.as_ref().map(|a| a.hash().to_string());

        let rect_changed = current_rect != self.last_marquee_rect_for_cb;
        self.save_persistent_ui_state_if_needed();
        let content_changed = rect_changed || current_asset_hash != self.last_marquee_asset_hash;
        if rect_changed || content_changed {
            self.on_marquee_changed();
        }

        if rect_changed {
            self.last_marquee_rect_for_cb = current_rect;
        }
        if current_asset_hash != self.last_marquee_asset_hash {
            self.last_marquee_asset_hash = current_asset_hash;
        }

        // Debug window
        #[cfg(debug_assertions)]
        {
            crate::debug::debug_window(&ctx);
        }
    }
}
