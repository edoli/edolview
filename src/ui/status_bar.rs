use eframe::egui;

use super::component::egui_ext::{Size, UiExt};
use crate::{
    model::{AppState, Image, MeanDim, Recti, SharedAsset},
    settings::AngleDisplayUnit,
    util::color::ColorDisplay,
};

pub(super) struct StatusBar {
    last_marquee_rect: Recti,
    marquee_rect_text: String,
}

impl StatusBar {
    pub(super) fn new(rect: Recti) -> Self {
        Self {
            last_marquee_rect: rect,
            marquee_rect_text: rect.to_string(),
        }
    }

    pub(super) fn reset_marquee_text(&mut self, rect: Recti) {
        self.last_marquee_rect = rect;
        self.marquee_rect_text = rect.to_string();
    }

    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut AppState,
        active_asset: Option<&SharedAsset>,
        zoom: f32,
        angle_unit: &mut AngleDisplayUnit,
    ) {
        let mut frame = egui::Frame::side_top_panel(ui.style());
        frame.inner_margin.bottom = 0;
        egui::Panel::bottom("bottom").frame(frame).show(ui, |ui| {
            let cursor_pixel = state.cursor_pixel_pos();
            let cursor_label =
                cursor_pixel.map_or_else(|| "Cursor: -, -".to_owned(), |p| format!("Cursor: {}, {}", p.x, p.y));
            let colors = active_asset.map(|asset| {
                let image = asset.image();
                let spec = image.spec();
                let cursor = image
                    .get_pixel_at(cursor_pixel.map_or(-1, |p| p.x), cursor_pixel.map_or(-1, |p| p.y))
                    .map(|pixel| pixel.iter().copied().collect::<Vec<f32>>())
                    .unwrap_or_else(|_| vec![0.0; spec.channels as usize]);
                let mean = image
                    .mean_value_in_rect(state.marquee_rect, MeanDim::All)
                    .map(|mean| mean.iter().map(|&v| v as f32).collect::<Vec<f32>>())
                    .unwrap_or_else(|_| vec![0.0; spec.channels as usize]);
                (cursor, mean, spec.dtype)
            });
            let image_label = state.asset.as_ref().map_or_else(
                || "No image loaded".to_owned(),
                |asset| {
                    let spec = asset.image().spec();
                    format!("{}×{} | {}", spec.width, spec.height, spec.dtype.name())
                },
            );
            let zoom_label = format!("Zoom: {:.2}x", zoom);
            let unit = *angle_unit;
            let angle_label = format_marquee_angle(marquee_angle(state.marquee_rect, unit));
            let font = egui::TextStyle::Body.resolve(ui.style());
            let text_width = |text: &str| {
                ui.painter()
                    .layout_no_wrap(text.to_owned(), font.clone(), ui.visuals().text_color())
                    .size()
                    .x
                    .ceil()
            };
            let color_width = colors.as_ref().map_or(0.0, |(cursor, mean, dtype)| {
                [cursor, mean]
                    .into_iter()
                    .map(|color| {
                        let text = if dtype.is_floating() {
                            color.to_rgba_string()
                        } else {
                            color.to_rgba_int_string(dtype.alpha())
                        };
                        text_width(&text)
                    })
                    .fold(0.0_f32, f32::max)
                    + ui.text_style_height(&egui::TextStyle::Body)
                    + ui.spacing().item_spacing.y
                    + ui.spacing().item_spacing.x * 0.5
            });
            // Capture may replace the editor text this frame when the selection changes.
            let rect_label = if state.marquee_rect != self.last_marquee_rect {
                state.marquee_rect.to_string()
            } else {
                self.marquee_rect_text.clone()
            };
            let minimum = [
                color_width,
                text_width(&cursor_label).max(text_width(&rect_label) + ui.spacing().button_padding.x * 2.0 + 2.0),
                text_width(&angle_label),
                0.0,
                text_width(&image_label).max(text_width(&zoom_label)),
            ];
            let (spacing, rows) = column_rows(ui.available_width(), ui.spacing().item_spacing.x, minimum);
            ui.spacing_mut().item_spacing.x = spacing;
            let mut row_heights = (ui.text_style_height(&egui::TextStyle::Body), ui.spacing().interact_size.y);
            for (row_index, widths) in rows.into_iter().enumerate() {
                ui.push_id(row_index, |ui| {
                    ui.columns_sized(widths.map(Size::exact), |columns| {
                        if widths[1] > 0.0 {
                            row_heights = columns[1]
                                .vertical(|ui| {
                                    let cursor_response = ui.label(&cursor_label);

                                    ui.spacing_mut().text_edit_width = ui.available_width();
                                    let response = ui
                                        .text_edit_value_capture(
                                            &mut self.marquee_rect_text,
                                            &mut state.marquee_rect,
                                            &mut self.last_marquee_rect,
                                        )
                                        .on_hover_text("Selected marquee rectangle bounds (x, y, width, height)");
                                    let row_heights = (cursor_response.rect.height(), response.rect.height());
                                    response.context_menu(|ui| {
                                        if ui.button("Copy Numpy Indexing").clicked() {
                                            let rect = state.marquee_rect.validate();
                                            let np_indexing = format!(
                                                "{}:{}, {}:{}",
                                                rect.min.y, rect.max.y, rect.min.x, rect.max.x,
                                            );
                                            arboard::Clipboard::new()
                                                .and_then(|mut cb| cb.set_text(np_indexing))
                                                .unwrap_or_else(|e| {
                                                    eprintln!("Failed to copy numpy indexing to clipboard: {e}");
                                                });
                                            ui.close();
                                        }
                                        if ui.button("Copy x1, y1, x2, y2").clicked() {
                                            let rect = state.marquee_rect.validate();
                                            let rect_str = format!(
                                                "{}, {}, {}, {}",
                                                rect.min.x, rect.min.y, rect.max.x, rect.max.y
                                            );
                                            arboard::Clipboard::new()
                                                .and_then(|mut cb| cb.set_text(rect_str))
                                                .unwrap_or_else(|e| {
                                                    eprintln!("Failed to copy rect to clipboard: {e}");
                                                });
                                            ui.close();
                                        }
                                    });
                                    row_heights
                                })
                                .inner;
                        }
                        let (cursor_row_height, selection_row_height) = row_heights;

                        if widths[0] > 0.0 {
                            columns[0].vertical(|ui| {
                                if let Some((cursor_color, mean_color, dtype)) = &colors {
                                    ui.label_with_colored_rect(cursor_color.clone(), *dtype, cursor_row_height);
                                    ui.label_with_colored_rect(mean_color.clone(), *dtype, selection_row_height);
                                }
                            });
                        }

                        if widths[2] > 0.0 {
                            columns[2].vertical(|ui| {
                                ui.label("");
                                let mut show_radians = unit == crate::settings::AngleDisplayUnit::Radians;
                                ui.allocate_ui_with_layout(
                                    egui::vec2(ui.available_width(), selection_row_height),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.label(&angle_label).on_hover_text(marquee_angle_tooltip(unit)).context_menu(
                                            |ui| {
                                                if ui.checkbox(&mut show_radians, "Show in radians").changed() {
                                                    *angle_unit = if show_radians {
                                                        crate::settings::AngleDisplayUnit::Radians
                                                    } else {
                                                        crate::settings::AngleDisplayUnit::Degrees
                                                    };
                                                    ui.close();
                                                }
                                            },
                                        );
                                    },
                                );
                            });
                        }

                        if widths[4] > 0.0 {
                            columns[4].with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                                ui.add(egui::Label::new(&image_label).wrap());
                                ui.allocate_ui_with_layout(
                                    egui::vec2(ui.available_width(), selection_row_height),
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| ui.label(&zoom_label),
                                );
                            });
                        }
                    });
                });
            }
        });
    }
}

