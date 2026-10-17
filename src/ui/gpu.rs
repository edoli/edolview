use std::{borrow::Cow, num::NonZeroU64, path::PathBuf, sync::Arc};

use bytemuck::{Pod, Zeroable};
use color_eyre::eyre::{eyre, Result};
use eframe::{
    egui::{self, Color32, Vec2},
    egui_wgpu::{self, wgpu},
};
use serde::{Deserialize, Serialize};

use crate::{
    model::{GpuImageTexture, Image, MinMaxTotal},
    util::path_ext::exe_dir_or_cwd,
};

const IMAGE_SHADER_CODE: &str = include_str!("gpu_image.frag");
const PARAM_SLOT_COUNT: u64 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScaleMode {
    #[default]
    Linear = 0,
    Inverse = 1,
    Log = 2,
    Absolute = 3,
}

impl ScaleMode {
    pub fn auto_normalize_enabled(self) -> bool {
        matches!(self, Self::Linear | Self::Absolute)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShaderParams {
    pub use_alpha: bool,
    pub offset: f32,
    pub exposure: f32,
    pub gamma: f32,
    pub min_v: f32,
    pub max_v: f32,
    pub auto_minmax: bool,
    pub scale_mode: ScaleMode,
    pub use_per_channel: bool,
    pub min_v_channels: [f32; 4],
    pub max_v_channels: [f32; 4],
    pub auto_minmax_channels: [bool; 4],
    pub scale_mode_channels: [ScaleMode; 4],
}

impl Default for ShaderParams {
    fn default() -> Self {
        Self {
            use_alpha: true,
            offset: 0.0,
            exposure: 0.0,
            gamma: 1.0,
            min_v: 0.0,
            max_v: 1.0,
            auto_minmax: false,
            scale_mode: ScaleMode::Linear,
            use_per_channel: false,
            min_v_channels: [0.0; 4],
            max_v_channels: [1.0; 4],
            auto_minmax_channels: [false; 4],
            scale_mode_channels: [ScaleMode::Linear; 4],
        }
    }
}

#[derive(Clone, Debug)]
pub struct MinMaxOverlay {
    pub enabled: bool,
    pub show_min_channels: [bool; 4],
    pub show_max_channels: [bool; 4],
    pub scope_rect: [i32; 4],
    pub channel_count: i32,
    pub value_scale: f32,
    pub compare_epsilon: f32,
    pub min_values: [f32; 4],
    pub max_values: [f32; 4],
}

impl Default for MinMaxOverlay {
    fn default() -> Self {
        Self {
            enabled: false,
            show_min_channels: [false; 4],
            show_max_channels: [false; 4],
            scope_rect: [0; 4],
            channel_count: 0,
            value_scale: 1.0,
            compare_epsilon: 0.0,
            min_values: [0.0; 4],
            max_values: [0.0; 4],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuParams {
    viewport_image: [f32; 4],
    transform: [f32; 4],
    color: [f32; 4],
    global_scale: [f32; 4],
    min_values: [f32; 4],
    max_values: [f32; 4],
    scale_modes: [f32; 4],
    overlay_flags: [f32; 4],
    overlay_show_min: [f32; 4],
    overlay_show_max: [f32; 4],
    overlay_scope: [f32; 4],
    overlay_min_values: [f32; 4],
    overlay_max_values: [f32; 4],
    background_color_a: [f32; 4],
    background_color_b: [f32; 4],
    background: [f32; 4],
}

impl GpuParams {
    #[allow(clippy::too_many_arguments)]
    fn image(
        viewport_size: Vec2,
        image_size: Vec2,
        channel_index: i32,
        min_max: &MinMaxTotal,
        scale: f32,
        position: Vec2,
        shader: &ShaderParams,
        overlay: &MinMaxOverlay,
        background_a: Color32,
        background_b: Color32,
    ) -> Self {
        let mut min_values = shader.min_v_channels;
        let mut max_values = shader.max_v_channels;
        if shader.use_per_channel {
            for index in 0..4 {
                if shader.auto_minmax_channels[index] && shader.scale_mode_channels[index].auto_normalize_enabled() {
                    (min_values[index], max_values[index]) = if shader.scale_mode_channels[index] == ScaleMode::Absolute
                    {
                        (min_max.min_abs(index), min_max.max_abs(index))
                    } else {
                        (min_max.min(index), min_max.max(index))
                    };
                }
            }
        }

        let (global_min, global_max) = if shader.auto_minmax && shader.scale_mode.auto_normalize_enabled() {
            if shader.scale_mode == ScaleMode::Absolute {
                (min_max.total_min_abs(), min_max.total_max_abs())
            } else {
                (min_max.total_min(), min_max.total_max())
            }
        } else {
            (shader.min_v, shader.max_v)
        };

        Self {
            viewport_image: [viewport_size.x, viewport_size.y, image_size.x, image_size.y],
            transform: [scale, position.x, position.y, channel_index as f32],
            color: [
                shader.use_alpha as u8 as f32,
                shader.exposure,
                shader.offset,
                shader.gamma,
            ],
            global_scale: [
                global_min,
                global_max,
                shader.scale_mode as i32 as f32,
                shader.use_per_channel as u8 as f32,
            ],
            min_values,
            max_values,
            scale_modes: shader.scale_mode_channels.map(|mode| mode as i32 as f32),
            overlay_flags: [
                overlay.enabled as u8 as f32,
                overlay.channel_count as f32,
                overlay.value_scale,
                overlay.compare_epsilon,
            ],
            overlay_show_min: overlay.show_min_channels.map(|value| value as u8 as f32),
            overlay_show_max: overlay.show_max_channels.map(|value| value as u8 as f32),
            overlay_scope: overlay.scope_rect.map(|value| value as f32),
            overlay_min_values: overlay.min_values,
            overlay_max_values: overlay.max_values,
            background_color_a: color_to_linear_f32(background_a),
            background_color_b: color_to_linear_f32(background_b),
            background: [16.0, 0.0, 0.0, 0.0],
        }
    }
}

fn color_to_linear_f32(color: Color32) -> [f32; 4] {
    let rgba = color.to_normalized_gamma_f32();
    [rgba[0], rgba[1], rgba[2], rgba[3]]
}

#[derive(Clone, Copy, Debug)]
pub enum ImageSlot {
    Primary,
    Secondary,
}

#[derive(Clone, Copy)]
pub struct PaneDraw {
    pub viewport_px: egui::Rect,
    pub slot: ImageSlot,
    pub uniform_slot: u32,
}

pub type ExportCompletion = Arc<dyn Fn(std::result::Result<Vec<u8>, String>) + Send + Sync>;

#[derive(Clone)]
pub struct ExportRequest {
    pub width: u32,
    pub height: u32,
    pub slot: ImageSlot,
    pub completion: ExportCompletion,
}

#[derive(Clone)]
pub struct ImagePaintCallback {
    pub panes: Vec<PaneDraw>,
    pub export: Option<ExportRequest>,
}

impl egui_wgpu::CallbackTrait for ImagePaintCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let (Some(export), Some(renderer)) = (self.export.as_ref(), resources.get_mut::<GpuRenderer>()) {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("edolview export encoder"),
            });
            match renderer.encode_export(device, &mut encoder, export) {
                Ok(readback) => {
                    queue.submit([encoder.finish()]);
                    readback.map();
                }
                Err(error) => (export.completion)(Err(error.to_string())),
            }
        }
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        if let Some(renderer) = resources.get::<GpuRenderer>() {
            renderer.paint(pass, &self.panes);
        }
    }
}

struct GpuImage {
    _texture: Arc<GpuImageTexture>,
    bind_group: wgpu::BindGroup,
    image_id: u64,
}

struct ExportReadback {
    buffer: Arc<wgpu::Buffer>,
    row_bytes: u32,
    padded_row_bytes: u32,
    width: u32,
    height: u32,
    completion: ExportCompletion,
}

impl ExportReadback {
    fn map(self) {
        let callback_buffer = Arc::clone(&self.buffer);
        self.buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| match result {
                Ok(()) => {
                    std::thread::spawn(move || {
                        let mapped = match callback_buffer.slice(..).get_mapped_range() {
                            Ok(mapped) => mapped,
                            Err(error) => {
                                callback_buffer.unmap();
                                (self.completion)(Err(format!("GPU export readback failed: {error}")));
                                return;
                            }
                        };
                        let mut pixels = Vec::with_capacity(self.row_bytes as usize * self.height as usize);
                        for row in mapped.chunks(self.padded_row_bytes as usize).take(self.height as usize) {
                            pixels.extend_from_slice(&row[..self.row_bytes as usize]);
                        }
                        drop(mapped);
                        callback_buffer.unmap();
                        debug_assert_eq!(pixels.len(), self.width as usize * self.height as usize * 4);
                        (self.completion)(Ok(pixels));
                    });
                }
                Err(error) => (self.completion)(Err(format!("GPU export readback failed: {error}"))),
            });
    }
}

