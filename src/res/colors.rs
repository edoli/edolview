use eframe::egui::Color32;

pub const UPDATE_ACCENT_TEXT: Color32 = Color32::from_rgb(120, 220, 120);
pub const UPDATE_ACCENT_FILL: Color32 = Color32::from_rgb(42, 84, 42);
pub const CONTROL_LISTENER_UNAVAILABLE_TEXT: Color32 = Color32::from_rgb(255, 140, 140);
pub const ASSET_SECONDARY_SELECTION_FILL: Color32 = Color32::from_rgb(140, 70, 30);
pub const ASSET_MULTI_SELECTION_FILL: Color32 = Color32::from_rgb(67, 43, 86);
pub const ASSET_MULTI_SELECTION_STROKE: Color32 = Color32::from_rgb(192, 140, 236);
pub const ASSET_MULTI_SELECTION_HOVER_STROKE: Color32 = Color32::from_rgb(232, 198, 255);
pub const DANGER_TEXT: Color32 = Color32::from_rgb(255, 100, 100);
pub const NOTICE_ERROR_TEXT: Color32 = Color32::from_rgb(255, 60, 60);
pub const NOTICE_WARNING_TEXT: Color32 = Color32::from_rgb(255, 210, 120);

pub const BACKGROUND_BLACK: Color32 = Color32::BLACK;
pub const BACKGROUND_GRAY: Color32 = Color32::from_gray(128);
pub const BACKGROUND_WHITE: Color32 = Color32::WHITE;
pub const BACKGROUND_LIGHT_CHECKER: [Color32; 2] = [Color32::from_gray(208), Color32::from_gray(240)];
pub const BACKGROUND_HIGH_CONTRAST_CHECKER: [Color32; 2] = [Color32::BLACK, Color32::WHITE];

pub const STATISTICS_MIN_TOGGLE_FILL: Color32 = Color32::from_rgb(0, 122, 255);
pub const STATISTICS_MAX_TOGGLE_FILL: Color32 = Color32::from_rgb(214, 48, 49);

pub const SELECTION_HANDLE_CLIPPED_STROKE: Color32 = Color32::from_rgb(120, 72, 0);
pub const MINIMAP_VIEWPORT_STROKE: Color32 = Color32::from_rgb(255, 210, 64);
pub const MINIMAP_VIEWPORT_OUTLINE: Color32 = Color32::BLACK;
pub const MINIMAP_OUTSIDE_VIEW_FILL: Color32 = Color32::from_black_alpha(100);
pub const TEXT_EDIT_PARSE_FAILED_FLASH: Color32 = Color32::from_rgba_premultiplied(255, 64, 64, 64);

pub const RGB_CHANNEL_LABEL_COLORS: [Color32; 3] = [
    Color32::from_rgb(255, 100, 100),
    Color32::from_rgb(100, 220, 120),
    Color32::from_rgb(100, 160, 255),
];

pub const HISTOGRAM_LABEL_COLORS: [Color32; 4] = [Color32::RED, Color32::GREEN, Color32::BLUE, Color32::YELLOW];

pub const MULTI_LINE_PLOT_COLORS: [Color32; 8] = [
    Color32::RED,
    Color32::GREEN,
    Color32::BLUE,
    Color32::YELLOW,
    Color32::LIGHT_BLUE,
    Color32::LIGHT_GREEN,
    Color32::LIGHT_RED,
    Color32::from_rgb(200, 100, 255),
];
pub const MULTI_LINE_PLOT_GRID_STROKE: Color32 = Color32::from_gray(36);

pub const PIXEL_VALUE_CHANNEL_COLORS: [Color32; 4] = [Color32::RED, Color32::GREEN, Color32::BLUE, Color32::GRAY];

pub const TOAST_INFO_ICON_COLOR: Color32 = Color32::from_rgb(0, 155, 255);
pub const TOAST_WARNING_ICON_COLOR: Color32 = Color32::from_rgb(255, 212, 0);
pub const TOAST_ERROR_ICON_COLOR: Color32 = Color32::from_rgb(255, 32, 0);
pub const TOAST_SUCCESS_ICON_COLOR: Color32 = Color32::from_rgb(0, 255, 32);

pub fn selection_handle_clipped_fill() -> Color32 {
    Color32::from_rgba_unmultiplied(255, 174, 174, 240)
}

pub fn histogram_bar_colors() -> [Color32; 4] {
    [
        Color32::from_rgba_unmultiplied(255, 0, 0, 150),
        Color32::from_rgba_unmultiplied(0, 255, 0, 150),
        Color32::from_rgba_unmultiplied(0, 0, 255, 150),
        Color32::from_rgba_unmultiplied(255, 255, 0, 150),
    ]
}
