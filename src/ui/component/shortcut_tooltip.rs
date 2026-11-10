use eframe::egui::{self, KeyboardShortcut, Response, Ui, WidgetText};

use crate::res::{IS_MAC, MODIFIER_NAMES, SHORTCUT_KEY_FILL, SHORTCUT_KEY_TEXT};

fn shortcut_text(shortcut: KeyboardShortcut) -> String {
    let names = egui::ModifierNames {
        is_short: true,
        ..MODIFIER_NAMES
    };
    shortcut.format(&names, IS_MAC)
}

fn shortcut_frame(ui: &Ui) -> egui::Frame {
    egui::Frame::new()
        .fill(SHORTCUT_KEY_FILL)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(3)
        .inner_margin(egui::Margin::symmetric(5, 1))
}

fn shortcut_label(text: String) -> egui::RichText {
    egui::RichText::new(text).monospace().strong().color(SHORTCUT_KEY_TEXT)
}

/// Draw a non-interactive shortcut badge, including platform-specific modifiers.
pub fn shortcut_key(ui: &mut Ui, shortcut: KeyboardShortcut) {
    shortcut_frame(ui).show(ui, |ui| {
        ui.label(shortcut_label(shortcut_text(shortcut)));
    });
}

/// Keep the standard button's full-row interaction and right-align its shortcut badge.
pub fn menu_button_with_shortcut<'a>(
    ui: &Ui,
    label: impl egui::IntoAtoms<'a>,
    shortcut: KeyboardShortcut,
) -> egui::Button<'a> {
    menu_button_with_hint(ui, label, shortcut_text(shortcut))
}

/// A menu badge for input gestures such as Ctrl+Click.
pub fn menu_button_with_hint<'a>(ui: &Ui, label: impl egui::IntoAtoms<'a>, hint: String) -> egui::Button<'a> {
    let badge = egui::ContainerAtom::new(shortcut_label(hint)).frame(shortcut_frame(ui));
    egui::Button::new(label).right_text(egui::Atom::container(badge))
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
