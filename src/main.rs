#![allow(dead_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use color_eyre::eyre::{eyre, Result};
use eframe::egui::{self, ViewportBuilder};
use std::{path::PathBuf, sync::Arc};

use crate::ui::ViewerApp;

#[cfg(diagnostics)]
#[macro_use]
mod diagnostics;

// Distribution builds enter the application directly, without diagnostics.
#[cfg(diagnostics)]
use diagnostics::main;
#[cfg(not(diagnostics))]
use run as main;

mod control;
mod model;
mod res;
mod settings;
mod supported_image;
mod ui;
mod update;
mod util;
#[cfg(target_os = "windows")]
mod windows_install;

#[cfg(debug_assertions)]
mod debug;

const ICON_DATA: &[u8] = include_bytes!("../icons/icon.png");
#[cfg(target_os = "windows")]
const WINDOWS_APP_USER_MODEL_ID: &str = "kr.edoli.edolview";

const REQUIRED_STORAGE_BUFFERS_PER_STAGE: u32 = 6;
const REQUIRED_STORAGE_TEXTURES_PER_STAGE: u32 = 1;
const REQUIRED_COMPUTE_INVOCATIONS_PER_WORKGROUP: u32 = 256;
const REQUIRED_COMPUTE_WORKGROUP_STORAGE_SIZE: u32 = 256 * 16;

fn supports_image_compute_limits(limits: &wgpu::Limits) -> bool {
    limits.max_bind_groups >= 1
        && limits.max_bindings_per_bind_group >= 8
        && limits.max_sampled_textures_per_shader_stage >= 2
        && limits.max_storage_buffers_per_shader_stage >= REQUIRED_STORAGE_BUFFERS_PER_STAGE
        && limits.max_storage_textures_per_shader_stage >= REQUIRED_STORAGE_TEXTURES_PER_STAGE
        && limits.max_uniform_buffers_per_shader_stage >= 1
        && limits.max_storage_buffer_binding_size > 0
        && limits.max_compute_workgroup_storage_size >= REQUIRED_COMPUTE_WORKGROUP_STORAGE_SIZE
        && limits.max_compute_invocations_per_workgroup >= REQUIRED_COMPUTE_INVOCATIONS_PER_WORKGROUP
        && limits.max_compute_workgroup_size_x >= REQUIRED_COMPUTE_INVOCATIONS_PER_WORKGROUP
        && limits.max_compute_workgroup_size_y >= 16
        && limits.max_compute_workgroup_size_z >= 1
        && limits.max_compute_workgroups_per_dimension > 0
}

fn supports_image_compute(adapter: &wgpu::Adapter) -> bool {
    if !adapter
        .get_downlevel_capabilities()
        .flags
        .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
        || !supports_image_compute_limits(&adapter.limits())
    {
        return false;
    }

    let required_texture_usages = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING;
    [wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureFormat::Rgba32Float]
        .into_iter()
        .all(|format| {
            adapter
                .get_texture_format_features(format)
                .allowed_usages
                .contains(required_texture_usages)
        })
}

fn apply_adapter_compute_limits(required: &mut wgpu::Limits, adapter: &wgpu::Limits) {
    required.max_texture_dimension_2d = adapter.max_texture_dimension_2d;
    required.max_storage_buffer_binding_size = adapter.max_storage_buffer_binding_size;
    required.max_buffer_size = adapter.max_buffer_size;
    required.max_storage_buffers_per_shader_stage = adapter.max_storage_buffers_per_shader_stage;
    required.max_storage_textures_per_shader_stage = adapter.max_storage_textures_per_shader_stage;
    required.max_compute_workgroup_storage_size = adapter.max_compute_workgroup_storage_size;
    required.max_compute_invocations_per_workgroup = adapter.max_compute_invocations_per_workgroup;
    required.max_compute_workgroup_size_x = adapter.max_compute_workgroup_size_x;
    required.max_compute_workgroup_size_y = adapter.max_compute_workgroup_size_y;
    required.max_compute_workgroup_size_z = adapter.max_compute_workgroup_size_z;
    required.max_compute_workgroups_per_dimension = adapter.max_compute_workgroups_per_dimension;
}

fn device_priority(device_type: wgpu::DeviceType, power_preference: wgpu::PowerPreference) -> u8 {
    match power_preference {
        wgpu::PowerPreference::LowPower => match device_type {
            wgpu::DeviceType::IntegratedGpu => 5,
            wgpu::DeviceType::VirtualGpu => 4,
            wgpu::DeviceType::DiscreteGpu => 3,
            wgpu::DeviceType::Other => 2,
            wgpu::DeviceType::Cpu => 1,
        },
        wgpu::PowerPreference::None | wgpu::PowerPreference::HighPerformance => match device_type {
            wgpu::DeviceType::DiscreteGpu => 5,
            wgpu::DeviceType::IntegratedGpu => 4,
            wgpu::DeviceType::VirtualGpu => 3,
            wgpu::DeviceType::Other => 2,
            wgpu::DeviceType::Cpu => 1,
        },
    }
}

