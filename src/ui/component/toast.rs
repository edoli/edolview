use std::time::{Duration, Instant};

use eframe::{
    egui::{self, Frame, Response, Shape, Stroke, StrokeKind, Ui, Widget, WidgetText},
    epaint::RectShape,
};

use crate::res::{TOAST_ERROR_ICON_COLOR, TOAST_INFO_ICON_COLOR, TOAST_SUCCESS_ICON_COLOR, TOAST_WARNING_ICON_COLOR};

pub struct ToastStyle {
    pub info_icon: WidgetText,
    pub warning_icon: WidgetText,
    pub error_icon: WidgetText,
    pub success_icon: WidgetText,
    pub close_button_text: WidgetText,
}

impl ToastStyle {
    fn new() -> Self {
        Self {
            info_icon: WidgetText::from("ℹ").color(TOAST_INFO_ICON_COLOR),
            warning_icon: WidgetText::from("⚠").color(TOAST_WARNING_ICON_COLOR),
            error_icon: WidgetText::from("❗").color(TOAST_ERROR_ICON_COLOR),
            success_icon: WidgetText::from("✔").color(TOAST_SUCCESS_ICON_COLOR),
            close_button_text: WidgetText::from("🗙"),
        }
    }
}

#[derive(Clone)]
pub enum ToastKind {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Clone)]
pub struct Toast {
    pub message: String,
    pub duration: Duration,
    pub kind: ToastKind,
    pub created_at: Instant,
}

const DEFAULT_TOAST_DURATION: Duration = Duration::from_secs(3);

impl Toast {
    pub fn new(message: String, duration: Option<Duration>, kind: ToastKind) -> Self {
        let duration = duration.unwrap_or(DEFAULT_TOAST_DURATION);
        Self {
            message,
            duration,
            kind,
            created_at: Instant::now(),
        }
    }

    pub fn info(message: String) -> Self {
        Self::new(message, None, ToastKind::Info)
    }
    pub fn warning(message: String) -> Self {
        Self::new(message, None, ToastKind::Warning)
    }
    pub fn error(message: String) -> Self {
        Self::new(message, None, ToastKind::Error)
    }
    pub fn success(message: String) -> Self {
        Self::new(message, None, ToastKind::Success)
    }
}

type Toasts = Vec<Toast>;

pub trait ToastsExt {
    fn add_toast(&mut self, toast: Toast);
    fn add_info(&mut self, message: String);
    fn add_warning(&mut self, message: String);
    fn add_error(&mut self, message: String);
    fn add_success(&mut self, message: String);
    fn retain_active(&mut self);
}

impl ToastsExt for Toasts {
    fn add_toast(&mut self, toast: Toast) {
        self.push(toast);
    }

    fn add_info(&mut self, message: String) {
        self.push(Toast::info(message));
    }

    fn add_warning(&mut self, message: String) {
        self.push(Toast::warning(message));
    }

    fn add_error(&mut self, message: String) {
        self.push(Toast::error(message));
    }

    fn add_success(&mut self, message: String) {
        self.push(Toast::success(message));
    }

    fn retain_active(&mut self) {
        let now = Instant::now();
        self.retain(|toast| now.duration_since(toast.created_at) < toast.duration);
    }
}

#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct ToastUi<'a> {
    pub toasts: &'a mut Toasts,
    pub style: ToastStyle,
}

impl<'a> ToastUi<'a> {
    pub fn new(toasts: &'a mut Toasts) -> Self {
        Self {
            toasts,
            style: ToastStyle::new(),
        }
    }
}

impl Widget for ToastUi<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let style = &self.style;
        egui::Area::new(egui::Id::unique("toasts_area"))
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 32.0))
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    for toast in self.toasts.iter_mut() {
                        default_toast_contents(ui, toast, style);
                    }
                });
            })
            .response
    }
}

fn default_toast_contents(ui: &mut Ui, toast: &mut Toast, style: &ToastStyle) -> Response {
    let inner_margin = 10.0;
    let frame = Frame::window(ui.style());
    let response = frame
        .inner_margin(inner_margin)
        .stroke(Stroke::NONE)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let icon = match toast.kind {
                    ToastKind::Info => style.info_icon.clone(),
                    ToastKind::Warning => style.warning_icon.clone(),
                    ToastKind::Error => style.error_icon.clone(),
                    ToastKind::Success => style.success_icon.clone(),
                };

                ui.label(icon);
                ui.label(toast.message.clone());

                // No close button for now
                // if ui.button(style.close_button_text.clone()).clicked() {
                //     toast.duration = Duration::ZERO;
                // }
            })
        })
        .response;

    // Draw the frame's stroke last
    let frame_shape = Shape::Rect(RectShape::stroke(
        response.rect,
        frame.corner_radius,
        ui.visuals().window_stroke,
        StrokeKind::Inside,
    ));
    ui.painter().add(frame_shape);

    response
}