#[derive(Clone, Copy)]
enum MarqueeAngleDisplay {
    Degrees(f32),
    Radians(f32),
}

fn marquee_angle_radian(rect: Recti) -> Option<f32> {
    let rect = rect.validate();
    if rect.empty() {
        return None;
    }

    Some((rect.height() as f32).atan2(rect.width() as f32))
}

fn marquee_angle(rect: Recti, unit: crate::settings::AngleDisplayUnit) -> Option<MarqueeAngleDisplay> {
    let angle_radians = marquee_angle_radian(rect)?;
    Some(match unit {
        crate::settings::AngleDisplayUnit::Degrees => MarqueeAngleDisplay::Degrees(angle_radians.to_degrees()),
        crate::settings::AngleDisplayUnit::Radians => MarqueeAngleDisplay::Radians(angle_radians),
    })
}

fn marquee_angle_tooltip(unit: crate::settings::AngleDisplayUnit) -> &'static str {
    match unit {
        crate::settings::AngleDisplayUnit::Degrees => {
            "Arc tan of marquee aspect ratio (height / width), shown in degrees."
        }
        crate::settings::AngleDisplayUnit::Radians => {
            "Arc tan of marquee aspect ratio (height / width), shown in radians."
        }
    }
}

fn format_marquee_angle(angle: Option<MarqueeAngleDisplay>) -> String {
    match angle {
        Some(MarqueeAngleDisplay::Degrees(value)) => format!("{value:.2}°"),
        Some(MarqueeAngleDisplay::Radians(value)) => format!("{value:.3}"),
        None => "-".to_string(),
    }
}

