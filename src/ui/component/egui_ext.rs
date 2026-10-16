use eframe::egui::{self, Color32, ComboBox, Image, InnerResponse, Label, Rangef, Response, Ui, Widget, WidgetText};

use crate::model::PixelType;
use crate::res::TEXT_EDIT_PARSE_FAILED_FLASH;
use crate::util::color::ColorDisplay;

type ColumnsContents<'c, R, const N: usize> = Box<dyn FnOnce(&mut [Ui; N]) -> R + 'c>;

#[derive(Clone, Debug, Copy)]
pub enum Size {
    /// Absolute size in points, with a given range of allowed sizes to resize within.
    Absolute { initial: f32, range: Rangef },

    /// Relative size relative to all available space.
    Relative { fraction: f32, range: Rangef },

    /// Multiple remainders each get the same space.
    Remainder { weight: f32, range: Rangef },
}

impl Size {
    pub fn exact(initial: f32) -> Self {
        Self::Absolute {
            initial,
            range: Rangef::new(initial, initial),
        }
    }

    pub fn initial(initial: f32) -> Self {
        Self::Absolute {
            initial,
            range: Rangef::new(0.0, f32::INFINITY),
        }
    }

    pub fn relative(fraction: f32) -> Self {
        Self::Relative {
            fraction,
            range: Rangef::new(0.0, f32::INFINITY),
        }
    }

    pub fn remainder(weight: f32) -> Self {
        Self::Remainder {
            weight,
            range: Rangef::new(0.0, f32::INFINITY),
        }
    }

    #[inline]
    pub fn at_least(mut self, minimum: f32) -> Self {
        self.range_mut().min = minimum;
        self
    }

    /// Won't grow above this size (in points).
    #[inline]
    pub fn at_most(mut self, maximum: f32) -> Self {
        self.range_mut().max = maximum;
        self
    }

    #[inline]
    pub fn with_range(mut self, range: Rangef) -> Self {
        *self.range_mut() = range;
        self
    }

    pub fn range(self) -> Rangef {
        match self {
            Self::Absolute { range, .. } | Self::Relative { range, .. } | Self::Remainder { range, .. } => range,
        }
    }

    pub fn range_mut(&mut self) -> &mut Rangef {
        match self {
            Self::Absolute { range, .. } | Self::Relative { range, .. } | Self::Remainder { range, .. } => range,
        }
    }
}

pub trait UiExt {
    fn raw_scroll_delta_y(&self) -> f32;
    fn colored_label(&mut self, color: impl Into<Color32>, text: impl ToString) -> Response;
    fn data_label(&mut self, text: impl Into<WidgetText>) -> Response;
    fn label_with_colored_rect(&mut self, color: Vec<f32>, dtype: PixelType, row_height: f32) -> Response;
    fn text_edit_t<T: std::fmt::Display + std::str::FromStr>(&mut self, value: &mut T) -> Response;
    fn text_edit_value<T: std::fmt::Display + std::str::FromStr>(
        &mut self,
        display_value: &mut String,
        value: &mut T,
    ) -> Response;
    fn text_edit_value_capture<T: std::fmt::Display + std::str::FromStr + PartialEq + Clone>(
        &mut self,
        display_value: &mut String,
        value: &mut T,
        last_value: &mut T,
    ) -> Response;
    fn toggle_icon<'a>(&mut self, selected: &mut bool, icon: impl Into<Image<'a>>, name: &str) -> Response;
    fn indicator_icon<'a>(&mut self, is_on: bool, icon: impl Into<Image<'a>>, name: &str) -> Response;
    fn radio_icon<'a, Value: PartialEq>(
        &mut self,
        current_value: &mut Value,
        alternative: Value,
        icon: impl Into<Image<'a>>,
        name: &str,
    ) -> Response;
    fn calc_sizes<const N: usize>(&self, sizes: [Size; N]) -> [f32; N];
    fn columns_sized<R, const N: usize>(
        &mut self,
        sizes: [Size; N],
        add_contents: impl FnOnce(&mut [Self; N]) -> R,
    ) -> R;
    fn columns_sized_dyn<'c, R, const N: usize>(
        &mut self,
        sizes: [Size; N],
        add_contents: ColumnsContents<'c, R, N>,
    ) -> R
    where
        Self: Sized;
}

