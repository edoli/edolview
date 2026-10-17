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
const SHOW_PIXEL_VALUE: &[u8] = include_bytes!("icons/show_pixel_value.svg");
const SHOW_CROSSHAIR: &[u8] = include_bytes!("icons/show_crosshair.svg");

const SCALE_LINEAR: &[u8] = include_bytes!("icons/scale_linear.svg");
const SCALE_INVERSE: &[u8] = include_bytes!("icons/scale_inverse.svg");
const SCALE_LOG: &[u8] = include_bytes!("icons/scale_log.svg");
const SCALE_ABSOLUTE: &[u8] = include_bytes!("icons/scale_absolute.svg");

const NORMALIZE: &[u8] = include_bytes!("icons/normalize.svg");
const DOWNLOADING: &[u8] = include_bytes!("icons/downloading.svg");

const REDUCE_COLUMN: &[u8] = include_bytes!("icons/reduce_column.svg");
const REDUCE_ROW: &[u8] = include_bytes!("icons/reduce_row.svg");
const REDUCE_AUTO: &[u8] = include_bytes!("icons/reduce_auto.svg");

const DELETE: &[u8] = include_bytes!("icons/delete.svg");
const COPY: &[u8] = include_bytes!("icons/copy.svg");
const ARROW_UP: &[u8] = include_bytes!("icons/arrow_up.svg");
const ARROW_DOWN: &[u8] = include_bytes!("icons/arrow_down.svg");

pub struct Icons {
    background: Mutex<HashMap<[egui::Color32; 2], egui::TextureHandle>>,
    show_pixel_value: OnceLock<egui::TextureHandle>,
    show_crosshair: OnceLock<egui::TextureHandle>,

    scale_linear: OnceLock<egui::TextureHandle>,
    scale_inverse: OnceLock<egui::TextureHandle>,
    scale_log: OnceLock<egui::TextureHandle>,
    scale_absolute: OnceLock<egui::TextureHandle>,

    normalize: OnceLock<egui::TextureHandle>,

    downloading: OnceLock<egui::TextureHandle>,

    reduce_column: OnceLock<egui::TextureHandle>,
    reduce_row: OnceLock<egui::TextureHandle>,
    reduce_auto: OnceLock<egui::TextureHandle>,

    delete: OnceLock<egui::TextureHandle>,
    copy: OnceLock<egui::TextureHandle>,
    arrow_up: OnceLock<egui::TextureHandle>,
    arrow_down: OnceLock<egui::TextureHandle>,
}

impl Icons {
    pub fn new() -> Self {
        Self {
            background: Mutex::new(HashMap::new()),
            show_pixel_value: OnceLock::new(),
            show_crosshair: OnceLock::new(),

            scale_linear: OnceLock::new(),
            scale_inverse: OnceLock::new(),
            scale_log: OnceLock::new(),
            scale_absolute: OnceLock::new(),

            normalize: OnceLock::new(),

            downloading: OnceLock::new(),

            reduce_column: OnceLock::new(),
            reduce_row: OnceLock::new(),
            reduce_auto: OnceLock::new(),

            delete: OnceLock::new(),
            copy: OnceLock::new(),
            arrow_up: OnceLock::new(),
            arrow_down: OnceLock::new(),
        }
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
    pub fn get_downloading<'c>(&self, ctx: &egui::Context) -> egui::Image<'c> {
        self.downloading
            .get_or_init(|| icon::load_svg_icon_texture(ctx, "downloading", DOWNLOADING))
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