pub struct GpuRenderer {
    target_format: wgpu::TextureFormat,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    uniform_buffer: wgpu::Buffer,
    uniform_stride: u64,
    background_pipeline: wgpu::RenderPipeline,
    image_pipeline: wgpu::RenderPipeline,
    export_pipeline: wgpu::RenderPipeline,
    primary: Option<GpuImage>,
    secondary: Option<GpuImage>,
    last_colormap: String,
    last_is_mono: bool,
    last_error: Option<String>,
}

impl GpuRenderer {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat, _stream_rgba_uploads: bool) -> Result<Self> {
        let uniform_stride = align_to(
            std::mem::size_of::<GpuParams>() as u64,
            device.limits().min_uniform_buffer_offset_alignment as u64,
        );
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("edolview image uniforms"),
            size: uniform_stride * PARAM_SLOT_COUNT,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("edolview image bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: NonZeroU64::new(std::mem::size_of::<GpuParams>() as u64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("edolview image pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let background_pipeline = create_background_pipeline(device, &pipeline_layout, target_format);
        let fragment_module = compile_fragment_module(device, "rgb", false)?;
        let image_pipeline = create_image_pipeline(device, &pipeline_layout, target_format, &fragment_module);
        let export_pipeline =
            create_image_pipeline(device, &pipeline_layout, wgpu::TextureFormat::Rgba8Unorm, &fragment_module);
        Ok(Self {
            target_format,
            bind_group_layout,
            pipeline_layout,
            uniform_buffer,
            uniform_stride,
            background_pipeline,
            image_pipeline,
            export_pipeline,
            primary: None,
            secondary: None,
            last_colormap: "rgb".to_owned(),
            last_is_mono: false,
            last_error: None,
        })
    }

