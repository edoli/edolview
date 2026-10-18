use std::sync::{mpsc, Arc, LazyLock, RwLock};

use bytemuck::{Pod, Zeroable};
use color_eyre::eyre::{eyre, Result};
use wgpu::util::DeviceExt as _;

use super::{image_io::DecodedImage, ComparisonMode, ImageSpec, MeanDim, PixelType, Recti};
use crate::util::math_ext::Vec2i;

const WORKGROUP_SIZE: u32 = 256;
const REDUCTION_GROUPS: u32 = 1024;

static GPU_COMPUTE: LazyLock<RwLock<Option<Arc<GpuComputeContext>>>> = LazyLock::new(|| RwLock::new(None));

pub fn install_gpu_compute(device: &wgpu::Device, queue: &wgpu::Queue, backend: wgpu::Backend) {
    let mut slot = GPU_COMPUTE.write().unwrap();
    if slot.is_none() {
        *slot = Some(Arc::new(GpuComputeContext::new(device.clone(), queue.clone(), backend)));
    }
}

pub fn gpu_compute() -> Result<Arc<GpuComputeContext>> {
    GPU_COMPUTE
        .read()
        .unwrap()
        .clone()
        .ok_or_else(|| eyre!("GPU compute is not initialized yet"))
}

#[derive(Clone)]
pub struct GpuImageTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub spec: ImageSpec,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ComputeParams {
    roi: [u32; 4],
    image: [u32; 4],
    operation: [u32; 4],
    values: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct NativeUploadParams {
    image: [u32; 4],
    memory: [u32; 4],
    format: [u32; 4],
    transform: [u32; 4],
}

impl ComputeParams {
    fn new(spec: &ImageSpec, rect: Recti) -> Self {
        let rect = normalized_rect(spec, rect);
        let (x, y, width, height) = rect.xywh();
        Self {
            roi: [x as u32, y as u32, width as u32, height as u32],
            image: [spec.width as u32, spec.height as u32, spec.channels as u32, 0],
            operation: [0; 4],
            values: [0.0; 4],
        }
    }
}

pub struct GpuComputeContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    layout: wgpu::BindGroupLayout,
    compute_module: wgpu::ShaderModule,
    compute_pipeline_layout: wgpu::PipelineLayout,
    minmax_pipeline: std::sync::OnceLock<wgpu::ComputePipeline>,
    histogram_pipeline: std::sync::OnceLock<wgpu::ComputePipeline>,
    sum_pipeline: std::sync::OnceLock<wgpu::ComputePipeline>,
    vector_mean_pipeline: std::sync::OnceLock<wgpu::ComputePipeline>,
    compare_pipeline: std::sync::OnceLock<wgpu::ComputePipeline>,
    ssim_pipeline: std::sync::OnceLock<wgpu::ComputePipeline>,
    rgb_upload_layout: wgpu::BindGroupLayout,
    rgb_upload_pipeline: wgpu::ComputePipeline,
    native_upload_layout: wgpu::BindGroupLayout,
    native_upload_pipeline: wgpu::ComputePipeline,
    native_u8_upload_layout: wgpu::BindGroupLayout,
    native_u8_upload_pipeline: wgpu::ComputePipeline,
    stream_rgba_uploads: bool,
    large_rgba_upload_chunk_bytes: u64,
    rgb_upload_max_binding_size: u64,
    dummy_source: GpuImageTexture,
    dummy_target: GpuImageTexture,
}

