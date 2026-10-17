use std::{cell::Cell, ops::RangeInclusive, rc::Rc};

use eframe::egui::{self, emath::Numeric};

use super::egui_ext::UiExt;

/// Native range slider with the app's handle shape, wheel editing, and primary-button-only dragging.
/// `step` sets both the native slider's precision and the wheel increment.
/// `configure` exposes egui's builder without duplicating its API here.
pub fn range_slider_ui<'a, Num: Numeric>(
    ui: &mut egui::Ui,
    low: &mut Num,
    high: &mut Num,
    range: impl Into<RangeInclusive<Num>>,
    step: f64,
    configure: impl FnOnce(egui::RangeSlider<'a>) -> egui::RangeSlider<'a>,
) -> egui::Response {
    // egui 0.37's RangeSlider writes values for any pointer button.
    // Include the release frame so opening a context menu cannot move a handle.
    // With no pointer gesture, keyboard and accessibility edits still work.
    let allow_update = ui.input(|i| {
        (!i.pointer.any_down() && !i.pointer.any_released()) || i.pointer.primary_down() || i.pointer.primary_released()
    });
    let range = range.into();
    // Stage native edits so the callbacks do not hold the caller's mutable borrows
    // while the wheel handler updates the same values after the widget is drawn.
    let values = Rc::new([Cell::new(*low), Cell::new(*high)]);
    let low_value = Rc::clone(&values);
    let high_value = Rc::clone(&values);
    let slider = egui::RangeSlider::from_get_set(
        range.start().to_f64()..=range.end().to_f64(),
        move |value| {
            if let Some(value) = value.filter(|_| allow_update) {
                low_value[0].set(Num::from_f64(value));
            }
            low_value[0].get().to_f64()
        },
        move |value| {
            if let Some(value) = value.filter(|_| allow_update) {
                high_value[1].set(Num::from_f64(value));
            }
            high_value[1].get().to_f64()
        },
    )
    .handle_shape(egui::style::HandleShape::Rect { aspect_ratio: 0.5 });
    let slider = if Num::INTEGRAL { slider.integer() } else { slider };
    let mut response = ui.add(configure(slider.step_by(step)));
    *low = values[0].get();
    *high = values[1].get();
    range_slider_scroll(ui, &mut response, low, high, range, step);
    response
}

/// Scroll the nearest handle of a native range slider by `step` per wheel unit.
/// Pass the same range and step as the slider. Values stay on that step grid.
fn range_slider_scroll<Num: Numeric>(
    ui: &egui::Ui,
    response: &mut egui::Response,
    low: &mut Num,
    high: &mut Num,
    range: RangeInclusive<Num>,
    step: f64,
) {
    if !response.hovered() || !response.enabled() || !step.is_finite() || step <= 0.0 {
        return;
    }
    // Keep wheel edits separate from pointer gestures, including context-menu clicks.
    if ui.input(|i| i.pointer.any_down() || i.pointer.any_released()) {
        return;
    }
    let delta = ui.raw_scroll_delta_y() as f64;
    if delta == 0.0 || !delta.is_finite() {
        return;
    }
    // egui 0.37 exposes each native handle's response under these child ids.
    // Use their actual positions so logarithmic and vertical sliders work too.
    let (Some(pointer), Some(low_handle), Some(high_handle)) = (
        response.hover_pos(),
        ui.ctx().read_response(response.id.with("low")),
        ui.ctx().read_response(response.id.with("high")),
    ) else {
        return;
    };
    let low_center = low_handle.rect.center();
    let high_center = high_handle.rect.center();
    let start = range.start().to_f64();
    let end = range.end().to_f64();
    let vertical = response.rect.height() > response.rect.width();
    let pointer_before = if vertical {
        pointer.y > low_center.y
    } else {
        pointer.x < low_center.x
    };
    let choose_low = low_center.distance_sq(pointer) < high_center.distance_sq(pointer)
        || (low_center == high_center && pointer_before == (start <= end));
    let min = start.min(end);
    let max = start.max(end);
    let bounds = if choose_low {
        min..=high.to_f64().min(max)
    } else {
        low.to_f64().max(min)..=max
    };
    let value = if choose_low { low } else { high };
    let before = value.to_f64();
    let move_by = if ui.input(|i| i.modifiers.shift_only()) {
        step / 10.0
    } else {
        step
    };
    let next = before + delta * move_by;
    let next = start + ((next - start) / step).round() * step;
    *value = Num::from_f64(next.clamp(*bounds.start(), *bounds.end()));
    if value.to_f64() != before {
        response.mark_changed();
        ui.ctx().request_repaint();
    }
}