    pub fn sync_image(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        slot: ImageSlot,
        image: Option<&impl Image>,
    ) -> Result<()> {
        let current = match slot {
            ImageSlot::Primary => &mut self.primary,
            ImageSlot::Secondary => &mut self.secondary,
        };
        let Some(image) = image else {
            *current = None;
            return Ok(());
        };
        if current.as_ref().is_some_and(|gpu_image| gpu_image.image_id == image.id()) {
            return Ok(());
        }
        *current = Some(upload_image(device, &self.bind_group_layout, &self.uniform_buffer, image)?);
        Ok(())
    }

    pub fn update_colormap(&mut self, device: &wgpu::Device, name: &str, is_mono: bool) {
        if self.last_colormap == name && self.last_is_mono == is_mono {
            return;
        }
        match compile_fragment_module(device, name, is_mono) {
            Ok(module) => {
                self.image_pipeline = create_image_pipeline(device, &self.pipeline_layout, self.target_format, &module);
                self.export_pipeline =
                    create_image_pipeline(device, &self.pipeline_layout, wgpu::TextureFormat::Rgba8Unorm, &module);
                self.last_colormap = name.to_owned();
                self.last_is_mono = is_mono;
                self.last_error = None;
            }
            Err(error) => {
                let message = error.to_string();
                eprintln!("{message}");
                self.last_error = Some(message);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn write_params(
        &self,
        queue: &wgpu::Queue,
        slot: u32,
        viewport_size: Vec2,
        image_size: Vec2,
        channel_index: i32,
        min_max: &MinMaxTotal,
        scale: f32,
        position: Vec2,
        shader: &ShaderParams,
        overlay: &MinMaxOverlay,
        background_a: Color32,
        background_b: Color32,
    ) {
        debug_assert!((slot as u64) < PARAM_SLOT_COUNT);
        let params = GpuParams::image(
            viewport_size,
            image_size,
            channel_index,
            min_max,
            scale,
            position,
            shader,
            overlay,
            background_a,
            background_b,
        );
        queue.write_buffer(
            &self.uniform_buffer,
            self.uniform_stride * slot as u64,
            bytemuck::bytes_of(&params),
        );
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    fn image(&self, slot: ImageSlot) -> Option<&GpuImage> {
        match slot {
            ImageSlot::Primary => self.primary.as_ref(),
            ImageSlot::Secondary => self.secondary.as_ref(),
        }
    }

    fn paint(&self, pass: &mut wgpu::RenderPass<'static>, panes: &[PaneDraw]) {
        for pane in panes {
            let width = pane.viewport_px.width().round().max(0.0);
            let height = pane.viewport_px.height().round().max(0.0);
            if width < 1.0 || height < 1.0 {
                continue;
            }
            let Some(image) = self.image(pane.slot) else {
                continue;
            };
            pass.set_viewport(
                pane.viewport_px.min.x.round(),
                pane.viewport_px.min.y.round(),
                width,
                height,
                0.0,
                1.0,
            );
            let offset = (self.uniform_stride * pane.uniform_slot as u64) as u32;
            pass.set_bind_group(0, &image.bind_group, &[offset]);
            pass.set_pipeline(&self.background_pipeline);
            pass.draw(0..4, 0..1);
            pass.set_pipeline(&self.image_pipeline);
            pass.draw(0..4, 0..1);
        }
    }

    fn encode_export(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        request: &ExportRequest,
    ) -> Result<ExportReadback> {
        let image = self
            .image(request.slot)
            .ok_or_else(|| eyre!("The requested image is not on the GPU"))?;
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("edolview export target"),
            size: wgpu::Extent3d {
                width: request.width,
                height: request.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("edolview export render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            let offset = (self.uniform_stride * 2) as u32;
            pass.set_pipeline(&self.export_pipeline);
            pass.set_bind_group(0, &image.bind_group, &[offset]);
            pass.draw(0..4, 0..1);
        }

        let row_bytes = request.width * 4;
        let padded_row_bytes = align_to(row_bytes as u64, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64) as u32;
        let buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("edolview export readback"),
            size: padded_row_bytes as u64 * request.height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        }));
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: Some(request.height),
                },
            },
            target.size(),
        );