impl GpuComputeContext {
    fn new(device: wgpu::Device, queue: wgpu::Queue, backend: wgpu::Backend) -> Self {
        const DEFAULT_RGBA_UPLOAD_CHUNK_BYTES: u64 = 16 * 1024 * 1024;
        const VULKAN_LARGE_RGBA_UPLOAD_CHUNK_BYTES: u64 = 40 * 1024 * 1024;
        const VULKAN_RGB_UPLOAD_CHUNK_BYTES: u64 = 8 * 1024 * 1024;
        let stream_rgba_uploads = matches!(backend, wgpu::Backend::Dx12 | wgpu::Backend::Vulkan);
        let large_rgba_upload_chunk_bytes = if backend == wgpu::Backend::Vulkan {
            VULKAN_LARGE_RGBA_UPLOAD_CHUNK_BYTES
        } else {
            DEFAULT_RGBA_UPLOAD_CHUNK_BYTES
        };
        let rgb_upload_max_binding_size = if backend == wgpu::Backend::Vulkan {
            VULKAN_RGB_UPLOAD_CHUNK_BYTES
        } else {
            device.limits().max_storage_buffer_binding_size
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("edolview image compute layout"),
            entries: &[
                texture_layout_entry(0),
                texture_layout_entry(1),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let compute_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("edolview image compute shader"),
            source: wgpu::ShaderSource::Wgsl(COMPUTE_SHADER.into()),
        });
        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("edolview image compute pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let rgb_upload_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("edolview RGB upload layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let rgb_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("edolview RGB upload shader"),
            source: wgpu::ShaderSource::Wgsl(RGB_UPLOAD_SHADER.into()),
        });
        let rgb_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("edolview RGB upload pipeline layout"),
            bind_group_layouts: &[Some(&rgb_upload_layout)],
            immediate_size: 0,
        });
        let rgb_upload_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("edolview RGB upload pipeline"),
            layout: Some(&rgb_pipeline_layout),
            module: &rgb_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let native_upload_layout = create_native_upload_layout(
            &device,
            wgpu::TextureFormat::Rgba32Float,
            "edolview native image upload layout",
        );
        let native_u8_upload_layout = create_native_upload_layout(
            &device,
            wgpu::TextureFormat::Rgba8Unorm,
            "edolview native U8 image upload layout",
        );
        let native_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("edolview native image upload shader"),
            source: wgpu::ShaderSource::Wgsl(NATIVE_UPLOAD_SHADER.into()),
        });
        let native_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("edolview native image upload pipeline layout"),
            bind_group_layouts: &[Some(&native_upload_layout)],
            immediate_size: 0,
        });
        let native_upload_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("edolview native image upload pipeline"),
            layout: Some(&native_pipeline_layout),
            module: &native_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let native_u8_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("edolview native U8 image upload shader"),
            source: wgpu::ShaderSource::Wgsl(NATIVE_UPLOAD_SHADER.replace("rgba32float", "rgba8unorm").into()),
        });
        let native_u8_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("edolview native U8 image upload pipeline layout"),
            bind_group_layouts: &[Some(&native_u8_upload_layout)],
            immediate_size: 0,
        });
        let native_u8_upload_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("edolview native U8 image upload pipeline"),
            layout: Some(&native_u8_pipeline_layout),
            module: &native_u8_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let dummy_source = create_empty_rgba_texture(&device, 1, 1, 4, "edolview compute dummy source");
        let dummy_target = create_empty_rgba_texture(&device, 1, 1, 4, "edolview compute dummy target");
        Self {
            device,
            queue,
            layout,
            compute_module,
            compute_pipeline_layout,
            minmax_pipeline: std::sync::OnceLock::new(),
            histogram_pipeline: std::sync::OnceLock::new(),
            sum_pipeline: std::sync::OnceLock::new(),
            vector_mean_pipeline: std::sync::OnceLock::new(),
            compare_pipeline: std::sync::OnceLock::new(),
            ssim_pipeline: std::sync::OnceLock::new(),
            rgb_upload_layout,
            rgb_upload_pipeline,
            native_upload_layout,
            native_upload_pipeline,
            native_u8_upload_layout,
            native_u8_upload_pipeline,
            stream_rgba_uploads,
            large_rgba_upload_chunk_bytes,
            rgb_upload_max_binding_size,
            dummy_source,
            dummy_target,
        }
    }

    pub(crate) fn upload_decoded(&self, image: &DecodedImage) -> Result<Arc<GpuImageTexture>> {
        if let Some(pixels) = image.f32_pixels() {
            return self.upload(
                &ImageSpec::new(image.width, image.height, image.channels, image.pixel_type),
                pixels,
            );
        }
        if image.is_canonical_direct()
            && matches!(image.pixels, crate::model::image_io::DecodedPixels::U8(_))
            && matches!(image.channels, 1 | 2 | 4)
        {
            return self.upload_u8_texture(image);
        }
        if matches!(image.pixels, crate::model::image_io::DecodedPixels::Raw(_))
            && !image.transform.flip_y
            && !image.transform.swap_bytes
            && image.transform.scale == 1.0
            && image.layout.planes == 1
            && image.layout.input_channels == image.channels as usize
            && image.layout.row_stride_bytes == image.width as usize * image.channels as usize * 4
            && matches!(image.channels, 1 | 2 | 4)
        {
            return self.upload_raw_f32_texture(image);
        }
        self.upload_native_fused(image)
    }

    fn upload_raw_f32_texture(&self, image: &DecodedImage) -> Result<Arc<GpuImageTexture>> {
        let format = texture_format(image.channels)?;
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("edolview raw F32 image"),
            size: wgpu::Extent3d {
                width: image.width as u32,
                height: image.height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let byte_count = image.layout.row_stride_bytes * image.height as usize;
        let start = image.transform.data_offset_bytes;
        let bytes = image
            .pixels
            .bytes()
            .get(start..start + byte_count)
            .ok_or_else(|| eyre!("Raw F32 image payload is truncated"))?;
        self.queue.write_texture(
            texture.as_image_copy(),
            bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.layout.row_stride_bytes as u32),
                rows_per_image: Some(image.height as u32),
            },
            texture.size(),
        );
        Ok(Arc::new(GpuImageTexture {
            view: texture.create_view(&Default::default()),
            texture,
            spec: ImageSpec::new(image.width, image.height, image.channels, image.pixel_type),
        }))
    }

    fn upload_u8_texture(&self, image: &DecodedImage) -> Result<Arc<GpuImageTexture>> {
        let format = match image.channels {
            1 => wgpu::TextureFormat::R8Unorm,
            2 => wgpu::TextureFormat::Rg8Unorm,
            4 => wgpu::TextureFormat::Rgba8Unorm,
            _ => unreachable!("the native U8 fast path validates channels"),
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("edolview native U8 image"),
            size: wgpu::Extent3d {
                width: image.width as u32,
                height: image.height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let bytes = image.pixels.bytes();
        let bytes_per_row = image.width as u32 * image.channels as u32;
        self.queue.write_texture(
            texture.as_image_copy(),
            bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(image.height as u32),
            },
            texture.size(),
        );
        Ok(Arc::new(GpuImageTexture {
            view: texture.create_view(&Default::default()),
            texture,
            spec: ImageSpec::new(image.width, image.height, image.channels, image.pixel_type),
        }))
    }

    fn upload_native_fused(&self, image: &DecodedImage) -> Result<Arc<GpuImageTexture>> {
        let pixel_kind = image
            .pixels
            .shader_kind()
            .ok_or_else(|| eyre!("This native pixel type cannot be normalized by WGSL"))?;
        let compact_u8_output = matches!(image.pixels, crate::model::image_io::DecodedPixels::U8(_))
            && matches!(image.color, crate::model::image_io::DecodedColor::Direct)
            && image.layout.bit_depth <= 8
            && image.layout.input_channels == image.channels as usize;
        let output_format = if compact_u8_output {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            wgpu::TextureFormat::Rgba32Float
        };
        let output = create_native_output_texture(
            &self.device,
            image.width as u32,
            image.height as u32,
            image.channels,
            output_format,
        );
        let palette_bytes = image.color.palette().map(bytemuck::cast_slice).unwrap_or(&[]);
        let palette = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("edolview native image palette"),
            contents: if palette_bytes.is_empty() {
                bytemuck::bytes_of(&0_u32)
            } else {
                palette_bytes
            },
            usage: wgpu::BufferUsages::STORAGE,
        });
        let source_bytes = image.pixels.bytes();
        let max_binding = self.device.limits().max_storage_buffer_binding_size as usize;
        if image.layout.planes > 5 {
            return Err(eyre!("Native planar images with more than five planes are not supported"));
        }
        if image.layout.row_stride_bytes > max_binding {
            return Err(eyre!(
                "A native image row exceeds the GPU storage binding limit: {} > {max_binding}",
                image.layout.row_stride_bytes
            ));
        }
        let rows_per_chunk = (max_binding / image.layout.row_stride_bytes).max(1).min(image.height as usize);
        let dummy_source = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("edolview native image unused plane"),
            contents: bytemuck::bytes_of(&0_u32),
            usage: wgpu::BufferUsages::STORAGE,
        });
        for base_y in (0..image.height as usize).step_by(rows_per_chunk) {
            let row_count = rows_per_chunk.min(image.height as usize - base_y);
            let plane_count = image.layout.planes.max(1) as usize;
            let mut sources = Vec::with_capacity(plane_count);
            for plane in 0..plane_count {
                let source_base_y = if image.transform.flip_y {
                    image.height as usize - base_y - row_count
                } else {
                    base_y
                };
                let start = if plane_count == 1 {
                    image.transform.data_offset_bytes + source_base_y * image.layout.row_stride_bytes
                } else {
                    image.transform.data_offset_bytes
                        + plane * image.layout.plane_stride_bytes
                        + source_base_y * image.layout.row_stride_bytes
                };
                let end = start + row_count * image.layout.row_stride_bytes;
                let bytes = source_bytes
                    .get(start..end)
                    .ok_or_else(|| eyre!("Native image plane or row is truncated"))?;
                sources.push(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("edolview native image source plane"),
                    contents: bytes,
                    usage: wgpu::BufferUsages::STORAGE,
                }));
            }
            let params = NativeUploadParams {
                image: [
                    image.width as u32,
                    image.height as u32,
                    image.layout.input_channels as u32,
                    image.channels as u32,
                ],
                memory: [
                    image.layout.row_stride_bytes as u32,
                    0,
                    image.layout.planes,
                    image.layout.bit_depth as u32,
                ],
                format: [pixel_kind, image.color.shader_kind(), base_y as u32, row_count as u32],
                transform: [
                    0,
                    u32::from(image.transform.flip_y) | (u32::from(image.transform.swap_bytes) << 1),
                    image.transform.scale.to_bits(),
                    0,
                ],
            };
            let params = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("edolview native image upload parameters"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });
            let upload_layout = if compact_u8_output {
                &self.native_u8_upload_layout
            } else {
                &self.native_upload_layout
            };
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("edolview native image upload bind group"),
                layout: upload_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: sources[0].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: sources.get(1).unwrap_or(&dummy_source).as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: sources.get(2).unwrap_or(&dummy_source).as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: sources.get(3).unwrap_or(&dummy_source).as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: sources.get(4).unwrap_or(&dummy_source).as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: palette.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(&output.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: params.as_entire_binding(),
                    },
                ],
            });
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("edolview native image upload encoder"),
            });
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("edolview fused native image conversion"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(if compact_u8_output {
                    &self.native_u8_upload_pipeline
                } else {
                    &self.native_upload_pipeline
                });
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups((image.width as u32).div_ceil(16), (row_count as u32).div_ceil(16), 1);
            }
            self.queue.submit([encoder.finish()]);
        }
        Ok(Arc::new(output))
    }

    pub fn upload(&self, spec: &ImageSpec, pixels: &[f32]) -> Result<Arc<GpuImageTexture>> {
        if spec.width <= 0 || spec.height <= 0 || !(1..=4).contains(&spec.channels) {
            return Err(eyre!("Invalid image dimensions or channels"));
        }
        let expected = spec.width as usize * spec.height as usize * spec.channels as usize;
        if pixels.len() != expected {
            return Err(eyre!("Unexpected image data length: {} != {expected}", pixels.len()));
        }
        let format = texture_format(spec.channels)?;
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("edolview shared image"),
            size: wgpu::Extent3d {
                width: spec.width as u32,
                height: spec.height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | if spec.channels == 3 {
                    wgpu::TextureUsages::STORAGE_BINDING
                } else {
                    wgpu::TextureUsages::empty()
                },
            view_formats: &[],
        });
        if spec.channels == 3 {
            self.upload_rgb(&texture, pixels)?;
        } else {
            self.upload_direct(&texture, spec, pixels)?;
        }
        Ok(Arc::new(GpuImageTexture {
            view: texture.create_view(&Default::default()),
            texture,
            spec: spec.clone(),
        }))
    }

    fn upload_direct(&self, texture: &wgpu::Texture, spec: &ImageSpec, pixels: &[f32]) -> Result<()> {
        const SMALL_RGBA_UPLOAD_CHUNK_BYTES: u64 = 16 * 1024 * 1024;
        const LARGE_UPLOAD_THRESHOLD_BYTES: u64 = 64 * 1024 * 1024;
        let bytes = bytemuck::cast_slice(pixels);
        let chunk_bytes = if bytes.len() as u64 <= LARGE_UPLOAD_THRESHOLD_BYTES {
            SMALL_RGBA_UPLOAD_CHUNK_BYTES
        } else {
            self.large_rgba_upload_chunk_bytes
        };
        let bytes_per_row = spec.width as u32 * spec.channels as u32 * 4;
        let should_stream = self.stream_rgba_uploads && spec.channels == 4 && bytes.len() as u64 > chunk_bytes;
        let rows_per_chunk = if should_stream {
            (chunk_bytes / u64::from(bytes_per_row)).max(1) as u32
        } else {
            spec.height as u32
        };
        for base_y in (0..spec.height as u32).step_by(rows_per_chunk as usize) {
            let row_count = rows_per_chunk.min(spec.height as u32 - base_y);
            let start = base_y as usize * bytes_per_row as usize;
            let end = (base_y + row_count) as usize * bytes_per_row as usize;
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: base_y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                &bytes[start..end],
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(row_count),
                },
                wgpu::Extent3d {
                    width: spec.width as u32,
                    height: row_count,
                    depth_or_array_layers: 1,
                },
            );
            if should_stream {
                self.queue.submit([]);
                self.device
                    .poll(wgpu::PollType::wait_indefinitely())
                    .map_err(|error| eyre!("Failed to flush an image upload chunk: {error}"))?;
            }
        }
        Ok(())
    }

    fn upload_rgb(&self, texture: &wgpu::Texture, pixels: &[f32]) -> Result<()> {
        let size = texture.size();
        let rows_per_chunk = rgb_rows_per_chunk(
            size.width,
            self.rgb_upload_max_binding_size
                .min(self.device.limits().max_storage_buffer_binding_size),
        )
        .min(size.height);
        let view = texture.create_view(&Default::default());
        for base_y in (0..size.height).step_by(rows_per_chunk as usize) {
            let row_count = rows_per_chunk.min(size.height - base_y);
            let start = base_y as usize * size.width as usize * 3;
            let end = (base_y + row_count) as usize * size.width as usize * 3;
            let input = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("edolview RGB upload buffer"),
                contents: bytemuck::cast_slice(&pixels[start..end]),
                usage: wgpu::BufferUsages::STORAGE,
            });
            let parameters = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("edolview RGB upload parameters"),
                contents: bytemuck::cast_slice(&[base_y, size.width, row_count, 0u32]),
                usage: wgpu::BufferUsages::UNIFORM,
            });
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("edolview RGB upload bind group"),
                layout: &self.rgb_upload_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: input.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: parameters.as_entire_binding(),
                    },
                ],
            });
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("edolview RGB upload encoder"),
            });
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("edolview RGB upload pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&self.rgb_upload_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(size.width.div_ceil(16), row_count.div_ceil(16), 1);
            }
            self.queue.submit([encoder.finish()]);
        }
        Ok(())
    }

    pub fn compare(
        &self,
        lhs: &GpuImageTexture,
        rhs: &GpuImageTexture,
        output_spec: &ImageSpec,
        mode: ComparisonMode,
        alpha: f32,
        strategy: u32,
    ) -> Result<Arc<GpuImageTexture>> {
        let output = create_empty_rgba_texture(
            &self.device,
            output_spec.width as u32,
            output_spec.height as u32,
            output_spec.channels,
            "edolview comparison image",
        );
        let mut params = ComputeParams::new(output_spec, Recti::ZERO);
        params.operation = [
            match mode {
                ComparisonMode::Diff => 0,
                ComparisonMode::Blend => 1,
                ComparisonMode::Split => 2,
            },
            lhs.spec.channels as u32,
            rhs.spec.channels as u32,
            strategy,
        ];
        params.values[0] = alpha.clamp(0.0, 1.0);
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("edolview comparison dummy output"),
            size: 4,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let (params_buffer, bind_group) = self.bind(lhs, Some(rhs), &output_buffer, &params, Some(&output));
        self.dispatch_2d(
            self.compute_pipeline(&self.compare_pipeline, "compare_images"),
            &bind_group,
            (output_spec.width as u32).div_ceil(16),
            (output_spec.height as u32).div_ceil(16),
        );
        drop(params_buffer);
        Ok(Arc::new(output))
    }

    pub fn minmax(&self, image: &GpuImageTexture, rect: Recti) -> Result<(Vec<f32>, Vec<f32>)> {
        let channels = image.spec.channels as usize;
        let init = [u32::MAX, u32::MAX, u32::MAX, u32::MAX, 0, 0, 0, 0];
        let output = self.storage_buffer(bytemuck::cast_slice(&init), "edolview minmax output");
        let params = ComputeParams::new(&image.spec, rect);
        let groups = reduction_group_count(&params);
        self.run_readback(
            self.compute_pipeline(&self.minmax_pipeline, "minmax"),
            image,
            None,
            &output,
            &params,
            groups,
            1,
            32,
            |bytes| {
                let words: &[u32] = bytemuck::cast_slice(bytes);
                let mut mins = Vec::with_capacity(channels);
                let mut maxs = Vec::with_capacity(channels);
                for channel in 0..channels {
                    mins.push(ordered_to_float(words[channel]));
                    maxs.push(ordered_to_float(words[4 + channel]));
                }
                (mins, maxs)
            },
        )
    }

    pub fn histogram(&self, image: &GpuImageTexture) -> Result<Vec<Vec<f32>>> {
        let size = 4 * 256 * std::mem::size_of::<u32>();
        let zeros = vec![0u8; size];
        let output = self.storage_buffer(&zeros, "edolview histogram output");
        let params = ComputeParams::new(&image.spec, Recti::ZERO);
        let groups = reduction_group_count(&params);
        self.run_readback(
            self.compute_pipeline(&self.histogram_pipeline, "histogram"),
            image,
            None,
            &output,
            &params,
            groups,
            1,
            size as u64,
            |bytes| {
                let words: &[u32] = bytemuck::cast_slice(bytes);
                (0..image.spec.channels as usize)
                    .map(|channel| words[channel * 256..(channel + 1) * 256].iter().map(|&v| v as f32).collect())
                    .collect()
            },
        )
    }

    pub fn mean(&self, image: &GpuImageTexture, rect: Recti, dim: MeanDim) -> Result<Vec<f64>> {
        let rect = normalized_rect(&image.spec, rect);
        if rect.empty() {
            return Ok(Vec::new());
        }
        match dim {
            MeanDim::All => {
                let mut params = ComputeParams::new(&image.spec, rect);
                params.operation[0] = 0;
                let groups = reduction_group_count(&params);
                let size = groups as u64 * 16;
                let output = self.empty_storage_buffer(size, "edolview mean partials");
                self.run_readback(
                    self.compute_pipeline(&self.sum_pipeline, "sum_values"),
                    image,
                    None,
                    &output,
                    &params,
                    groups,
                    1,
                    size,
                    |bytes| {
                        let partials: &[f32] = bytemuck::cast_slice(bytes);
                        let count = (params.roi[2] as f64) * (params.roi[3] as f64);
                        (0..image.spec.channels as usize)
                            .map(|channel| {
                                partials.chunks_exact(4).map(|partial| partial[channel] as f64).sum::<f64>() / count
                            })
                            .collect()
                    },
                )
            }
            MeanDim::Column | MeanDim::Row => {
                let mut params = ComputeParams::new(&image.spec, rect);
                params.operation[0] = u32::from(matches!(dim, MeanDim::Row));
                let count = if matches!(dim, MeanDim::Column) {
                    params.roi[2]
                } else {
                    params.roi[3]
                };
                let size = count as u64 * 16;
                let output = self.empty_storage_buffer(size, "edolview vector mean output");
                self.run_readback(
                    self.compute_pipeline(&self.vector_mean_pipeline, "vector_mean"),
                    image,
                    None,
                    &output,
                    &params,
                    count,
                    1,
                    size,
                    |bytes| {
                        let values: &[f32] = bytemuck::cast_slice(bytes);
                        values
                            .chunks_exact(4)
                            .flat_map(|pixel| pixel[..image.spec.channels as usize].iter().map(|&v| v as f64))
                            .collect()
                    },
                )
            }
        }
    }

    pub fn psnr(
        &self,
        lhs: &GpuImageTexture,
        rhs: &GpuImageTexture,
        rect: Recti,
        data_range: f64,
        scale: f64,
    ) -> Result<(f64, f64)> {
        ensure_compatible(lhs, rhs)?;
        let mut params = ComputeParams::new(&lhs.spec, rect);
        params.operation[0] = 1;
        let groups = reduction_group_count(&params);
        let size = groups as u64 * 16;
        let output = self.empty_storage_buffer(size, "edolview squared error partials");
        self.run_readback(
            self.compute_pipeline(&self.sum_pipeline, "sum_values"),
            lhs,
            Some(rhs),
            &output,
            &params,
            groups,
            1,
            size,
            |bytes| {
                let values: &[f32] = bytemuck::cast_slice(bytes);
                let squared_error: f64 = values.iter().map(|&v| v as f64).sum();
                let sample_count = params.roi[2] as f64 * params.roi[3] as f64 * lhs.spec.channels as f64;
                let mse = squared_error / sample_count;
                let psnr = if mse == 0.0 {
                    f64::INFINITY
                } else {
                    10.0 * ((data_range * data_range) / mse).log10()
                };
                let rmse = scale * mse.sqrt();
                (psnr, rmse)
            },
        )
    }

    pub fn ssim(&self, lhs: &GpuImageTexture, rhs: &GpuImageTexture, rect: Recti) -> Result<f64> {
        ensure_compatible(lhs, rhs)?;
        let params = ComputeParams::new(&lhs.spec, rect);
        let groups = reduction_group_count(&params);
        let size = groups as u64 * 16;
        let output = self.empty_storage_buffer(size, "edolview ssim partials");
        self.run_readback(
            self.compute_pipeline(&self.ssim_pipeline, "ssim"),
            lhs,
            Some(rhs),
            &output,
            &params,
            groups,
            1,
            size,
            |bytes| {
                let values: &[f32] = bytemuck::cast_slice(bytes);
                let sum: f64 = values.iter().map(|&v| v as f64).sum();
                let count = params.roi[2] as f64 * params.roi[3] as f64 * lhs.spec.channels as f64;
                sum / count
            },
        )
    }

    pub fn wait_idle(&self) -> Result<()> {
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| eyre!("GPU wait failed: {error}"))?;
        Ok(())
    }

    fn compute_pipeline<'a>(
        &'a self,
        slot: &'a std::sync::OnceLock<wgpu::ComputePipeline>,
        entry_point: &'static str,
    ) -> &'a wgpu::ComputePipeline {
        slot.get_or_init(|| {
            self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&self.compute_pipeline_layout),
                module: &self.compute_module,
                entry_point: Some(entry_point),
                compilation_options: Default::default(),
                cache: None,
            })
        })
    }

    fn bind(
        &self,
        image: &GpuImageTexture,
        second: Option<&GpuImageTexture>,
        output: &wgpu::Buffer,
        params: &ComputeParams,
        target: Option<&GpuImageTexture>,
    ) -> (wgpu::Buffer, wgpu::BindGroup) {
        let params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("edolview compute parameters"),
            contents: bytemuck::bytes_of(params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("edolview image compute bind group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&image.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        second.map_or(&self.dummy_source.view, |texture| &texture.view),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: output.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(
                        target.map_or(&self.dummy_target.view, |texture| &texture.view),
                    ),
                },
            ],
        });
        (params_buffer, bind_group)
    }

    #[allow(clippy::too_many_arguments)]
    fn run_readback<T>(
        &self,
        pipeline: &wgpu::ComputePipeline,
        image: &GpuImageTexture,
        second: Option<&GpuImageTexture>,
        output: &wgpu::Buffer,
        params: &ComputeParams,
        groups_x: u32,
        groups_y: u32,
        output_size: u64,
        decode: impl FnOnce(&[u8]) -> T,
    ) -> Result<T> {
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("edolview compute readback"),
            size: output_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let (_params_buffer, bind_group) = self.bind(image, second, output, params, None);
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("edolview image compute encoder"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("edolview image compute pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(groups_x.max(1), groups_y.max(1), 1);
        }
        encoder.copy_buffer_to_buffer(output, 0, &readback, 0, output_size);
        self.queue.submit([encoder.finish()]);
        let slice = readback.slice(..);
        let (tx, rx) = mpsc::sync_channel(1);
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| eyre!("GPU compute poll failed: {error}"))?;
        rx.recv().map_err(|_| eyre!("GPU readback callback disconnected"))??;
        let mapped = slice
            .get_mapped_range()
            .map_err(|error| eyre!("GPU compute readback failed: {error}"))?;
        let result = decode(&mapped);
        drop(mapped);
        readback.unmap();
        Ok(result)
    }

    fn dispatch_2d(&self, pipeline: &wgpu::ComputePipeline, bind_group: &wgpu::BindGroup, x: u32, y: u32) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("edolview image generation encoder"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("edolview image generation pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.dispatch_workgroups(x.max(1), y.max(1), 1);
        }
        self.queue.submit([encoder.finish()]);
    }

    fn storage_buffer(&self, contents: &[u8], label: &'static str) -> wgpu::Buffer {
        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        })
    }

    fn empty_storage_buffer(&self, size: u64, label: &'static str) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    }
}

