use eframe::egui::{self, FontData};
use std::{collections::HashSet, fs, path::PathBuf, sync::mpsc, thread};

pub(crate) type LoadedFallbackFonts = Vec<(String, FontData)>;

fn collect_fallback_font_sources() -> Vec<(String, PathBuf)> {
    let mut sources = Vec::new();

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            let font_path = dir.join("fallback_font.ttf");
            if font_path.is_file() {
                sources.push(("user_fallback".to_owned(), font_path));
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let fonts_dir = PathBuf::from(std::env::var_os("WINDIR").unwrap_or_else(|| r"C:\Windows".into())).join("Fonts");
        // Prefer Windows 11's system glyphs; MDL2 provides the same caption
        // codepoints on Windows versions without Segoe Fluent Icons.
        sources.push((
            crate::res::icons::WINDOW_ICON_FONT_FAMILY.to_owned(),
            fonts_dir.join("SegoeIcons.ttf"),
        ));
        sources.push((
            crate::res::icons::WINDOW_ICON_FONT_FAMILY.to_owned(),
            fonts_dir.join("segmdl2.ttf"),
        ));
    }
    append_platform_cjk_font_sources(&mut sources);
    sources
}

#[cfg(target_os = "windows")]
fn append_platform_cjk_font_sources(sources: &mut Vec<(String, PathBuf)>) {
    sources.extend([
        ("cjk_kr_malgun".to_owned(), PathBuf::from(r"C:\Windows\Fonts\malgun.ttf")),
        ("cjk_zh_microsoft_yahei".to_owned(), PathBuf::from(r"C:\Windows\Fonts\msyh.ttc")),
        ("cjk_jp_yu_gothic".to_owned(), PathBuf::from(r"C:\Windows\Fonts\YuGothM.ttc")),
        ("cjk_jp_ms_gothic".to_owned(), PathBuf::from(r"C:\Windows\Fonts\msgothic.ttc")),
    ]);
}

#[cfg(target_os = "linux")]
fn append_platform_cjk_font_sources(sources: &mut Vec<(String, PathBuf)>) {
    sources.extend([
        (
            "cjk_noto_sans_cjk".to_owned(),
            PathBuf::from("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
        ),
        (
            "cjk_noto_sans_cjk_alt".to_owned(),
            PathBuf::from("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc"),
        ),
        (
            "cjk_noto_sans_cjk_truetype".to_owned(),
            PathBuf::from("/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc"),
        ),
        (
            "cjk_wqy_microhei".to_owned(),
            PathBuf::from("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc"),
        ),
    ]);
}

#[cfg(target_os = "macos")]
fn append_platform_cjk_font_sources(sources: &mut Vec<(String, PathBuf)>) {
    sources.extend([
        (
            "cjk_ko_apple_sd_gothic_neo".to_owned(),
            PathBuf::from("/System/Library/Fonts/AppleSDGothicNeo.ttc"),
        ),
        (
            "cjk_zh_pingfang".to_owned(),
            PathBuf::from("/System/Library/Fonts/PingFang.ttc"),
        ),
        (
            "cjk_jp_hiragino_sans_w3".to_owned(),
            PathBuf::from("/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc"),
        ),
        (
            "cjk_jp_hiragino_sans_w6".to_owned(),
            PathBuf::from("/System/Library/Fonts/ヒラギノ角ゴシック W6.ttc"),
        ),
    ]);
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn append_platform_cjk_font_sources(_: &mut Vec<(String, PathBuf)>) {}

pub(crate) fn spawn_fallback_font_loader() -> Option<mpsc::Receiver<LoadedFallbackFonts>> {
    let sources = collect_fallback_font_sources();
    if sources.is_empty() {
        return None;
    }

    let (tx_font, rx_font) = mpsc::channel();
    thread::spawn(move || {
        let mut seen_paths = HashSet::new();
        let mut loaded_fonts = Vec::new();
        let mut seen_keys = HashSet::new();

        for (font_key, font_path) in sources {
            if seen_keys.contains(&font_key) || !seen_paths.insert(font_path.clone()) || !font_path.is_file() {
                continue;
            }

            match fs::read(&font_path) {
                Ok(bytes) => {
                    seen_keys.insert(font_key.clone());
                    loaded_fonts.push((font_key, FontData::from_owned(bytes)));
                }
                Err(err) => {
                    eprintln!("Failed to read fallback font '{}': {err}", font_path.display());
                }
            }
        }

        let _ = tx_font.send(loaded_fonts);
    });

    Some(rx_font)
}

pub(crate) fn apply_fallback_fonts(ctx: &egui::Context, fallback_fonts: LoadedFallbackFonts) {
    if fallback_fonts.is_empty() {
        return;
    }

    let mut fonts = egui::FontDefinitions::default();
    let mut font_order = Vec::with_capacity(fallback_fonts.len());

    for (font_key, font_data) in fallback_fonts {
        #[cfg(target_os = "windows")]
        if font_key == crate::res::icons::WINDOW_ICON_FONT_FAMILY {
            fonts
                .families
                .insert(egui::FontFamily::Name(font_key.clone().into()), vec![font_key.clone()]);
            fonts.font_data.insert(font_key, font_data.into());
            continue;
        }
        font_order.push(font_key.clone());
        fonts.font_data.insert(font_key, font_data.into());
    }

    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        family.extend(font_order.iter().cloned());
    }
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        family.extend(font_order);
    }

    ctx.set_fonts(fonts);
}