impl UiExt for Ui {
    #[inline]
    fn raw_scroll_delta_y(&self) -> f32 {
        self.input(|input| {
            input
                .events
                .iter()
                .filter_map(|event| match event {
                    egui::Event::MouseWheel { delta, .. } => Some(delta.y),
                    _ => None,
                })
                .sum()
        })
    }

    #[inline]
    fn colored_label(&mut self, color: impl Into<Color32>, text: impl ToString) -> Response {
        Label::new(egui::RichText::new(text.to_string()).color(color.into())).ui(self)
    }

    #[inline]
    fn data_label(&mut self, text: impl Into<WidgetText>) -> Response {
        Label::new(text).ui(self)
    }

    #[inline]
    fn label_with_colored_rect(&mut self, color: Vec<f32>, dtype: PixelType, row_height: f32) -> Response {
        let color32 = color.to_color32();

        let text_height = self.text_style_height(&egui::TextStyle::Body);
        let row_height = row_height.max(text_height);
        // Use the space between text rows for a larger swatch without changing their height.
        let rect_size = text_height + self.spacing().item_spacing.y;
        self.allocate_ui_with_layout(
            egui::vec2(self.available_width(), row_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x *= 0.5;
                let (id, row_rect) = ui.allocate_space(egui::vec2(rect_size, row_height));
                let rect = egui::Rect::from_center_size(row_rect.center(), egui::Vec2::splat(rect_size));
                let resp = ui.interact(rect, id, egui::Sense::click());
                ui.painter().rect_filled(rect, 4.0, color32);

                let color_text = if dtype.is_floating() {
                    color.to_rgba_string()
                } else {
                    color.to_rgba_int_string(dtype.alpha())
                };

                resp.context_menu(|ui| {
                    if ui.button("Copy #hex").clicked() {
                        ui.ctx().copy_text(color.to_hex_string());
                        ui.close();
                    }
                    if ui.button("Copy rgba() ").clicked() {
                        ui.ctx().copy_text(color_text.clone());
                        ui.close();
                    }
                });

                ui.label(&color_text);
            },
        )
        .response
    }

    fn text_edit_t<T: std::fmt::Display + std::str::FromStr>(&mut self, value: &mut T) -> Response {
        let mut text = value.to_string();
        let resp = egui::TextEdit::singleline(&mut text).ui(self);
        if resp.changed() {
            if let Ok(v) = text.parse::<T>() {
                *value = v;
            }
        }
        resp
    }

    fn text_edit_value<T: std::fmt::Display + std::str::FromStr>(
        &mut self,
        display_value: &mut String,
        value: &mut T,
    ) -> Response {
        let resp = egui::TextEdit::singleline(display_value).ui(self);

        // Track whether parsing failed this frame (trigger a short visual flash)
        let mut parse_failed = false;
        resp.on_enter(false, || {
            match display_value.parse::<T>() {
                Ok(v) => {
                    *value = v;
                    // Normalize the text to the parsed value on success
                }
                Err(_) => {
                    // Keep the user's text; flag an error to flash the field
                    parse_failed = true;
                }
            }
            *display_value = format!("{}", value);
        });

        // TODO: animate the color flash instead of just showing it for one frame
        if parse_failed {
            self.painter().rect_filled(resp.rect, 4.0, TEXT_EDIT_PARSE_FAILED_FLASH);
            self.ctx().request_repaint();
        }

        resp
    }

    fn text_edit_value_capture<T: std::fmt::Display + std::str::FromStr + PartialEq + Clone>(
        &mut self,
        display_value: &mut String,
        value: &mut T,
        last_value: &mut T,
    ) -> Response {
        if value != last_value {
            *display_value = format!("{}", value);
            last_value.clone_from(value);
        }

        self.text_edit_value(display_value, value)
    }