fn texture_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn create_native_upload_layout(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    label: &'static str,
) -> wgpu::BindGroupLayout {
    let mut entries = (0..=5)
        .map(|binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        })
        .collect::<Vec<_>>();
    entries.push(wgpu::BindGroupLayoutEntry {
        binding: 6,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format,
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    });
    entries.push(wgpu::BindGroupLayoutEntry {
        binding: 7,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    });
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &entries,
    })
}

fn create_native_output_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    channels: i32,
    format: wgpu::TextureFormat,
) -> GpuImageTexture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("edolview normalized native image"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });
    GpuImageTexture {
        view: texture.create_view(&Default::default()),
        texture,
        spec: ImageSpec::new(width as i32, height as i32, channels, PixelType::F32),
    }
}

fn create_empty_rgba_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    channels: i32,
    label: &'static str,
) -> GpuImageTexture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });
    GpuImageTexture {
        view: texture.create_view(&Default::default()),
        texture,
        spec: ImageSpec {
            width: width as i32,
            height: height as i32,
            channels,
            dtype: PixelType::F32,
        },
    }
}

fn texture_format(channels: i32) -> Result<wgpu::TextureFormat> {
    match channels {
        1 => Ok(wgpu::TextureFormat::R32Float),
        2 => Ok(wgpu::TextureFormat::Rg32Float),
        3 | 4 => Ok(wgpu::TextureFormat::Rgba32Float),
        _ => Err(eyre!("Unsupported channel count: {channels}")),
    }
}

