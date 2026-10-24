use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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
    #[cfg(target_os = "linux")]
    if env::current_exe().is_ok_and(|exe| exe == Path::new("/usr/bin/edolview")) {
        return PathBuf::from("/usr/share/edolview/colormap");
    }
    exe_dir_or_cwd().join("colormap")
}

pub fn custom_colormap_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    if env::current_exe().is_ok_and(|exe| exe == Path::new("/usr/bin/edolview")) {
        return app_config_dir().join("colormap");
    }
    colormap_dir()
}

pub fn colormap_file(relative: &Path) -> PathBuf {
    let custom = custom_colormap_dir().join(relative);
    if custom.is_file() {
        custom
    } else {
        colormap_dir().join(relative)
    }
}
