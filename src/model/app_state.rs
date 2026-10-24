use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use clipboard_rs::{Clipboard, ClipboardContext, ContentFormat};
use color_eyre::eyre::{eyre, Result};
use indexmap::IndexMap;

use crate::{
    model::{
        AssetType, ClipboardAsset, ComparisonAsset, ComparisonMode, FileAsset, Image, ImageData, Recti, SharedAsset,
        SocketInfo, SocketState, Statistics,
    },
    settings::BackgroundStyle,
    ui::gpu::ShaderParams,
    util::math_ext::{vec2i, Vec2i},
};

pub struct AppState {
    pub path: Option<PathBuf>,
    pub asset: Option<SharedAsset>,
    pub asset_primary: Option<SharedAsset>,
    pub asset_secondary: Option<SharedAsset>,
    pub comparison_mode: ComparisonMode,
    pub comparison_blend: f32,
    pub comparison_notice: Option<String>,
    pub shader_params: ShaderParams,
    pub cursor_pos: Option<Vec2i>,
    pub cursor_on_secondary: bool,
    pub marquee_rect: Recti,

    pub channel_index: i32,
    pub colormap_rgb: String,
    pub colormap_mono: String,
    pub colormap_rgb_list: Vec<String>,
    pub colormap_mono_list: Vec<String>,

    pub background_style: BackgroundStyle,
    pub is_show_pixel_value: bool,
    pub is_show_crosshair: bool,
    pub is_show_minimap: bool,
    pub is_show_sidebar: bool,
    pub is_show_statusbar: bool,

    // Copy behavior: when true, Ctrl+C copies marquee at original pixel size regardless of zoom.
    pub copy_use_original_size: bool,

    // File navigation + watcher
    pub file_nav: crate::model::FileNav,

    pub statistics: Statistics,

    pub socket_state: Arc<SocketState>,
    pub socket_info: Arc<Mutex<SocketInfo>>,

    pub assets: IndexMap<String, SharedAsset>,
}

fn list_colormaps(rel_dir: &str) -> Vec<String> {
    // get file names ending with .glsl in the directory; silently ignore IO errors
    let relative = rel_dir.strip_prefix("colormap/").unwrap_or(rel_dir);
    let roots = [
        crate::util::path_ext::colormap_dir(),
        crate::util::path_ext::custom_colormap_dir(),
    ];
    let mut files = Vec::new();
    for root in roots {
        if let Ok(entries) = std::fs::read_dir(root.join(relative)) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".glsl") {
                        if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                            files.push(stem.to_string());
                        }
                    }
                }
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

impl AppState {
    pub fn empty() -> Self {
        Self {
            path: None,
            asset: None,
            asset_primary: None,
            asset_secondary: None,
            comparison_mode: ComparisonMode::Diff,
            comparison_blend: 0.5,
            comparison_notice: None,
            shader_params: ShaderParams::default(),
            cursor_pos: None,
            cursor_on_secondary: false,
            marquee_rect: Recti::ZERO,
            channel_index: -1,
            colormap_rgb: String::from("rgb"),
            colormap_mono: String::from("gray"),
            colormap_rgb_list: list_colormaps("colormap/rgb"),
            colormap_mono_list: list_colormaps("colormap/mono"),
            background_style: BackgroundStyle::default(),
            is_show_pixel_value: true,
            is_show_crosshair: false,
            is_show_minimap: true,
            is_show_sidebar: true,
            is_show_statusbar: true,
            copy_use_original_size: true,
            file_nav: crate::model::FileNav::new(),
            statistics: Statistics::default(),
            socket_state: Arc::new(SocketState::new()),
            socket_info: Arc::new(Mutex::new(SocketInfo::new())),
            assets: IndexMap::new(),
        }
    }

    pub fn load_from_path(&mut self, path: PathBuf) -> Result<()> {
        #[cfg(debug_assertions)]
        let _timer = crate::util::timer::ScopedTimer::new("Total image load time [from path]");

        let hash_str = FileAsset::hash_from_path(&path)?;

        if self.assets.contains_key(&hash_str) {
            self.set_file_asset_primary_by_hash_and_path(&hash_str, &path);
            return Ok(());
        }

        let image = ImageData::load_from_path(&path)?;
        self.apply_loaded_file_asset(path, hash_str, image);
        Ok(())
    }

    pub fn apply_loaded_file_asset(&mut self, path: PathBuf, hash: String, image: ImageData) {
        let path_str = path.to_string_lossy().to_string();
        let asset: SharedAsset = Arc::new(FileAsset::new(path_str, hash.clone(), image));

        self.set_primary_asset(asset);
        // `set_primary_asset` syncs file navigation via `asset_primary.name()`.
        // Keep the concrete load path as the navigation source when the caller already has it.
        self.sync_file_navigation_for_path(&path);
    }

    pub fn set_file_asset_primary_by_hash_and_path(&mut self, hash: &str, path: &PathBuf) {
        self.set_asset_primary_by_hash(hash);
        // `set_asset_primary_by_hash` syncs file navigation from `asset_primary.name()`.
        // When a caller already has the resolved file path, prefer it as the authoritative source.
        self.sync_file_navigation_for_path(path);
    }

