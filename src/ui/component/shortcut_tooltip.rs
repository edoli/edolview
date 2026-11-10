use eframe::egui::{self, KeyboardShortcut, Response, Ui, WidgetText};

use crate::res::{IS_MAC, MODIFIER_NAMES, SHORTCUT_KEY_FILL, SHORTCUT_KEY_TEXT};

/// Draw a non-interactive shortcut badge, including platform-specific modifiers.
pub fn shortcut_key(ui: &mut Ui, shortcut: KeyboardShortcut) {
    let names = egui::ModifierNames {
        is_short: true,
        ..MODIFIER_NAMES
    };
    egui::Frame::new()
        .fill(SHORTCUT_KEY_FILL)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(3)
        .inner_margin(egui::Margin::symmetric(5, 1))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(shortcut.format(&names, IS_MAC))
                    .monospace()
                    .strong()
                    .color(SHORTCUT_KEY_TEXT),
            );
        });
}

/// A tooltip row that can also be composed into a multi-line tooltip.
pub fn shortcut_hint(ui: &mut Ui, description: impl Into<WidgetText>, shortcut: Option<KeyboardShortcut>) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.label(description);
        if let Some(shortcut) = shortcut {
            shortcut_key(ui, shortcut);
        }
    });
}

pub trait ShortcutTooltipExt {
    fn on_hover_shortcut(self, description: impl Into<WidgetText>, shortcut: Option<KeyboardShortcut>) -> Self;
}

impl ShortcutTooltipExt for Response {
    fn on_hover_shortcut(self, description: impl Into<WidgetText>, shortcut: Option<KeyboardShortcut>) -> Self {
        if shortcut.is_some() {
            self.on_hover_ui(|ui| shortcut_hint(ui, description, shortcut))
        } else {
            self.on_hover_text(description)
        }
    }
}
