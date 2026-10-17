use eframe::egui::{self, pos2, vec2, Rect, Vec2};

use crate::{
    res::{MINIMAP_OUTSIDE_VIEW_FILL, MINIMAP_VIEWPORT_OUTLINE, MINIMAP_VIEWPORT_STROKE},
    ui::gpu::{ImagePaintCallback, ImageSlot, PaneDraw},
};

pub struct ViewportMinimap {
    pub image_rect: Rect,
    pub image_size: Vec2,
    pub scale: f32,
    visible_rect: Option<Rect>,
}

impl ViewportMinimap {
    pub fn new(viewport: Rect, image_size: Vec2, visible_image: Rect) -> Option<Self> {
        let max_size = (viewport.size() * 0.28).min(vec2(180.0, 140.0));
        if max_size.x.min(max_size.y) < 36.0 || !image_size.is_finite() || image_size.x <= 0.0 || image_size.y <= 0.0 {
            return None;
        }
        let scale = (max_size.x / image_size.x).min(max_size.y / image_size.y);
        let size = image_size * scale;
        let image_rect = Rect::from_min_size(pos2(viewport.right() - 12.0 - size.x, viewport.top() + 12.0), size);
        let visible = visible_image.intersect(Rect::from_min_size(egui::Pos2::ZERO, image_size));
        let visible_rect = (visible.width() > 0.0 && visible.height() > 0.0).then(|| {
            Rect::from_min_max(
                image_rect.min + visible.min.to_vec2() * scale,
                image_rect.min + visible.max.to_vec2() * scale,
            )
        });
        Some(Self {
            image_rect,
            image_size,
            scale,
            visible_rect,
        })
    }

    pub fn paint(&self, ui: &egui::Ui, slot: ImageSlot, uniform_slot: u32) {
        if !ui.is_rect_visible(self.image_rect) {
            return;
        }
        let painter = ui.painter().with_clip_rect(self.image_rect);
        // Reuse the viewer's GPU texture, colormap, and display pipeline. Only
        // the small viewport and transform differ; no image readback is needed.
        painter.add(eframe::egui_wgpu::Callback::new_paint_callback(
            self.image_rect,
            ImagePaintCallback {
                panes: vec![PaneDraw {
                    viewport_px: self.image_rect * ui.ctx().pixels_per_point(),
                    slot,
                    uniform_slot,
                }],
                export: None,
            },
        ));
        if let Some(visible) = self.visible_rect {
            let image = self.image_rect;
            for outside in [
                Rect::from_min_max(image.min, pos2(image.right(), visible.top())),
                Rect::from_min_max(pos2(image.left(), visible.bottom()), image.max),
                Rect::from_min_max(pos2(image.left(), visible.top()), visible.left_bottom()),
                Rect::from_min_max(visible.right_top(), pos2(image.right(), visible.bottom())),
            ] {
                if outside.width() > 0.0 && outside.height() > 0.0 {
                    painter.rect_filled(outside, 0.0, MINIMAP_OUTSIDE_VIEW_FILL);
                }
            }
        }
        painter.rect_stroke(self.image_rect, 0.0, ui.visuals().window_stroke, egui::StrokeKind::Inside);
        if let Some(visible) = self.visible_rect {
            // Keep even subpixel regions visible at very high zoom levels.
            let marker =
                Rect::from_center_size(visible.center(), visible.size().max(vec2(3.0, 3.0))).intersect(self.image_rect);
            // Draw above the minimap border and allow the centered stroke to
            // extend beyond the thumbnail when the visible region reaches an edge.
            let painter = ui.painter().with_clip_rect(self.image_rect.expand(3.0));
            painter.rect_stroke(
                marker,
                0.0,
                egui::Stroke::new(3.5, MINIMAP_VIEWPORT_OUTLINE),
                egui::StrokeKind::Middle,
            );
            painter.rect_stroke(
                marker,
                0.0,
                egui::Stroke::new(1.5, MINIMAP_VIEWPORT_STROKE),
                egui::StrokeKind::Middle,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_visible_image_region_without_changing_aspect_ratio() {
        let pane = Rect::from_min_size(pos2(40.0, 80.0), vec2(1000.0, 500.0));
        let visible = Rect::from_min_size(pos2(500.0, 250.0), vec2(1000.0, 500.0));
        let map = ViewportMinimap::new(pane, vec2(4000.0, 2000.0), visible).unwrap();
        assert_eq!(map.image_rect.size(), vec2(180.0, 90.0));
        let roi = map.visible_rect.unwrap();
        assert_eq!(roi.min - map.image_rect.min, vec2(22.5, 11.25));
        assert_eq!(roi.size(), vec2(45.0, 22.5));
        assert!(pane.contains_rect(map.image_rect));
    }

    #[test]
    fn clips_viewport_to_image_and_handles_an_offscreen_image() {
        let pane = Rect::from_min_size(egui::Pos2::ZERO, vec2(1000.0, 800.0));
        let size = vec2(200.0, 400.0);
        let full = Rect::from_min_max(pos2(-50.0, -50.0), pos2(400.0, 800.0));
        let map = ViewportMinimap::new(pane, size, full).unwrap();
        assert_eq!(map.visible_rect, Some(map.image_rect));
        assert_eq!(map.image_rect.size(), vec2(70.0, 140.0));

        let partial = Rect::from_min_max(pos2(-50.0, 200.0), pos2(100.0, 800.0));
        let map = ViewportMinimap::new(pane, size, partial).unwrap();
        let visible = map.visible_rect.unwrap();
        assert_eq!(visible.min, map.image_rect.min + vec2(0.0, 70.0));
        assert_eq!(visible.size(), vec2(35.0, 70.0));

        let offscreen = Rect::from_min_size(pos2(500.0, 500.0), size);
        assert!(ViewportMinimap::new(pane, size, offscreen).unwrap().visible_rect.is_none());
    }

    #[test]
    fn skips_tiny_viewports_and_invalid_image_sizes() {
        let pane = Rect::from_min_size(egui::Pos2::ZERO, vec2(100.0, 100.0));
        assert!(ViewportMinimap::new(pane, vec2(10.0, 10.0), pane).is_none());
        let pane = Rect::from_min_size(egui::Pos2::ZERO, vec2(800.0, 600.0));
        assert!(ViewportMinimap::new(pane, Vec2::ZERO, pane).is_none());
        assert!(ViewportMinimap::new(pane, vec2(f32::NAN, 100.0), pane).is_none());
    }
}