/// Column widths for color, coordinates, angle, a spacer, and image details.
fn column_rows(available: f32, spacing: f32, minimum: [f32; 5]) -> (f32, Vec<[f32; 5]>) {
    let preferred = [400.0_f32, 160.0, 40.0, 0.0, 108.0];
    let minimum_total: f32 = minimum.iter().sum();
    // Remove only unused space; never squeeze a column below its current text.
    let spacing = if minimum_total <= available {
        spacing.min((available - minimum_total) / 4.0)
    } else {
        spacing.min(available / 4.0)
    };
    let budget = (available - spacing * 4.0).max(0.0);
    let fit = |indices: &[usize]| {
        let mut widths = [0.0; 5];
        let natural: f32 = indices.iter().map(|&i| minimum[i]).sum();
        let slack: f32 = indices.iter().map(|&i| (preferred[i] - minimum[i]).max(0.0)).sum();
        let fraction = if slack > 0.0 {
            ((budget - natural) / slack).clamp(0.0, 1.0)
        } else {
            0.0
        };
        for &i in indices {
            widths[i] = (minimum[i] + (preferred[i] - minimum[i]).max(0.0) * fraction).min(budget);
        }
        widths[3] = (budget - widths.iter().sum::<f32>()).max(0.0);
        widths
    };
    if minimum_total <= budget {
        return (spacing, vec![fit(&[0, 1, 2, 4])]);
    }

    // Very long values need another row after all horizontal padding is gone.
    let mut rows = Vec::new();
    for pair in [[0, 1], [2, 4]] {
        if pair.iter().map(|&i| minimum[i]).sum::<f32>() <= budget {
            rows.push(fit(&pair));
        } else {
            // Keep the coordinate editor in the first row so resizing does not
            // change its egui ID and interrupt an in-progress edit.
            let order = if pair == [0, 1] { [1, 0] } else { pair };
            rows.extend(order.map(|i| fit(&[i])));
        }
    }
    (spacing, rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_layout_preserves_existing_positions_and_narrow_layout_fits_content() {
        let minimum = [120.0, 130.0, 40.0, 0.0, 105.0];
        let (gap, rows) = column_rows(1280.0, 8.0, minimum);
        assert_eq!(gap, 8.0);
        assert_eq!(rows, vec![[400.0, 160.0, 40.0, 540.0, 108.0]]);
        for width in [740.0, 739.0, 720.0, 500.0, 427.0, 410.0, 395.0] {
            let (gap, rows) = column_rows(width, 8.0, minimum);
            assert_eq!(rows.len(), 1);
            assert!((rows[0].iter().sum::<f32>() + gap * 4.0 - width).abs() < 0.001);
            for i in [0, 1, 2, 4] {
                assert!(rows[0][i] >= minimum[i]);
            }
        }
    }

    #[test]
    fn long_values_get_space_and_wrap_groups_only_when_they_cannot_fit_together() {
        let minimum = [360.0, 240.0, 48.0, 0.0, 160.0];
        for width in [1280.0, 840.0, 808.0, 807.0, 720.0, 500.0] {
            let (gap, rows) = column_rows(width, 8.0, minimum);
            assert_eq!(rows.len() > 1, width < minimum.iter().sum());
            for row in &rows {
                assert!((row.iter().sum::<f32>() + gap * 4.0 - width).abs() < 0.001);
            }
            for i in [0, 1, 2, 4] {
                assert_eq!(rows.iter().filter(|row| row[i] >= minimum[i]).count(), 1);
            }
        }
    }
}
