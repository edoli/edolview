use std::{sync::mpsc, thread};

use eframe::egui::{self, vec2, Color32};

use crate::{
    model::{Image, ImageData, ImageSpec, MeanDim, Recti},
    res::icons::Icons,
    util::{color::ColorDisplay, math_ext::Vec2i},
};

struct ColorReadout {
    display_text: String,
    copy_text: String,
    swatch: Color32,
}

impl ColorReadout {
    fn new<T: Copy + Into<f64> + std::fmt::Display>(spec: &ImageSpec, values: &[T]) -> Self {
        let color: Vec<f32> = values.iter().map(|&value| value.into() as f32).collect();
        let swatch = if let [r, g, b, a] = color.as_slice() {
            let byte = |value: f32| (value * 255.0).round() as u8;
            Color32::from_rgba_unmultiplied(byte(*r), byte(*g), byte(*b), byte(*a))
        } else {
            color.to_color32()
        };
        Self {
            display_text: spec.pixel_values_to_string(values),
            copy_text: values
                .iter()
                .map(|value| {
                    if spec.dtype.is_floating() {
                        value.to_string()
                    } else {
                        // Undo integer normalization without exposing its floating-point noise.
                        ((*value).into() * spec.dtype.alpha()).round().to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(", "),
            swatch,
        }
    }

    fn mean(spec: &ImageSpec, values: &[f64]) -> Self {
        let mut readout = Self::new(spec, values);
        // Integer source pixels can still have a fractional regional mean.
        readout.display_text = values
            .iter()
            .map(|value| {
                format!("{:.4}", value * spec.dtype.alpha())
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_owned()
            })
            .collect::<Vec<_>>()
            .join(", ");
        readout.copy_text = values
            .iter()
            .map(|value| (value * spec.dtype.alpha()).to_string())
            .collect::<Vec<_>>()
            .join(", ");
        readout
    }
}

enum ReadoutUpdate {
    Cursor((u64, Vec2i), Result<ColorReadout, String>),
    Mean((u64, Recti), Result<ColorReadout, String>),
}

pub struct ContextValueCopy {
    pub label: &'static str,
    pub text: String,
}

pub struct ImageContextInfo {
    cursor_key: Option<(u64, Vec2i)>,
    mean_key: Option<(u64, Recti)>,
    cursor: Option<Result<ColorReadout, String>>,
    mean: Option<Result<ColorReadout, String>>,
    tx: mpsc::Sender<ReadoutUpdate>,
    rx: mpsc::Receiver<ReadoutUpdate>,
}

impl Default for ImageContextInfo {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            cursor_key: None,
            mean_key: None,
            cursor: None,
            mean: None,
            tx,
            rx,
        }
    }
}

impl ImageContextInfo {
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        image: &ImageData,
        cursor: Option<Vec2i>,
        rect: Recti,
    ) -> Vec<String> {
        let cursor_key = cursor.map(|pos| (image.id(), pos));
        if self.cursor_key != cursor_key {
            self.cursor_key = cursor_key;
            self.cursor = None;
            if let Some(key @ (_, pos)) = cursor_key {
                let image = image.clone();
                let tx = self.tx.clone();
                let ctx = ctx.clone();
                thread::spawn(move || {
                    let result = image
                        .get_pixel_at(pos.x, pos.y)
                        .map(|values| ColorReadout::new(&image.spec(), &values))
                        .map_err(|error| format!("Failed to read cursor color: {error}"));
                    let _ = tx.send(ReadoutUpdate::Cursor(key, result));
                    ctx.request_repaint();
                });
            }
        }

        let mean_key = (!rect.empty()).then_some((image.id(), rect));
        if self.mean_key != mean_key {
            self.mean_key = mean_key;
            self.mean = None;
            if let Some(key) = mean_key {
                let image = image.clone();
                let tx = self.tx.clone();
                let ctx = ctx.clone();
                // ImageData shares its storage. Reuse the existing mean processor
                // off the UI thread and retain the result while the region is unchanged.
                thread::spawn(move || {
                    let result = image
                        .mean_value_in_rect(rect, MeanDim::All)
                        .map(|values| ColorReadout::mean(&image.spec(), &values))
                        .map_err(|error| format!("Failed to read rectangle mean color: {error}"));
                    let _ = tx.send(ReadoutUpdate::Mean(key, result));
                    ctx.request_repaint();
                });
            }
        }

        self.poll()
    }

