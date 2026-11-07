use eframe::egui::{self, pos2, Color32, Pos2, Rect, Sense, Stroke, TextStyle, Vec2};

use super::{channel_toggle_ui, CopyExport, ExportAction, SaveExport};
use crate::model::{Histogram, HistogramRange, HistogramWorker, Image, ImageData, PixelType, HISTOGRAM_BINS};
use crate::res::{histogram_bar_colors, HISTOGRAM_GRID_STROKE, HISTOGRAM_LABEL_COLORS, NOTICE_WARNING_TEXT};
use crate::util::series::channel_label;
#[cfg(debug_assertions)]
use crate::util::timer::ScopedTimer;

#[derive(Clone)]
struct HistogramHoverState {
    bin_idx: usize,
    text: String,
    lines: Vec<(String, Color32)>,
}

#[derive(Default, PartialEq)]
enum RangeMode {
    #[default]
    Unit,
    Data,
    Custom,
}

#[derive(Default)]
pub struct HistogramPanel {
    mode: RangeMode,
    custom_range: [f64; 2],
    image_id: Option<u64>,
    worker: HistogramWorker,
}

impl HistogramPanel {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        image: &ImageData,
        mask: &mut [bool; 4],
    ) -> color_eyre::Result<Option<ExportAction>> {
        let spec = image.spec();
        let scale = spec.dtype.alpha();
        if self.image_id != Some(image.id()) {
            self.image_id = Some(image.id());
            self.custom_range = [0.0, scale];
        }
        ui.horizontal(|ui| {
            ui.label("Range");
            egui::ComboBox::from_id_salt("histogram_range")
                .selected_text(match self.mode {
                    RangeMode::Unit => "0–1 (normalized)",
                    RangeMode::Data => "Full data range",
                    RangeMode::Custom => "Custom",
                })
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.mode, RangeMode::Unit, "0–1 (normalized)");
                    ui.selectable_value(&mut self.mode, RangeMode::Data, "Full data range");
                    ui.selectable_value(&mut self.mode, RangeMode::Custom, "Custom");
                });
        });
        if self.mode == RangeMode::Custom {
            ui.horizontal(|ui| {
                for (i, label) in ["Min", "Max"].into_iter().enumerate() {
                    ui.label(label);
                    ui.add(
                        egui::DragValue::new(&mut self.custom_range[i])
                            .speed(scale / 1000.0)
                            .max_decimals(6),
                    )
                    .on_hover_text(format!("Histogram {label} in source pixel values"));
                }
            });
        }
        let range = match self.mode {
            RangeMode::Unit => HistogramRange::Unit,
            RangeMode::Data => HistogramRange::Data,
            RangeMode::Custom => {
                let bounds = self.custom_range.map(|v| (v / scale) as f32);
                if !bounds.iter().all(|v| v.is_finite()) || bounds[0] >= bounds[1] {
                    ui.colored_label(NOTICE_WARNING_TEXT, "Min must be finite and smaller than Max.");
                    return Ok(None);
                }
                HistogramRange::Custom(bounds)
            }
        };
        let histogram = self.worker.request(image, range, ui.ctx())?;
        let action = if let Some(histogram) = &histogram {
            let mono_mask = [true];
            let visible_mask: &[bool] = if spec.channels == 1 { &mono_mask } else { mask };
            ui.push_id(image.id(), |ui| {
                draw_histogram(ui, Vec2::new(ui.available_width(), 130.0), histogram, visible_mask, spec.dtype)
            })
            .inner
        } else {
            ui.horizontal(|ui| {
                if self.worker.failed(image, range) {
                    ui.label("Histogram unavailable.");
                } else {
                    ui.spinner();
                    ui.label("Computing histogram…");
                }
            });
            None
        };
        ui.horizontal(|ui| {
            if spec.channels > 1 {
                channel_toggle_ui(ui, mask, spec.channels as usize);
            }
            if let Some(histogram) = &histogram {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.menu_button("Details", |ui| {
                        ui.strong("Excluded pixels");
                        ui.separator();
                        let mut shown = false;
                        for (i, counts) in histogram.excluded.iter().enumerate() {
                            if spec.channels > 1 && !mask.get(i).copied().unwrap_or(false) {
                                continue;
                            }
                            if shown {
                                ui.separator();
                            }
                            shown = true;
                            ui.strong(channel_label(i, histogram.bins.len()));
                            for (label, count) in
                                ["Below range", "Above range", "NaN / infinity"].into_iter().zip(counts)
                            {
                                ui.label(format!("{label}: {count} ({:.2}%)", histogram.percent(*count)));
                            }
                        }
                        if !shown {
                            ui.label("Select a channel to display.");
                        }
                        ui.separator();
                        ui.small(format!("% of all {} pixels per channel", histogram.pixels));
                    });
                });
            }
        });
        Ok(action)
    }
}

