use eframe::egui;

use crate::res::RGB_CHANNEL_LABEL_COLORS;

const CHANNELS: &[&str] = &["R", "G", "B", "A"];

const CHANNEL_TOOLTIPS: &[&str] = &[
    "Show/hide Red channel",
    "Show/hide Green channel",
    "Show/hide Blue channel",
    "Show/hide Alpha channel",
];

pub fn channel_toggle_ui(ui: &mut egui::Ui, channel: &mut [bool; 4], num_channels: usize) {
    ui.horizontal(|ui| {
        for c in 0..num_channels {
            let mut label = egui::RichText::new(CHANNELS[c]);
            if let Some(&color) = RGB_CHANNEL_LABEL_COLORS.get(c) {
                label = label.color(color);
            }
            ui.checkbox(&mut channel[c], label).on_hover_text(CHANNEL_TOOLTIPS[c]);
        }
    });
}