    fn toggle_icon<'a>(&mut self, selected: &mut bool, icon: impl Into<Image<'a>>, name: &str) -> Response {
        let tint_color = if *selected {
            self.style().visuals.selection.bg_fill.linear_multiply(1.2)
        } else {
            self.style().visuals.text_color()
        };
        let image_button = egui::Button::image(icon.into().tint(tint_color));

        let resp = self.add(image_button).on_hover_text(name);
        if resp.clicked() {
            *selected = !*selected;
        }

        resp
    }

    fn indicator_icon<'a>(&mut self, is_on: bool, icon: impl Into<Image<'a>>, name: &str) -> Response {
        let tint_color = if is_on {
            self.style().visuals.selection.bg_fill
        } else {
            self.style().visuals.weak_text_color()
        };
        let image = icon.into().tint(tint_color);

        self.add(image).on_hover_text(name)
    }

    fn radio_icon<'a, Value: PartialEq>(
        &mut self,
        current_value: &mut Value,
        alternative: Value,
        icon: impl Into<Image<'a>>,
        name: &str,
    ) -> Response {
        let tint_color = if *current_value == alternative {
            self.style().visuals.selection.bg_fill.linear_multiply(1.2)
        } else {
            self.style().visuals.text_color()
        };
        let image_button = egui::Button::image(icon.into().tint(tint_color));

        let resp = self.add(image_button).on_hover_text(name);
        if resp.clicked() {
            *current_value = alternative;
        }

        resp
    }

    fn calc_sizes<const N: usize>(&self, sizes: [Size; N]) -> [f32; N] {
        let total_width = self.available_width();
        let spacing = self.spacing().item_spacing.x;

        let mut results = [0.0f32; N];

        let mut total_absolute = 0.0;
        let mut total_relative_fraction = 0.0;
        let mut total_remainders = 0.0;

        for (i, size) in sizes.iter().enumerate() {
            match size {
                Size::Absolute { initial, range } => {
                    let clamped = initial.clamp(range.min, range.max);
                    results[i] = clamped;
                    total_absolute += clamped;
                }
                Size::Relative { fraction, range: _ } => {
                    total_relative_fraction += *fraction;
                }
                Size::Remainder { weight, range: _ } => {
                    total_remainders += *weight;
                }
            }
        }

        let remaining_space = (total_width - total_absolute).max(0.0);

        if total_relative_fraction > 0.0 {
            for (i, size) in sizes.iter().enumerate() {
                if let Size::Relative { fraction, range } = size {
                    let allocated = (fraction / total_relative_fraction) * remaining_space;
                    let clamped = allocated.clamp(range.min, range.max);
                    results[i] = clamped;
                }
            }
        }

        let used_space: f32 = results.iter().sum();
        let remaining_for_remainders = (total_width - used_space - spacing * (sizes.len() - 1) as f32).max(0.0);

        if total_remainders > 0.0 {
            let per_remainder = remaining_for_remainders / total_remainders;
            for (i, size) in sizes.iter().enumerate() {
                if let Size::Remainder { weight, range } = size {
                    let clamped = (per_remainder * weight).clamp(range.min, range.max);
                    results[i] = clamped;
                }
            }
        }

        results
    }

    #[inline]
    fn columns_sized<R, const N: usize>(
        &mut self,
        sizes: [Size; N],
        add_contents: impl FnOnce(&mut [Self; N]) -> R,
    ) -> R {
        self.columns_sized_dyn(sizes, Box::new(add_contents))
    }

    fn columns_sized_dyn<'c, R, const N: usize>(
        &mut self,
        sizes: [Size; N],
        add_contents: Box<dyn FnOnce(&mut [Self; N]) -> R + 'c>,
    ) -> R {
        let spacing = self.spacing().item_spacing.x;
        let actual_sizes = self.calc_sizes(sizes);
        let top_left = self.cursor().min;
        let mut current_left = 0.0;

        let mut columns: [Self; N] = std::array::from_fn(|col_idx| {
            let pos = top_left + egui::vec2(current_left, 0.0);
            let cell_size = actual_sizes[col_idx];
            let child_rect =
                egui::Rect::from_min_max(pos, egui::pos2(pos.x + cell_size, self.max_rect().right_bottom().y));
            current_left += cell_size + spacing;

            let mut column_ui = self.new_child(
                egui::UiBuilder::new()
                    .max_rect(child_rect)
                    .layout(egui::Layout::top_down_justified(egui::Align::Center)),
            );
            column_ui.set_width(cell_size);
            column_ui
        });

        let result = add_contents(&mut columns);

        let mut max_height = 0.0;
        for column in &columns {
            max_height = column.min_size().y.max(max_height);
        }

        // Make sure we fit everything next frame:
        let total_required_width = current_left - spacing; // remove last spacing

        let size = egui::vec2(self.available_width().max(total_required_width), max_height);
        self.advance_cursor_after_rect(egui::Rect::from_min_size(top_left, size));
        result
    }
}