fn configure_wgpu(options: &mut eframe::NativeOptions) {
    let egui_wgpu::WgpuSetup::CreateNew(setup) = &mut options.wgpu_options.wgpu_setup else {
        return;
    };

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        // Use the same tuned Vulkan path on Windows and Linux. Setting the
        // instance backends explicitly also prevents WGPU_BACKEND from
        // selecting DX12 or GL at runtime.
        setup.instance_descriptor.backends = wgpu::Backends::VULKAN;
    }
    #[cfg(target_os = "macos")]
    {
        setup.instance_descriptor.backends = wgpu::Backends::METAL;
    }

    let power_preference = setup.power_preference;
    setup.native_adapter_selector = Some(Arc::new(move |adapters, surface| {
        adapters
            .iter()
            .filter(|adapter| surface.is_none_or(|surface| adapter.is_surface_supported(surface)))
            .filter(|adapter| supports_image_compute(adapter))
            .max_by_key(|adapter| {
                let info = adapter.get_info();
                (
                    info.device_type != wgpu::DeviceType::Cpu,
                    device_priority(info.device_type, power_preference),
                )
            })
            .cloned()
            .ok_or_else(|| {
                let available = adapters
                    .iter()
                    .map(|adapter| {
                        let info = adapter.get_info();
                        format!("{} ({:?}, {:?})", info.name, info.backend, info.device_type)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "No graphics adapter supports the compute and storage capabilities required by edolview. Available adapters: {available}"
                )
            })
    }));

    let default_device_descriptor = Arc::clone(&setup.device_descriptor);
    setup.device_descriptor = Arc::new(move |adapter| {
        let mut descriptor = default_device_descriptor(adapter);
        apply_adapter_compute_limits(&mut descriptor.required_limits, &adapter.limits());
        const MIB: u64 = 1024 * 1024;
        descriptor.memory_hints = match adapter.get_info().backend {
            wgpu::Backend::Dx12 => {
                // Keep the initial DX12 heaps small enough for low VRAM usage while
                // allowing the allocator to grow for workloads with many resources.
                // wgpu-hal derives the host range as half of this device range.
                wgpu::MemoryHints::Manual {
                    suballocated_device_memory_block_size: 40 * MIB..256 * MIB,
                }
            }
            wgpu::Backend::Vulkan => {
                // Vulkan's default Performance hint starts with much larger device
                // and host blocks than the image upload path needs. A smaller range
                // lowers peak VRAM without crossing the allocator's slow path.
                wgpu::MemoryHints::Manual {
                    suballocated_device_memory_block_size: 40 * MIB..192 * MIB,
                }
            }
            _ => descriptor.memory_hints.clone(),
        };
        descriptor
    });
}

/// GPU-accelerated image viewer using image-rs for decoding and egui for GUI.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// Image file paths to open (optional)
    images: Vec<PathBuf>,
}

fn run() -> Result<()> {
    color_eyre::install()?;
    #[cfg(diagnostics)]
    diagnostics::install_panic_hook();
    set_windows_app_user_model_id();
    let args = Args::parse();

    let settings = crate::settings::AppSettings::load().unwrap_or_else(|err| {
        eprintln!("Failed to load settings: {err}");
        crate::settings::AppSettings::default()
    });

    if !args.images.is_empty() && settings.external_open_mode == crate::settings::ExternalOpenMode::ExistingWindow {
        match crate::control::try_forward_paths_to_last_active(&args.images) {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(err) => eprintln!("Failed to forward files to an existing Edolview window: {err}"),
        }
    }

    #[cfg(target_os = "linux")]
    {
        std::env::remove_var("WAYLAND_DISPLAY");
    }

    let icon = match image::load_from_memory(ICON_DATA) {
        Ok(icon) => {
            let icon = icon.into_rgba8();
            egui::IconData {
                width: icon.width(),
                height: icon.height(),
                rgba: icon.into_raw(),
            }
        }
        Err(error) => {
            eprintln!("Failed to load icon image; continuing without icon: {error}");
            egui::IconData::default()
        }
    };

    let mut native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("edolview")
            .with_inner_size(egui::vec2(1280.0, 720.0))
            .with_min_inner_size(egui::vec2(720.0, 480.0))
            .with_icon(icon),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    configure_wgpu(&mut native_options);

    if let Err(e) = eframe::run_native(
        "edolview",
        native_options,
        Box::new(|_cc| Ok(Box::new(ViewerApp::new().with_paths(args.images)))),
    ) {
        return Err(eyre!("eframe initialization failed: {e}"));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn set_windows_app_user_model_id() {
    use windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

    let mut app_id = WINDOWS_APP_USER_MODEL_ID.encode_utf16().collect::<Vec<_>>();
    app_id.push(0);

    unsafe {
        let _ = SetCurrentProcessExplicitAppUserModelID(app_id.as_ptr());
    }
}

#[cfg(not(target_os = "windows"))]
fn set_windows_app_user_model_id() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webgl2_limits_are_rejected_for_image_compute() {
        assert!(!supports_image_compute_limits(&wgpu::Limits::downlevel_webgl2_defaults()));
    }

    #[test]
    fn standard_wgpu_limits_support_image_compute() {
        assert!(supports_image_compute_limits(&wgpu::Limits::default()));
    }

    #[test]
    fn adapter_compute_limits_replace_webgl2_zeroes() {
        let mut requested = wgpu::Limits::downlevel_webgl2_defaults();
        let supported = wgpu::Limits::default();

        apply_adapter_compute_limits(&mut requested, &supported);

        assert!(supports_image_compute_limits(&requested));
        assert_eq!(
            requested.max_storage_buffers_per_shader_stage,
            supported.max_storage_buffers_per_shader_stage
        );
        assert_eq!(
            requested.max_compute_invocations_per_workgroup,
            supported.max_compute_invocations_per_workgroup
        );
    }
}