fn format_value(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    if value.abs() < 0.001 || value.abs() >= 1_000_000.0 {
        return format!("{value:.6e}");
    }
    format!("{value:.6}").trim_end_matches('0').trim_end_matches('.').to_owned()
}

fn bin_label(hist: &Histogram, bin: usize, dtype: PixelType) -> String {
    let [low, high] = hist.bin_bounds(bin).map(|v| v * dtype.alpha());
    if !dtype.is_floating() {
        if hist.range[0] == hist.range[1] {
            return format_value(low.round());
        }
        let low = low.ceil();
        let high = if bin + 1 == HISTOGRAM_BINS || hist.range[0] == hist.range[1] {
            high.floor()
        } else {
            high.ceil() - 1.0
        };
        if low > high {
            return "No integer values".to_owned();
        }
        if low == high {
            return format_value(low);
        }
        return format!("{}–{}", format_value(low), format_value(high));
    }
    format!(
        "[{}, {}{}",
        format_value(low),
        format_value(high),
        if bin + 1 == HISTOGRAM_BINS || low == high {
            "]"
        } else {
            ")"
        }
    )
}

fn histogram_csv(hist: &Histogram, mask: &[bool], dtype: PixelType) -> String {
    use std::fmt::Write;
    let channels: Vec<_> = (0..hist.bins.len())
        .filter(|&i| mask.get(i).copied().unwrap_or(false))
        .collect();
    let mut csv = "bin,value_min,value_max,max_inclusive".to_owned();
    for &i in &channels {
        let label = channel_label(i, hist.bins.len());
        let _ = write!(csv, ",{label}_pixels,{label}_percent");
    }
    csv.push('\n');
    for bin in 0..HISTOGRAM_BINS {
        let [low, high] = hist.bin_bounds(bin).map(|v| v * dtype.alpha());
        let _ = write!(csv, "{bin},{low},{high},{}", bin + 1 == HISTOGRAM_BINS || low == high);
        for &i in &channels {
            let count = hist.bins[i][bin];
            let _ = write!(csv, ",{count},{}", hist.percent(count));
        }
        csv.push('\n');
    }
    // Explicit rows preserve excluded samples and the denominator when exporting.
    for (category, offset) in [("below_range", 0), ("above_range", 1), ("non_finite", 2)] {
        let _ = write!(csv, "{category},,,");
        for &i in &channels {
            let count = hist.excluded[i][offset];
            let _ = write!(csv, ",{count},{}", hist.percent(count));
        }
        csv.push('\n');
    }
    csv
}

