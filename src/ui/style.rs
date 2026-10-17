use eframe::egui::{self, vec2, CornerRadius, Visuals};

pub(super) fn apply(ctx: &egui::Context) {
    ctx.global_style_mut(|style| {
        style.visuals = Visuals::dark();

        // Preserve egui 0.35's compact spacing and corner radii while using 0.37's widgets.
        style.spacing.button_padding = vec2(4.0, 1.0);
        style.spacing.interact_size = vec2(40.0, 18.0);
        let widgets = &mut style.visuals.widgets;
        widgets.noninteractive.corner_radius = CornerRadius::same(2);
        widgets.inactive.corner_radius = CornerRadius::same(2);
        widgets.hovered.corner_radius = CornerRadius::same(3);
        widgets.active.corner_radius = CornerRadius::same(2);
        widgets.open.corner_radius = CornerRadius::same(2);
    });
}