fn normalized_rect(spec: &ImageSpec, rect: Recti) -> Recti {
    let bounds = Recti::from_min_size(Vec2i::new(0, 0), Vec2i::new(spec.width, spec.height));
    let rect = rect.validate();
    if rect.empty() {
        bounds
    } else {
        rect.intersect(bounds)
    }
}

fn reduction_group_count(params: &ComputeParams) -> u32 {
    let count = params.roi[2].saturating_mul(params.roi[3]);
    count.div_ceil(WORKGROUP_SIZE).clamp(1, REDUCTION_GROUPS)
}

pub(crate) fn rgb_rows_per_chunk(width: u32, max_binding_size: u64) -> u32 {
    let bytes_per_row = u64::from(width) * 3 * std::mem::size_of::<f32>() as u64;
    (max_binding_size / bytes_per_row).max(1).min(u64::from(u32::MAX)) as u32
}

fn ordered_to_float(value: u32) -> f32 {
    let bits = if value & 0x8000_0000 != 0 {
        value ^ 0x8000_0000
    } else {
        !value
    };
    f32::from_bits(bits)
}

fn ensure_compatible(lhs: &GpuImageTexture, rhs: &GpuImageTexture) -> Result<()> {
    if lhs.spec.width != rhs.spec.width || lhs.spec.height != rhs.spec.height || lhs.spec.channels != rhs.spec.channels
    {
        return Err(eyre!("Images must have matching dimensions and channels"));
    }
    Ok(())
}

const COMPUTE_SHADER: &str = r#"
struct Params {
    roi: vec4<u32>,
    image: vec4<u32>,
    operation: vec4<u32>,
    values: vec4<f32>,
}

@group(0) @binding(0) var source_a: texture_2d<f32>;
@group(0) @binding(1) var source_b: texture_2d<f32>;
@group(0) @binding(2) var<storage, read_write> output: array<atomic<u32>>;
@group(0) @binding(3) var<uniform> params: Params;
@group(0) @binding(4) var destination: texture_storage_2d<rgba32float, write>;

var<workgroup> scratch: array<vec4<f32>, 256>;

fn finite(value: f32) -> bool {
    return value == value && abs(value) <= 3.402823466e+38;
}

fn float_to_ordered(value: f32) -> u32 {
    let bits = bitcast<u32>(value);
    if ((bits & 0x80000000u) != 0u) {
        return ~bits;
    }
    return bits ^ 0x80000000u;
}

