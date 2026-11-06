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
                let socket_address = state.socket_info.lock().unwrap().address.clone();
                let layout = ToolbarLayout::new(ui, &socket_address, state.titlebar);
                if layout.grouped {
                    group_menu(ui, "open_group", icons.get_open_file(&ctx), "Open Images", |ui| {
                        action = open_controls(ui, icons, true);
                    });
                    group_menu(ui, "view_group", icons.get_fit_view(&ctx), "View Controls", |ui| {
                        view_controls(ui, icons, state, viewer, true);
                    });
                    group_menu(ui, "display_group", icons.get_view_settings(&ctx), "View Settings", |ui| {
                        view_settings(ui, icons, state, true);
                    });
                } else {
                    ui.push_id("open_controls", |ui| action = open_controls(ui, icons, false));
                    ui.add(egui::Separator::default().spacing(GROUP_SEPARATOR_WIDTH));
                    ui.push_id("view_controls", |ui| view_controls(ui, icons, state, viewer, false));
                    ui.add(egui::Separator::default().spacing(GROUP_SEPARATOR_WIDTH));
                    ui.push_id("view_settings", |ui| view_settings(ui, icons, state, false));
                }
                ui.push_id("socket_address", |ui| {
                    socket_address_label(ui, icons, &socket_address, layout.compact_address);
                });
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

const GROUP_SEPARATOR_WIDTH: f32 = 6.0;
const MIN_TITLE_WIDTH: f32 = 120.0;
const PREFERRED_TITLE_WIDTH: f32 = 320.0;

struct ToolbarLayout {
    grouped: bool,
    compact_address: bool,
}

impl ToolbarLayout {
    fn new(ui: &egui::Ui, address: &str, titlebar: TitlebarState) -> Self {
        let gap = ui.spacing().item_spacing.x;
        let icon = crate::res::icons::TOOLBAR_ICON_SIZE;
        let button = icon + ui.spacing().button_padding.x * 2.0 + gap;
        let address_width = ui
            .painter()
            .layout_no_wrap(
                address.to_owned(),
                egui::TextStyle::Body.resolve(ui.style()),
                ui.visuals().weak_text_color(),
            )
            .size()
            .x;
        let caption =
            if !ui.input(|i| i.viewport().fullscreen.unwrap_or(false)) && titlebar.trailing_controls_width > 0.0 {
                titlebar.trailing_controls_width + gap
            } else {
                0.0
            };
        // Three right-hand toggles, two separators, receiving/loading indicators,
        // and the spacing after the socket address. Caption buttons have no inner gaps.
        let fixed = 3.0 * button
            + 2.0 * (GROUP_SEPARATOR_WIDTH + gap)
            + icon
            + crate::res::icons::IMAGE_LOADING_ICON_SIZE
            + 3.0 * gap
            + caption;
        let title_min = if titlebar.is_integrated { MIN_TITLE_WIDTH } else { 0.0 };
        let smallest_address = if titlebar.is_integrated { icon } else { address_width };
        let grouped = ui.available_width() < 9.0 * button + fixed + smallest_address + title_min;
        let title_preferred = if titlebar.is_integrated {
            PREFERRED_TITLE_WIDTH
        } else {
            0.0
        };
        let controls = if grouped { 3.0 } else { 9.0 } * button;
        let compact_address = titlebar.is_integrated
            && ui.available_width()
                < controls + fixed
                    - if grouped {
                        2.0 * (GROUP_SEPARATOR_WIDTH + gap)
                    } else {
                        0.0
                    }
                    + address_width
                    + title_preferred;
        Self {
            grouped,
            compact_address,
        }
    }
}

fn group_menu(ui: &mut egui::Ui, id: &str, icon: egui::Image<'_>, name: &str, contents: impl FnOnce(&mut egui::Ui)) {
    ui.push_id(id, |ui| {
        let response = ui.menu_image_button(icon.tint(ui.visuals().text_color()), contents).response;
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), name));
        response.on_hover_text(name);
    });
}

// Share the actions between the inline icons and their named menu items.
fn control_button(
    ui: &mut egui::Ui,
    icon: egui::Image<'_>,
    name: &str,
    description: &str,
    shortcut: Option<egui::KeyboardShortcut>,
    in_menu: bool,
) -> egui::Response {
    let icon = icon.tint(ui.visuals().text_color());
    let button = if in_menu {
        let button = egui::Button::image_and_text(icon, name);
        if let Some(shortcut) = shortcut {
            button.shortcut_text(shortcut.format_sys())
        } else {
            button
        }
    } else {
        egui::Button::image(icon)
    };
    let response = ui.add(button);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), name));
    response.on_hover_text(shortcut.map_or_else(
        || description.to_owned(),
        |shortcut| format!("{description} ({})", shortcut.format_sys()),
    ))
}

