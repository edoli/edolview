//! Egui presentation of the Windows title bar. Native pointer input lives in `native`.
mod native;
pub(super) use native::{store_regions, NativeChrome, Regions};

use crate::res::{
    icons::{WindowControlIcon, WINDOW_BUTTON_SIZE, WINDOW_ICON_FONT_FAMILY, WINDOW_ICON_FONT_SIZE},
    WINDOW_CLOSE_HOVER, WINDOW_CLOSE_PRESSED,
};
use eframe::egui::{self, Rect, Sense};
use native::Button;

// Right-to-left toolbar order. These same rectangles are published to hit testing.
pub(super) fn controls(ui: &mut egui::Ui) -> [Rect; 3] {
    if ui.input(|i| i.viewport().fullscreen.unwrap_or(false)) {
        return [Rect::NOTHING; 3];
    }
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let restore = native::is_maximized(ui.ctx());
        [
            window_button(ui, Button::Close, WindowControlIcon::Close, "Close"),
            window_button(
                ui,
                Button::Maximize,
                if restore {
                    WindowControlIcon::Restore
                } else {
                    WindowControlIcon::Maximize
                },
                if restore { "Restore" } else { "Maximize" },
            ),
            window_button(ui, Button::Minimize, WindowControlIcon::Minimize, "Minimize"),
        ]
    })
    .inner
}

fn window_button(ui: &mut egui::Ui, button: Button, icon: WindowControlIcon, name: &str) -> Rect {
    let (rect, response) = ui.allocate_exact_size(WINDOW_BUTTON_SIZE, Sense::click());
    let (hovered, pressed) = native::button_state(ui.ctx(), button);
    let hovered = hovered || response.hovered();
    let visuals = if pressed {
        &ui.visuals().widgets.active
    } else if hovered {
        &ui.visuals().widgets.hovered
    } else {
        &ui.visuals().widgets.inactive
    };
    if hovered || pressed {
        let fill = if button == Button::Close {
            if pressed {
                WINDOW_CLOSE_PRESSED
            } else {
                WINDOW_CLOSE_HOVER
            }
        } else {
            visuals.weak_bg_fill
        };
        ui.painter().rect_filled(rect, 0, fill);
    }
    let tint = if button == Button::Close && (hovered || pressed) {
        egui::Color32::WHITE
    } else {
        visuals.fg_stroke.color
    };
    let family = egui::FontFamily::Name(WINDOW_ICON_FONT_FAMILY.into());
    let loaded = ui.fonts_mut(|fonts| fonts.definitions().families.contains_key(&family));
    let (glyph, font) = if loaded {
        (icon.glyph(), egui::FontId::new(WINDOW_ICON_FONT_SIZE, family))
    } else {
        (icon.fallback_glyph(), egui::FontId::proportional(12.0))
    };
    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, glyph, font, tint);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), name));
    // Native pointer tracking and keyboard/accessibility share WM_SYSCOMMAND.
    // Egui must not execute a real mouse click a second time.
    if response.clicked() && !response.clicked_by(egui::PointerButton::Primary) {
        native::activate(ui.ctx(), button);
    }
    response.on_hover_text(name);
    rect
}

pub(super) fn title(ui: &mut egui::Ui, image_name: Option<&str>) -> Rect {
    let width = ui.available_width().max(0.0);
    if width == 0.0 {
        return Rect::NOTHING;
    }
    let native_title = ui
        .input(|i| i.viewport().title.clone())
        .unwrap_or_else(|| "edolview".to_owned());
    let text = caption_title(&native_title, image_name);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, WINDOW_BUTTON_SIZE.y), Sense::hover());
    ui.place(
        rect,
        egui::Label::new(text).truncate().halign(egui::Align::Center).selectable(false),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Pane, ui.is_enabled(), "Window Title"));
    response.on_hover_text(image_name.map_or(native_title, str::to_owned));
    rect
}

fn caption_title(native_title: &str, image_name: Option<&str>) -> String {
    image_name.map_or_else(
        || native_title.to_owned(),
        |name| {
            let filename = std::path::Path::new(name)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new(name));
            filename.to_string_lossy().into_owned()
        },
    )
}
