use std::sync::Arc;

use eframe::egui::epaint::Mesh;
use eframe::egui::{self, pos2, Color32, Pos2, Rect, Sense, Stroke, TextStyle, Vec2};

use super::{channel_toggle_ui, CopyExport, ExportAction, SaveExport};
use crate::model::{Histogram, HistogramRange, HistogramWorker, Image, ImageData, PixelType, HISTOGRAM_BINS};
use crate::res::{
    histogram_bar_colors, HISTOGRAM_GRID_STROKE, HISTOGRAM_LABEL_COLORS, HISTOGRAM_RGB_ADDITIVE_COLORS,
    NOTICE_WARNING_TEXT,
};
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
    mesh_cache: HistogramMeshCache,
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
        let standard_range_label = format!("Standard range (0-{scale:.0})");
        if self.image_id != Some(image.id()) {
            self.image_id = Some(image.id());
            self.custom_range = [0.0, scale];
        }
        ui.horizontal(|ui| {
            ui.label("Range");
            egui::ComboBox::from_id_salt("histogram_range")
                .selected_text(match self.mode {
                    RangeMode::Unit => standard_range_label.as_str(),
                    RangeMode::Data => "Full data range",
                    RangeMode::Custom => "Custom",
                })
                .width(ui.available_width())
                .truncate()
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.mode, RangeMode::Unit, &standard_range_label);
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
                draw_histogram(
                    ui,
                    Vec2::new(ui.available_width(), 130.0),
                    histogram,
                    visible_mask,
                    spec.dtype,
                    &mut self.mesh_cache,
                )
            })
            .inner
        } else {
            self.mesh_cache = HistogramMeshCache::default();
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

fn histogram_bar_mesh(series: &[u32], rect: Rect, max_freq: f32, color: Color32) -> Mesh {
    let mut mesh = Mesh::default();
    if !rect.is_positive() || max_freq <= 0.0 {
        return mesh;
    }
    mesh.reserve_vertices(series.len().min(HISTOGRAM_BINS) * 4);
    mesh.reserve_triangles(series.len().min(HISTOGRAM_BINS) * 2);
    let edge_x = |bin: usize| rect.left() + rect.width() * (bin as f32 / HISTOGRAM_BINS as f32);
    for (bin, &count) in series.iter().take(HISTOGRAM_BINS).enumerate() {
        if count == 0 {
            continue;
        }
        let height = (count as f32 / max_freq).clamp(0.0, 1.0) * rect.height();
        let bar = Rect::from_min_max(pos2(edge_x(bin), rect.bottom() - height), pos2(edge_x(bin + 1), rect.bottom()));
        // Matching bin edges tile without the per-rectangle feathering that
        // produces vertical seams on sub-point-wide bars. Keep fractional
        // coordinates so narrow bins retain their exact widths and positions.
        mesh.add_colored_rect(bar, color);
    }
    mesh
}

#[derive(PartialEq)]
struct HistogramMeshKey {
    // Vertices use absolute coordinates, so scrolling must also invalidate them.
    rect: Rect,
    max_freq: f32,
    mask: [bool; 4],
    colors: [Color32; 4],
}

struct HistogramMeshes {
    // Retain the immutable result to make pointer identity safe across image switches.
    histogram: Arc<Histogram>,
    key: HistogramMeshKey,
    channels: [Option<Arc<Mesh>>; 4],
}

#[derive(Default)]
struct HistogramMeshCache {
    // Only the current plot is retained, independently of the histogram data caches.
    current: Option<HistogramMeshes>,
}

impl HistogramMeshCache {
    fn get(
        &mut self,
        histogram: &Arc<Histogram>,
        rect: Rect,
        max_freq: f32,
        mask: &[bool],
        colors: [Color32; 4],
    ) -> &[Option<Arc<Mesh>>; 4] {
        let key = HistogramMeshKey {
            rect,
            max_freq,
            mask: std::array::from_fn(|i| mask.get(i).copied().unwrap_or(false)),
            colors,
        };
        let reuse = self
            .current
            .as_ref()
            .is_some_and(|cached| Arc::ptr_eq(&cached.histogram, histogram) && cached.key == key);
        if !reuse {
            let channels = std::array::from_fn(|i| {
                histogram
                    .bins
                    .get(i)
                    .filter(|_| key.mask[i])
                    .map(|values| Arc::new(histogram_bar_mesh(values, rect, max_freq, colors[i])))
            });
            self.current = Some(HistogramMeshes {
                histogram: Arc::clone(histogram),
                key,
                channels,
            });
        }
        &self.current.as_ref().unwrap().channels
    }
}

fn draw_histogram(
    ui: &mut egui::Ui,
    desired_size: Vec2,
    histogram: &Arc<Histogram>,
    mask: &[bool],
    dtype: PixelType,
    mesh_cache: &mut HistogramMeshCache,
) -> Option<ExportAction> {
    #[cfg(debug_assertions)]
    let _timer = ScopedTimer::new("ui.histogram_plot.draw");

    let series = &histogram.bins;
    if series.is_empty() || mask.iter().all(|&m| !m) {
        *mesh_cache = HistogramMeshCache::default();
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

    let bin_width = rect.width() / bins as f32;

    let mut bar_colors = histogram_bar_colors();
    let rgb = series.len() >= 3;
    if rgb {
        bar_colors[..3].copy_from_slice(&HISTOGRAM_RGB_ADDITIVE_COLORS);
    }
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

    // Alpha remains a conventional overlay underneath the RGB light channels,
    // so it cannot tint the white produced by all three RGB channels.
    let draw_order = if rgb { [3, 0, 1, 2] } else { [0, 1, 2, 3] };
    let meshes = mesh_cache.get(histogram, rect, max_freq, mask, bar_colors);
    for i in draw_order {
        if let Some(mesh) = &meshes[i] {
            painter.add(egui::Shape::Mesh(Arc::clone(mesh)));
        }
    }

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

    #[test]
    fn histogram_mesh_cache_reuses_buffers_and_preserves_rendered_geometry() {
        let mut hist = histogram([0.0, 1.0]);
        hist.bins = vec![vec![64; HISTOGRAM_BINS]; 4];
        let hist = Arc::new(hist);
        let rect = Rect::from_min_size(pos2(10.25, 20.5), Vec2::new(222.5, 114.0));
        let colors = histogram_bar_colors();
        let mask = [true, true, false, true];
        let mut cache = HistogramMeshCache::default();
        let first = cache.get(&hist, rect, 64.0, &mask, colors).clone();
        assert!(first[2].is_none());
        for _ in 0..10 {
            let reused = cache.get(&Arc::clone(&hist), rect, 64.0, &mask, colors);
            for i in [0, 1, 3] {
                let mesh = reused[i].as_ref().unwrap();
                assert!(Arc::ptr_eq(first[i].as_ref().unwrap(), mesh));
                let expected = histogram_bar_mesh(&hist.bins[i], rect, 64.0, colors[i]);
                assert_eq!(mesh.vertices, expected.vertices);
                assert_eq!(mesh.indices, expected.indices);
            }
        }
    }

    #[test]
    fn histogram_mesh_cache_invalidates_for_position_size_scale_mask_and_color() {
        let mut hist = histogram([0.0, 1.0]);
        hist.bins = vec![vec![32; HISTOGRAM_BINS]; 4];
        let hist = Arc::new(hist);
        let rect = Rect::from_min_size(pos2(10.0, 20.0), Vec2::new(256.0, 114.0));
        let colors = histogram_bar_colors();
        let mut changed_colors = colors;
        changed_colors[0] = HISTOGRAM_RGB_ADDITIVE_COLORS[0];
        for (new_rect, max_freq, mask, colors) in [
            (rect.translate(Vec2::new(1.0, 0.0)), 32.0, [true; 4], colors),
            (rect.translate(Vec2::new(0.0, -1.0)), 32.0, [true; 4], colors),
            (Rect::from_min_size(rect.min, Vec2::new(257.0, 114.0)), 32.0, [true; 4], colors),
            (Rect::from_min_size(rect.min, Vec2::new(256.0, 115.0)), 32.0, [true; 4], colors),
            (rect, 64.0, [true; 4], colors),
            (rect, 32.0, [true, true, true, false], colors),
            (rect, 32.0, [true; 4], changed_colors),
        ] {
            let mut cache = HistogramMeshCache::default();
            let original = Arc::clone(
                cache.get(&hist, rect, 32.0, &[true; 4], histogram_bar_colors())[0]
                    .as_ref()
                    .unwrap(),
            );
            let meshes = cache.get(&hist, new_rect, max_freq, &mask, colors);
            let updated = meshes[0].as_ref().unwrap();
            assert!(!Arc::ptr_eq(&original, updated));
            let expected = histogram_bar_mesh(&hist.bins[0], new_rect, max_freq, colors[0]);
            assert_eq!(updated.vertices, expected.vertices);
            assert_eq!(updated.indices, expected.indices);
            assert_eq!(meshes[3].is_some(), mask[3]);
        }
    }

    #[test]
    fn histogram_mesh_cache_replaces_data_and_releases_previous_entry() {
        let mut hist = histogram([0.0, 1.0]);
        hist.bins[0][0] = 4;
        let hist = Arc::new(hist);
        let old_hist = Arc::downgrade(&hist);
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(256.0, 114.0));
        let mut cache = HistogramMeshCache::default();
        let meshes = cache.get(&hist, rect, 4.0, &[true], histogram_bar_colors());
        let old_mesh = Arc::downgrade(meshes[0].as_ref().unwrap());
        let old_vertices = meshes[0].as_ref().unwrap().vertices.clone();
        drop(hist);
        assert!(old_hist.upgrade().is_some());

        // Same range, dimensions and peak, but a different distribution.
        let mut next = histogram([0.0, 1.0]);
        next.bins[0][255] = 4;
        let next = Arc::new(next);
        let meshes = cache.get(&next, rect, 4.0, &[true], histogram_bar_colors());
        assert_ne!(meshes[0].as_ref().unwrap().vertices, old_vertices);
        assert!(old_hist.upgrade().is_none());
        assert!(old_mesh.upgrade().is_none());
        let current_mesh = Arc::downgrade(meshes[0].as_ref().unwrap());
        drop(cache);
        assert!(current_mesh.upgrade().is_none());
        assert_eq!(Arc::strong_count(&next), 1);
    }

    #[test]
    fn histogram_rgb_additive_combinations_are_independent_of_draw_order() {
        for order in [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]] {
            for mask in 0..8 {
                let mut blended = Color32::BLACK;
                for channel in order {
                    if mask & (1 << channel) != 0 {
                        blended = blended.blend(HISTOGRAM_RGB_ADDITIVE_COLORS[channel]);
                    }
                }
                assert_eq!(
                    blended,
                    Color32::from_rgb(
                        if mask & 1 != 0 { 255 } else { 0 },
                        if mask & 2 != 0 { 255 } else { 0 },
                        if mask & 4 != 0 { 255 } else { 0 },
                    )
                );
            }
        }
        let mesh = histogram_bar_mesh(
            &[1; HISTOGRAM_BINS],
            Rect::from_min_size(Pos2::ZERO, Vec2::new(256.0, 100.0)),
            1.0,
            HISTOGRAM_RGB_ADDITIVE_COLORS[0],
        );
        assert!(mesh
            .vertices
            .iter()
            .all(|vertex| vertex.color == HISTOGRAM_RGB_ADDITIVE_COLORS[0]));
    }

    #[test]
    fn histogram_bar_mesh_tiles_fractional_bin_edges_without_transparent_seams() {
        let rect = Rect::from_min_size(pos2(1049.34375, 382.0), Vec2::new(222.65625, 114.0));
        let color = histogram_bar_colors()[0];
        let mesh = histogram_bar_mesh(&[64; HISTOGRAM_BINS], rect, 64.0, color);
        assert!(mesh.is_valid());
        assert_eq!(mesh.vertices.len(), HISTOGRAM_BINS * 4);
        assert_eq!(mesh.indices.len(), HISTOGRAM_BINS * 6);
        assert_eq!(mesh.vertices[0].pos, rect.left_top());
        assert_eq!(mesh.vertices.last().unwrap().pos, rect.right_bottom());
        for (i, bar) in mesh.vertices.chunks_exact(4).enumerate() {
            assert!(bar.iter().all(|vertex| vertex.color == color));
            assert_eq!(bar[0].pos.y, rect.top());
            assert_eq!(bar[2].pos.y, rect.bottom());
            if i + 1 < HISTOGRAM_BINS {
                assert_eq!(bar[1].pos.x.to_bits(), mesh.vertices[(i + 1) * 4].pos.x.to_bits());
            }
        }
    }

    #[test]
    fn histogram_bar_mesh_preserves_isolated_spikes_and_zero_neighbors() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), Vec2::new(256.0, 100.0));
        let mut bins = [0; HISTOGRAM_BINS];
        bins[128] = 100;
        bins[130] = 25;
        let mesh = histogram_bar_mesh(&bins, rect, 100.0, histogram_bar_colors()[0]);
        assert_eq!(mesh.vertices.len(), 8);
        assert_eq!(mesh.vertices[0].pos, pos2(128.0, 0.0));
        assert_eq!(mesh.vertices[3].pos, pos2(129.0, 100.0));
        assert_eq!(mesh.vertices[4].pos, pos2(130.0, 75.0));
        assert_eq!(mesh.vertices[7].pos, pos2(131.0, 100.0));
        assert!(histogram_bar_mesh(&[0; HISTOGRAM_BINS], rect, 1.0, Color32::RED)
            .vertices
            .is_empty());
    }

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
