use eframe::egui::{self, Color32};

use crate::{
    model::ImageData,
    res::icons::Icons,
    switch,
    ui::{
        component::{
            egui_ext::{Size, UiExt},
            CustomSlider,
        },
        gpu::ScaleMode,
    },
    util::expression::parse_number_expression,
};

fn scale_mode_name(scale_mode: ScaleMode) -> &'static str {
    match scale_mode {
        ScaleMode::Linear => "Linear",
        ScaleMode::Absolute => "Absolute",
        ScaleMode::Inverse => "Inverse",
        ScaleMode::Log => "Log",
    }
}

fn scale_mode_icon<'a>(icons: &'a Icons, ctx: &'a egui::Context, scale_mode: ScaleMode) -> egui::Image<'a> {
    match scale_mode {
        ScaleMode::Linear => icons.get_scale_linear(ctx),
        ScaleMode::Absolute => icons.get_scale_absolute(ctx),
        ScaleMode::Inverse => icons.get_scale_inverse(ctx),
        ScaleMode::Log => icons.get_scale_log(ctx),
    }
}

#[allow(clippy::too_many_arguments)] // These controls are edited together by the caller.
pub fn display_controls_ui(
    ui: &mut egui::Ui,
    icons: &Icons,
    image: &ImageData,
    channel: i32,
    scale_mode: &mut ScaleMode,
    auto_minmax: &mut bool,
    min_v: &mut f32,
    max_v: &mut f32,
) {
    let ctx = ui.ctx().clone();
    let normalize_enabled = scale_mode.auto_normalize_enabled();
    if !normalize_enabled && *auto_minmax {
        *auto_minmax = false;
    }
    let locked = *auto_minmax && normalize_enabled;

    ui.menu_image_button(scale_mode_icon(icons, &ctx, *scale_mode), |ui| {
        for mode in [ScaleMode::Linear, ScaleMode::Absolute, ScaleMode::Inverse] {
            if ui
                .add(egui::Button::image_and_text(
                    scale_mode_icon(icons, &ctx, mode),
                    scale_mode_name(mode),
                ))
                .on_hover_text(scale_mode_name(mode))
                .clicked()
            {
                *scale_mode = mode;
                ui.close();
            }
        }
    })
    .response
    .on_hover_text(format!("Pixel value display mode: {}", scale_mode_name(*scale_mode)));

    ui.separator();

    let original_spacing = ui.style().spacing.item_spacing;
    ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);

    let mut tmp_min = *min_v;
    let mut tmp_max = *max_v;

    if locked {
        if channel == -1 {
            if scale_mode == &ScaleMode::Absolute {
                tmp_min = image.minmax().total_min_abs();
                tmp_max = image.minmax().total_max_abs();
            } else {
                tmp_min = image.minmax().total_min();
                tmp_max = image.minmax().total_max();
            }
        } else {
            if scale_mode == &ScaleMode::Absolute {
                tmp_min = image.minmax().min_abs(channel as usize);
                tmp_max = image.minmax().max_abs(channel as usize);
            } else {
                tmp_min = image.minmax().min(channel as usize);
                tmp_max = image.minmax().max(channel as usize);
            }
        }
    }

    ui.columns_sized(
        [
            Size::remainder(1.0),
            Size::exact(24.0),
            Size::remainder(1.0),
            Size::exact(original_spacing.x),
            Size::exact(16.0),
        ],
        |columns| {
            // Min control
            columns[0].add_enabled_ui(!locked, |ui| {
                ui.add(
                    egui::DragValue::new(switch!(locked => &mut tmp_min, min_v))
                        .speed(0.01)
                        .custom_parser(parse_number_expression),
                )
                .on_hover_text("Min value")
                .context_menu(|ui| {
                    if ui.button("Reset").clicked() {
                        *min_v = 0.0;
                        ui.close();
                    }
                });
            });

            // Swap min/max button
            if columns[1]
                .add_enabled_ui(!locked, |ui| ui.button("↔").on_hover_text("Switch min/max"))
                .inner
                .clicked()
            {
                std::mem::swap(min_v, max_v);
            }

            // Max control
            columns[2].add_enabled_ui(!locked, |ui| {
                ui.add(
                    egui::DragValue::new(switch!(locked => &mut tmp_max, max_v))
                        .speed(0.01)
                        .custom_parser(parse_number_expression),
                )
                .on_hover_text("Max value")
                .context_menu(|ui| {
                    if ui.button("Reset").clicked() {
                        *max_v = 1.0;
                        ui.close();
                    }
                });
            });

            // Auto min/max toggle
            let auto_tip = if normalize_enabled {
                "Use min / max values of image for normalization"
            } else {
                "Auto min/max disabled for Inverse/Log modes"
            };
            columns[4].add_enabled_ui(normalize_enabled, |ui| {
                let response = ui.toggle_icon(auto_minmax, icons.get_normalize(&ctx), auto_tip);
                if locked {
                    response.context_menu(|ui| {
                        if ui.button("Set current values as manual").clicked() {
                            *min_v = tmp_min;
                            *max_v = tmp_max;
                            *auto_minmax = false;
                            ui.close();
                        }
                    });
                }
            });
        },
    );
}

pub fn display_profile_slider(
    ui: &mut egui::Ui,
    value: &mut f32,
    min: f32,
    max: f32,
    baseline: f64,
    text: &str,
) -> egui::Response {
    let value_width = 56.0;
    let font_id = egui::TextStyle::Body.resolve(ui.style());
    let label_width = ["Offset", "Exposure", "Gamma", text]
        .into_iter()
        .map(|label| {
            ui.painter()
                .layout_no_wrap(label.to_owned(), font_id.clone(), ui.visuals().text_color())
                .size()
                .x
        })
        .fold(0.0_f32, f32::max);
    // Keep the fields aligned and fit the full row inside the sidebar at any font scale.
    let reserved_width = value_width + label_width + 2.0 * ui.spacing().item_spacing.x;
    ui.spacing_mut().slider_width = (ui.available_width() - reserved_width).max(0.0);
    let response = ui.add(
        CustomSlider::new(value, min..=max)
            .value_width(value_width)
            .text(text)
            .step_by(0.01)
            .move_by(0.1)
            .smart_aim(false)
            .handle_shape(egui::style::HandleShape::Rect { aspect_ratio: 0.5 })
            .trailing_fill(true)
            .trailing_start(baseline)
            .trailing_color_pos(Color32::from_hex("#4EADE4").unwrap())
            .trailing_color_neg(Color32::from_hex("#FF6B6B").unwrap()),
    );
    response.context_menu(|ui| {
        if ui.button("Reset").clicked() {
            *value = baseline as f32;
            ui.close();
        }
    });
    response
}
