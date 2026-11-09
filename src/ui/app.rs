use color_eyre::eyre::Report;
use eframe::egui::{self, pos2, vec2, Color32, Rangef};
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

use super::asset_list_selection::AssetListSelection;
use super::clipboard_input::{clipboard_action, translate_text_clipboard_shortcuts, ClipboardAction};
use super::image_loader::{ImageLoadRequest, ImageLoadResult, ImageLoader};
use super::status_bar::StatusBar;
use super::toolbar::{Toolbar, ToolbarAction};
#[cfg(debug_assertions)]
use crate::util::timer::ScopedTimer;
use crate::{
    model::{
        start_server_with_retry, AppState, AssetType, ComparisonMode, Image, MeanDim, Recti, SocketAsset,
        StatisticsScope, StatisticsType, StatisticsUpdate, StatisticsWorker,
    },
    res::{
        icons::Icons, KeyboardShortcutExt, ASSET_MULTI_SELECTION_FILL, ASSET_MULTI_SELECTION_HOVER_STROKE,
        ASSET_MULTI_SELECTION_STROKE, ASSET_SECONDARY_SELECTION_FILL, CONTROL_LISTENER_UNAVAILABLE_TEXT, DANGER_TEXT,
        NOTICE_ERROR_TEXT, NOTICE_WARNING_TEXT, RGB_CHANNEL_LABEL_COLORS, STATISTICS_MAX_TOGGLE_FILL,
        STATISTICS_MIN_TOGGLE_FILL, UPDATE_ACCENT_FILL, UPDATE_ACCENT_TEXT,
    },
    ui::{
        component::{
            channel_toggle_ui, colormap_popup_contents, display_controls_ui, display_profile_slider,
            draw_multi_line_plot,
            egui_ext::{ComboBoxExt, ResponseExt, Size, UiExt},
            range_slider_context_menu, range_slider_tooltip, range_slider_ui, show_roi_window, CopyExport,
            ExportAction, HistogramPanel, MonoColormapPreviews, RoiJumpMode, SaveExport, Toast, ToastUi, ToastsExt,
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
struct Roi {
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

fn statistics_value_text(ui: &egui::Ui, value: f64, width: f32) -> String {
    if !value.is_finite() {
        return "N/A".to_owned();
    }
    let scientific = value != 0.0 && (value.abs() < 0.0001 || value.abs() >= 10_000.0);
    let font = egui::TextStyle::Body.resolve(ui.style());
    for precision in (0..=4).rev() {
        let text = if scientific {
            format!("{value:.precision$e}")
        } else {
            format!("{value:.precision$}")
        };
        let fits = ui.fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(text.clone(), font.clone(), ui.visuals().text_color())
                .size()
                .x
                <= width
        });
        if fits {
            return text;
        }
    }
    format!("{value:.0e}")
}

fn statistics_overlay_toggle(
    ui: &mut egui::Ui,
    selected: &mut bool,
    value: f64,
    active_fill: Color32,
    hover_text: String,
    width: f32,
) -> egui::Response {
    if !value.is_finite() {
        *selected = false;
    }
    let label = statistics_value_text(ui, value, width);
    let mut text = egui::RichText::new(&label);
    if *selected {
        text = text.background_color(active_fill).color(Color32::WHITE);
    }
    // Clickable text keeps all four channels compact without button padding.
    let response = ui
        .add_enabled_ui(value.is_finite(), |ui| {
            ui.add_sized(
                vec2(width, ui.text_style_height(&egui::TextStyle::Body)),
                egui::Label::new(text).truncate().selectable(false).sense(egui::Sense::click()),
            )
        })
        .inner
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(if value.is_finite() {
            format!("{value}\n{hover_text}")
        } else {
            "No finite pixels in this channel.".to_owned()
        })
        .on_disabled_hover_text("No finite pixels in this channel.");
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::Role::Button,
            ui.is_enabled() && value.is_finite(),
            *selected,
            label.as_str(),
        )
    });
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
    mono_colormap_previews: MonoColormapPreviews,
    asset_list_selection: AssetListSelection,
    last_path: Option<PathBuf>,
    image_loader: ImageLoader,
    pending_navigation_path: Option<PathBuf>,
    status_bar: StatusBar,
    #[cfg(target_os = "windows")]
    native_chrome: Option<super::window_chrome::NativeChrome>,
    is_start_background_event_handlers_called: bool,

    // Marquee change callbacks
    last_marquee_rect_for_cb: Recti,
    last_marquee_asset_hash: Option<String>,
    rois: Vec<Roi>,
    active_roi_index: Option<usize>,
    scroll_to_new_roi: bool,
    roi_jump_mode: RoiJumpMode,

    show_plot_channels: [bool; 4],
    plot_percentiles: [f64; 2],

    show_histogram: bool,
    show_histogram_channels: [bool; 4],
    histogram_panel: HistogramPanel,

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

    update_rx: Option<mpsc::Receiver<crate::update::UpdateEvent>>,
    update_apply_rx: Option<mpsc::Receiver<Result<crate::update::PreparedUpdate, String>>>,
    update_status: UpdateStatus,
    update_toast_shown: bool,
    close_for_update: bool,
    pending_update_confirmation: Option<crate::update::AvailableUpdate>,
    app_settings: crate::settings::AppSettings,
    show_settings_modal: bool,
    show_rois_modal: bool,
    control_rx: mpsc::Receiver<crate::control::ControlOpenEvent>,
    control_instance: Option<crate::control::ControlInstance>,
    last_control_touch: Instant,
    was_focused_last_frame: bool,
    last_image_save_dir: Option<PathBuf>,
    pending_image_save_dialog: Option<PendingImageSaveDialog>,
    colormap_folder_rx: Option<mpsc::Receiver<Result<(), String>>>,
}