/// Opens a context menu for the range slider's drag-only response.
pub fn range_slider_context_menu(response: &egui::Response, contents: impl FnOnce(&mut egui::Ui)) {
    // Adding a click interaction over the drag response would steal its handles.
    let open_menu = (response.hovered() && response.ctx.input(|i| i.pointer.secondary_clicked()))
        .then_some(egui::SetOpenCommand::Bool(true));
    egui::Popup::context_menu(response).open_memory(open_menu).show(contents);
}

/// Show help on hover and separate handle values while dragging or scrolling.
pub fn range_slider_tooltip(ui: &egui::Ui, response: &egui::Response, description: &str, value_text: [String; 2]) {
    let dragging = response.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_down());
    let scrolling = response.hovered() && ui.raw_scroll_delta_y() != 0.0;
    let now = ui.input(|i| i.time);
    let expiry_id = response.id.with("scroll_value_until");
    let expiry = ui.data_mut(|data| {
        if scrolling {
            // Keep wheel feedback readable between individual wheel events.
            data.insert_temp(expiry_id, now + 0.6);
        }
        data.get_temp::<f64>(expiry_id).unwrap_or(f64::NEG_INFINITY)
    });
    if dragging || (response.hovered() && now < expiry) {
        range_slider_value_labels(ui, response, value_text);
        if now < expiry {
            ui.ctx().request_repaint_after(std::time::Duration::from_secs_f64(expiry - now));
        }
    } else if !ui.input(|i| i.pointer.any_down() || i.pointer.any_released()) {
        response.clone().on_hover_text(description);
    }
}

fn range_slider_value_labels(ui: &egui::Ui, response: &egui::Response, values: [String; 2]) {
    let (Some(low), Some(high)) = (
        ui.ctx().read_response(response.id.with("low")),
        ui.ctx().read_response(response.id.with("high")),
    ) else {
        return;
    };
    // Paint above the UI without allocating space or intercepting handle input.
    let painter = ui
        .ctx()
        .layer_painter(egui::LayerId::new(egui::Order::Tooltip, response.id.with("handle_values")));
    let frame = egui::Frame::popup(ui.style());
    let margin = frame.total_margin();
    let bounds = ui.ctx().content_rect().shrink(8.0);
    let font = egui::TextStyle::Body.resolve(ui.style());
    let text = values.map(|value| painter.layout_no_wrap(value, font.clone(), ui.visuals().text_color()));
    let handles = [low.rect, high.rect];
    let mut labels = std::array::from_fn::<_, 2, _>(|index| {
        let size = text[index].size() + margin.sum();
        let handle = handles[index];
        let left = (handle.center().x - size.x / 2.0).clamp(bounds.left(), bounds.right() - size.x);
        let top = if handle.bottom() + 6.0 + size.y <= bounds.bottom() {
            handle.bottom() + 6.0
        } else {
            handle.top() - 6.0 - size.y
        };
        egui::Rect::from_min_size(egui::pos2(left, top), size)
    });
    let gap = 8.0;
    if labels[0].expand(gap / 2.0).intersects(labels[1].expand(gap / 2.0)) {
        // Move both bubbles as a group so equal/nearby values remain readable,
        // including at the screen edge. Keep low on the left and high on the right.
        let width = labels[0].width() + gap + labels[1].width();
        let center = (handles[0].center().x + handles[1].center().x) / 2.0;
        let left = (center - width / 2.0).clamp(bounds.left(), bounds.right() - width);
        labels[0] = egui::Rect::from_min_size(egui::pos2(left, labels[0].top()), labels[0].size());
        labels[1] = egui::Rect::from_min_size(egui::pos2(labels[0].right() + gap, labels[1].top()), labels[1].size());
    }
    for (index, label) in labels.into_iter().enumerate() {
        let handle = handles[index];
        let below = label.top() >= handle.bottom();
        painter.line_segment(
            [
                if below {
                    handle.center_bottom()
                } else {
                    handle.center_top()
                },
                if below {
                    label.center_top()
                } else {
                    label.center_bottom()
                },
            ],
            ui.visuals().window_stroke(),
        );
        let content = label - margin;
        painter.add(frame.paint(content));
        painter.galley(content.min, text[index].clone(), ui.visuals().text_color());
    }
}