    fn sync_file_navigation_for_path(&mut self, path: &PathBuf) {
        self.path = Some(path.to_path_buf());

        #[cfg(debug_assertions)]
        let _timer_path = crate::util::timer::ScopedTimer::new("Path scan");

        // Refresh directory listing and select current index
        if let Some(dir) = path.parent() {
            if self.file_nav.check_is_current_dir(dir) {
                // Same directory, no need to refresh
                return;
            }
            self.file_nav.refresh_dir_listing_for(dir.to_path_buf());
            self.file_nav.select_index_for_path(path);
            let _ = self.file_nav.start_dir_watcher(dir.to_path_buf());
        } else {
            self.file_nav.clear();
        }
    }

    fn sync_primary_file_navigation(&mut self) {
        match self
            .asset_primary
            .as_ref()
            .map(|asset| (asset.asset_type(), asset.name().to_string()))
        {
            Some((AssetType::File, path)) => self.sync_file_navigation_for_path(&PathBuf::from(path)),
            _ => self.path = None,
        }
    }

    pub fn load_from_clipboard(&mut self) -> Result<()> {
        #[cfg(debug_assertions)]
        let _timer = crate::util::timer::ScopedTimer::new("Total image load time [from clipboard]");

        let image = ImageData::load_from_clipboard()
            .or_else(|_| ImageData::load_from_url(arboard::Clipboard::new().unwrap().get_text()?.as_str()));

        if image.is_err() {
            let ctx = ClipboardContext::new().unwrap();

            if !ctx.has(ContentFormat::Files) {
                return Err(eyre!("Clipboard does not contain files"));
            }
            let uris = ctx.get_files().unwrap();
            if uris.is_empty() {
                return Err(eyre!("Clipboard does not contain image or file data"));
            }

            let paths: Vec<PathBuf> = uris.iter().map(PathBuf::from).collect();

            for path in &paths {
                if path.exists() && path.is_file() {
                    let image = ImageData::load_from_path(path)?;
                    let path_str = path.to_string_lossy().to_string();
                    let hash_str = FileAsset::hash_from_path(path)?;
                    self.set_primary_asset(Arc::new(FileAsset::new(path_str, hash_str, image)));
                }
            }
        } else {
            self.set_primary_asset(Arc::new(ClipboardAsset::new(image?)));
        }

        self.path = None;
        self.file_nav.clear();

        Ok(())
    }

    pub fn set_asset_primary_by_hash(&mut self, hash: &str) {
        self.asset_primary = self.assets.get(hash).cloned();
        self.sync_primary_file_navigation();

        self.update_asset();
        self.validate_marquee_rect();
    }

    pub fn set_primary_asset(&mut self, asset: SharedAsset) {
        let hash = asset.hash().to_string();

        self.assets.insert(hash.clone(), asset.clone());

        self.asset_primary = self.assets.get(&hash).cloned();
        self.sync_primary_file_navigation();

        self.update_asset();
        self.validate_marquee_rect();
    }

    pub fn set_asset_secondary_by_hash(&mut self, hash: &str) {
        self.asset_secondary = self.assets.get(hash).cloned();

        self.update_asset();
        self.validate_marquee_rect();
    }

    pub fn set_secondary_asset(&mut self, asset: Option<SharedAsset>) {
        if let Some(asset) = asset {
            let hash = asset.hash().to_string();

            self.assets.insert(hash.clone(), asset.clone());

            self.asset_secondary = self.assets.get(&hash).cloned();
        } else {
            self.asset_secondary = None;
        }

        self.update_asset();
        self.validate_marquee_rect();
    }

    pub fn reorder_asset_by_hash(&mut self, hash: &str, insertion_index: usize) -> bool {
        let len = self.assets.len();
        if len <= 1 || insertion_index > len {
            return false;
        }

        let Some(from_index) = self.assets.get_index_of(hash) else {
            return false;
        };

        let to_index = if insertion_index > from_index {
            insertion_index.saturating_sub(1)
        } else {
            insertion_index
        };

        if to_index >= len || to_index == from_index {
            return false;
        }

        self.assets.move_index(from_index, to_index);
        true
    }

    pub fn is_comparison(&self) -> bool {
        if let Some(asset) = &self.asset {
            asset.asset_type() == AssetType::Comparison
        } else {
            false
        }
    }