impl Drop for ViewerApp {
    fn drop(&mut self) {
        if let Some(mut control_instance) = self.control_instance.take() {
            if let Err(err) = control_instance.remove() {
                eprintln!("Failed to remove active window registration: {err}");
            }
            control_instance.shutdown();
        }
        if let Some(socket_server) = self.socket_server.as_mut() {
            if let Err(err) = socket_server.shutdown() {
                eprintln!("Failed to stop socket listener: {err}");
            }
        }
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
        state.is_show_minimap = persisted_ui_state.is_show_minimap;
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
        let (control_tx, control_rx, control_nx) = mpsc_with_notify::<crate::control::ControlOpenEvent>();

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
            mono_colormap_previews: MonoColormapPreviews::default(),
            asset_list_selection: AssetListSelection::default(),

            last_path: None,
            image_loader: ImageLoader::default(),
            pending_navigation_path: None,

            status_bar: StatusBar::new(marquee_rect),
            #[cfg(target_os = "windows")]
            native_chrome: None,
            is_start_background_event_handlers_called: false,

            last_marquee_rect_for_cb: marquee_rect,
            last_marquee_asset_hash: None,
            rois: Vec::new(),
            active_roi_index: None,
            scroll_to_new_roi: false,
            roi_jump_mode: RoiJumpMode::Center,

            show_plot_channels: [true, true, true, false],
            plot_percentiles: [0.0, 100.0],

            show_histogram: false,
            show_histogram_channels: [true, true, true, false],
            histogram_panel: HistogramPanel::default(),

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
            show_rois_modal: false,
            control_rx,
            control_instance,
            last_control_touch: Instant::now(),
            was_focused_last_frame: false,
            last_image_save_dir: None,
            pending_image_save_dialog: None,
            colormap_folder_rx: None,
        }
    }

    #[cfg(target_os = "windows")]
    pub fn with_window_chrome(mut self, cc: &eframe::CreationContext<'_>) -> Self {
        match super::window_chrome::NativeChrome::attach(cc) {
            Ok(chrome) => {
                self.native_chrome = Some(chrome);
                self.state.titlebar =
                    crate::model::TitlebarState::integrated(crate::res::icons::WINDOW_BUTTON_SIZE.x * 3.0);
            }
            Err(error) => {
                eprintln!("{error}");
                self.toasts.add_error(error.to_string());
            }
        }
        self
    }

    #[inline]
    pub fn with_paths(mut self, paths: Vec<PathBuf>) -> Self {
        if !paths.is_empty() {
            self.image_loader.enqueue(ImageLoadRequest::Files(paths));
        }
        self
    }

    fn start_image_loading(&mut self, ctx: &egui::Context) {
        self.image_loader.start(ctx, self.state.assets.keys().cloned());
    }

    fn open_file_paths(&mut self, paths: Vec<PathBuf>, ctx: &egui::Context) {
        self.image_loader.enqueue(ImageLoadRequest::Files(paths));
        self.start_image_loading(ctx);
        ctx.request_repaint();
    }

    fn navigate_file(&mut self, forward: bool, ctx: &egui::Context) {
        if !self
            .state
            .asset_primary
            .as_ref()
            .is_some_and(|asset| asset.asset_type() == AssetType::File)
        {
            return;
        }
        // Rapid arrow presses advance from the last requested file, even while
        // an older RAW is still decoding and updating the displayed asset.
        if let Some(path) = &self.pending_navigation_path {
            self.state.file_nav.select_index_for_path(path);
        }
        let path = if forward {
            self.state.file_nav.navigate_next()
        } else {
            self.state.file_nav.navigate_prev()
        };
        if let Some(path) = path {
            self.pending_navigation_path = Some(path.clone());
            self.open_file_paths(vec![path], ctx);
        }
    }

    fn load_fail(toasts: &mut Vec<Toast>, message: &str, path: Option<&PathBuf>, e: &Report) {
        eprintln!("{message}: {e}");
        let detail = path.map_or_else(|| e.to_string(), |path| path.display().to_string());
        toasts.add_error(format!("{message}: {detail}"));
    }

    fn open_from_clipboard(&mut self, ctx: &egui::Context) {
        self.image_loader.enqueue(ImageLoadRequest::Clipboard);
        self.start_image_loading(ctx);
        ctx.request_repaint();
    }

    // Native clipboard translation is disabled so Ctrl/Cmd+V arrives on key-down,
    // including when the clipboard contains only a file list. Never load on release.
    fn consume_open_from_clipboard_shortcut(input: &mut egui::InputState) -> bool {
        let legacy = crate::res::OPEN_FROM_CLIPBOARD_LEGACY_SC;
        let mut triggered = false;
        input.events.retain(|event| {
            if let egui::Event::Key {
                key,
                pressed: true,
                repeat,
                modifiers,
                ..
            } = event
            {
                if clipboard_action(*key, *modifiers) == Some(ClipboardAction::Paste)
                    || (*key == legacy.logical_key && modifiers.matches_exact(legacy.modifiers))
                {
                    // Consume auto-repeat too, but load only on the initial key-down.
                    triggered |= !repeat;
                    return false;
                }
            }
            if matches!(event, egui::Event::Paste(_) | egui::Event::PasteImage(_)) {
                triggered = true;
                return false;
            }
            true
        });
        triggered
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

    fn open_colormap_folder(&mut self, ctx: &egui::Context, is_mono: bool) {
        if self.colormap_folder_rx.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.colormap_folder_rx = Some(rx);
        let ctx = ctx.clone();
        thread::spawn(move || {
            // Installed Linux defaults are read-only; custom colormaps belong to the user.
            let directory = crate::util::path_ext::custom_colormap_dir().join(if is_mono { "mono" } else { "rgb" });
            let result = fs::create_dir_all(&directory)
                .map_err(|error| format!("Failed to create colormap folder '{}': {error}", directory.display()))
                .and_then(|()| {
                    opener::open(&directory)
                        .map_err(|error| format!("Failed to open colormap folder '{}': {error}", directory.display()))
                });
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }

    fn poll_colormap_folder_open(&mut self) {
        let Some(rx) = &self.colormap_folder_rx else {
            return;
        };
        let result = match rx.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => Err("Colormap folder worker disconnected".to_string()),
        };
        self.colormap_folder_rx = None;
        if let Err(error) = result {
            eprintln!("{error}");
            self.toasts.add_error(error);
        }
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
        ui_state.is_show_minimap = self.state.is_show_minimap;
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

    fn current_roi_rect(&self) -> Option<Recti> {
        let rect = self.state.marquee_rect.validate();
        if !rect.empty() {
            return Some(rect);
        }
        None
    }

    fn toggle_roi(&mut self) {
        let Some(rect) = self.current_roi_rect() else {
            self.toasts.add_info("No selection to add as ROI".to_string());
            return;
        };

        let roi = Roi { rect };
        if let Some(index) = self.rois.iter().position(|entry| entry == &roi) {
            self.remove_roi(index);
            self.toasts.add_success("Removed ROI".to_string());
            return;
        }

        self.rois.push(roi);
        self.active_roi_index = Some(self.rois.len() - 1);
        self.scroll_to_new_roi = true;
        self.toasts.add_success(format!("Added ROI {}", self.rois.len()));
    }

    fn remove_roi(&mut self, index: usize) {
        if index >= self.rois.len() {
            return;
        }

        if index + 1 == self.rois.len() {
            self.scroll_to_new_roi = false;
        }
        self.rois.remove(index);
        self.active_roi_index = match self.active_roi_index {
            Some(active) if active == index => None,
            Some(active) if active > index => Some(active - 1),
            active => active,
        };
    }

    fn move_roi(&mut self, index: usize, direction: i32) {
        if index >= self.rois.len() {
            return;
        }

        let target = if direction < 0 {
            index.saturating_sub(1)
        } else {
            (index + 1).min(self.rois.len().saturating_sub(1))
        };

        if target == index {
            return;
        }

        self.rois.swap(index, target);
        self.active_roi_index = self.active_roi_index.map(|active| {
            if active == index {
                target
            } else if active == target {
                index
            } else {
                active
            }
        });
    }

    fn clear_rois(&mut self) {
        self.rois.clear();
        self.active_roi_index = None;
        self.scroll_to_new_roi = false;
    }

    fn navigate_roi(&mut self, direction: i32, ctx: &egui::Context) {
        if self.rois.is_empty() {
            self.toasts.add_info("No ROIs".to_string());
            return;
        }

        let len = self.rois.len();
        let current = match self.active_roi_index {
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

        self.apply_roi(next, ctx);
    }

    fn apply_roi(&mut self, index: usize, ctx: &egui::Context) {
        let Some(roi) = self.rois.get(index).cloned() else {
            return;
        };
        let Some(asset) = &self.state.asset else {
            self.toasts.add_info("No image loaded".to_string());
            return;
        };
        let spec = asset.image().spec();
        let image_rect = Recti::from_min_size(vec2i(0, 0), vec2i(spec.width, spec.height));
        let requested_rect = roi.rect.validate();
        let clipped_rect = requested_rect.intersect(image_rect);

        self.state.set_marquee_rect(requested_rect);
        self.status_bar.reset_marquee_text(self.state.marquee_rect);
        if clipped_rect.empty() {
        } else {
            match self.roi_jump_mode {
                RoiJumpMode::None => {}
                RoiJumpMode::Center => self.viewer.center_rect(self.state.marquee_rect),
                RoiJumpMode::Fit => self.viewer.fit_rect(self.state.marquee_rect),
            }
        }

        self.active_roi_index = Some(index);
        ctx.request_repaint();
        if clipped_rect.empty() {
            self.toasts.add_info("The ROI is outside the current image bounds".to_string());
        } else if clipped_rect != requested_rect {
            self.toasts
                .add_info("The ROI is clipped to the current image bounds".to_string());
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
        let overlay_layer = egui::LayerId::new(egui::Order::Middle, egui::Id::unique("confirm_update_overlay"));
        ctx.layer_painter(overlay_layer)
            .rect_filled(screen_rect, 0.0, Color32::from_black_alpha(180));

        egui::Area::new(egui::Id::unique("confirm_update_modal"))
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

    fn show_rois_dialog(&mut self, ctx: &egui::Context) {
        let roi_rects: Vec<_> = self.rois.iter().map(|roi| roi.rect).collect();
        let actions = show_roi_window(
            ctx,
            &self.icons,
            &mut self.show_rois_modal,
            &roi_rects,
            self.active_roi_index,
            &mut self.scroll_to_new_roi,
            &mut self.roi_jump_mode,
            &crate::res::ROI_ADD.format_sys(),
        );

        if let Some(index) = actions.move_up {
            self.move_roi(index, -1);
        } else if let Some(index) = actions.move_down {
            self.move_roi(index, 1);
        }

        if let Some(index) = actions.remove_index {
            self.remove_roi(index);
        }

        if actions.clear_all {
            self.clear_rois();
        }

        if actions.add_current {
            self.toggle_roi();
        }

        if let Some(index) = actions.jump_to {
            self.apply_roi(index, ctx);
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
                .run_statistics(img.clone(), img.spec().dtype.alpha(), scope.clone());
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
            // Show the previous update outcome without waiting for the network check.
            let _ = update_tx.send(crate::update::UpdateEvent::PreviousResult(crate::update::take_update_result()));
            Self::request_root_repaint(&update_ctx);
            let result = crate::update::check_for_update();
            let _ = update_tx.send(crate::update::UpdateEvent::Checked(result));
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
                    if let Some(error) = &result.error {
                        self.toasts.add_error(error.clone());
                    }
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
                            self.state.statistics.min_max.value.min = result.value.iter().step_by(4).cloned().collect();
                            self.state.statistics.min_max.value.max =
                                result.value.iter().skip(1).step_by(4).cloned().collect();
                            self.state.statistics.min_max.value.variance =
                                result.value.iter().skip(2).step_by(4).cloned().collect();
                            self.state.statistics.min_max.value.std_dev =
                                result.value.iter().skip(3).step_by(4).cloned().collect();
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
                Ok(request) => {
                    if self.close_for_update || ctx.input(|i| i.viewport().close_requested()) {
                        continue;
                    }
                    let Some(paths) = request.accept() else {
                        continue;
                    };
                    // Acknowledge a responsive UI before decoding. Reuse the
                    // background loader so a large image cannot prevent the
                    // window from being shown or accepting the next request.
                    self.open_file_paths(paths, ctx);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    ctx.request_repaint();
                    if let Some(control_instance) = &self.control_instance {
                        if let Err(err) = control_instance.touch_active() {
                            eprintln!("Failed to update active window registry after external open: {err}");
                        } else {
                            self.last_control_touch = Instant::now();
                        }
                    }
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
            loop {
                match rx.try_recv() {
                    Ok(crate::update::UpdateEvent::PreviousResult(result)) => match result {
                        Ok(Some(crate::update::UpdateResult::Success(message))) => self.toasts.add_success(message),
                        Ok(Some(crate::update::UpdateResult::Failure(error))) | Err(error) => {
                            eprintln!("{error}");
                            self.toasts.add_error(error);
                        }
                        Ok(None) => {}
                    },
                    Ok(crate::update::UpdateEvent::Checked(result)) => match result {
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
                    },
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        if self.update_status == UpdateStatus::Checking {
                            self.update_status = UpdateStatus::Error("Update check disconnected".to_string());
                        }
                        break;
                    }
                }
            }
        }

        while let Some(result) = self.image_loader.poll() {
            let path = match &result {
                ImageLoadResult::FileLoaded { path, .. } | ImageLoadResult::FileReused { path, .. } => Some(path),
                ImageLoadResult::Failed { path, .. } => path.as_ref(),
                ImageLoadResult::ClipboardLoaded(_) => None,
            };
            if path.is_some() && self.pending_navigation_path.as_ref() == path {
                self.pending_navigation_path = None;
            }
            match result {
                ImageLoadResult::FileLoaded { path, hash, image } => {
                    self.state.apply_loaded_file_asset(path, hash, image)
                }
                ImageLoadResult::FileReused { path, hash } => {
                    self.state.set_file_asset_primary_by_hash_and_path(&hash, &path)
                }
                ImageLoadResult::ClipboardLoaded(image) => self.state.apply_loaded_clipboard_image(image),
                ImageLoadResult::Failed { path, error } => {
                    let message = if path.is_some() {
                        "Failed to load image"
                    } else {
                        "Failed to load image from clipboard"
                    };
                    Self::load_fail(&mut self.toasts, message, path.as_ref(), &error);
                }
            }
            Self::request_root_repaint(ctx);
        }
        self.start_image_loading(ctx);
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
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(p.display().to_string()));
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
        if !ctx.text_edit_focused() && ctx.input_mut(Self::consume_open_from_clipboard_shortcut) {
            self.open_from_clipboard(ctx);
        }
        if ctx.egui_wants_keyboard_input() {
            return;
        }

        let mut request_save = false;
        let mut apply_view_preset = None;
        let mut save_view_preset = None;
        let mut toggle_roi_panel = false;
        let mut add_roi = false;
        let mut navigate_prev_roi = false;
        let mut navigate_next_roi = false;
        let mut navigate_file = None;
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
                    self.status_bar.reset_marquee_text(img_rect);
                }
            }
            if i.consume_shortcut(&crate::res::SELECT_NONE_SC) {
                self.state.reset_marquee_rect();
            }
            request_save |= i.consume_shortcut(&crate::res::SAVE_IMAGE_SC);
            toggle_roi_panel |= i.consume_shortcut(&crate::res::ROI_PANEL_TOGGLE);
            add_roi |= i.consume_shortcut(&crate::res::ROI_ADD);
            navigate_prev_roi |= i.consume_shortcut(&crate::res::ROI_PREV);
            navigate_next_roi |= i.consume_shortcut(&crate::res::ROI_NEXT);
            if i.consume_shortcut(&crate::res::NAVIGATE_PREV) {
                navigate_file = Some(false);
            }
            if i.consume_shortcut(&crate::res::NAVIGATE_NEXT) {
                navigate_file = Some(true);
            }
            if i.consume_shortcut(&crate::res::NAVIGATE_ASSET_PREV)
                || i.consume_shortcut(&crate::res::NAVIGATE_ASSET_PREV_TAB)
            {
                self.state.navigate_asset_prev();
            }
            if i.consume_shortcut(&crate::res::NAVIGATE_ASSET_NEXT)
                || i.consume_shortcut(&crate::res::NAVIGATE_ASSET_NEXT_TAB)
            {
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
        if toggle_roi_panel {
            self.show_rois_modal = !self.show_rois_modal;
            ctx.request_repaint();
        } else if add_roi {
            self.toggle_roi();
            ctx.request_repaint();
        } else if navigate_prev_roi {
            self.navigate_roi(-1, ctx);
        } else if navigate_next_roi {
            self.navigate_roi(1, ctx);
        }
        if let Some(forward) = navigate_file {
            self.navigate_file(forward, ctx);
        }
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        if dropped_files.is_empty() {
            return;
        }

        self.open_file_paths(dropped_files.into_iter().map(|file| file.path().to_path_buf()).collect(), ctx);
    }

    fn run_logic(&mut self, ctx: &egui::Context) {
        #[cfg(debug_assertions)]
        let _timer = ScopedTimer::new("ui.app.logic");

        self.prepare_clipboard_input(ctx);
        self.poll_pending_image_save_dialog(ctx);
        self.poll_colormap_folder_open();

        super::style::apply(ctx);

        if !self.is_start_background_event_handlers_called {
            self.start_background_event_handlers(ctx);
            self.is_start_background_event_handlers_called = true;
        }

        self.start_image_loading(ctx);
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

        egui::Area::new(egui::Id::unique("asset_drag_preview"))
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

impl ViewerApp {
    fn prepare_clipboard_input(&mut self, ctx: &egui::Context) {
        let text_edit_focused = ctx.text_edit_focused();
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        let unicode_editor = self.viewer.ocr_text_has_focus(ctx);
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let unicode_editor = false;
        let error = ctx.input_mut(|input| {
            let error = translate_text_clipboard_shortcuts(&mut input.events, text_edit_focused, || {
                arboard::Clipboard::new().and_then(|mut clipboard| clipboard.get_text())
            });
            if !unicode_editor {
                // Key-based text paste is translated after RawInput's filter.
                super::field_input::filter_field_events(&mut input.events);
            }
            error
        });
        if let Some(error) = error {
            eprintln!("Could not paste clipboard text: {error}");
            self.toasts.add_error(format!("Could not paste clipboard text: {error}"));
        }
    }
}

impl eframe::App for ViewerApp {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        if self.viewer.ocr_text_has_focus(ctx) {
            // OCR results are free-form Unicode text; numeric expression fields
            // still use the existing ASCII and IME filtering below.
            return;
        }
        super::field_input::filter_field_input(raw_input);
    }

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
            egui::Area::new(egui::Id::unique("file_hover_overlay"))
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

        let toolbar_action = Toolbar {
            state: &mut self.state,
            viewer: &mut self.viewer,
            icons: &self.icons,
            show_settings: &mut self.show_settings_modal,
            show_rois: &mut self.show_rois_modal,
            is_loading: self.image_loader.is_loading(),
            update_available: matches!(self.update_status, UpdateStatus::Available(_)),
        }
        .show(ui);
        match toolbar_action {
            Some(ToolbarAction::OpenFile) => {
                let supported_extensions = crate::supported_image::supported_image_extensions();
                if let Some(path) = FileDialog::new()
                    .add_filter("Images", supported_extensions.as_slice())
                    .pick_file()
                {
                    self.viewer.reset_view();
                    self.open_file_paths(vec![path], &ctx);
                }
            }
            Some(ToolbarAction::OpenFromClipboard) => self.open_from_clipboard(&ctx),
            None => {}
        }

        self.show_update_confirmation_dialog(&ctx);
        self.show_update_progress_dialog(&ctx);
        self.show_settings_dialog(&ctx);
        self.show_rois_dialog(&ctx);

        if self.state.is_show_statusbar {
            let active_asset = self.active_display_asset().cloned();
            self.status_bar.show(
                ui,
                &mut self.state,
                active_asset.as_ref(),
                self.viewer.zoom(),
                &mut self.app_settings.ui_state.angle_display_unit,
            );
        }

        if self.state.is_show_sidebar {
            egui::Panel::right("right")
                .resizable(true)
                .size_range(Rangef::new(240.0, f32::INFINITY))
                .show(ui, |ui| {
                    // Keep the histogram axes and range controls reachable on short windows,
                    // while reserving room for the independently scrolling image list.
                    let settings_height = (ui.available_height() - 100.0).max(140.0);
                    egui::ScrollArea::vertical()
                        .id_salt("view_settings_scroll")
                        .max_height(settings_height)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading("View Settings");
                            });
                            ui.style_mut().spacing.slider_rail_height = 4.0;

                            let channels = self.state.asset.as_ref().map(|a| a.image().spec().channels).unwrap_or(0);
                            let is_mono = self.state.channel_index != -1 || channels == 1;

                            if is_mono {
                                if let Some(render_state) = frame.wgpu_render_state() {
                                    for error in self.mono_colormap_previews.prepare(&ctx, render_state, &self.state.colormap_mono_list) {
                                        eprintln!("{error}");
                                        self.toasts.add_error(error);
                                    }
                                }
                            }

                            // ComboBox widths are minimums. Bound each control to its own column
                            // so its natural text width cannot grow the panel on every frame.
                            let sizes = [Size::exact(68.0), Size::remainder(1.0), Size::exact(if is_mono { 0.0 } else { 54.0 })];
                            let mut open_custom_colormap = false;
                            ui.columns_sized(sizes, |[channel_ui, colormap_ui, alpha_ui]| {
                                let channel_values: Vec<i32> = if channels > 1 {
                                    (-1..channels).collect()
                                } else {
                                    vec![-1]
                                };
                                channel_ui.add_enabled_ui(channels > 1, |ui| {
                                    egui::ComboBox::from_id_salt("channel_index")
                                        .width(ui.available_width())
                                        .truncate()
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

                                let colormap_combo = egui::ComboBox::from_id_salt(if is_mono { "colormap_mono" } else { "colormap_rgb" })
                                    .width(colormap_ui.available_width())
                                    .truncate();
                                if is_mono {
                                    self.mono_colormap_previews.show(
                                        colormap_ui,
                                        colormap_combo,
                                        &mut self.state.colormap_mono,
                                        &self.state.colormap_mono_list,
                                        &mut open_custom_colormap,
                                    )
                                } else {
                                    let response = colormap_combo
                                        .height(f32::INFINITY)
                                        .selected_text(self.state.colormap_rgb.as_str())
                                        .show_ui(colormap_ui, |ui| {
                                            open_custom_colormap |= colormap_popup_contents(ui, |ui| {
                                            for name in &self.state.colormap_rgb_list {
                                                ui.selectable_value(&mut self.state.colormap_rgb, name.clone(), name);
                                            }
                                            });
                                        });
                                    response.response.hover_scroll(
                                        colormap_ui,
                                        &self.state.colormap_rgb_list,
                                        &mut self.state.colormap_rgb,
                                        false,
                                    );
                                    response
                                }
                                .response
                                .on_hover_text("Colormap");

                                if !is_mono {
                                    alpha_ui.checkbox(&mut self.state.shader_params.use_alpha, "Alpha");
                                }
                            });
                            if open_custom_colormap {
                                self.open_colormap_folder(&ctx, is_mono);
                            }

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
                                                    (1, 0) => "C",
                                                    (_, 0) => "R",
                                                    (_, 1) => "G",
                                                    (_, 2) => "B",
                                                    _ => "A",
                                                };
                                                let mut label = egui::RichText::new(label);
                                                if channels > 1 && i < 3 {
                                                    label = label.color(RGB_CHANNEL_LABEL_COLORS[i as usize]);
                                                }
                                                ui.label(label);

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

                                let mut style: egui::Style = ui.style().as_ref().clone();
                                egui::containers::menu::menu_style(&mut style);
                                style.spacing.item_spacing.x = 4.0;
                                let menu_style = std::sync::Arc::new(style);
                                let slider_width = ui.spacing().interact_size.y;
                                let alpha_scale = asset.image().spec().dtype.alpha();
                                // Use part of the panel's right padding without increasing the row's reported width.
                                egui::Frame::new().inner_margin(egui::Margin { right: -4, ..Default::default() }).show(ui, |ui| {
                                    ui.spacing_mut().item_spacing.x = 2.0;
                                    ui.columns_sized([Size::remainder(1.0), Size::exact(slider_width)], |[plot_ui, slider_ui]| {
                                        let plot_refs = plot_data.iter().map(Vec::as_slice).collect::<Vec<_>>();
                                        let mask = if channels > 1 { &self.show_plot_channels[..] } else { &[true] };
                                        if let Some(export) = draw_multi_line_plot(
                                            plot_ui,
                                            egui::vec2(plot_ui.available_width(), desired_size_plot.y),
                                            SeriesRef::new(&plot_refs),
                                            mask,
                                            self.plot_percentiles,
                                            alpha_scale,
                                            position_label,
                                            position_offset,
                                        ) {
                                            self.handle_export_action(export);
                                        }

                                        slider_ui.set_style(menu_style.clone());
                                        slider_ui.spacing_mut().slider_width = desired_size_plot.y;
                                        let [low, high] = &mut self.plot_percentiles;
                                        let response = slider_ui.scope(|ui| {
                                            // Only restyle native painting; keep the handle geometry and input unchanged.
                                            let inset = 2.0;
                                            let radius = slider_width.max(ui.text_style_height(&egui::TextStyle::Body)) / 2.5;
                                            let rail_width = 8.0;
                                            // egui shares the rail fill with the idle handle, so paint the rail separately.
                                            // Native handles retain all input, focus, and per-handle hover behavior.
                                            ui.spacing_mut().slider_rail_height = 0.0;
                                            let rail_shape = ui.painter().add(egui::Shape::Noop);
                                            let visuals = ui.visuals_mut();
                                            let rail_fill = visuals.widgets.inactive.bg_fill;
                                            let range_fill = visuals.selection.bg_fill;
                                            let widgets = &mut visuals.widgets;
                                            for visuals in [
                                                &mut widgets.noninteractive,
                                                &mut widgets.inactive,
                                                &mut widgets.hovered,
                                                &mut widgets.active,
                                            ] {
                                                visuals.expansion = rail_width / 2.0 - radius;
                                                visuals.corner_radius = egui::CornerRadius::same(1);
                                                visuals.bg_fill = visuals.fg_stroke.color;
                                                visuals.fg_stroke = egui::Stroke::NONE;
                                            }
                                            let handle_expansion = widgets.inactive.expansion;
                                            let response = range_slider_ui(ui, low, high, 0.0..=100.0, 1.0, |slider| {
                                                slider.vertical().show_value(false).smart_aim(false)
                                                    .handle_shape(egui::style::HandleShape::Rect {
                                                        aspect_ratio: 0.5 + inset / radius,
                                                    })
                                            });
                                            if let (Some(low), Some(high)) = (
                                                ui.ctx().read_response(response.id.with("low")),
                                                ui.ctx().read_response(response.id.with("high")),
                                            ) {
                                                let rail = egui::Rect::from_center_size(
                                                    response.rect.center(),
                                                    // Native handles change their painted bounds by expansion at both ends.
                                                    // Match those bounds without changing value mapping or the hit target.
                                                    egui::vec2(rail_width, (response.rect.height() + 2.0 * handle_expansion).max(0.0)),
                                                );
                                                let range = egui::Rect::from_min_max(
                                                    egui::pos2(rail.left(), high.rect.center().y - 1.0),
                                                    egui::pos2(rail.right(), low.rect.center().y + 1.0),
                                                ).intersect(rail);
                                                ui.painter().set(rail_shape, egui::Shape::Vec(vec![
                                                    egui::Shape::rect_filled(rail, 1, rail_fill),
                                                    egui::Shape::rect_filled(range, 1, range_fill),
                                                ]));
                                            }
                                            response
                                        }).inner;
                                        if response.changed() {
                                            ctx.request_repaint();
                                        }
                                        range_slider_tooltip(
                                            slider_ui,
                                            &response,
                                            "Adjust the plot's Y-axis limits using lower and upper percentiles.",
                                            self.plot_percentiles.map(|value| format!("{value:.0}%")),
                                        );
                                        range_slider_context_menu(&response, |ui| {
                                            if ui.button("Reset (0–100%)").clicked() {
                                                self.plot_percentiles = [0.0, 100.0];
                                                ctx.request_repaint();
                                                ui.close();
                                            }
                                        });
                                    });
                                });

                                ui.scope(|ui| {
                                    ui.columns_sized([Size::exact(68.0), Size::remainder(1.0)], |[axis_ui, channel_ui]| {
                                        axis_ui.set_style(menu_style.clone());
                                        axis_ui.horizontal(|ui| {
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
                                        });

                                        if channels > 1 {
                                            channel_toggle_ui(channel_ui, &mut self.show_plot_channels, channels);
                                        }
                                    });
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
                                if let Some(asset) = &self.state.asset {
                                    match self.histogram_panel.show(ui, asset.image(), &mut self.show_histogram_channels) {
                                        Ok(Some(export)) => self.handle_export_action(export),
                                        Ok(None) => {},
                                        Err(error) => {
                                            eprintln!("Histogram failed: {error}");
                                            self.toasts.add_error(format!("Histogram failed: {error}"));
                                        }
                                    }
                                } else {
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(ui.available_width(), 142.0),
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
                                .on_hover_text("Show per-channel min/max, variance and standard deviation for the selection, or the full image if no selection. NaN and infinity are excluded. Comparison metrics are also shown when comparing images.")
                                .clicked()
                                && self.show_statistics {
                                self.update_statistics();
                            }

                            if self.show_statistics {
                                let min_values = &self.state.statistics.min_max.value.min;
                                let max_values = &self.state.statistics.min_max.value.max;
                                let variance_values = &self.state.statistics.min_max.value.variance;
                                let std_dev_values = &self.state.statistics.min_max.value.std_dev;
                                let num_channels = min_values.len().min(max_values.len()).min(4);

                                if num_channels > 0 {
                                    let gap = 4.0;
                                    let label_width = 32.0;
                                    let value_width = ((ui.available_width() - label_width - gap * num_channels as f32)
                                        / num_channels as f32).max(0.0);
                                    egui::Grid::new("statistics_min_max_grid")
                                        .num_columns(num_channels + 1)
                                        .min_col_width(0.0)
                                        .max_col_width(value_width.max(label_width))
                                        .spacing(vec2(gap, 2.0))
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
                                                    value_width,
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
                                                    value_width,
                                                );
                                            }
                                            ui.end_row();

                                            for (label, values, tooltip) in [
                                                ("Var:", variance_values, "Variance per channel. Values use squared pixel units; NaN and infinity are excluded."),
                                                ("Std:", std_dev_values, "Standard deviation per channel: the square root of variance, in pixel units. NaN and infinity are excluded."),
                                            ] {
                                                ui.add(egui::Label::new(label).extend()).on_hover_text(tooltip);
                                                for &value in values.iter().take(num_channels) {
                                                    ui.add_sized(vec2(value_width, ui.text_style_height(&egui::TextStyle::Body)), egui::Label::new(statistics_value_text(ui, value, value_width)).truncate())
                                                        .on_hover_text(if value.is_finite() { value.to_string() } else { "Unavailable: no finite pixels, or the calculation exceeded the numeric range.".to_owned() });
                                                }
                                                ui.end_row();
                                            }
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

                        });
                    self.asset_list_selection.retain(|hash| self.state.assets.contains_key(hash));
                    ui.horizontal(|ui| {
                        ui.heading("Image List");
                        if self.asset_list_selection.len() > 0 {
                            ui.label(
                                egui::RichText::new(format!("{} selected", self.asset_list_selection.len()))
                                    .small()
                                    .color(ASSET_MULTI_SELECTION_STROKE),
                            );
                        }
                        ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                            ui.button("Clear").clicked().then(|| {
                                self.state.assets.clear();
                                self.state.clear_asset();
                                self.asset_list_selection.clear();
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
                                    let btn = if self.asset_list_selection.contains(&hash) {
                                        let visuals = ui.visuals_mut();
                                        visuals.selection.bg_fill = ASSET_MULTI_SELECTION_FILL;
                                        // Let egui account for the border during layout instead of
                                        // overriding the button stroke after its hover compensation.
                                        let stroke = egui::Stroke::new(1.0, ASSET_MULTI_SELECTION_STROKE);
                                        for widget in [
                                            &mut visuals.widgets.inactive,
                                            &mut visuals.widgets.hovered,
                                            &mut visuals.widgets.active,
                                            &mut visuals.widgets.open,
                                        ] {
                                            widget.bg_stroke = stroke;
                                        }
                                        for widget in [
                                            &mut visuals.widgets.hovered,
                                            &mut visuals.widgets.active,
                                            &mut visuals.widgets.open,
                                        ] {
                                            widget.bg_stroke.color = ASSET_MULTI_SELECTION_HOVER_STROKE;
                                        }
                                        ui.add(
                                            egui::Button::selectable(true, &display_name)
                                                .sense(egui::Sense::click_and_drag()),
                                        )
                                    } else if Some(hash.as_str()) == asset_primary_hash.as_deref() {
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
                                    let row_response = btn.clone();
                                    if row_response.secondary_clicked() && !self.asset_list_selection.contains(&hash) {
                                        self.asset_list_selection.activate(&hash);
                                    }
                                    row_response.context_menu(|ui| {
                                        let bulk_delete = self.asset_list_selection.contains(&hash);
                                        ui.visuals_mut().override_text_color = Some(DANGER_TEXT);
                                        if ui.button("Delete").clicked() {
                                            if bulk_delete {
                                                to_remove.extend(self.asset_list_selection.hashes().cloned());
                                            } else {
                                                to_remove.insert(hash.clone());
                                            }
                                            ui.close();
                                        }
                                        if bulk_delete {
                                            return;
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
                                                if ui.button("Copy Name").clicked() {
                                                    let name = std::path::Path::new(asset.name())
                                                        .file_name()
                                                        .unwrap_or_default()
                                                        .to_string_lossy()
                                                        .into_owned();
                                                    if let Err(e) = arboard::Clipboard::new()
                                                        .and_then(|mut cb| cb.set_text(name))
                                                    {
                                                        eprintln!("Failed to copy name to clipboard: {e}");
                                                        self.toasts.add_error("Failed to copy name to clipboard".to_string());
                                                    }
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
                                        let modifiers = ui.input(|i| i.modifiers);
                                        if modifiers.shift {
                                            let order: Vec<_> = self.state.assets.keys().map(String::as_str).collect();
                                            self.asset_list_selection.select_range(
                                                &order, &hash, asset_primary_hash.as_deref(), modifiers.command,
                                            );
                                        } else if modifiers.command {
                                            self.asset_list_selection.activate(&hash);
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
                                            self.asset_list_selection.activate(&hash);
                                            // Normal click: set primary
                                            to_set_primary = Some(asset.clone());
                                        }
                                    }

                                    row_response
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

                        self.asset_list_selection.retain(|hash| self.state.assets.contains_key(hash));
                        if self.state.asset_primary.as_ref().is_some_and(|asset| !self.state.assets.contains_key(asset.hash())) {
                            self.state.clear_asset();
                        } else if self.state.asset_secondary.as_ref().is_some_and(|asset| !self.state.assets.contains_key(asset.hash())) {
                            self.state.set_secondary_asset(None);
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
                    &self.icons,
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
                    egui::Area::new(egui::Id::unique("comparison_notice_overlay"))
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
                    egui::Area::new(egui::Id::unique("statistics_rect_mismatch_overlay"))
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

        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        for error in self.viewer.show_ocr(&ctx) {
            eprintln!("{error}");
            self.toasts.add_error(error);
            ctx.request_repaint();
        }

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

        #[cfg(target_os = "windows")]
        if let Some(chrome) = &self.native_chrome {
            for error in chrome.update(&ctx) {
                eprintln!("{error}");
                self.toasts.add_error(error);
            }
        }

        // Debug window
        #[cfg(debug_assertions)]
        {
            crate::debug::debug_window(&ctx);
        }
    }
}

#[cfg(test)]
mod clipboard_shortcut_tests {
    use super::*;

    fn key(pressed: bool, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key {
            key: crate::res::OPEN_FROM_CLIPBOARD_SC.logical_key,
            physical_key: None,
            pressed,
            // Like egui-winit, let egui detect repeated key-downs.
            repeat: false,
            modifiers,
        }
    }

    fn frame(ctx: &egui::Context, events: Vec<egui::Event>) -> bool {
        ctx.begin_pass(egui::RawInput {
            events,
            ..Default::default()
        });
        let triggered = ctx.input_mut(ViewerApp::consume_open_from_clipboard_shortcut);
        ctx.end_pass().textures_delta.clear();
        triggered
    }

    #[test]
    fn clipboard_shortcut_loads_once_per_press_and_release() {
        let ctx = egui::Context::default();
        let modifiers = crate::res::OPEN_FROM_CLIPBOARD_SC.modifiers;
        assert!(frame(&ctx, vec![key(true, modifiers)]));
        for _ in 0..10 {
            assert!(!frame(&ctx, vec![key(true, modifiers)]));
        }
        assert!(!frame(&ctx, vec![key(false, modifiers)]));
        assert!(frame(&ctx, vec![key(true, modifiers)]));
        // Releasing the modifiers before V must also allow the next press.
        assert!(!frame(&ctx, vec![key(false, egui::Modifiers::NONE)]));
        assert!(frame(&ctx, vec![key(true, modifiers)]));
    }

    #[test]
    fn clipboard_shortcut_handles_a_quick_tap_without_a_release_load() {
        let ctx = egui::Context::default();
        let modifiers = crate::res::OPEN_FROM_CLIPBOARD_SC.modifiers;
        assert!(frame(&ctx, vec![key(true, modifiers), key(false, modifiers)]));
        assert!(!frame(&ctx, vec![]));
        assert!(frame(&ctx, vec![key(true, modifiers), key(false, modifiers)]));
    }

    #[test]
    fn clipboard_shortcut_accepts_native_paste_events_and_legacy_binding() {
        let ctx = egui::Context::default();
        assert!(frame(&ctx, vec![egui::Event::Paste("123".into())]));
        let legacy = crate::res::OPEN_FROM_CLIPBOARD_LEGACY_SC.modifiers;
        assert!(frame(&ctx, vec![key(true, legacy), key(false, legacy)]));
        assert!(!frame(
            &ctx,
            vec![key(true, egui::Modifiers::NONE), key(false, egui::Modifiers::NONE)]
        ));
        let alt = crate::res::OPEN_FROM_CLIPBOARD_SC.modifiers.plus(egui::Modifiers::ALT);
        assert!(!frame(&ctx, vec![key(true, alt), key(false, alt)]));
    }
}
