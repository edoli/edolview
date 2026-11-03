use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Read a copied absolute path or file URI without treating ordinary text as a file.
pub fn clipboard_text_to_path(text: &str) -> Option<PathBuf> {
    let text = text.trim();
    // Explorer's "Copy as path" surrounds the path with double quotes.
    let text = text.strip_prefix('"').and_then(|text| text.strip_suffix('"')).unwrap_or(text);
    let path = PathBuf::from(text);
    if path.is_absolute() {
        // Literal paths may contain '%' and '#'; only decode escapes in URIs.
        return Some(path);
    }
    url::Url::parse(text).ok()?.to_file_path().ok()
}

pub fn is_ascii_path(path: &Path) -> bool {
    match path.to_str() {
        Some(s) => s.is_ascii(),
        None => false,
    }
}

/// Crossplatform safe temporary directory path
/// 1. If std::env::temp_dir() is representable in ASCII, use it
/// 2. Otherwise, fallback to environment variables / hardcoded ASCII paths
pub fn safe_temp_dir() -> PathBuf {
    let tmp = env::temp_dir();

    if is_ascii_path(&tmp) {
        return tmp;
    }

    // 2. fallback candidates: (allow user override)
    //    - RUST_SAFE_TEMP (custom)
    //    - TMPDIR / TMP / TEMP (platform conventions)
    let candidates = [
        env::var_os("RUST_SAFE_TEMP"),
        env::var_os("TMPDIR"),
        env::var_os("TMP"),
        env::var_os("TEMP"),
    ];

    for cand in candidates.iter().flatten() {
        let p = PathBuf::from(cand);
        if is_ascii_path(&p) {
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }

    // 3. Last : completely hardcoded ASCII path
    #[cfg(windows)]
    let p = PathBuf::from(r"C:\Temp");

    #[cfg(not(windows))]
    let p = PathBuf::from("/tmp");

    let _ = fs::create_dir_all(&p);
    p
}

pub fn exe_dir_or_cwd() -> PathBuf {
    if let Ok(exe_path) = env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            return dir.to_path_buf();
        }
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn app_config_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(dir) = env::var_os("LOCALAPPDATA") {
            let path = PathBuf::from(dir).join("Edolview");
            let _ = fs::create_dir_all(&path);
            return path;
        }
        if let Some(dir) = env::var_os("APPDATA") {
            let path = PathBuf::from(dir).join("Edolview");
            let _ = fs::create_dir_all(&path);
            return path;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = env::var_os("HOME") {
            let path = PathBuf::from(home).join("Library").join("Application Support").join("Edolview");
            let _ = fs::create_dir_all(&path);
            return path;
        }
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        if let Some(dir) = env::var_os("XDG_CONFIG_HOME") {
            let path = PathBuf::from(dir).join("edolview");
            let _ = fs::create_dir_all(&path);
            return path;
        }
        if let Some(home) = env::var_os("HOME") {
            let path = PathBuf::from(home).join(".config").join("edolview");
            let _ = fs::create_dir_all(&path);
            return path;
        }
    }

    let path = safe_temp_dir().join("edolview-config");
    let _ = fs::create_dir_all(&path);
    path
}

pub fn colormap_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    if let Some(resources) = macos_bundle_resources() {
        return resources.join("colormap");
    }
    #[cfg(target_os = "linux")]
    if env::current_exe().is_ok_and(|exe| exe == Path::new("/usr/bin/edolview")) {
        return PathBuf::from("/usr/share/edolview/colormap");
    }
    exe_dir_or_cwd().join("colormap")
}

pub fn custom_colormap_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    if macos_bundle_resources().is_some() {
        return app_config_dir().join("colormap");
    }
    #[cfg(target_os = "linux")]
    if env::current_exe().is_ok_and(|exe| exe == Path::new("/usr/bin/edolview")) {
        return app_config_dir().join("colormap");
    }
    colormap_dir()
}

#[cfg(target_os = "macos")]
fn macos_bundle_resources() -> Option<PathBuf> {
    let executable_dir = exe_dir_or_cwd();
    let contents = executable_dir.parent()?;
    let bundle = contents.parent()?;
    (executable_dir.file_name()? == "MacOS" && contents.file_name()? == "Contents" && bundle.extension()? == "app")
        .then(|| contents.join("Resources"))
}

pub fn colormap_file(relative: &Path) -> PathBuf {
    let custom = custom_colormap_dir().join(relative);
    if custom.is_file() {
        custom
    } else {
        colormap_dir().join(relative)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uri_decodes_escaped_path_characters() {
        let path = if cfg!(windows) {
            PathBuf::from("D:/images/한 글/100% #1.png")
        } else {
            PathBuf::from("/images/한 글/100% #1.png")
        };
        let uri = url::Url::from_file_path(&path).unwrap();
        assert_eq!(clipboard_text_to_path(&format!("  {uri}\r\n")), Some(path));
    }

    #[cfg(windows)]
    #[test]
    fn file_uri_opens_windows_drive_and_unc_paths() {
        assert_eq!(
            clipboard_text_to_path("file:///D:/workspace/hg-hsi-line-spread-correction/artifacts/line_spread/endpoint_investigation/244_common_ghost/assets/ghost_3709547.png"),
            Some(PathBuf::from(r"D:\workspace\hg-hsi-line-spread-correction\artifacts\line_spread\endpoint_investigation\244_common_ghost\assets\ghost_3709547.png"))
        );
        assert_eq!(
            clipboard_text_to_path("file://server/share/image%20one.png"),
            Some(PathBuf::from(r"\\server\share\image one.png"))
        );
    }

    #[test]
    fn other_clipboard_content_is_not_a_local_path() {
        for text in [
            "",
            "hello",
            "https://example.com/image.png",
            "data:image/png;base64,AA==",
            "images/example.png",
            "D:example.png",
        ] {
            assert_eq!(clipboard_text_to_path(text), None, "{text}");
        }
    }

    #[test]
    fn clipboard_absolute_path_preserves_literal_characters_and_quotes() {
        let text = if cfg!(windows) {
            "D:/images/한 글/100%20 #1.png"
        } else {
            "/images/한 글/100%20 #1.png"
        };
        assert_eq!(clipboard_text_to_path(text), Some(PathBuf::from(text)));
        assert_eq!(clipboard_text_to_path(&format!(" \"{text}\"\r\n")), Some(PathBuf::from(text)));
    }

    #[cfg(windows)]
    #[test]
    fn clipboard_windows_paths_accept_both_separators_and_unc() {
        for text in [
            "D:/workspace/hg-hsi-line-spread-correction/artifacts/line_spread/endpoint_investigation/244_common_ghost/assets/ghost_3709547.png",
            r"D:\images\image.png",
            r"\\server\share\image one.png",
        ] {
            assert_eq!(clipboard_text_to_path(text), Some(PathBuf::from(text)));
        }
    }
}
