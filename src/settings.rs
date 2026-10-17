use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::ui::gpu::ShaderParams;

pub const VIEW_PRESET_COUNT: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ExternalOpenMode {
    #[default]
    NewWindow,
    ExistingWindow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum AngleDisplayUnit {
    #[default]
    Degrees,
    Radians,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundStyle {
    Black,
    Gray,
    White,
    #[default]
    DarkChecker,
    LightChecker,
    HighContrastChecker,
}

impl BackgroundStyle {
    pub const ALL: [Self; 6] = [
        Self::Black,
        Self::Gray,
        Self::White,
        Self::DarkChecker,
        Self::LightChecker,
        Self::HighContrastChecker,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Black => "Black",
            Self::Gray => "Gray",
            Self::White => "White",
            Self::DarkChecker => "Dark checkerboard",
            Self::LightChecker => "Light checkerboard",
            Self::HighContrastChecker => "High-contrast checkerboard",
        }
    }

    pub const fn colors(self, dark_checker_colors: [eframe::egui::Color32; 2]) -> [eframe::egui::Color32; 2] {
        match self {
            Self::Black => [crate::res::BACKGROUND_BLACK; 2],
            Self::Gray => [crate::res::BACKGROUND_GRAY; 2],
            Self::White => [crate::res::BACKGROUND_WHITE; 2],
            Self::DarkChecker => dark_checker_colors,
            Self::LightChecker => crate::res::BACKGROUND_LIGHT_CHECKER,
            Self::HighContrastChecker => crate::res::BACKGROUND_HIGH_CONTRAST_CHECKER,
        }
    }
}

impl ExternalOpenMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::NewWindow => "Open in a new window",
            Self::ExistingWindow => "Send to the last active window",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub external_open_mode: ExternalOpenMode,
    /// Build a CPU summed-area table for O(1) marquee mean queries.
    /// This costs `(width + 1) * (height + 1) * channels * 8` bytes per
    /// active image, so keeping it configurable is important for very large images.
    #[serde(default = "default_integral_table_precompute")]
    pub integral_table_precompute: bool,
    #[serde(default)]
    pub ui_state: PersistentUiState,
    #[serde(default = "default_view_presets")]
    pub view_presets: Vec<Option<ViewPreset>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewPreset {
    pub colormap_rgb: String,
    pub colormap_mono: String,
    pub shader_params: ShaderParams,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistentUiState {
    #[serde(default)]
    pub background_style: BackgroundStyle,
    pub is_show_pixel_value: bool,
    pub is_show_crosshair: bool,
    pub is_show_minimap: bool,
    pub is_show_sidebar: bool,
    pub is_show_statusbar: bool,
    pub copy_use_original_size: bool,
    #[serde(default)]
    pub angle_display_unit: AngleDisplayUnit,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            external_open_mode: ExternalOpenMode::NewWindow,
            integral_table_precompute: default_integral_table_precompute(),
            ui_state: PersistentUiState::default(),
            view_presets: default_view_presets(),
        }
    }
}

impl Default for PersistentUiState {
    fn default() -> Self {
        Self {
            background_style: BackgroundStyle::default(),
            is_show_pixel_value: true,
            is_show_crosshair: false,
            is_show_minimap: false,
            is_show_sidebar: true,
            is_show_statusbar: true,
            copy_use_original_size: true,
            angle_display_unit: AngleDisplayUnit::Degrees,
        }
    }
}

impl AppSettings {
    pub fn load() -> Result<Self, String> {
        let path = settings_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let body =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read settings file '{}': {e}", path.display()))?;
        let mut settings: Self = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse settings file '{}': {e}", path.display()))?;
        settings.normalize();
        Ok(settings)
    }

    pub fn save(&self) -> Result<(), String> {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings directory '{}': {e}", parent.display()))?;
        }

        let body = serde_json::to_string_pretty(self).map_err(|e| format!("Failed to serialize settings: {e}"))?;
        fs::write(&path, body).map_err(|e| format!("Failed to write settings file '{}': {e}", path.display()))
    }

    pub fn view_preset(&self, slot: usize) -> Option<&ViewPreset> {
        self.view_presets.get(slot).and_then(|preset| preset.as_ref())
    }

    pub fn set_view_preset(&mut self, slot: usize, preset: ViewPreset) {
        self.normalize();
        if let Some(entry) = self.view_presets.get_mut(slot) {
            *entry = Some(preset);
        }
    }

    fn normalize(&mut self) {
        self.view_presets.resize(VIEW_PRESET_COUNT, None);
        self.view_presets.truncate(VIEW_PRESET_COUNT);
    }
}

fn settings_path() -> PathBuf {
    crate::util::path_ext::app_config_dir().join("settings.json")
}

fn default_view_presets() -> Vec<Option<ViewPreset>> {
    vec![None; VIEW_PRESET_COUNT]
}

const fn default_integral_table_precompute() -> bool {
    true
}