pub trait ResponseExt {
    fn on_enter<'c>(&self, lost_focus: bool, f: impl FnMut() + 'c) -> &Self;
    fn hover_scroll<T: PartialEq + Clone>(&self, ui: &Ui, values: &[T], current: &mut T, is_cycle: bool);
}

impl ResponseExt for Response {
    fn on_enter<'c>(&self, lost_focus: bool, mut f: impl FnMut() + 'c) -> &Self {
        if self.lost_focus() && self.ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            f();
            if !lost_focus {
                self.request_focus();
            }
        }
        self
    }

    fn hover_scroll<T: PartialEq + Clone>(&self, ui: &Ui, values: &[T], current: &mut T, is_cycle: bool) {
        if self.hovered() {
            let scroll = ui.raw_scroll_delta_y();

            if scroll.abs() > 0.0 {
                if let Some(cur_idx) = values.iter().position(|n| n == current) {
                    let mut new_idx = cur_idx as isize + if scroll < 0.0 { 1 } else { -1 };
                    if new_idx < 0 {
                        new_idx = if is_cycle { (values.len() - 1) as isize } else { 0 };
                    } else if new_idx as usize >= values.len() {
                        new_idx = if is_cycle { 0 } else { (values.len() - 1) as isize };
                    }
                    if new_idx as usize != cur_idx {
                        if let Some(new_value) = values.get(new_idx as usize) {
                            *current = new_value.clone();
                            // 필요 시 뷰 갱신
                            ui.ctx().request_repaint();
                        }
                    }
                }
            }
        }
    }
}

pub trait InnerRespExt {
    fn hover_scroll<T: PartialEq + Clone>(self, ui: &Ui, values: &[T], current: &mut T, is_cycle: bool) -> Self;
}

impl<R> InnerRespExt for InnerResponse<R> {
    fn hover_scroll<T: PartialEq + Clone>(self, ui: &Ui, values: &[T], current: &mut T, is_cycle: bool) -> Self {
        self.response.hover_scroll(ui, values, current, is_cycle);
        self
    }
}

pub trait ComboBoxExt {
    fn combo(self, ui: &mut Ui, selected_text: &mut String, list: &[String]) -> InnerResponse<Option<()>>;
    fn combo_i32(self, ui: &mut Ui, selected_text: &mut i32, list: &[i32]) -> InnerResponse<Option<()>>;
    fn combo_i32_with<F: Fn(i32) -> String>(
        self,
        ui: &mut Ui,
        selected_value: &mut i32,
        list: &[i32],
        to_label: F,
    ) -> InnerResponse<Option<()>>;
}

impl ComboBoxExt for ComboBox<'_> {
    fn combo(self, ui: &mut Ui, selected_text: &mut String, list: &[String]) -> InnerResponse<Option<()>> {
        self.selected_text(selected_text.as_str())
            .show_ui(ui, |ui| {
                for name in list {
                    ui.selectable_value(selected_text, name.clone(), name);
                }
            })
            .hover_scroll(ui, list, selected_text, false)
    }

    fn combo_i32(self, ui: &mut Ui, selected_text: &mut i32, list: &[i32]) -> InnerResponse<Option<()>> {
        self.selected_text(selected_text.to_string())
            .show_ui(ui, |ui| {
                for index in list {
                    ui.selectable_value(selected_text, *index, index.to_string());
                }
            })
            .hover_scroll(ui, list, selected_text, false)
    }

    fn combo_i32_with<F: Fn(i32) -> String>(
        self,
        ui: &mut Ui,
        selected_value: &mut i32,
        list: &[i32],
        to_label: F,
    ) -> InnerResponse<Option<()>> {
        // We capture labels first so hover_scroll can still work with underlying values.
        self.selected_text(to_label(*selected_value))
            .show_ui(ui, |ui| {
                for index in list {
                    let label = to_label(*index);
                    ui.selectable_value(selected_value, *index, label);
                }
            })
            .hover_scroll(ui, list, selected_value, false)
    }
}