fn roi_pixel(index: u32) -> vec2<i32> {
    let x = index % params.roi.z;
    let y = index / params.roi.z;
    return vec2<i32>(i32(params.roi.x + x), i32(params.roi.y + y));
}

fn channel_mask(value: vec4<f32>, channels: u32) -> vec4<f32> {
    return value * vec4<f32>(
        select(0.0, 1.0, channels > 0u),
        select(0.0, 1.0, channels > 1u),
        select(0.0, 1.0, channels > 2u),
        select(0.0, 1.0, channels > 3u));
}

@compute @workgroup_size(256)
fn minmax(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(num_workgroups) workgroups: vec3<u32>) {
    let count = params.roi.z * params.roi.w;
    let stride = max(1u, workgroups.x * 256u);
    var index = gid.x;
    loop {
        if (index >= count) { break; }
        let value = textureLoad(source_a, roi_pixel(index), 0);
        for (var channel = 0u; channel < params.image.z; channel++) {
            let v = value[channel];
            if (finite(v)) {
                let ordered = float_to_ordered(v);
                atomicMin(&output[channel], ordered);
                atomicMax(&output[4u + channel], ordered);
            }
        }
        index += stride;
    }
}

@compute @workgroup_size(256)
fn histogram(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(num_workgroups) workgroups: vec3<u32>) {
    let count = params.roi.z * params.roi.w;
    let stride = max(1u, workgroups.x * 256u);
    var index = gid.x;
    loop {
        if (index >= count) { break; }
        let value = textureLoad(source_a, roi_pixel(index), 0);
        for (var channel = 0u; channel < params.image.z; channel++) {
            let v = value[channel];
            if (v >= 0.0 && v < 1.0) {
                let bin = min(255u, u32(floor(v * 256.0)));
                atomicAdd(&output[channel * 256u + bin], 1u);
            }
        }
        index += stride;
    }
}

@compute @workgroup_size(256)
fn sum_values(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(num_workgroups) workgroups: vec3<u32>) {
    let count = params.roi.z * params.roi.w;
    let stride = max(1u, workgroups.x * 256u);
    var sum = vec4<f32>(0.0);
    var index = gid.x;
    loop {
        if (index >= count) { break; }
        let a = channel_mask(textureLoad(source_a, roi_pixel(index), 0), params.image.z);
        if (params.operation.x == 0u) {
            sum += a;
        } else {
            let b = channel_mask(textureLoad(source_b, roi_pixel(index), 0), params.image.z);
            let d = a - b;
            sum += d * d;
        }
        index += stride;
    }
    scratch[lid.x] = sum;
    workgroupBarrier();
    var offset = 128u;
    loop {
        if (offset == 0u) { break; }
        if (lid.x < offset) { scratch[lid.x] += scratch[lid.x + offset]; }
        workgroupBarrier();
        offset /= 2u;
    }
    if (lid.x == 0u) {
        let base = wid.x * 4u;
        for (var channel = 0u; channel < 4u; channel++) {
            atomicStore(&output[base + channel], bitcast<u32>(scratch[0][channel]));
        }
    }
}

@compute @workgroup_size(256)
fn vector_mean(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wid: vec3<u32>) {
    let output_count = select(params.roi.z, params.roi.w, params.operation.x == 1u);
    if (wid.x >= output_count) { return; }
    var sum = vec4<f32>(0.0);
    if (params.operation.x == 0u) {
        var row = lid.x;
        loop {
            if (row >= params.roi.w) { break; }
            sum += textureLoad(source_a, vec2<i32>(i32(params.roi.x + wid.x), i32(params.roi.y + row)), 0);
            row += 256u;
        }
    } else {
        var column = lid.x;
        loop {
            if (column >= params.roi.z) { break; }
            sum += textureLoad(source_a, vec2<i32>(i32(params.roi.x + column), i32(params.roi.y + wid.x)), 0);
            column += 256u;
        }
    }
    scratch[lid.x] = sum;
    workgroupBarrier();
    var offset = 128u;
    loop {
        if (offset == 0u) { break; }
        if (lid.x < offset) { scratch[lid.x] += scratch[lid.x + offset]; }
        workgroupBarrier();
        offset /= 2u;
    }
    if (lid.x == 0u) {
        let divisor = select(f32(params.roi.w), f32(params.roi.z), params.operation.x == 1u);
        let value = scratch[0] / divisor;
        let base = wid.x * 4u;
        for (var channel = 0u; channel < 4u; channel++) {
            atomicStore(&output[base + channel], bitcast<u32>(value[channel]));
        }
    }
}

fn normalize_channels(a: vec4<f32>, b: vec4<f32>) -> array<vec4<f32>, 2> {
    var result: array<vec4<f32>, 2>;
    result[0] = a;
    result[1] = b;
    switch params.operation.w {
        case 1u: { result[0] = vec4<f32>(a.x); }
        case 2u: { result[1] = vec4<f32>(b.x); }
        default: {}
    }
    return result;
}

@compute @workgroup_size(16, 16)
fn compare_images(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= params.image.x || gid.y >= params.image.y) { return; }
    let coord = vec2<i32>(gid.xy);
    let pair = normalize_channels(textureLoad(source_a, coord, 0), textureLoad(source_b, coord, 0));
    var value = pair[0] - pair[1];
    if (params.operation.x == 1u) {
        value = mix(pair[0], pair[1], params.values.x);
    }
    // Comparison textures are stored as RGBA even for images without an alpha
    // channel. Keep that storage alpha opaque so an RGB diff remains visible.
    if (params.image.z < 4u) {
        value.w = 1.0;
    }
    textureStore(destination, coord, value);
}

fn reflect101(value: i32, length: i32) -> i32 {
    if (length <= 1) { return 0; }
    var result = value;
    loop {
        if (result >= 0 && result < length) { break; }
        if (result < 0) { result = -result; }
        if (result >= length) { result = 2 * length - result - 2; }
    }
    return result;
}

fn gaussian_weight(offset: i32) -> f32 {
    let x = f32(offset);
    return exp(-(x * x) / 4.5);
}

fn ssim_pixel(index: u32) -> vec4<f32> {
    let local_x = i32(index % params.roi.z);
    let local_y = i32(index / params.roi.z);
    let width = i32(params.roi.z);
    let height = i32(params.roi.w);
    var mu_a = vec4<f32>(0.0);
    var mu_b = vec4<f32>(0.0);
    var aa = vec4<f32>(0.0);
    var bb = vec4<f32>(0.0);
    var ab = vec4<f32>(0.0);
    var weight_sum = 0.0;
    for (var oy = -5; oy <= 5; oy++) {
        let sy = reflect101(local_y + oy, height) + i32(params.roi.y);
        let wy = gaussian_weight(oy);
        for (var ox = -5; ox <= 5; ox++) {
            let sx = reflect101(local_x + ox, width) + i32(params.roi.x);
            let weight = wy * gaussian_weight(ox);
            let a = textureLoad(source_a, vec2<i32>(sx, sy), 0);
            let b = textureLoad(source_b, vec2<i32>(sx, sy), 0);
            mu_a += a * weight;
            mu_b += b * weight;
            aa += a * a * weight;
            bb += b * b * weight;
            ab += a * b * weight;
            weight_sum += weight;
        }
    }
    mu_a /= weight_sum;
    mu_b /= weight_sum;
    let variance_a = aa / weight_sum - mu_a * mu_a;
    let variance_b = bb / weight_sum - mu_b * mu_b;
    let covariance = ab / weight_sum - mu_a * mu_b;
    let numerator = (2.0 * mu_a * mu_b + vec4<f32>(0.0001)) * (2.0 * covariance + vec4<f32>(0.0009));
    let denominator = (mu_a * mu_a + mu_b * mu_b + vec4<f32>(0.0001))
        * (variance_a + variance_b + vec4<f32>(0.0009));
    return channel_mask(numerator / denominator, params.image.z);
}

@compute @workgroup_size(256)
fn ssim(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(num_workgroups) workgroups: vec3<u32>) {
    let count = params.roi.z * params.roi.w;
    let stride = max(1u, workgroups.x * 256u);
    var sum = vec4<f32>(0.0);
    var index = gid.x;
    loop {
        if (index >= count) { break; }
        sum += ssim_pixel(index);
        index += stride;
    }
    scratch[lid.x] = sum;
    workgroupBarrier();
    var offset = 128u;
    loop {
        if (offset == 0u) { break; }
        if (lid.x < offset) { scratch[lid.x] += scratch[lid.x + offset]; }
        workgroupBarrier();
        offset /= 2u;
    }
    if (lid.x == 0u) {
        let base = wid.x * 4u;
        for (var channel = 0u; channel < 4u; channel++) {
            atomicStore(&output[base + channel], bitcast<u32>(scratch[0][channel]));
        }
    }
}
"#;

