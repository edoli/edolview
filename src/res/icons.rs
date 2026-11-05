use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use eframe::egui;

use crate::settings::BackgroundStyle;
use crate::ui::icon::{self, IconExt};

const BACKGROUND_ICON_SIZE: usize = 24;
const BACKGROUND_ICON_INSET: usize = 3;
const BACKGROUND_ICON_CELL_SIZE: usize = 6;
pub const IMAGE_LOADING_ICON_SIZE: f32 = 16.0;
pub const SOCKET_ADDRESS_ICON_SIZE: f32 = 16.0;
#[cfg(target_os = "windows")]
pub const WINDOW_BUTTON_SIZE: egui::Vec2 = egui::vec2(46.0, 32.0);
#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
pub enum WindowControlIcon {
    Minimize,
    Maximize,
    Restore,
    Close,
}
#[cfg(target_os = "windows")]
pub const WINDOW_ICON_FONT_FAMILY: &str = "Segoe Fluent Icons";
#[cfg(target_os = "windows")]
pub const WINDOW_ICON_FONT_SIZE: f32 = 10.0;
#[cfg(target_os = "windows")]
impl WindowControlIcon {
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Minimize => "\u{e921}",
            Self::Maximize => "\u{e922}",
            Self::Restore => "\u{e923}",
            Self::Close => "\u{e8bb}",
        }
    }

    pub fn fallback_glyph(self) -> &'static str {
        match self {
            Self::Minimize => "−",
            Self::Maximize => "□",
            Self::Restore => "❐",
            Self::Close => "×",
        }
    }
}
const OPEN_FILE: &[u8] = include_bytes!("icons/open_file.svg");
const OPEN_CLIPBOARD: &[u8] = include_bytes!("icons/open_clipboard.svg");
const SHOW_PIXEL_VALUE: &[u8] = include_bytes!("icons/show_pixel_value.svg");
const SHOW_CROSSHAIR: &[u8] = include_bytes!("icons/show_crosshair.svg");
const SHOW_MINIMAP: &[u8] = include_bytes!("icons/show_minimap.svg");
const RESET_VIEW: &[u8] = include_bytes!("icons/reset_view.svg");
const FIT_VIEW: &[u8] = include_bytes!("icons/fit_view.svg");
const CENTER_VIEW: &[u8] = include_bytes!("icons/center_view.svg");
const ROIS_PANEL: [&[u8]; 2] = [
    include_bytes!("icons/rois_hidden.svg"),
    include_bytes!("icons/rois_shown.svg"),
];
const SIDEBAR_PANEL: [&[u8]; 2] = [
    include_bytes!("icons/sidebar_hidden.svg"),
    include_bytes!("icons/sidebar_shown.svg"),
];
const STATUSBAR_PANEL: [&[u8]; 2] = [
    include_bytes!("icons/statusbar_hidden.svg"),
    include_bytes!("icons/statusbar_shown.svg"),
];

const SCALE_LINEAR: &[u8] = include_bytes!("icons/scale_linear.svg");
const SCALE_INVERSE: &[u8] = include_bytes!("icons/scale_inverse.svg");
const SCALE_LOG: &[u8] = include_bytes!("icons/scale_log.svg");
const SCALE_ABSOLUTE: &[u8] = include_bytes!("icons/scale_absolute.svg");

const NORMALIZE: &[u8] = include_bytes!("icons/normalize.svg");
const SOCKET_ADDRESS: &[u8] = include_bytes!("icons/socket_address.svg");
const DOWNLOADING: &[u8] = include_bytes!("icons/downloading.svg");
const IMAGE_READY: &[u8] = include_bytes!("icons/image_ready.svg");

const REDUCE_COLUMN: &[u8] = include_bytes!("icons/reduce_column.svg");
const REDUCE_ROW: &[u8] = include_bytes!("icons/reduce_row.svg");
const REDUCE_AUTO: &[u8] = include_bytes!("icons/reduce_auto.svg");

const DELETE: &[u8] = include_bytes!("icons/delete.svg");
const COPY: &[u8] = include_bytes!("icons/copy.svg");
const INFO: &[u8] = include_bytes!("icons/info.svg");
const SETTINGS: &[u8] = include_bytes!("icons/settings.svg");
const ARROW_UP: &[u8] = include_bytes!("icons/arrow_up.svg");
const ARROW_DOWN: &[u8] = include_bytes!("icons/arrow_down.svg");

