use std::sync::atomic::Ordering;

use eframe::egui;

use super::{
    component::egui_ext::{ResponseExt, UiExt},
    ImageViewer,
};
use crate::{
    model::{AppState, Image, Recti, TitlebarState},
    res::{icons::Icons, KeyboardShortcutExt, UPDATE_ACCENT_TEXT},
    util::math_ext::vec2i,
};

pub(super) enum ToolbarAction {
    OpenFile,
    OpenFromClipboard,
}

pub(super) struct Toolbar<'a> {
    pub state: &'a mut AppState,
    pub viewer: &'a mut ImageViewer,
    pub icons: &'a Icons,
    pub show_settings: &'a mut bool,
    pub show_rois: &'a mut bool,
    pub is_loading: bool,
    pub update_available: bool,
}

impl Toolbar<'_> {
    pub(super) fn show(self, ui: &mut egui::Ui) -> Option<ToolbarAction> {
        let Self {
            state,
            viewer,
            icons,
            show_settings,
            show_rois,
            is_loading,
            update_available,
        } = self;
        let ctx = ui.ctx().clone();
        let mut action = None;
        #[cfg(target_os = "windows")]
        let mut regions = super::window_chrome::Regions {
            title: egui::Rect::NOTHING,
            buttons: [egui::Rect::NOTHING; 3],
            layer: ui.layer_id(),
        };
        let panel = egui::Panel::top("top");
        #[cfg(target_os = "windows")]
        let panel = if state.titlebar.is_integrated {
            panel
                .show_separator_line(true)
                .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin {
                    left: 8,
                    right: 0,
                    top: 0,
                    bottom: 0,
                }))
        } else {
            panel
        };
        panel.show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                #[cfg(target_os = "windows")]
                if state.titlebar.is_integrated {
                    ui.set_min_height(crate::res::icons::WINDOW_BUTTON_SIZE.y);
                }
                let open_button = ui
                    .add(egui::Button::image(icons.get_open_file(&ctx).tint(ui.visuals().text_color())))
                    .on_hover_text("Open image file...");
                open_button.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), "Open"));
                if open_button.clicked() {
                    action = Some(ToolbarAction::OpenFile);
                }
                let clipboard_button = ui
                    .add(egui::Button::image(
                        icons.get_open_clipboard(&ctx).tint(ui.visuals().text_color()),
                    ))
                    .on_hover_text(format!(
                        "Open from clipboard ({})",
                        crate::res::OPEN_FROM_CLIPBOARD_SC.format_sys()
                    ));
                clipboard_button.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), "Open from clipboard")
                });
                if clipboard_button.clicked() {
                    action = Some(ToolbarAction::OpenFromClipboard);
                }

                ui.separator();
                let reset_button = ui
                    .add(egui::Button::image(icons.get_reset_view(&ctx).tint(ui.visuals().text_color())))
                    .on_hover_text(format!(
                        "Reset zoom and pan to original ({})",
                        crate::res::RESET_VIEW.format_sys()
                    ));
                reset_button.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), "Reset"));
                if reset_button.clicked() {
                    viewer.reset_view();
                }

                let fit_button = ui
                    .add(egui::Button::image(icons.get_fit_view(&ctx).tint(ui.visuals().text_color())))
                    .on_hover_text("Fit image or marquee to view");
                fit_button.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), "Fit"));
                if fit_button.clicked() {
                    let rect = state.marquee_rect.validate();
                    if rect.empty() {
                        if let Some(asset) = &state.asset {
                            let spec = asset.image().spec();
                            let img_rect = Recti::from_min_size(vec2i(0, 0), vec2i(spec.width, spec.height));
                            viewer.fit_rect(img_rect);
                        }
                    } else {
                        viewer.fit_rect(rect);
                    }
                }
                let center_button = ui
                    .add(egui::Button::image(icons.get_center_view(&ctx).tint(ui.visuals().text_color())))
                    .on_hover_text("Center image or marquee in view");
                center_button.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), "Center"));
                if center_button.clicked() {
                    let rect = state.marquee_rect.validate();
                    if rect.empty() {
                        if let Some(asset) = &state.asset {
                            let spec = asset.image().spec();
                            let img_rect = Recti::from_min_size(vec2i(0, 0), vec2i(spec.width, spec.height));
                            viewer.center_rect(img_rect);
                        }
                    } else {
                        viewer.center_rect(rect);
                    }
                }

                ui.separator();
                ui.toggle_icon(
                    &mut state.is_show_pixel_value,
                    icons.get_show_pixel_value(&ctx),
                    "Show Pixel Value",
                    Some("Zoom in to see values"),
                );
                ui.toggle_icon(
                    &mut state.is_show_crosshair,
                    icons.get_show_crosshair(&ctx),
                    "Show Crosshair",
                    None,
                );
                ui.toggle_icon(
                    &mut state.is_show_minimap,
                    icons.get_show_minimap(&ctx),
                    "Show Minimap",
                    Some("Current visible region"),
                );
                let background_button = ui
                    .add(egui::Button::image(icons.get_background(
                        &ctx,
                        state.background_style,
                        ui.visuals(),
                    )))
                    .on_hover_text(format!(
                        "Background: {}\nClick to choose a background; scroll to change it.",
                        state.background_style.label()
                    ));
                background_button.hover_scroll(
                    ui,
                    &crate::settings::BackgroundStyle::ALL,
                    &mut state.background_style,
                    false,
                );
                egui::Popup::menu(&background_button).show(|ui| {
                    for style in crate::settings::BackgroundStyle::ALL {
                        if ui
                            .add(
                                egui::Button::image_and_text(
                                    icons.get_background(&ctx, style, ui.visuals()),
                                    style.label(),
                                )
                                .selected(state.background_style == style),
                            )
                            .clicked()
                        {
                            state.background_style = style;
                            ui.close();
                        }
                    }
                });

                let socket_address = state.socket_info.lock().unwrap().address.clone();
                socket_address_label(ui, icons, &socket_address, state.titlebar);
                ui.indicator_icon(
                    state.socket_state.is_socket_receiving.load(Ordering::Relaxed),
                    icons.get_downloading(&ctx),
                    "Image Receiving",
                );
                // Both states paint into the same slot so loading never shifts the toolbar.
                let (loading_rect, loading_response) = ui.allocate_exact_size(
                    egui::Vec2::splat(crate::res::icons::IMAGE_LOADING_ICON_SIZE),
                    egui::Sense::hover(),
                );
                if is_loading {
                    egui::Spinner::new().paint_at(ui, loading_rect);
                } else {
                    icons
                        .get_image_ready(&ctx)
                        .tint(ui.visuals().weak_text_color())
                        .paint_at(ui, loading_rect);
                }
                loading_response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::ProgressIndicator, ui.is_enabled(), "Image Loading")
                });
                loading_response.on_hover_text(if is_loading {
                    "Loading images..."
                } else {
                    "No images loading"
                });

                // Right end: panel visibility toggles
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    #[cfg(target_os = "windows")]
                    if state.titlebar.is_integrated {
                        regions.buttons = super::window_chrome::controls(ui);
                    }
                    let settings_button = ui.toggle_icon(show_settings, icons.get_settings(&ctx), "Settings", None);
                    if update_available {
                        let indicator_center = settings_button.rect.right_bottom() + egui::vec2(-4.0, -4.0);
                        ui.painter().circle_filled(indicator_center, 2.0, UPDATE_ACCENT_TEXT);
                    }

                    let statusbar_icon = icons.get_statusbar_panel(&ctx, state.is_show_statusbar);
                    ui.toggle_icon(&mut state.is_show_statusbar, statusbar_icon, "Status Bar", None);
                    let sidebar_icon = icons.get_sidebar_panel(&ctx, state.is_show_sidebar);
                    ui.toggle_icon(&mut state.is_show_sidebar, sidebar_icon, "Sidebar", None);
                    let rois_icon = icons.get_rois_panel(&ctx, *show_rois);
                    ui.toggle_icon(show_rois, rois_icon, "ROIs", Some(&crate::res::ROI_PANEL_TOGGLE.format_sys()));
                    #[cfg(target_os = "windows")]
                    if state.titlebar.is_integrated {
                        regions.title = super::window_chrome::title(ui, state.asset.as_ref().map(|asset| asset.name()));
                    }
                });
            });
        });

        #[cfg(target_os = "windows")]
        super::window_chrome::store_regions(&ctx, regions);
        action
    }
}

