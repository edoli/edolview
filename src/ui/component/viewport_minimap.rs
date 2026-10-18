use eframe::egui::{self, pos2, vec2, Rect, Vec2};

use crate::{
    res::{
        MINIMAP_DRAG_STROKE, MINIMAP_HOVER_STROKE, MINIMAP_OUTSIDE_VIEW_FILL, MINIMAP_VIEWPORT_OUTLINE,
        MINIMAP_VIEWPORT_STROKE,
    },
    ui::gpu::{ImagePaintCallback, ImageSlot, PaneDraw},
};

pub struct ViewportMinimap {
    pub image_rect: Rect,
    pub image_size: Vec2,
    pub scale: f32,
    visible_image: Rect,
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
            visible_image,
            visible_rect,
        })
    }

    pub fn interact(&self, ui: &mut egui::Ui, id: egui::Id) -> egui::Response {
        ui.interact(self.image_rect, id, egui::Sense::click_and_drag())
            .on_hover_cursor(egui::CursorIcon::Grab)
            .on_hover_text("Click or drag to move the view")
    }

    pub fn grab_offset(&self, pointer: egui::Pos2) -> Vec2 {
        if self.visible_rect.is_some_and(|visible| visible.contains(pointer)) {
            (pointer - self.image_rect.min) / self.scale - self.visible_image.center().to_vec2()
        } else {
            Vec2::ZERO
        }
    }

    pub fn navigation_center(&self, pointer: egui::Pos2, grab_offset: Vec2) -> egui::Pos2 {
        ((pointer - self.image_rect.min) / self.scale - grab_offset).to_pos2()
    }

    pub fn paint(&self, ui: &egui::Ui, slot: ImageSlot, uniform_slot: u32, hovered: bool, dragging: bool) {
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
        if hovered || dragging {
            // Outline the whole input target immediately, independently of the
            // yellow marker showing which part of the image is currently visible.
            let painter = ui.painter().with_clip_rect(self.image_rect.expand(5.0));
            let frame = self.image_rect.expand(1.5);
            let (width, color) = if dragging {
                (3.0, MINIMAP_DRAG_STROKE)
            } else {
                (2.0, MINIMAP_HOVER_STROKE)
            };
            painter.rect_stroke(
                frame,
                2.0,
                egui::Stroke::new(width + 2.0, MINIMAP_VIEWPORT_OUTLINE),
                egui::StrokeKind::Middle,
            );
            painter.rect_stroke(frame, 2.0, egui::Stroke::new(width, color), egui::StrokeKind::Middle);
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

    #[test]
    fn navigation_preserves_the_grab_point_and_allows_moving_beyond_image_edges() {
        let pane = Rect::from_min_size(pos2(40.0, 80.0), vec2(1000.0, 500.0));
        let visible = Rect::from_min_size(pos2(500.0, 250.0), vec2(1000.0, 500.0));
        let map = ViewportMinimap::new(pane, vec2(4000.0, 2000.0), visible).unwrap();
        let grab = map.image_rect.min + vec2(700.0, 400.0) * map.scale;
        let offset = map.grab_offset(grab);
        assert!((map.navigation_center(grab, offset) - visible.center()).length() < 0.002);
        let moved = grab + vec2(200.0, 100.0) * map.scale;
        assert!((map.navigation_center(moved, offset) - pos2(1200.0, 600.0)).length() < 0.002);

        let jump = map.image_rect.min + vec2(3000.0, 1500.0) * map.scale;
        assert_eq!(map.grab_offset(jump), Vec2::ZERO);
        assert!((map.navigation_center(jump, Vec2::ZERO) - pos2(3000.0, 1500.0)).length() < 0.002);
        let outside_min = map.image_rect.min - vec2(100.0, 100.0) * map.scale;
        let outside_max = map.image_rect.max + vec2(100.0, 100.0) * map.scale;
        assert!((map.navigation_center(outside_min, Vec2::ZERO) - pos2(-100.0, -100.0)).length() < 0.002);
        assert!((map.navigation_center(outside_max, Vec2::ZERO) - pos2(4100.0, 2100.0)).length() < 0.002);
    }

    #[test]
    fn navigation_does_not_jump_for_partially_offscreen_or_oversized_views() {
        let pane = Rect::from_min_size(egui::Pos2::ZERO, vec2(1000.0, 800.0));
        for (size, visible) in [
            (
                vec2(4000.0, 2000.0),
                Rect::from_min_size(pos2(-300.0, -200.0), vec2(1000.0, 500.0)),
            ),
            (
                vec2(4000.0, 2000.0),
                Rect::from_min_size(pos2(3600.0, 1900.0), vec2(1000.0, 500.0)),
            ),
            (
                vec2(200.0, 400.0),
                Rect::from_min_size(pos2(-100.0, -100.0), vec2(1000.0, 800.0)),
            ),
        ] {
            let map = ViewportMinimap::new(pane, size, visible).unwrap();
            let pointer = map.visible_rect.unwrap().center();
            let offset = map.grab_offset(pointer);
            assert!((map.navigation_center(pointer, offset) - visible.center()).length() < 0.002);
            let delta = vec2(32.0, -24.0);
            assert!(
                (map.navigation_center(pointer + delta * map.scale, offset) - (visible.center() + delta)).length()
                    < 0.002
            );
        }
    }

    #[test]
    fn minimap_owns_the_drag_even_when_the_pointer_leaves_its_bounds() {
        let ctx = egui::Context::default();
        let mut map_rect = Rect::NOTHING;
        let mut run_frame = |events: Vec<egui::Event>| {
            let mut state = (false, false, false);
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(800.0, 600.0))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    let (pane, image_response) =
                        ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
                    let map = ViewportMinimap::new(pane, vec2(4000.0, 2000.0), pane).unwrap();
                    map_rect = map.image_rect;
                    let response = map.interact(ui, ui.scope_id().with("minimap"));
                    state = (
                        response.is_pointer_button_down_on(),
                        response.dragged(),
                        image_response.drag_started(),
                    );
                },
            );
            output.textures_delta.clear();
            (map_rect, state)
        };
        let (rect, _) = run_frame(vec![]);
        let point = rect.center();
        run_frame(vec![egui::Event::PointerMoved(point)]);
        let (_, (down, _, image_drag)) = run_frame(vec![egui::Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }]);
        assert!(down);
        assert!(!image_drag);
        let outside = rect.min - vec2(100.0, 100.0);
        let (_, (_, dragged, image_drag)) = run_frame(vec![egui::Event::PointerMoved(outside)]);
        assert!(dragged);
        assert!(!image_drag);
    }
}