pub struct Icons {
    background: Mutex<HashMap<[egui::Color32; 2], egui::TextureHandle>>,
    open_file: OnceLock<egui::TextureHandle>,
    open_clipboard: OnceLock<egui::TextureHandle>,
    show_pixel_value: OnceLock<egui::TextureHandle>,
    show_crosshair: OnceLock<egui::TextureHandle>,
    show_minimap: OnceLock<egui::TextureHandle>,
    reset_view: OnceLock<egui::TextureHandle>,
    fit_view: OnceLock<egui::TextureHandle>,
    center_view: OnceLock<egui::TextureHandle>,
    rois_panel: [OnceLock<egui::TextureHandle>; 2],
    sidebar_panel: [OnceLock<egui::TextureHandle>; 2],
    statusbar_panel: [OnceLock<egui::TextureHandle>; 2],

    scale_linear: OnceLock<egui::TextureHandle>,
    scale_inverse: OnceLock<egui::TextureHandle>,
    scale_log: OnceLock<egui::TextureHandle>,
    scale_absolute: OnceLock<egui::TextureHandle>,

    normalize: OnceLock<egui::TextureHandle>,

    socket_address: OnceLock<egui::TextureHandle>,
    downloading: OnceLock<egui::TextureHandle>,
    image_ready: OnceLock<egui::TextureHandle>,

    reduce_column: OnceLock<egui::TextureHandle>,
    reduce_row: OnceLock<egui::TextureHandle>,
    reduce_auto: OnceLock<egui::TextureHandle>,

    delete: OnceLock<egui::TextureHandle>,
    copy: OnceLock<egui::TextureHandle>,
    info: OnceLock<egui::TextureHandle>,
    settings: OnceLock<egui::TextureHandle>,
    arrow_up: OnceLock<egui::TextureHandle>,
    arrow_down: OnceLock<egui::TextureHandle>,
}

impl Icons {
    pub fn new() -> Self {
        Self {
            background: Mutex::new(HashMap::new()),
            open_file: OnceLock::new(),
            open_clipboard: OnceLock::new(),
            show_pixel_value: OnceLock::new(),
            show_crosshair: OnceLock::new(),
            show_minimap: OnceLock::new(),
            reset_view: OnceLock::new(),
            fit_view: OnceLock::new(),
            center_view: OnceLock::new(),
            rois_panel: [const { OnceLock::new() }; 2],
            sidebar_panel: [const { OnceLock::new() }; 2],
            statusbar_panel: [const { OnceLock::new() }; 2],

            scale_linear: OnceLock::new(),
            scale_inverse: OnceLock::new(),
            scale_log: OnceLock::new(),
            scale_absolute: OnceLock::new(),

            normalize: OnceLock::new(),

            socket_address: OnceLock::new(),
            downloading: OnceLock::new(),
            image_ready: OnceLock::new(),

            reduce_column: OnceLock::new(),
            reduce_row: OnceLock::new(),
            reduce_auto: OnceLock::new(),

            delete: OnceLock::new(),
            copy: OnceLock::new(),
            info: OnceLock::new(),
            settings: OnceLock::new(),
            arrow_up: OnceLock::new(),
            arrow_down: OnceLock::new(),
        }
    }