const NATIVE_UPLOAD_SHADER: &str = r#"
struct NativeUploadParams {
    image: vec4<u32>,
    memory: vec4<u32>,
    format: vec4<u32>,
    transform: vec4<u32>,
}

@group(0) @binding(0) var<storage, read> source0: array<u32>;
@group(0) @binding(1) var<storage, read> source1: array<u32>;
@group(0) @binding(2) var<storage, read> source2: array<u32>;
@group(0) @binding(3) var<storage, read> source3: array<u32>;
@group(0) @binding(4) var<storage, read> source4: array<u32>;
@group(0) @binding(5) var<storage, read> palette: array<u32>;
@group(0) @binding(6) var output_texture: texture_storage_2d<rgba32float, write>;
@group(0) @binding(7) var<uniform> params: NativeUploadParams;

fn load_word(plane: u32, index: u32) -> u32 {
    if plane == 1u { return source1[index]; }
    if plane == 2u { return source2[index]; }
    if plane == 3u { return source3[index]; }
    if plane == 4u { return source4[index]; }
    return source0[index];
}

fn load_byte(plane: u32, offset: u32) -> u32 {
    let word = load_word(plane, offset >> 2u);
    return (word >> ((offset & 3u) * 8u)) & 255u;
}

fn load_u16(plane: u32, offset: u32) -> u32 {
    return load_byte(plane, offset) | (load_byte(plane, offset + 1u) << 8u);
}

fn load_palette_u16(index: u32) -> u32 {
    let word = palette[index >> 1u];
    return (word >> ((index & 1u) * 16u)) & 65535u;
}

fn sample_bytes() -> u32 {
    let kind = params.format.x;
    if kind <= 1u { return 1u; }
    if kind <= 3u || kind == 5u { return 2u; }
    return 4u;
}

fn unsigned_maximum() -> f32 {
    let bits = params.memory.w;
    if bits >= 32u { return 4294967295.0; }
    return f32((1u << bits) - 1u);
}

fn signed_maximum() -> f32 {
    let bits = params.memory.w;
    if bits >= 32u { return 2147483647.0; }
    return f32((1u << (bits - 1u)) - 1u);
}

fn sample_offset(x: u32, y: u32, channel: u32) -> u32 {
    let bytes = sample_bytes();
    let source_y = select(y, params.format.w - y - 1u, (params.transform.y & 1u) != 0u);
    if params.memory.z > 1u {
        return source_y * params.memory.x + x * bytes;
    }
    return source_y * params.memory.x + (x * params.image.z + channel) * bytes;
}

fn packed_sample(x: u32, y: u32, channel: u32) -> f32 {
    let source_y = select(y, params.format.w - y - 1u, (params.transform.y & 1u) != 0u);
    var base = source_y * params.memory.x;
    var sample = x * params.image.z + channel;
    var plane = 0u;
    if params.memory.z > 1u {
        plane = channel;
        base = source_y * params.memory.x;
        sample = x;
    }
    let bit = sample * params.memory.w;
    let byte = load_byte(plane, base + bit / 8u);
    let shift = 8u - params.memory.w - bit % 8u;
    let mask = (1u << params.memory.w) - 1u;
    return f32((byte >> shift) & mask) / f32(mask);
}

fn normalized_sample(x: u32, y: u32, channel: u32) -> f32 {
    if params.memory.w < 8u {
        return packed_sample(x, y, channel);
    }
    let offset = sample_offset(x, y, channel);
    let plane = select(0u, channel, params.memory.z > 1u);
    let kind = params.format.x;
    if kind == 0u { return f32(load_byte(plane, offset)) / unsigned_maximum(); }
    if kind == 1u {
        let signed = bitcast<i32>(load_byte(plane, offset) << 24u) >> 24;
        return f32(signed) / signed_maximum();
    }
    if kind == 2u { return f32(load_u16(plane, offset)) / unsigned_maximum(); }
    if kind == 3u {
        let signed = bitcast<i32>(load_u16(plane, offset) << 16u) >> 16;
        return f32(signed) / signed_maximum();
    }
    if kind == 4u {
        let raw = load_byte(plane, offset) | (load_byte(plane, offset + 1u) << 8u) |
            (load_byte(plane, offset + 2u) << 16u) | (load_byte(plane, offset + 3u) << 24u);
        return f32(bitcast<i32>(raw)) / signed_maximum();
    }
    if kind == 5u {
        let word_offset = offset & ~3u;
        let raw = load_word(plane, word_offset >> 2u);
        let pair = unpack2x16float(raw);
        return select(pair.x, pair.y, (offset & 2u) != 0u);
    }
    let raw = load_byte(plane, offset) | (load_byte(plane, offset + 1u) << 8u) |
        (load_byte(plane, offset + 2u) << 16u) | (load_byte(plane, offset + 3u) << 24u);
    if kind == 6u {
        var bits = raw;
        if (params.transform.y & 2u) != 0u {
            bits = ((raw & 0x000000ffu) << 24u) | ((raw & 0x0000ff00u) << 8u) |
                ((raw & 0x00ff0000u) >> 8u) | ((raw & 0xff000000u) >> 24u);
        }
        return bitcast<f32>(bits) * bitcast<f32>(params.transform.z);
    }
    return f32(raw) / unsigned_maximum();
}