        Ok(ExportReadback {
            buffer,
            row_bytes,
            padded_row_bytes,
            width: request.width,
            height: request.height,
            completion: Arc::clone(&request.completion),
        })
    }
}

fn align_to(value: u64, alignment: u64) -> u64 {
    value.div_ceil(alignment) * alignment
}

fn upload_image(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
    image: &impl Image,
) -> Result<GpuImage> {
    #[cfg(debug_assertions)]
    let _timer = crate::util::timer::ScopedTimer::new("Upload texture");

    let texture = image.gpu_texture()?;
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("edolview image bind group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: uniform_buffer,
                    offset: 0,
                    size: NonZeroU64::new(std::mem::size_of::<GpuParams>() as u64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
        ],
    });
    Ok(GpuImage {
        _texture: texture,
        bind_group,
        image_id: image.id(),
    })
}

fn compile_fragment_module(device: &wgpu::Device, colormap: &str, is_mono: bool) -> Result<wgpu::ShaderModule> {
    let source = build_fragment_source(colormap, is_mono)?;
    compile_fragment_source(device, &source)
}

fn compile_fragment_source(device: &wgpu::Device, source: &str) -> Result<wgpu::ShaderModule> {
    let mut frontend = naga::front::glsl::Frontend::default();
    let mut module = frontend
        .parse(&naga::front::glsl::Options::from(naga::ShaderStage::Fragment), source)
        .map_err(|errors| eyre!(errors.emit_to_string(source)))?;
    // GLSL leaves the default fragment sampling point unspecified, while WGSL
    // makes the matching vertex output explicitly center-sampled.
    for entry_point in &mut module.entry_points {
        for argument in &mut entry_point.function.arguments {
            if let Some(naga::Binding::Location { sampling, .. }) = &mut argument.binding {
                sampling.get_or_insert(naga::Sampling::Center);
            }
        }
    }
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .map_err(|error| eyre!(error.emit_to_string(source)))?;
    Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("edolview image fragment shader"),
        source: wgpu::ShaderSource::Naga(Cow::Owned(module)),
    }))
}

