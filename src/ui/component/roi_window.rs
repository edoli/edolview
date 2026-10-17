use eframe::egui;

use crate::{
    model::Recti,
    res::icons::Icons,
    ui::component::egui_ext::{Size, UiExt},
};

#[derive(Default)]
pub struct RoiWindowActions {
    pub add_current: bool,
    pub clear_all: bool,
    pub jump_to: Option<usize>,
    pub move_up: Option<usize>,
    pub move_down: Option<usize>,
    pub remove_index: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum RoiJumpMode {
    None,
    #[default]
    Center,
    Fit,
}

impl RoiJumpMode {
    fn label(self) -> &'static str {
        match self {
            Self::None => "Do Nothing",
            Self::Center => "Center",
            Self::Fit => "Fit",
        }
    }
}

pub fn show_roi_window(
    ctx: &egui::Context,
    icons: &Icons,
    open: &mut bool,
    rois: &[Recti],
    active_roi_index: Option<usize>,
    jump_mode: &mut RoiJumpMode,
    add_shortcut_label: &str,
) -> RoiWindowActions {
    if !*open {
        return RoiWindowActions::default();
    }

    let mut actions = RoiWindowActions::default();

    egui::Window::new("ROIs")
        .default_pos(egui::pos2(1024.0, 72.0))
        .collapsible(false)
        .resizable(true)
        .default_width(320.0)
        .open(open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let add_clicked = ui
                    .button("Add Current")
                    .on_hover_text(format!("Add current selection as ROI ({add_shortcut_label})"))
                    .clicked();
                if add_clicked {
                    actions.add_current = true;
                }

                if ui.add_enabled(!rois.is_empty(), egui::Button::new("Clear All")).clicked() {
                    actions.clear_all = true;
                }
                ui.label("Jump:");
                egui::ComboBox::from_id_salt("roi_jump_mode")
                    .selected_text(jump_mode.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(jump_mode, RoiJumpMode::None, RoiJumpMode::None.label());
                        ui.selectable_value(jump_mode, RoiJumpMode::Center, RoiJumpMode::Center.label());
                        ui.selectable_value(jump_mode, RoiJumpMode::Fit, RoiJumpMode::Fit.label());
                    });
            });
            ui.separator();

            if rois.is_empty() {
                ui.label("No ROIs");
                return;
            }

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                for (index, rect) in rois.iter().enumerate() {
                    let selected = active_roi_index == Some(index);

                    ui.scope(|ui| {
                        let mut style = (**ui.style()).clone();
                        style.spacing.button_padding = egui::vec2(2.0, 2.0);
                        ui.set_style(style);

                        ui.columns_sized(
                            [
                                Size::remainder(1.0),
                                Size::exact(16.0),
                                Size::exact(16.0),
                                Size::exact(16.0),
                            ],
                            |columns| {
                                let rect_label = format!("{}: {}", index + 1, rect);

                                columns[0].with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                                    if ui.selectable_label(selected, rect_label).clicked() {
                                        actions.jump_to = Some(index);
                                    }
                                });

                                let mut add_button =
                                    |col: usize,
                                     enabled: bool,
                                     icon_fn: fn(&Icons, &egui::Context) -> egui::Image<'static>,
                                     hover_text: &str,
                                     action: &mut Option<usize>| {
                                        if columns[col]
                                            .add_enabled(
                                                enabled,
                                                egui::Button::image(
                                                    icon_fn(icons, ctx).fit_to_exact_size(egui::vec2(14.0, 14.0)),
                                                ),
                                            )
                                            .on_hover_text(hover_text)
                                            .clicked()
                                        {
                                            *action = Some(index);
                                        }
                                    };

                                let is_first = index == 0;
                                let is_last = index + 1 == rois.len();

                                add_button(1, !is_first, Icons::get_arrow_up, "Move ROI up", &mut actions.move_up);
                                add_button(2, !is_last, Icons::get_arrow_down, "Move ROI down", &mut actions.move_down);
                                add_button(3, true, Icons::get_delete, "Delete ROI", &mut actions.remove_index);
                            },
                        );
                    });
                }
            });
        });

    actions
}