fn lab_to_rgb(l_in: f32, a_in: f32, b_in: f32) -> vec3<f32> {
    let l = l_in * 100.0;
    let a = a_in * 255.0 - 128.0;
    let b = b_in * 255.0 - 128.0;
    let fy = (l + 16.0) / 116.0;
    let f = vec3<f32>(fy + a / 500.0, fy, fy - b / 200.0);
    let cube = f * f * f;
    let xyz = vec3<f32>(0.95047, 1.0, 1.08883) * select((116.0 * f - 16.0) / 903.3, cube, cube > vec3<f32>(0.008856));
    let linear = vec3<f32>(
        3.240454 * xyz.x - 1.537139 * xyz.y - 0.498531 * xyz.z,
        -0.969266 * xyz.x + 1.876011 * xyz.y + 0.041556 * xyz.z,
        0.055643 * xyz.x - 0.204026 * xyz.y + 1.057225 * xyz.z
    );
    let positive = max(linear, vec3<f32>(0.0));
    return clamp(select(12.92 * positive, 1.055 * pow(positive, vec3<f32>(1.0 / 2.4)) - 0.055,
        positive > vec3<f32>(0.0031308)), vec3<f32>(0.0), vec3<f32>(1.0));
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= params.image.x || id.y >= params.format.w { return; }
    var input: array<f32, 5>;
    for (var channel = 0u; channel < params.image.z; channel++) {
        input[channel] = normalized_sample(id.x, id.y, channel);
    }
    var output = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let color = params.format.y;
    if color == 0u {
        output.x = input[0];
        if params.image.w > 1u { output.y = input[1]; }
        if params.image.w > 2u { output.z = input[2]; }
        if params.image.w > 3u { output.w = input[3]; }
    } else if color == 1u {
        let entries = 1u << params.memory.w;
        let index = u32(round(input[0] * f32(entries - 1u)));
        output = vec4<f32>(
            f32(load_palette_u16(index)) / 65535.0,
            f32(load_palette_u16(entries + index)) / 65535.0,
            f32(load_palette_u16(2u * entries + index)) / 65535.0,
            1.0
        );
    } else if color == 2u || color == 3u {
        let k = 1.0 - input[3];
        output = vec4<f32>((vec3<f32>(1.0) - vec3<f32>(input[0], input[1], input[2])) * k, select(1.0, input[4], color == 3u));
    } else if color == 4u {
        let cb = input[1] - 0.5;
        let cr = input[2] - 0.5;
        output = vec4<f32>(clamp(vec3<f32>(
            input[0] + 1.402 * cr,
            input[0] - 0.344136 * cb - 0.714136 * cr,
            input[0] + 1.772 * cb
        ), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
    } else {
        output = vec4<f32>(lab_to_rgb(input[0], input[1], input[2]), 1.0);
    }
    textureStore(output_texture, vec2<i32>(i32(id.x), i32(params.format.z + id.y)), output);
}
"#;

const RGB_UPLOAD_SHADER: &str = r#"
struct Params {
    base_y: u32,
    width: u32,
    height: u32,
    _padding: u32,
}

@group(0) @binding(0) var<storage, read> rgb: array<f32>;
@group(0) @binding(1) var output: texture_storage_2d<rgba32float, write>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= params.width || gid.y >= params.height) {
        return;
    }
    let index = (gid.y * params.width + gid.x) * 3u;
    textureStore(
        output,
        vec2<i32>(i32(gid.x), i32(params.base_y + gid.y)),
        vec4<f32>(rgb[index], rgb[index + 1u], rgb[index + 2u], 1.0));
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::image_io::{DecodedColor, DecodedImage, DecodedLayout, DecodedPixels, DecodedTransform};
    use crate::model::{Image, ImageData, PixelType};

    fn context() -> Arc<GpuComputeContext> {
        static CONTEXT: std::sync::OnceLock<Arc<GpuComputeContext>> = std::sync::OnceLock::new();
        Arc::clone(CONTEXT.get_or_init(|| {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
            let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
                ..Default::default()
            }))
            .expect("GPU adapter");
            let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("edolview compute test device"),
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            }))
            .expect("GPU device");
            install_gpu_compute(&device, &queue, adapter.get_info().backend);
            gpu_compute().unwrap()
        }))
    }

    fn test_image(offset: f32) -> ImageData {
        let spec = ImageSpec::new(64, 32, 4, PixelType::F32);
        let pixels = (0..spec.width * spec.height)
            .flat_map(|index| {
                let value = index as f32 / (spec.width * spec.height) as f32 + offset;
                [value, value * 0.5, -value, 1.0]
            })
            .collect();
        ImageData::from_f32(spec, pixels).unwrap()
    }

    fn cpu_ssim(lhs: &ImageData, rhs: &ImageData) -> f64 {
        let spec = lhs.spec();
        let lhs = lhs.pixels().unwrap();
        let rhs = rhs.pixels().unwrap();
        let reflect = |mut value: i32, length: i32| {
            while value < 0 || value >= length {
                if value < 0 {
                    value = -value;
                }
                if value >= length {
                    value = 2 * length - value - 2;
                }
            }
            value
        };
        let mut total = 0.0f64;
        for y in 0..spec.height {
            for x in 0..spec.width {
                for channel in 0..spec.channels {
                    let mut weight_sum = 0.0f64;
                    let mut mean_a = 0.0f64;
                    let mut mean_b = 0.0f64;
                    let mut aa = 0.0f64;
                    let mut bb = 0.0f64;
                    let mut ab = 0.0f64;
                    for oy in -5..=5 {
                        let sy = reflect(y + oy, spec.height);
                        for ox in -5..=5 {
                            let sx = reflect(x + ox, spec.width);
                            let weight = (-(ox * ox + oy * oy) as f64 / 4.5).exp();
                            let index = ((sy * spec.width + sx) * spec.channels + channel) as usize;
                            let a = lhs[index] as f64;
                            let b = rhs[index] as f64;
                            weight_sum += weight;
                            mean_a += a * weight;
                            mean_b += b * weight;
                            aa += a * a * weight;
                            bb += b * b * weight;
                            ab += a * b * weight;
                        }
                    }
                    mean_a /= weight_sum;
                    mean_b /= weight_sum;
                    let variance_a = aa / weight_sum - mean_a * mean_a;
                    let variance_b = bb / weight_sum - mean_b * mean_b;
                    let covariance = ab / weight_sum - mean_a * mean_b;
                    total += ((2.0 * mean_a * mean_b + 0.0001) * (2.0 * covariance + 0.0009))
                        / ((mean_a * mean_a + mean_b * mean_b + 0.0001) * (variance_a + variance_b + 0.0009));
                }
            }
        }
        total / (spec.width * spec.height * spec.channels) as f64
    }

    #[test]
    fn disabled_precompute_reuses_mean_results_and_invalidates_changed_keys() {
        let compute = context();
        let image = ImageData::from_f32(ImageSpec::new(2, 2, 1, PixelType::F32), vec![1.0, 2.0, 3.0, 4.0]).unwrap();
        let processor = crate::model::MeanProcessor::new();
        processor.set_precompute_enabled(false);
        let full = Recti::from_min_size(Vec2i::ZERO, Vec2i::new(2, 2));
        let expected = [
            (MeanDim::All, vec![2.5]),
            (MeanDim::Column, vec![2.0, 3.0]),
            (MeanDim::Row, vec![1.5, 3.5]),
        ];
        for (dim, values) in &expected {
            assert_eq!(processor.compute(&image, full, *dim).unwrap(), *values);
        }

        // Alter only the GPU texture to make a repeated reduction observable.
        // A cache hit must still return the original values in every dimension.
        compute
            .upload_direct(&image.gpu_texture().unwrap().texture, &image.spec(), &[9.0; 4])
            .unwrap();
        for (dim, values) in &expected {
            assert_eq!(processor.compute(&image.clone(), full, *dim).unwrap(), *values);
        }
        let reversed = Recti::from_min_max(full.max, full.min);
        assert_eq!(processor.compute(&image, reversed, MeanDim::All).unwrap(), vec![2.5]);
        assert!(processor.compute(&image, Recti::ZERO, MeanDim::All).unwrap().is_empty());

        let column = Recti::from_min_size(Vec2i::ZERO, Vec2i::new(1, 2));
        assert_eq!(processor.compute(&image, column, MeanDim::All).unwrap(), vec![9.0]);
        assert_eq!(processor.compute(&image, full, MeanDim::All).unwrap(), vec![9.0]);
        let replacement = ImageData::from_f32(image.spec(), vec![5.0; 4]).unwrap();
        assert_eq!(processor.compute(&replacement, full, MeanDim::All).unwrap(), vec![5.0]);
    }

    #[test]
    fn enabling_precompute_bypasses_cached_gpu_means() {
        let compute = context();
        let image = ImageData::from_f32(ImageSpec::new(2, 2, 1, PixelType::F32), vec![1.0; 4]).unwrap();
        let processor = crate::model::MeanProcessor::new();
        processor.set_precompute_enabled(false);
        let full = Recti::from_min_size(Vec2i::ZERO, Vec2i::new(2, 2));
        assert_eq!(processor.compute(&image, full, MeanDim::All).unwrap(), vec![1.0]);
        compute
            .upload_direct(&image.gpu_texture().unwrap().texture, &image.spec(), &[9.0; 4])
            .unwrap();
        processor.set_precompute_enabled(true);
        assert_eq!(processor.compute(&image, full, MeanDim::All).unwrap(), vec![9.0]);
    }

    #[test]
    fn gpu_statistics_match_expected_values() {
        let compute = context();
        let image = test_image(0.0);
        let texture = image.gpu_texture().unwrap();
        let (mins, maxs) = compute.minmax(&texture, Recti::ZERO).unwrap();
        assert!((mins[0] - 0.0).abs() < 1e-6);
        assert!((maxs[0] - 2047.0 / 2048.0).abs() < 1e-6);
        assert!((mins[2] + 2047.0 / 2048.0).abs() < 1e-6);
        assert_eq!(mins[3], 1.0);
        assert_eq!(maxs[3], 1.0);

        let mean = compute.mean(&texture, Recti::ZERO, MeanDim::All).unwrap();
        assert!((mean[0] - 2047.0 / 4096.0).abs() < 1e-5);
        assert!((mean[1] - 2047.0 / 8192.0).abs() < 1e-5);
        assert!((mean[2] + 2047.0 / 4096.0).abs() < 1e-5);
        assert!((mean[3] - 1.0).abs() < 1e-6);

        let columns = compute.mean(&texture, Recti::ZERO, MeanDim::Column).unwrap();
        let rows = compute.mean(&texture, Recti::ZERO, MeanDim::Row).unwrap();
        assert_eq!(columns.len(), 64 * 4);
        assert_eq!(rows.len(), 32 * 4);

        let histogram = compute.histogram(&texture).unwrap();
        assert_eq!(histogram.len(), 4);
        assert_eq!(histogram[0].iter().sum::<f32>(), 2048.0);
        assert_eq!(histogram[3].iter().sum::<f32>(), 0.0);
    }

    #[test]
    fn native_upload_normalizes_and_fuses_layout_and_palette() {
        let compute = context();

        let rgba = ImageData::from_raw_bytes(&[0, 64, 128, 255, 255, 128, 64, 0], 2, 1, 4, PixelType::U8).unwrap();
        let (mins, maxs) = compute.minmax(&rgba.gpu_texture().unwrap(), Recti::ZERO).unwrap();
        assert_eq!(mins, vec![0.0, 64.0 / 255.0, 64.0 / 255.0, 0.0]);
        assert_eq!(maxs, vec![1.0, 128.0 / 255.0, 128.0 / 255.0, 1.0]);

        let rgb16 = ImageData::from_raw_bytes(
            bytemuck::cast_slice(&[0_u16, 32768, u16::MAX, u16::MAX, 16384, 0]),
            2,
            1,
            3,
            PixelType::U16,
        )
        .unwrap();
        let (mins, maxs) = compute.minmax(&rgb16.gpu_texture().unwrap(), Recti::ZERO).unwrap();
        assert!((mins[0] - 0.0).abs() < 1e-6 && (maxs[0] - 1.0).abs() < 1e-6);
        assert!((mins[1] - 16384.0 / 65535.0).abs() < 1e-6);
        assert!((maxs[1] - 32768.0 / 65535.0).abs() < 1e-6);

        let planar = DecodedImage::new_with_layout(
            2,
            1,
            3,
            PixelType::U8,
            DecodedPixels::U8(vec![0, 255, 128, 64, 255, 0]),
            DecodedLayout {
                row_stride_bytes: 2,
                plane_stride_bytes: 2,
                planes: 3,
                bit_depth: 8,
                input_channels: 3,
            },
            DecodedColor::Direct,
        )
        .unwrap();
        let planar = ImageData::from_decoded(planar).unwrap();
        let (mins, maxs) = compute.minmax(&planar.gpu_texture().unwrap(), Recti::ZERO).unwrap();
        for (actual, expected) in mins.iter().zip([0.0, 64.0 / 255.0, 0.0]) {
            assert!((actual - expected).abs() < 1e-6);
        }
        for (actual, expected) in maxs.iter().zip([1.0, 128.0 / 255.0, 1.0]) {
            assert!((actual - expected).abs() < 1e-6);
        }

        let mut palette = vec![0_u16; 16 * 3];
        palette[15] = u16::MAX;
        palette[16] = u16::MAX;
        palette[2 * 16 + 15] = u16::MAX;
        let packed = DecodedImage::new_with_layout(
            2,
            1,
            3,
            PixelType::U16,
            DecodedPixels::U8(vec![0x0f]),
            DecodedLayout {
                row_stride_bytes: 1,
                plane_stride_bytes: 1,
                planes: 1,
                bit_depth: 4,
                input_channels: 1,
            },
            DecodedColor::Palette(palette),
        )
        .unwrap();
        let packed = ImageData::from_decoded(packed).unwrap();
        let (mins, maxs) = compute.minmax(&packed.gpu_texture().unwrap(), Recti::ZERO).unwrap();
        assert_eq!(mins, vec![0.0, 0.0, 0.0]);
        assert_eq!(maxs, vec![1.0, 1.0, 1.0]);

        let mut raw = vec![0_u8; 4];
        for value in [1.0_f32, 2.0, 3.0, 4.0] {
            raw.extend_from_slice(&value.to_be_bytes());
        }
        let transformed = DecodedImage::new_with_transform(
            2,
            2,
            1,
            PixelType::F32,
            DecodedPixels::Raw(raw),
            DecodedLayout {
                row_stride_bytes: 8,
                plane_stride_bytes: 16,
                planes: 1,
                bit_depth: 32,
                input_channels: 1,
            },
            DecodedColor::Direct,
            DecodedTransform {
                data_offset_bytes: 4,
                flip_y: true,
                swap_bytes: true,
                scale: 2.0,
            },
        )
        .unwrap();
        let transformed = ImageData::from_decoded(transformed).unwrap();
        let texture = transformed.gpu_texture().unwrap();
        assert_eq!(texture.texture.format(), wgpu::TextureFormat::Rgba32Float);
        let rows = compute.mean(&texture, Recti::ZERO, MeanDim::Row).unwrap();
        assert_eq!(rows, vec![7.0, 3.0]);
    }

    #[test]
    #[ignore = "manual supported-format GPU pipeline validation"]
    fn supported_format_matrix_reaches_gpu_statistics() {
        let root =
            std::path::PathBuf::from(std::env::var("EDOLVIEW_GPU_MATRIX_DIR").expect("set EDOLVIEW_GPU_MATRIX_DIR"));
        let compute = context();
        let mut tested = 0;
        for entry in std::fs::read_dir(&root).unwrap() {
            let path = entry.unwrap().path();
            let extension = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if !crate::supported_image::is_supported_image_extension(&extension) {
                continue;
            }
            let image = ImageData::load_from_path(&path)
                .unwrap_or_else(|error| panic!("{} failed to decode: {error:#}", path.display()));
            let texture = image
                .gpu_texture()
                .unwrap_or_else(|error| panic!("{} failed GPU upload: {error:#}", path.display()));
            let (mins, maxs) = compute
                .minmax(&texture, Recti::ZERO)
                .unwrap_or_else(|error| panic!("{} failed GPU statistics: {error:#}", path.display()));
            assert_eq!(mins.len(), image.spec().channels as usize, "{}", path.display());
            assert_eq!(maxs.len(), image.spec().channels as usize, "{}", path.display());
            tested += 1;
        }
        assert!(tested >= 100, "only {tested} supported samples were tested");
        eprintln!("validated {tested} supported images through decode, GPU upload, and min/max");
    }

    #[test]
    #[ignore = "manual decode and GPU upload benchmark"]
    fn benchmark_decode_and_gpu_upload() {
        let files = std::env::var("EDOLVIEW_GPU_BENCH_FILES").expect("set EDOLVIEW_GPU_BENCH_FILES");
        let iterations = std::env::var("EDOLVIEW_GPU_BENCH_ITERATIONS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(5)
            .max(1);
        let compute = context();
        for path in files.split(';').filter(|value| !value.is_empty()) {
            let path = std::path::PathBuf::from(path);
            let mut decode_ms = Vec::new();
            let mut upload_ms = Vec::new();
            let mut output_format = None;
            for iteration in 0..=iterations {
                let start = std::time::Instant::now();
                let image = ImageData::load_from_path(&path).unwrap();
                let decoded = start.elapsed().as_secs_f64() * 1_000.0;
                let start = std::time::Instant::now();
                let texture = image.gpu_texture().unwrap();
                compute
                    .device
                    .poll(wgpu::PollType::wait_indefinitely())
                    .expect("wait for native upload");
                let uploaded = start.elapsed().as_secs_f64() * 1_000.0;
                output_format = Some(texture.texture.format());
                if iteration > 0 {
                    decode_ms.push(decoded);
                    upload_ms.push(uploaded);
                }
            }
            decode_ms.sort_by(f64::total_cmp);
            upload_ms.sort_by(f64::total_cmp);
            eprintln!(
                "GPU_BENCH,{},{:.3},{:.3},{:.3},{:?}",
                path.file_name().unwrap().to_string_lossy(),
                decode_ms[decode_ms.len() / 2],
                upload_ms[upload_ms.len() / 2],
                decode_ms[decode_ms.len() / 2] + upload_ms[upload_ms.len() / 2],
                output_format.unwrap()
            );
        }
    }

    #[test]
    fn gpu_comparison_psnr_and_ssim_are_consistent() {
        let compute = context();
        let lhs = test_image(0.0);
        let rhs = test_image(0.01);
        let lhs_texture = lhs.gpu_texture().unwrap();
        let rhs_texture = rhs.gpu_texture().unwrap();
        let (psnr, rmse) = compute.psnr(&lhs_texture, &rhs_texture, Recti::ZERO, 1.0, 1.0).unwrap();
        assert!((rmse - 0.0075).abs() < 1e-5);
        assert!((psnr - 42.498775).abs() < 1e-3);

        let identical_ssim = compute.ssim(&lhs_texture, &lhs_texture, Recti::ZERO).unwrap();
        assert!((identical_ssim - 1.0).abs() < 1e-4);
        let expected_ssim = cpu_ssim(&lhs, &rhs);
        let actual_ssim = compute.ssim(&lhs_texture, &rhs_texture, Recti::ZERO).unwrap();
        assert!((actual_ssim - expected_ssim).abs() < 1e-4);

        let derived = ImageData::derived_comparison(lhs, rhs, lhs_texture.spec.clone(), ComparisonMode::Diff, 0.5, 0);
        let diff = derived.gpu_texture().unwrap();
        let (mins, maxs) = compute.minmax(&diff, Recti::ZERO).unwrap();
        for (channel, expected) in [-0.01, -0.005, 0.01, 0.0].into_iter().enumerate() {
            assert!((mins[channel] - expected).abs() < 1e-5);
            assert!((maxs[channel] - expected).abs() < 1e-5);
        }
    }
}