    #[inline]
    pub fn get_open_file<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.open_file
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "open_file", OPEN_FILE))
            .to_icon()
    }

    #[inline]
    pub fn get_open_clipboard<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.open_clipboard
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "open_clipboard", OPEN_CLIPBOARD))
            .to_icon()
    }

    #[inline]
    pub fn get_background<'c>(
        &self,
        ctx: &egui::Context,
        style: BackgroundStyle,
        visuals: &egui::Visuals,
    ) -> egui::Image<'c> {
        // Share the viewer's palette, including theme-dependent dark checker colors.
        let colors = style.colors([visuals.extreme_bg_color, visuals.faint_bg_color]);
        let border_color = visuals.window_stroke.color;
        self.background
            .lock()
            .unwrap()
            .entry([colors[0], colors[1]])
            .or_insert_with(|| {
                let mut image = egui::ColorImage::filled([BACKGROUND_ICON_SIZE; 2], egui::Color32::TRANSPARENT);
                for y in 1..BACKGROUND_ICON_SIZE - 1 {
                    for x in 1..BACKGROUND_ICON_SIZE - 1 {
                        let inset = BACKGROUND_ICON_INSET;
                        image[(x, y)] = if x < inset
                            || y < inset
                            || x >= BACKGROUND_ICON_SIZE - inset
                            || y >= BACKGROUND_ICON_SIZE - inset
                        {
                            border_color
                        } else {
                            let parity =
                                ((x - inset) / BACKGROUND_ICON_CELL_SIZE + (y - inset) / BACKGROUND_ICON_CELL_SIZE) % 2;
                            colors[parity]
                        };
                    }
                }
                ctx.load_texture(format!("background_{}", style.label()), image, Default::default())
            })
            .to_icon()
    }

    #[inline]
    pub fn get_show_pixel_value<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.show_pixel_value
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "show_pixel_value", SHOW_PIXEL_VALUE))
            .to_icon()
    }
    #[inline]
    pub fn get_show_crosshair<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.show_crosshair
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "show_crosshair", SHOW_CROSSHAIR))
            .to_icon()
    }

    #[inline]
    pub fn get_show_minimap<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.show_minimap
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "show_minimap", SHOW_MINIMAP))
            .to_icon()
    }

    #[inline]
    pub fn get_reset_view<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.reset_view
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "reset_view", RESET_VIEW))
            .to_icon()
    }

    #[inline]
    pub fn get_fit_view<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.fit_view
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "fit_view", FIT_VIEW))
            .to_icon()
    }

    #[inline]
    pub fn get_center_view<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.center_view
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "center_view", CENTER_VIEW))
            .to_icon()
    }

    #[inline]
    pub fn get_rois_panel<'c>(&self, ctx: &egui::Context, shown: bool) -> egui::Image<'c> {
        let index = usize::from(shown);
        self.rois_panel[index]
            .get_or_init(|| icon::load_svg_icon_texture(ctx, format!("rois_panel_{shown}"), ROIS_PANEL[index]))
            .to_icon()
    }

    #[inline]
    pub fn get_sidebar_panel<'c>(&self, ctx: &egui::Context, shown: bool) -> egui::Image<'c> {
        let index = usize::from(shown);
        self.sidebar_panel[index]
            .get_or_init(|| icon::load_svg_icon_texture(ctx, format!("sidebar_panel_{shown}"), SIDEBAR_PANEL[index]))
            .to_icon()
    }

    #[inline]
    pub fn get_statusbar_panel<'c>(&self, ctx: &egui::Context, shown: bool) -> egui::Image<'c> {
        let index = usize::from(shown);
        self.statusbar_panel[index]
            .get_or_init(|| {
                icon::load_svg_icon_texture(ctx, format!("statusbar_panel_{shown}"), STATUSBAR_PANEL[index])
            })
            .to_icon()
    }

    #[inline]
    pub fn get_scale_linear<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.scale_linear
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "scale_linear", SCALE_LINEAR))
            .to_icon()
    }

    #[inline]
    pub fn get_scale_inverse<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.scale_inverse
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "scale_inverse", SCALE_INVERSE))
            .to_icon()
    }

    #[inline]
    pub fn get_scale_log<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.scale_log
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "scale_log", SCALE_LOG))
            .to_icon()
    }

    #[inline]
    pub fn get_scale_absolute<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.scale_absolute
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "scale_absolute", SCALE_ABSOLUTE))
            .to_icon()
    }

    #[inline]
    pub fn get_normalize<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.normalize
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "normalize", NORMALIZE))
            .to_icon()
    }

    #[inline]
    pub fn get_socket_address<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.socket_address
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "socket_address", SOCKET_ADDRESS))
            .to_icon()
    }

    #[inline]
    pub fn get_downloading<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.downloading
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "downloading", DOWNLOADING))
            .to_icon()
    }

    #[inline]
    pub fn get_image_ready<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.image_ready
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "image_ready", IMAGE_READY))
            .to_icon()
    }

    #[inline]
    pub fn get_reduce_column<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.reduce_column
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "reduce_column", REDUCE_COLUMN))
            .to_icon()
    }

    #[inline]
    pub fn get_reduce_row<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.reduce_row
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "reduce_row", REDUCE_ROW))
            .to_icon()
    }

    #[inline]
    pub fn get_reduce_auto<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.reduce_auto
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "reduce_auto", REDUCE_AUTO))
            .to_icon()
    }

    #[inline]
    pub fn get_delete<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.delete
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "delete", DELETE))
            .to_icon()
    }

    #[inline]
    pub fn get_copy<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.copy
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "copy", COPY))
            .to_icon()
    }

    #[inline]
    pub fn get_info<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.info
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "info", INFO))
            .to_icon()
    }

    #[inline]
    pub fn get_settings<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.settings
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "settings", SETTINGS))
            .to_icon()
    }

    #[inline]
    pub fn get_arrow_up<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.arrow_up
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "arrow_up", ARROW_UP))
            .to_icon()
    }

    #[inline]
    pub fn get_arrow_down<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.arrow_down
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "arrow_down", ARROW_DOWN))
            .to_icon()
    }
}