    pub fn poll(&mut self) -> Vec<String> {
        let mut errors = Vec::new();
        for update in self.rx.try_iter() {
            let (target, result) = match update {
                ReadoutUpdate::Cursor(key, result) if self.cursor_key == Some(key) => (&mut self.cursor, result),
                ReadoutUpdate::Mean(key, result) if self.mean_key == Some(key) => (&mut self.mean, result),
                _ => continue,
            };
            if let Err(error) = &result {
                errors.push(error.clone());
            }
            *target = Some(result);
        }
        errors
    }

    pub fn show(&self, ui: &mut egui::Ui, icons: &Icons) -> Option<ContextValueCopy> {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 4.0);
            ui.spacing_mut().button_padding = vec2(3.0, 2.0);
            let cursor_color = self.cursor.as_ref().and_then(|result| result.as_ref().ok());
            let mean_color = self.mean.as_ref().and_then(|result| result.as_ref().ok());
            let position = self.cursor_key.map(|(_, pos)| format!("{}, {}", pos.x, pos.y));
            let bound = self.mean_key.map(|(_, rect)| rect.to_string());
            let cursor_placeholder = self.placeholder(self.cursor_key.is_some(), &self.cursor, "Outside image");
            let mean_placeholder = self.placeholder(self.mean_key.is_some(), &self.mean, "No selection");
            let columns = ReadoutColumns::new(
                ui,
                [
                    (cursor_color.map(|color| color.display_text.as_str()), cursor_placeholder),
                    (position.as_deref(), "Outside image"),
                    (mean_color.map(|color| color.display_text.as_str()), mean_placeholder),
                    (bound.as_deref(), "No selection"),
                ],
            );
            let mut copy = None;
            ui.separator();
            ui.strong("Cursor");
            if columns.value_row(
                ui,
                icons,
                "Color",
                cursor_color.map(|color| color.display_text.as_str()),
                cursor_color.map(|color| color.swatch),
                cursor_placeholder,
                "Source pixel values before colormap and display adjustments",
            ) {
                copy = cursor_color.map(|color| ContextValueCopy {
                    label: "cursor color",
                    text: color.copy_text.clone(),
                });
            }
            if columns.value_row(
                ui,
                icons,
                "Position",
                position.as_deref(),
                None,
                "Outside image",
                "x, y in pixels (zero-based)",
            ) {
                copy = position.map(|text| ContextValueCopy {
                    label: "cursor position",
                    text,
                });
            }

            ui.separator();
            ui.strong("Rectangle");
            if columns.value_row(
                ui,
                icons,
                "Color",
                mean_color.map(|color| color.display_text.as_str()),
                mean_color.map(|color| color.swatch),
                mean_placeholder,
                "Mean source values inside the selected rectangle",
            ) {
                copy = mean_color.map(|color| ContextValueCopy {
                    label: "rectangle mean color",
                    text: color.copy_text.clone(),
                });
            }
            if columns.value_row(
                ui,
                icons,
                "Bound",
                bound.as_deref(),
                None,
                "No selection",
                "x, y, width, height in pixels",
            ) {
                copy = bound.map(|text| ContextValueCopy {
                    label: "rectangle bound",
                    text,
                });
            }
            copy
        })
        .inner
    }

    fn placeholder<'a>(
        &self,
        available: bool,
        value: &Option<Result<ColorReadout, String>>,
        empty: &'a str,
    ) -> &'a str {
        if !available {
            empty
        } else if matches!(value, Some(Err(_))) {
            "Unavailable"
        } else {
            "Reading…"
        }
    }
}