pub fn draw_histogram(
    ui: &mut egui::Ui,
    desired_size: Vec2,
    histogram: &Histogram,
    mask: &[bool],
    dtype: PixelType,
) -> Option<ExportAction> {
    #[cfg(debug_assertions)]
    let _timer = ScopedTimer::new("ui.histogram_plot.draw");

    let series = &histogram.bins;
    if series.is_empty() || mask.iter().all(|&m| !m) {
        ui.allocate_ui_with_layout(
            desired_size,
            egui::Layout::centered_and_justified(eframe::egui::Direction::LeftToRight),
            |ui| {
                ui.label(if mask.iter().all(|&m| !m) {
                    "Select a channel to display."
                } else {
                    "No data to display."
                });
            },
        );
        return None;
    }

    let max_freq = histogram.visible_max(mask).max(1) as f32;
    let (outer_rect, response) = ui.allocate_exact_size(desired_size, Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::Role::Image,
            ui.is_enabled(),
            format!(
                "Histogram, peak {} pixels, range {} to {}",
                histogram.visible_max(mask),
                format_value(histogram.range[0] as f64 * dtype.alpha()),
                format_value(histogram.range[1] as f64 * dtype.alpha())
            ),
        )
    });
    let painter = ui.painter_at(outer_rect);
    let axis_font = TextStyle::Small.resolve(ui.style());
    let rect = Rect::from_min_max(outer_rect.min + Vec2::new(0.0, 4.0), outer_rect.max - Vec2::new(0.0, 12.0));
    let mut export_action = None;

    if max_freq <= 0.0 {
        return None;
    }

    let bins = HISTOGRAM_BINS;
    if bins == 0 {
        return None;
    }

    let full_w = rect.width();
    let baseline_y = rect.bottom();
    let left_x = rect.left();

    let bin_width = full_w / bins as f32;
    let inner_w = bin_width.max(0.0);

    let bar_colors = histogram_bar_colors();
    let label_colors = HISTOGRAM_LABEL_COLORS;

    for fraction in [0.0, 0.5, 1.0] {
        let y = rect.bottom() - fraction * rect.height();
        painter.line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], HISTOGRAM_GRID_STROKE);
    }
    for (fraction, align) in [
        (0.0, egui::Align2::LEFT_TOP),
        (0.5, egui::Align2::CENTER_TOP),
        (1.0, egui::Align2::RIGHT_TOP),
    ] {
        let low = histogram.range[0] as f64;
        let value = (low + fraction * (histogram.range[1] as f64 - low)) * dtype.alpha();
        painter.text(
            pos2(rect.left() + fraction as f32 * rect.width(), rect.bottom() + 4.0),
            align,
            format_value(value),
            axis_font.clone(),
            ui.visuals().weak_text_color(),
        );
    }

    if histogram.visible_max(mask) == 0 {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No pixels in this range.",
            axis_font.clone(),
            ui.visuals().text_color(),
        );
    }

    let draw_series = |series: &[u32], color: Color32| {
        for i in 0..bins {
            let v = *series.get(i).unwrap_or(&0) as f32;
            if v <= 0.0 {
                continue;
            }
            let h_px = (v / max_freq).clamp(0.0, 1.0) * rect.height();
            if h_px <= 0.0 {
                continue;
            }

            let x0 = left_x + i as f32 * bin_width;
            let x1 = x0 + inner_w;
            let y_top = baseline_y - h_px;

            let x0c = x0.max(rect.left());
            let x1c = x1.min(rect.right());
            if x1c <= x0c {
                continue;
            }
            let y0c = y_top.max(rect.top());
            let y1c = baseline_y.min(rect.bottom());
            if y1c <= y0c {
                continue;
            }

            let r = Rect::from_min_max(pos2(x0c, y0c), pos2(x1c, y1c));
            painter.rect_filled(r, 0.0, color);
        }
    };

    series.iter().take(4).enumerate().for_each(|(i, s)| {
        if mask.get(i).copied().unwrap_or(false) {
            draw_series(s, bar_colors[i])
        }
    });

    let hover_state_id = response
        .id
        .with(("hovered_state", histogram.range.map(f32::to_bits), dtype as u32, mask));
    let context_menu_open = response.context_menu_opened();

    let hover_state = if context_menu_open {
        ui.ctx().data(|data| data.get_temp::<HistogramHoverState>(hover_state_id))
    } else if let Some(mouse_pos) = response.hover_pos() {
        if rect.contains(mouse_pos) {
            let x_norm = ((mouse_pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            let bin_idx = ((x_norm * bins as f32).floor() as usize).min(bins.saturating_sub(1));

            let mut lines: Vec<(String, Color32)> = Vec::new();
            lines.push((format!("Value: {}", bin_label(histogram, bin_idx, dtype)), Color32::WHITE));
            let total_channels = series.len();
            for (i, channel) in series.iter().take(4).enumerate() {
                if !mask.get(i).copied().unwrap_or(false) {
                    continue;
                }
                let value = channel.get(bin_idx).copied().unwrap_or(0);
                lines.push((
                    format!(
                        "{}: {} px ({:.2}%)",
                        channel_label(i, total_channels),
                        value,
                        histogram.percent(value)
                    ),
                    label_colors[i % label_colors.len()],
                ));
            }
            Some(HistogramHoverState {
                bin_idx,
                text: lines.iter().map(|(text, _)| text.as_str()).collect::<Vec<_>>().join("\n"),
                lines,
            })
        } else {
            None
        }
    } else {
        None
    };

    if let Some(hover_state) = hover_state.as_ref() {
        let bin_idx = hover_state.bin_idx;
        let lines = &hover_state.lines;
        let x = rect.left() + (bin_idx as f32 + 0.5) * bin_width;

        let dash_len = 6.0f32;
        let gap_len = 4.0f32;
        let mut y0 = rect.top();
        let stroke = Stroke::new(1.0, Color32::from_gray(180));
        while y0 < rect.bottom() {
            let y1 = (y0 + dash_len).min(rect.bottom());
            painter.line_segment([Pos2 { x, y: y0 }, Pos2 { x, y: y1 }], stroke);
            y0 += dash_len + gap_len;
        }

        if !context_menu_open {
            // Plot readouts follow the pointer immediately instead of waiting
            // for the normal tooltip delay or for the pointer to stop moving.
            egui::Tooltip::for_widget(&response).at_pointer().gap(12.0).show(|ui| {
                ui.set_max_width(300.0);
                for (text, color) in lines {
                    ui.label(egui::RichText::new(text).monospace().color(*color));
                }
                ui.small(format!("% of all {} pixels per channel", histogram.pixels));
            });
        }

        ui.ctx().data_mut(|data| data.insert_temp(hover_state_id, hover_state.clone()));
    }

    response.context_menu(|ui| {
        if ui.button("Copy Hovered Data").clicked() {
            let hover_state = hover_state
                .clone()
                .or_else(|| ui.ctx().data(|data| data.get_temp::<HistogramHoverState>(hover_state_id)));
            if let Some(hover_state) = hover_state {
                export_action = Some(ExportAction::Copy(CopyExport {
                    title: "hovered histogram data",
                    text: hover_state.text,
                }));
            }
            ui.close();
        }
        ui.separator();
        if ui.button("Copy Data (CSV)").clicked() {
            export_action = Some(ExportAction::Copy(CopyExport {
                title: "histogram data",
                text: histogram_csv(histogram, mask, dtype),
            }));
            ui.close();
        }
        if ui.button("Save Data as CSV...").clicked() {
            export_action = Some(ExportAction::Save(SaveExport {
                title: "histogram data",
                suggested_file_name: "histogram.csv",
                text: histogram_csv(histogram, mask, dtype),
            }));
            ui.close();
        }
    });

    export_action
}

#[cfg(test)]
mod tests {
    use super::*;
    fn histogram(range: [f32; 2]) -> Histogram {
        Histogram {
            bins: vec![vec![0; HISTOGRAM_BINS]],
            excluded: vec![[0; 3]],
            range,
            pixels: 4,
        }
    }
    #[test]
    fn histogram_value_intervals_use_source_bit_depth_and_include_last_endpoint() {
        let hist = histogram([0.0, 1.0]);
        assert_eq!(bin_label(&hist, 128, PixelType::U8), "128");
        assert_eq!(bin_label(&hist, 128, PixelType::U16), "32768–33023");
        assert_eq!(bin_label(&hist, 255, PixelType::U16), "65280–65535");
        assert_eq!(bin_label(&hist, 0, PixelType::F32), "[0, 0.003906)");
        assert_eq!(bin_label(&hist, 255, PixelType::F32), "[0.996094, 1]");
        assert_eq!(bin_label(&histogram([128.0 / 255.0; 2]), 0, PixelType::U8), "128");
        assert_eq!(bin_label(&histogram([-2.0, 2.0]), 0, PixelType::F32), "[-2, -1.984375)");
    }
    #[test]
    fn histogram_export_preserves_counts_percentages_and_excluded_samples() {
        let mut hist = histogram([-1.0, 1.0]);
        hist.bins[0][0] = 1;
        hist.excluded[0] = [1, 0, 2];
        let csv = histogram_csv(&hist, &[true], PixelType::F32);
        assert!(
            csv.starts_with("bin,value_min,value_max,max_inclusive,L_pixels,L_percent\n0,-1,-0.9921875,false,1,25\n")
        );
        assert!(csv.contains("below_range,,,,1,25\n"));
        assert!(csv.contains("non_finite,,,,2,50\n"));
    }
}
