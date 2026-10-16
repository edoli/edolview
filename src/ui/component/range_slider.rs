use std::ops::RangeInclusive;

use eframe::egui::{self, emath::Numeric};

/// Native range slider with the app's handle shape and primary-button-only editing.
/// `configure` exposes egui's builder without duplicating its API here.
pub fn range_slider_ui<'a, Num: Numeric>(
    ui: &mut egui::Ui,
    low: &'a mut Num,
    high: &'a mut Num,
    range: impl Into<RangeInclusive<Num>>,
    configure: impl FnOnce(egui::RangeSlider<'a>) -> egui::RangeSlider<'a>,
) -> egui::Response {
    // egui 0.37's RangeSlider writes values for any pointer button.
    // Include the release frame so opening a context menu cannot move a handle.
    // With no pointer gesture, keyboard and accessibility edits still work.
    let allow_update = ui.input(|i| {
        (!i.pointer.any_down() && !i.pointer.any_released()) || i.pointer.primary_down() || i.pointer.primary_released()
    });
    let range = range.into();
    let slider = egui::RangeSlider::from_get_set(
        range.start().to_f64()..=range.end().to_f64(),
        move |value| {
            if let Some(value) = value.filter(|_| allow_update) {
                *low = Num::from_f64(value);
            }
            low.to_f64()
        },
        move |value| {
            if let Some(value) = value.filter(|_| allow_update) {
                *high = Num::from_f64(value);
            }
            high.to_f64()
        },
    )
    .handle_shape(egui::style::HandleShape::Rect { aspect_ratio: 0.5 });
    let slider = if Num::INTEGRAL { slider.integer() } else { slider };
    ui.add(configure(slider))
}

/// Opens a context menu for the range slider's drag-only response.
pub fn range_slider_context_menu(response: &egui::Response, contents: impl FnOnce(&mut egui::Ui)) {
    // Adding a click interaction over the drag response would steal its handles.
    let open_menu = (response.hovered() && response.ctx.input(|i| i.pointer.secondary_clicked()))
        .then_some(egui::SetOpenCommand::Bool(true));
    egui::Popup::context_menu(response).open_memory(open_menu).show(contents);
}