fn socket_address_label(ui: &mut egui::Ui, icons: &Icons, address: &str, titlebar: TitlebarState) {
    let gap = ui.spacing().item_spacing.x;
    let icon_size = crate::res::icons::SOCKET_ADDRESS_ICON_SIZE;
    let color = ui.visuals().weak_text_color();
    let text = ui
        .painter()
        .layout_no_wrap(address.to_owned(), egui::TextStyle::Body.resolve(ui.style()), color);
    // Reserve receiving/loading indicators, the four right-hand toggles, and
    // enough title space before deciding whether the address needs to collapse.
    let toggles = 4.0 * (icon_size + ui.spacing().button_padding.x * 2.0 + gap);
    let indicators = icon_size + crate::res::icons::IMAGE_LOADING_ICON_SIZE + gap * 3.0;
    let caption = if !ui.input(|i| i.viewport().fullscreen.unwrap_or(false)) && titlebar.trailing_controls_width > 0.0 {
        titlebar.trailing_controls_width + gap
    } else {
        0.0
    };
    let title = if titlebar.is_integrated { 120.0 } else { 0.0 };
    let compact =
        titlebar.is_integrated && ui.available_width() < text.size().x + toggles + indicators + caption + title;
    // One stable response keeps hover/context-menu state across both layouts.
    let size = if compact {
        egui::Vec2::splat(icon_size)
    } else {
        text.size()
    };
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    if compact {
        icons.get_socket_address(ui.ctx()).tint(color).paint_at(ui, rect);
    } else {
        ui.painter().galley(rect.min, text, color);
    }
    let response = response.on_hover_cursor(egui::CursorIcon::ContextMenu);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::Role::Label,
            ui.is_enabled(),
            format!("Socket Listener Address: {address}"),
        )
    });
    response
        .on_hover_text(format!("Socket Listener Address\n{address}"))
        .context_menu(|ui| {
            if ui.button("Copy Address").clicked() {
                arboard::Clipboard::new()
                    .and_then(|mut cb| cb.set_text(address.to_owned()))
                    .unwrap_or_else(|e| eprintln!("Failed to copy socket address to clipboard: {e}"));
                ui.close();
            }
            ui.separator();
            if ui
                .hyperlink_to(
                    "Remote Viewer Guide",
                    "https://github.com/edoli/edolview/wiki/Remote%E2%80%90Viewer",
                )
                .clicked()
            {
                ui.close();
            }
        });
}
