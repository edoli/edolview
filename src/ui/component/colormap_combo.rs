use std::{collections::HashMap, sync::mpsc, thread};

use eframe::{
    egui::{self, vec2},
    egui_wgpu::{self, wgpu},
};

use super::egui_ext::ResponseExt;
use crate::ui::gpu::mono_colormap_preview_pipeline;

type PreviewResult = (String, Result<wgpu::RenderPipeline, String>);

#[derive(Default)]
pub struct MonoColormapPreviews {
    pipelines: HashMap<String, wgpu::RenderPipeline>,
    pending: Option<mpsc::Receiver<PreviewResult>>,
    started: bool,
}

impl MonoColormapPreviews {
    pub fn prepare(
        &mut self,
        ctx: &egui::Context,
        render_state: &egui_wgpu::RenderState,
        names: &[String],
    ) -> Vec<String> {
        if !self.started {
            self.started = true;
            let (tx, rx) = mpsc::channel();
            self.pending = Some(rx);
            let device = render_state.device.clone();
            let format = render_state.target_format;
            let names = names.to_vec();
            let ctx = ctx.clone();
            // Compile once off the UI thread, then reuse the GPU pipelines on every popup opening.
            thread::spawn(move || {
                for name in names {
                    let result = mono_colormap_preview_pipeline(&device, format, &name)
                        .map_err(|error| format!("Failed to preview colormap '{name}': {error}"));
                    if tx.send((name, result)).is_err() {
                        break;
                    }
                    ctx.request_repaint();
                }
            });
        }

        let mut errors = Vec::new();
        if let Some(rx) = &self.pending {
            loop {
                match rx.try_recv() {
                    Ok((name, Ok(pipeline))) => {
                        self.pipelines.insert(name, pipeline);
                    }
                    Ok((_, Err(error))) => errors.push(error),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.pending = None;
                        break;
                    }
                }
            }
        }
        errors
    }

    pub fn show(
        &self,
        ui: &mut egui::Ui,
        combo: egui::ComboBox<'_>,
        selected: &mut String,
        names: &[String],
    ) -> egui::InnerResponse<Option<()>> {
        let response = combo.selected_text(selected.as_str()).show_ui(ui, |ui| {
            let preview_size = vec2(52.0, 12.0);
            let padding = ui.spacing().button_padding;
            let font = egui::TextStyle::Button.resolve(ui.style());
            let label_width = names
                .iter()
                .map(|name| {
                    ui.painter()
                        .layout_no_wrap(name.clone(), font.clone(), ui.visuals().text_color())
                        .size()
                        .x
                })
                .fold(0.0_f32, f32::max);
            let row_width = ui.available_width().max(label_width + preview_size.x + padding.x * 2.0 + 12.0);
            let row_height = ui.spacing().interact_size.y.max(preview_size.y + padding.y * 2.0);
            for name in names {
                let row = ui.add_sized(
                    vec2(row_width, row_height),
                    egui::Button::selectable(*selected == *name, "").left_text(name.as_str()),
                );
                let bar = egui::Rect::from_center_size(
                    egui::pos2(row.rect.right() - padding.x - preview_size.x * 0.5, row.rect.center().y),
                    preview_size,
                );
                if ui.is_rect_visible(bar) {
                    if let Some(pipeline) = self.pipelines.get(name) {
                        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                            bar,
                            ColormapPreviewCallback(pipeline.clone()),
                        ));
                    } else {
                        ui.painter().rect_filled(bar, 0.0, ui.visuals().faint_bg_color);
                    }
                    ui.painter().rect_stroke(
                        bar,
                        0.0,
                        ui.visuals().widgets.noninteractive.bg_stroke,
                        egui::StrokeKind::Outside,
                    );
                }
                if row.clicked() {
                    *selected = name.clone();
                    ui.close();
                }
            }
        });
        response.response.hover_scroll(ui, names, selected, false);
        response
    }
}

struct ColormapPreviewCallback(wgpu::RenderPipeline);

impl egui_wgpu::CallbackTrait for ColormapPreviewCallback {
    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        _resources: &egui_wgpu::CallbackResources,
    ) {
        pass.set_pipeline(&self.0);
        pass.draw(0..3, 0..1);
    }
}