struct ReadoutColumns {
    label_width: f32,
    text_width: f32,
}

impl ReadoutColumns {
    fn new(ui: &egui::Ui, values: [(Option<&str>, &str); 4]) -> Self {
        let measure = |text: &str, style: egui::TextStyle| {
            ui.painter()
                .layout_no_wrap(text.to_owned(), style.resolve(ui.style()), ui.visuals().text_color())
                .size()
                .x
                .ceil()
        };
        Self {
            label_width: measure("Position", egui::TextStyle::Body),
            text_width: values
                .into_iter()
                .map(|(value, placeholder)| match value {
                    Some(text) => measure(text, egui::TextStyle::Monospace),
                    None => measure(placeholder, egui::TextStyle::Body),
                })
                .fold(0.0, f32::max),
        }
    }

    fn value_row(
        &self,
        ui: &mut egui::Ui,
        icons: &Icons,
        label: &str,
        value: Option<&str>,
        swatch: Option<Color32>,
        placeholder: &str,
        description: &str,
    ) -> bool {
        // Measure all rows together on each frame so columns align without retaining
        // the width of a previously opened menu with longer values.
        ui.allocate_ui_with_layout(vec2(0.0, 22.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.allocate_ui_with_layout(
                vec2(self.label_width, 22.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.set_min_width(self.label_width);
                    ui.label(egui::RichText::new(label).weak()).on_hover_text(description);
                },
            );
            if let Some(color) = swatch {
                color_swatch(ui, color);
            } else {
                ui.allocate_exact_size(vec2(18.0, 18.0), egui::Sense::hover());
            }
            ui.allocate_ui_with_layout(
                vec2(self.text_width, 22.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.set_min_width(self.text_width);
                    let text = match value {
                        Some(text) => egui::RichText::new(text).monospace(),
                        None => egui::RichText::new(placeholder).weak(),
                    };
                    ui.add(egui::Label::new(text).extend()).on_hover_text(description);
                },
            );
            let icon = icons
                .get_copy(ui.ctx())
                .fit_to_exact_size(vec2(16.0, 16.0))
                .tint(ui.visuals().text_color())
                .alt_text(format!("Copy {label}"));
            ui.add_enabled(value.is_some(), egui::Button::image(icon).min_size(vec2(22.0, 22.0)))
                .on_hover_text(format!("Copy {label}"))
                .clicked()
        })
        .inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{model::PixelType, util::math_ext::vec2i};
    use std::time::Duration;

    #[test]
    fn readouts_preserve_source_units_alpha_and_fractional_means() {
        let spec = ImageSpec::new(1, 1, 4, PixelType::U8);
        let cursor = ColorReadout::new(&spec, &[128.0 / 255.0, 64.0 / 255.0, 32.0 / 255.0, 128.0 / 255.0]);
        assert_eq!(cursor.display_text, "128, 64, 32, 128");
        assert_eq!(cursor.copy_text, "128, 64, 32, 128");
        assert_eq!(cursor.swatch, Color32::from_rgba_unmultiplied(128, 64, 32, 128));
        assert_eq!(
            ColorReadout::mean(&spec, &[0.5, 0.0, 1.0, 1.0]).display_text,
            "127.5, 0, 255, 255"
        );
        let spec = ImageSpec::new(1, 1, 1, PixelType::F32);
        let gray = ColorReadout::new(&spec, &[0.5]);
        assert_eq!(gray.display_text, "0.5000");
        assert_eq!(gray.swatch, Color32::from_gray(128));
    }

    #[test]
    fn copying_preserves_precision_instead_of_using_the_display_rounding() {
        let spec = ImageSpec::new(1, 1, 1, PixelType::F32);
        let pixel = 0.12345679_f32;
        let cursor = ColorReadout::new(&spec, &[pixel]);
        assert_eq!(cursor.display_text, "0.1235");
        assert_eq!(cursor.copy_text.parse::<f32>().unwrap().to_bits(), pixel.to_bits());
        assert_ne!(cursor.copy_text, cursor.display_text);

        let average = 0.1234567890123456_f64;
        let mean = ColorReadout::mean(&spec, &[average]);
        assert_eq!(mean.display_text, "0.1235");
        assert_eq!(mean.copy_text.parse::<f64>().unwrap().to_bits(), average.to_bits());
        let integer_spec = ImageSpec::new(1, 1, 1, PixelType::U8);
        let scaled_mean = ColorReadout::mean(&integer_spec, &[average]);
        assert_eq!(
            scaled_mean.copy_text.parse::<f64>().unwrap().to_bits(),
            (average * 255.0).to_bits()
        );
        assert_eq!(ColorReadout::new(&integer_spec, &[128.0_f32 / 255.0]).copy_text, "128");
    }

    #[test]
    fn swatch_is_a_single_color_layer_that_fades_with_the_menu() {
        let ctx = egui::Context::default();
        for alpha in [255, 128, 0] {
            let color = Color32::from_rgba_unmultiplied(5, 3, 2, alpha);
            for opacity in [0.1, 0.5, 1.0] {
                let mut rect = egui::Rect::NOTHING;
                let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                    rect = ui
                        .scope(|ui| {
                            ui.multiply_opacity(opacity);
                            color_swatch(ui, color);
                        })
                        .response
                        .rect;
                });
                output.textures_delta.clear();
                let fills = output
                    .shapes
                    .iter()
                    .filter_map(|clipped| match &clipped.shape {
                        egui::Shape::Rect(shape) if shape.rect == rect && shape.stroke.width == 0.0 => Some(shape.fill),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                assert_eq!(fills, [color.gamma_multiply(opacity)]);
                // Both halves must show the same source color, with only the
                // menu's opacity applied. No checker or other fill is underneath.
                for offset in [4.5, 13.5] {
                    let point = rect.min + vec2(offset, 4.5);
                    let pixel = output.shapes.iter().fold(Color32::BLACK, |pixel, clipped| {
                        if let egui::Shape::Rect(shape) = &clipped.shape {
                            if clipped.clip_rect.contains(point) && shape.rect.contains(point) {
                                return pixel.blend(shape.fill);
                            }
                        }
                        pixel
                    });
                    assert_eq!(
                        pixel,
                        Color32::BLACK.blend(color.gamma_multiply(opacity)),
                        "alpha={alpha}, menu opacity={opacity}"
                    );
                }
            }
        }
    }

    #[test]
    fn menu_columns_align_and_width_fits_when_reopened_after_a_wider_readout() {
        fn menu_size(ctx: &egui::Context, values: &[f32]) -> egui::Vec2 {
            // Inspector values are captured at opening, so changing images means
            // closing the previous popup and opening it again for the new source.
            for _ in 0..2 {
                let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
                output.textures_delta.clear();
            }
            let spec = ImageSpec::new(1, 1, values.len() as i32, PixelType::F32);
            let info = ImageContextInfo {
                cursor_key: Some((1, vec2i(128, 256))),
                mean_key: Some((1, Recti::from_min_size(vec2i(100, 200), vec2i(320, 180)))),
                cursor: Some(Ok(ColorReadout::new(&spec, values))),
                mean: Some(Ok(ColorReadout::mean(
                    &spec,
                    &values.iter().map(|value| *value as f64).collect::<Vec<_>>(),
                ))),
                ..Default::default()
            };
            let icons = Icons::new();
            let labels = ["Color", "Position", "Color", "Bound"];
            let readouts = [
                info.cursor.as_ref().unwrap().as_ref().unwrap().display_text.clone(),
                "128, 256".to_owned(),
                info.mean.as_ref().unwrap().as_ref().unwrap().display_text.clone(),
                info.mean_key.unwrap().1.to_string(),
            ];
            let mut size = egui::Vec2::ZERO;
            for frame in 0..8 {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1000.0, 700.0))),
                        ..Default::default()
                    },
                    |ui| {
                        let image = ui.allocate_response(vec2(300.0, 300.0), egui::Sense::click());
                        let popup = egui::Popup::context_menu(&image)
                            .open(true)
                            .at_position(egui::pos2(0.0, 0.0))
                            .show(|ui| {
                                let _ = ui.button("Copy Selected Image (Ctrl+C)");
                                let _ = ui.button("Save Selected Image As... (Ctrl+S)");
                                info.show(ui, &icons);
                            })
                            .unwrap();
                        size = popup.response.rect.size();
                    },
                );
                output.textures_delta.clear();
                if frame == 7 {
                    // Inspect the positions actually painted by egui, since a
                    // requested child width alone does not reserve that space.
                    let mut label_x = Vec::new();
                    let mut value_x = Vec::new();
                    let mut copy_x = Vec::new();
                    for clipped in &output.shapes {
                        match &clipped.shape {
                            egui::Shape::Text(text) => {
                                if labels.contains(&text.galley.job.text.as_str()) {
                                    label_x.push(text.pos.x);
                                }
                                if readouts.contains(&text.galley.job.text) {
                                    value_x.push(text.pos.x);
                                }
                            }
                            egui::Shape::Mesh(mesh) => copy_x.push(mesh.calc_bounds().left()),
                            egui::Shape::Rect(rect) if rect.brush.is_some() => copy_x.push(rect.rect.left()),
                            _ => {}
                        }
                    }
                    for (name, positions) in [("labels", label_x), ("values", value_x), ("copy icons", copy_x)] {
                        assert_eq!(positions.len(), 4, "{name}: {positions:?}");
                        assert!(
                            positions.iter().all(|x| (x - positions[0]).abs() <= 1.0),
                            "{name} are misaligned: {positions:?}"
                        );
                    }
                }
            }
            size
        }
        let ctx = egui::Context::default();
        crate::ui::style::apply(&ctx);
        let mono = menu_size(&ctx, &[0.12345679]);
        let rgba = menu_size(&ctx, &[0.12345679; 4]);
        let mono_again = menu_size(&ctx, &[0.12345679]);
        eprintln!("menu sizes: mono={mono:?}, rgba={rgba:?}, mono again={mono_again:?}");
        assert!(mono.x < rgba.x && rgba.x < 420.0, "{mono:?}, {rgba:?}");
        assert!(mono_again.x <= mono.x + 1.0, "{mono:?}, {mono_again:?}");
        assert!(rgba.y < 230.0, "{rgba:?}");
    }

    #[test]
    fn background_cursor_readout_ignores_stale_results_and_invalid_locations() {
        let ctx = egui::Context::default();
        let image = ImageData::from_f32(ImageSpec::new(1, 1, 1, PixelType::F32), vec![0.25]).unwrap();
        let mut info = ImageContextInfo::default();
        assert!(info.update(&ctx, &image, Some(vec2i(0, 0)), Recti::ZERO).is_empty());
        if info.cursor.is_none() {
            let update = info.rx.recv_timeout(Duration::from_secs(2)).unwrap();
            info.tx.send(update).unwrap();
            assert!(info.update(&ctx, &image, Some(vec2i(0, 0)), Recti::ZERO).is_empty());
        }
        assert_eq!(info.cursor.as_ref().unwrap().as_ref().unwrap().display_text, "0.2500");
        assert!(info.mean_key.is_none());
        assert!(info.update(&ctx, &image, None, Recti::ZERO).is_empty());
        info.tx
            .send(ReadoutUpdate::Cursor((image.id(), vec2i(0, 0)), Err("stale error".to_owned())))
            .unwrap();
        assert!(info.update(&ctx, &image, None, Recti::ZERO).is_empty());
        assert!(info.cursor.is_none());
    }
}

fn color_swatch(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(18.0, 18.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, color);
    ui.painter().rect_stroke(
        rect,
        0.0,
        ui.visuals().widgets.noninteractive.bg_stroke,
        egui::StrokeKind::Outside,
    );
}