fn open_controls(ui: &mut egui::Ui, icons: &Icons, in_menu: bool) -> Option<ToolbarAction> {
    let ctx = ui.ctx().clone();
    let action = if control_button(ui, icons.get_open_file(&ctx), "Open", "Open image file...", None, in_menu).clicked()
    {
        Some(ToolbarAction::OpenFile)
    } else {
        None
    };
    let action = if control_button(
        ui,
        icons.get_open_clipboard(&ctx),
        "Open from clipboard",
        "Open from clipboard",
        Some(crate::res::OPEN_FROM_CLIPBOARD_SC),
        in_menu,
    )
    .clicked()
    {
        Some(ToolbarAction::OpenFromClipboard)
    } else {
        action
    };
    if in_menu && action.is_some() {
        ui.close();
    }
    action
}

fn view_controls(ui: &mut egui::Ui, icons: &Icons, state: &AppState, viewer: &mut ImageViewer, in_menu: bool) {
    let ctx = ui.ctx().clone();
    if control_button(
        ui,
        icons.get_reset_view(&ctx),
        "Reset",
        "Reset zoom and pan to original",
        Some(crate::res::RESET_VIEW),
        in_menu,
    )
    .clicked()
    {
        viewer.reset_view();
        if in_menu {
            ui.close();
        }
    }
    let fit = control_button(
        ui,
        icons.get_fit_view(&ctx),
        "Fit",
        "Fit image or marquee to view",
        None,
        in_menu,
    )
    .clicked();
    let center = control_button(
        ui,
        icons.get_center_view(&ctx),
        "Center",
        "Center image or marquee in view",
        None,
        in_menu,
    )
    .clicked();
    if fit || center {
        let marquee = state.marquee_rect.validate();
        let rect = if !marquee.empty() {
            Some(marquee)
        } else {
            state.asset.as_ref().map(|asset| {
                let spec = asset.image().spec();
                Recti::from_min_size(vec2i(0, 0), vec2i(spec.width, spec.height))
            })
        };
        if let Some(rect) = rect {
            if fit {
                viewer.fit_rect(rect);
            } else {
                viewer.center_rect(rect);
            }
        }
        if in_menu {
            ui.close();
        }
    }
}

fn view_settings(ui: &mut egui::Ui, icons: &Icons, state: &mut AppState, in_menu: bool) {
    let ctx = ui.ctx().clone();
    for (selected, icon, name, description) in [
        (
            &mut state.is_show_pixel_value,
            icons.get_show_pixel_value(&ctx),
            "Show Pixel Value",
            Some("Zoom in to see values"),
        ),
        (
            &mut state.is_show_crosshair,
            icons.get_show_crosshair(&ctx),
            "Show Crosshair",
            None,
        ),
        (
            &mut state.is_show_minimap,
            icons.get_show_minimap(&ctx),
            "Show Minimap",
            Some("Current visible region"),
        ),
    ] {
        if in_menu {
            if ui
                .add(egui::Button::image_and_text(icon.tint(ui.visuals().text_color()), name).selected(*selected))
                .clicked()
            {
                *selected = !*selected;
                ui.close();
            }
        } else {
            ui.toggle_icon(selected, icon, name, description);
        }
    }
    let background = icons.get_background(&ctx, state.background_style, ui.visuals());
    if in_menu {
        ui.menu_image_text_button(background, "Background", |ui| background_options(ui, icons, state));
    } else {
        let response = ui.add(egui::Button::image(background)).on_hover_text(format!(
            "Background: {}\nClick to choose a background; scroll to change it.",
            state.background_style.label(),
        ));
        response.hover_scroll(ui, &crate::settings::BackgroundStyle::ALL, &mut state.background_style, false);
        egui::Popup::menu(&response).show(|ui| background_options(ui, icons, state));
    }
}

fn background_options(ui: &mut egui::Ui, icons: &Icons, state: &mut AppState) {
    for style in crate::settings::BackgroundStyle::ALL {
        if ui
            .add(
                egui::Button::image_and_text(icons.get_background(ui.ctx(), style, ui.visuals()), style.label())
                    .selected(state.background_style == style),
            )
            .clicked()
        {
            state.background_style = style;
            ui.close();
        }
    }
}

fn socket_address_label(ui: &mut egui::Ui, icons: &Icons, address: &str, compact: bool) {
    let icon_size = crate::res::icons::SOCKET_ADDRESS_ICON_SIZE;
    let color = ui.visuals().weak_text_color();
    let text = ui
        .painter()
        .layout_no_wrap(address.to_owned(), egui::TextStyle::Body.resolve(ui.style()), color);
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