    pub fn update_asset(&mut self) {
        self.comparison_notice = None;
        if self.comparison_mode != ComparisonMode::Split || self.asset_secondary.is_none() {
            self.cursor_on_secondary = false;
        }
        if let Some(asset_primary) = &self.asset_primary {
            if let Some(asset_secondary) = &self.asset_secondary {
                if asset_primary.hash() == asset_secondary.hash() {
                    self.asset = Some(asset_primary.clone());
                } else {
                    let primary_spec = asset_primary.image().spec();
                    let secondary_spec = asset_secondary.image().spec();
                    if primary_spec.width != secondary_spec.width || primary_spec.height != secondary_spec.height {
                        self.comparison_notice = Some(format!(
                            "Error: image size mismatch: primary is {}x{}, secondary is {}x{}.",
                            primary_spec.width, primary_spec.height, secondary_spec.width, secondary_spec.height
                        ));
                        self.cursor_on_secondary = false;
                        self.asset = None;
                        return;
                    }

                    // Different assets, create a comparison asset
                    let (comp_asset, comparison_notice) = ComparisonAsset::new(
                        asset_primary.clone(),
                        asset_secondary.clone(),
                        self.comparison_mode,
                        self.comparison_blend,
                    );
                    self.comparison_notice = comparison_notice;
                    self.asset = Some(Arc::new(comp_asset));
                }
            } else {
                self.asset = Some(asset_primary.clone());
            }

            if let Some(asset) = &self.asset {
                let num_channels = asset.image().spec().channels;

                if num_channels == 1 {
                    self.channel_index = -1;
                } else if self.channel_index >= num_channels {
                    self.channel_index = num_channels - 1;
                }

                crate::model::image::MEAN_PROCESSOR.precompute_async(asset.image());
            }
        } else {
            self.asset = None;
        }
    }

    pub fn clear_asset(&mut self) {
        self.asset = None;
        self.asset_primary = None;
        self.asset_secondary = None;
        self.comparison_notice = None;
        self.cursor_on_secondary = false;
        self.path = None;
        self.file_nav.clear();
        self.marquee_rect = Recti::ZERO;
    }

    pub fn set_marquee_rect(&mut self, rect: Recti) {
        // check marquee rect exceed image bounds
        if let Some(asset) = &self.asset {
            let spec = asset.image().spec();
            let img_rect = Recti::from_min_size(vec2i(0, 0), vec2i(spec.width, spec.height));
            let r = rect.validate().intersect(img_rect);
            self.marquee_rect = r;
        } else {
            self.marquee_rect = Recti::ZERO;
        }
    }

    pub fn validate_marquee_rect(&mut self) {
        if let Some(asset) = &self.asset {
            let spec = asset.image().spec();
            let img_rect = Recti::from_min_size(vec2i(0, 0), vec2i(spec.width, spec.height));
            self.marquee_rect = self.marquee_rect.validate().intersect(img_rect);
        } else {
            self.marquee_rect = Recti::ZERO;
        }
    }

    pub fn reset_marquee_rect(&mut self) {
        self.marquee_rect = Recti::ZERO;
    }

    pub fn navigate_next(&mut self) -> Result<Option<PathBuf>> {
        if let Some(asset) = &self.asset {
            match asset.asset_type() {
                AssetType::File => {
                    if let Some(path) = self.file_nav.navigate_next() {
                        self.load_from_path(path.clone())?;
                        return Ok(Some(path));
                    }
                }
                _ => return Ok(None),
            }
        }
        Ok(None)
    }

    pub fn navigate_prev(&mut self) -> Result<Option<PathBuf>> {
        if let Some(asset) = &self.asset {
            match asset.asset_type() {
                AssetType::File => {
                    if let Some(path) = self.file_nav.navigate_prev() {
                        self.load_from_path(path.clone())?;
                        return Ok(Some(path));
                    }
                }
                _ => return Ok(None),
            }
        }
        Ok(None)
    }

    pub fn navigate_asset_next(&mut self) -> bool {
        let Some(current_hash) = self.asset_primary.as_ref().map(|asset| asset.hash().to_string()) else {
            return false;
        };
        if self.assets.is_empty() {
            return false;
        }

        let Some(current_index) = self.assets.get_index_of(&current_hash) else {
            return false;
        };
        let next_index = (current_index + 1) % self.assets.len();
        let Some((next_hash, _)) = self.assets.get_index(next_index) else {
            return false;
        };
        let next_hash = next_hash.clone();

        if next_hash == current_hash {
            return false;
        }

        self.set_asset_primary_by_hash(&next_hash);
        true
    }

    pub fn navigate_asset_prev(&mut self) -> bool {
        let Some(current_hash) = self.asset_primary.as_ref().map(|asset| asset.hash().to_string()) else {
            return false;
        };
        if self.assets.is_empty() {
            return false;
        }

        let Some(current_index) = self.assets.get_index_of(&current_hash) else {
            return false;
        };
        let prev_index = if current_index == 0 {
            self.assets.len() - 1
        } else {
            current_index - 1
        };
        let Some((prev_hash, _)) = self.assets.get_index(prev_index) else {
            return false;
        };
        let prev_hash = prev_hash.clone();

        if prev_hash == current_hash {
            return false;
        }

        self.set_asset_primary_by_hash(&prev_hash);
        true
    }

    pub fn process_watcher_events(&mut self) {
        self.file_nav.process_watcher_events();
        // Keep current index in sync when list commits
        if let Some(cur_path) = self.path.clone() {
            // Ensures index stays aligned if list changed.
            self.file_nav.select_index_for_path(&cur_path);
        }
    }
}