pub(super) fn mono_colormap_preview_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    name: &str,
) -> Result<wgpu::RenderPipeline> {
    // Use the same GLSL and color helpers as the image, with a fixed 0..1 input.
    // Exposure, gamma and normalization settings must not change the reference bar.
    let source = build_fragment_source(name, true)?;
    let (helpers, _) = source
        .split_once("void main()")
        .ok_or_else(|| eyre!("Missing image shader entry point"))?;
    let source = format!("{helpers}\nvoid main() {{ frag_color = vec4(colormap(v_tex_coord.x), 1.0); }}");
    let fragment = compile_fragment_source(device, &source)?;
    let vertex = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("edolview colormap preview vertex shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
            r#"
struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};
@vertex fn main(@builtin(vertex_index) index: u32) -> VertexOut {
    let positions = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    var out: VertexOut;
    out.position = vec4(positions[index], 0.0, 1.0);
    out.uv = (positions[index] + vec2(1.0)) * 0.5;
    return out;
}
"#,
        )),
    });
    Ok(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("edolview mono colormap preview pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &vertex,
            entry_point: Some("main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment,
            entry_point: Some("main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    }))
}

fn build_fragment_source(colormap: &str, is_mono: bool) -> Result<String> {
    let relative_path: PathBuf = if is_mono {
        format!("colormap/mono/{colormap}.glsl")
    } else {
        format!("colormap/rgb/{colormap}.glsl")
    }
    .into();
    let installed_path = exe_dir_or_cwd().join(&relative_path);
    let path = if installed_path.is_file() {
        installed_path
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(relative_path)
    };
    let colormap_code = std::fs::read_to_string(&path)
        .map_err(|error| eyre!("Failed to read colormap file '{}': {error}", path.display()))?;
    let color_process = if is_mono {
        "float v = color_proc(tex.r); vec3 cm = colormap(v);"
    } else {
        "vec3 v; v.r = color_proc(tex.r); v.g = color_proc(tex.g); v.b = color_proc(tex.b); vec3 cm = colormap(v);"
    };
    let mut base = IMAGE_SHADER_CODE
        .lines()
        .filter(|line| !line.trim_start().starts_with("uniform "))
        .collect::<Vec<_>>()
        .join("\n");
    base = base
        .replace("#version 330 core", "#version 450 core")
        .replace("out vec4 frag_color;", "layout(location = 0) out vec4 frag_color;")
        .replace("in vec2 v_tex_coord;", "layout(location = 0) in vec2 v_tex_coord;")
        .replace("texture2D(u_texture, v_tex_coord)", "sample_image(v_tex_coord)")
        .replace("%colormap_function%", &colormap_code)
        .replace("%color_process%", color_process);

    let declarations = r#"
layout(set = 0, binding = 0, std140) uniform Params {
    vec4 viewport_image;
    vec4 transform;
    vec4 color;
    vec4 global_scale;
    vec4 min_values;
    vec4 max_values;
    vec4 scale_modes;
    vec4 overlay_flags;
    vec4 overlay_show_min;
    vec4 overlay_show_max;
    vec4 overlay_scope;
    vec4 overlay_min_values;
    vec4 overlay_max_values;
    vec4 background_color_a;
    vec4 background_color_b;
    vec4 background;
} p;
layout(set = 0, binding = 1) uniform texture2D u_texture;

#define u_image_size p.viewport_image.zw
#define u_channel_index int(p.transform.w)
#define u_use_alpha int(p.color.x)
#define u_exposure p.color.y
#define u_offset p.color.z
#define u_gamma p.color.w
#define u_min_v p.global_scale.x
#define u_max_v p.global_scale.y
#define u_scale_mode int(p.global_scale.z)
#define u_use_per_channel int(p.global_scale.w)
#define u_min_v0 p.min_values.x
#define u_min_v1 p.min_values.y
#define u_min_v2 p.min_values.z
#define u_min_v3 p.min_values.w
#define u_max_v0 p.max_values.x
#define u_max_v1 p.max_values.y
#define u_max_v2 p.max_values.z
#define u_max_v3 p.max_values.w
#define u_scale_mode0 int(p.scale_modes.x)
#define u_scale_mode1 int(p.scale_modes.y)
#define u_scale_mode2 int(p.scale_modes.z)
#define u_scale_mode3 int(p.scale_modes.w)
#define u_min_max_overlay_enabled int(p.overlay_flags.x)
#define u_min_max_channel_count int(p.overlay_flags.y)
#define u_min_max_value_scale p.overlay_flags.z
#define u_min_max_compare_epsilon p.overlay_flags.w
#define u_min_max_show_min_channels ivec4(p.overlay_show_min)
#define u_min_max_show_max_channels ivec4(p.overlay_show_max)
#define u_min_max_scope ivec4(p.overlay_scope)
#define u_min_max_min_values p.overlay_min_values
#define u_min_max_max_values p.overlay_max_values

vec4 load_clamped(ivec2 pixel) {
    ivec2 extent = textureSize(u_texture, 0);
    return texelFetch(u_texture, clamp(pixel, ivec2(0), extent - ivec2(1)), 0);
}

vec4 sample_image(vec2 uv) {
    ivec2 extent = textureSize(u_texture, 0);
    if (p.transform.x >= 1.0) {
        return load_clamped(ivec2(floor(uv * vec2(extent))));
    }
    vec2 texel = uv * vec2(extent) - vec2(0.5);
    ivec2 lo = ivec2(floor(texel));
    vec2 f = fract(texel);
    vec4 top = mix(load_clamped(lo), load_clamped(lo + ivec2(1, 0)), f.x);
    vec4 bottom = mix(load_clamped(lo + ivec2(0, 1)), load_clamped(lo + ivec2(1, 1)), f.x);
    return mix(top, bottom, f.y);
}
"#;
    Ok(base.replacen("#version 450 core", &format!("#version 450 core\n{declarations}"), 1))
}

fn create_image_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    fragment: &wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
    let vertex = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("edolview image vertex shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Owned(format!("{PARAMS_WGSL}{COMMON_VERTEX_SHADER}"))),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("edolview image pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &vertex,
            entry_point: Some("vs_image"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: fragment,
            entry_point: Some("main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn create_background_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("edolview checker shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Owned(format!("{PARAMS_WGSL}{BACKGROUND_SHADER}"))),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("edolview checker pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_background"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_background"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    })
}

const PARAMS_WGSL: &str = r#"
struct Params {
    viewport_image: vec4<f32>, transform: vec4<f32>, color: vec4<f32>, global_scale: vec4<f32>,
    min_values: vec4<f32>, max_values: vec4<f32>, scale_modes: vec4<f32>, overlay_flags: vec4<f32>,
    overlay_show_min: vec4<f32>, overlay_show_max: vec4<f32>, overlay_scope: vec4<f32>,
    overlay_min_values: vec4<f32>, overlay_max_values: vec4<f32>,
    background_color_a: vec4<f32>, background_color_b: vec4<f32>, background: vec4<f32>,
};
@group(0) @binding(0) var<uniform> p: Params;
"#;

const RGB_UPLOAD_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read> rgb: array<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba32float, write>;
struct UploadParams { base_y: u32, width: u32, row_count: u32, _padding: u32 };
@group(0) @binding(2) var<uniform> params: UploadParams;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.width || id.y >= params.row_count) {
        return;
    }
    let index = (id.y * params.width + id.x) * 3u;
    textureStore(
        output_texture,
        vec2(id.x, id.y + params.base_y),
        vec4(rgb[index], rgb[index + 1u], rgb[index + 2u], 1.0),
    );
}
"#;

const COMMON_VERTEX_SHADER: &str = r#"
struct VertexOut { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs_image(@builtin(vertex_index) index: u32) -> VertexOut {
    let positions = array<vec2<f32>, 4>(vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 1.0));
    let a_pos = positions[index];
    var pos = (a_pos * p.viewport_image.zw * 2.0 * p.transform.x) / p.viewport_image.xy;
    pos.x = pos.x + p.transform.y / p.viewport_image.x * 2.0 - 1.0;
    pos.y = -(pos.y + p.transform.z / p.viewport_image.y * 2.0 - 1.0);
    var out: VertexOut;
    out.position = vec4(pos, 0.0, 1.0);
    out.uv = a_pos;
    return out;
}
"#;

const BACKGROUND_SHADER: &str = r#"
struct BackgroundOut { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs_background(@builtin(vertex_index) index: u32) -> BackgroundOut {
    let positions = array<vec2<f32>, 4>(vec2(-1.0, 1.0), vec2(1.0, 1.0), vec2(-1.0, -1.0), vec2(1.0, -1.0));
    var out: BackgroundOut;
    out.position = vec4(positions[index], 0.0, 1.0);
    out.uv = (positions[index] + vec2(1.0)) * 0.5;
    out.uv.y = 1.0 - out.uv.y;
    return out;
}
@fragment fn fs_background(in: BackgroundOut) -> @location(0) vec4<f32> {
    let g = floor((in.uv * p.viewport_image.xy - p.transform.yz) / p.background.x);
    let parity = (i32(g.x + g.y) & 1) != 0;
    return select(p.background_color_a, p.background_color_b, parity);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mono_colormap_previews_render_reference_bars() {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).expect("GPU adapter");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).expect("GPU device");
        let mut names: Vec<_> = std::fs::read_dir("colormap/mono")
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("glsl"))
            .map(|path| path.file_stem().unwrap().to_str().unwrap().to_owned())
            .collect();
        names.sort();
        let pipelines: Vec<_> = names
            .iter()
            .map(|name| mono_colormap_preview_pipeline(&device, wgpu::TextureFormat::Rgba8Unorm, name).unwrap())
            .collect();
        let extent = wgpu::Extent3d {
            width: 256,
            height: names.len() as u32,
            depth_or_array_layers: 1,
        };
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("colormap preview test"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("colormap preview test readback"),
            size: 1024 * names.len() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            for (index, pipeline) in pipelines.iter().enumerate() {
                pass.set_viewport(0.0, index as f32, 256.0, 1.0, 0.0, 1.0);
                pass.set_pipeline(pipeline);
                pass.draw(0..3, 0..1);
            }
        }
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(1024),
                    rows_per_image: None,
                },
            },
            extent,
        );
        queue.submit([encoder.finish()]);
        let slice = readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        rx.recv().unwrap().unwrap();
        let pixels = slice.get_mapped_range().unwrap();
        for (name, row) in names.iter().zip(pixels.chunks_exact(1024)) {
            assert!(row.chunks_exact(4).all(|pixel| pixel[3] == 255), "{name}: opaque preview");
            // Cyclic maps such as twilight can have identical endpoints.
            assert!(
                row.chunks_exact(4).any(|pixel| pixel[..3] != row[..3]),
                "{name}: varying reference colors"
            );
            if name == "gray" || name == "invalid_viz" {
                for (index, pixel) in row.chunks_exact(4).enumerate() {
                    assert_eq!(pixel, &[index as u8, index as u8, index as u8, 255]);
                }
            }
        }
        drop(pixels);
        readback.unmap();
    }

    #[test]
    fn all_bundled_colormaps_parse_as_wgpu_glsl() {
        for (directory, is_mono) in [("colormap/mono", true), ("colormap/rgb", false)] {
            let entries = std::fs::read_dir(directory).unwrap();
            for entry in entries {
                let path = entry.unwrap().path();
                if path.extension().and_then(|extension| extension.to_str()) != Some("glsl") {
                    continue;
                }
                let name = path.file_stem().unwrap().to_str().unwrap();
                let source = build_fragment_source(name, is_mono).unwrap();
                naga::front::glsl::Frontend::default()
                    .parse(&naga::front::glsl::Options::from(naga::ShaderStage::Fragment), &source)
                    .unwrap_or_else(|errors| panic!("{}: {}", path.display(), errors.emit_to_string(&source)));
            }
        }
    }

    #[test]
    fn large_rgb_uploads_are_split_to_fit_storage_binding_limits() {
        let rows = crate::model::rgb_rows_per_chunk(4096, 128 * 1024 * 1024);
        assert_eq!(rows, 2730);
        assert_eq!(4096_u32.div_ceil(rows), 2);
    }
}
