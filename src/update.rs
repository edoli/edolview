use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::util::path_ext::app_config_dir;
#[cfg(not(target_os = "windows"))]
use crate::util::path_ext::safe_temp_dir;
#[cfg(windows)]
use crate::windows_install::install_scope;
use serde::Deserialize;
use sha2::{Digest, Sha256};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::time::Instant;
const RESULT_FILE: &str = "update-result.txt";
pub const RELEASES_PAGE_URL: &str = "https://github.com/edoli/edolview/releases";
const LATEST_RELEASE_API_URL: &str = "https://api.github.com/repos/edoli/edolview/releases/latest";
const WINDOWS_INSTALLER_ASSET: &str = "edolview-installer-exe.exe";
const CURRENT_VERSION: &str = match option_env!("EDOLVIEW_BUILD_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

pub fn current_version() -> &'static str {
    CURRENT_VERSION
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvailableUpdate {
    pub version: String,
    pub html_url: String,
    pub asset_name: String,
    pub asset_download_url: String,
    pub asset_size: u64,
    pub asset_sha256: String,
    pub target: UpdateTarget,
}

pub struct UpdateCheck {
    pub available: Result<Option<AvailableUpdate>, String>,
    pub previous_failure: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateTarget {
    PortableZip,
    WindowsInno,
    MacDmg,
    LinuxDeb,
    LinuxAppImage,
}

impl UpdateTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::PortableZip => "portable zip",
            Self::WindowsInno => "Inno Setup installer",
            Self::MacDmg => "DMG installer",
            Self::LinuxDeb => "DEB package",
            Self::LinuxAppImage => "AppImage",
        }
    }
}

#[derive(Deserialize)]
struct ReleaseResponse {
    tag_name: String,
    html_url: String,
    assets: Vec<ReleaseAsset>,
}

#[derive(Clone, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    digest: Option<String>,
}

pub fn check_for_update() -> Result<Option<AvailableUpdate>, String> {
    let release = fetch_latest_release()?;
    if !is_newer_version(release.tag_name.as_str(), CURRENT_VERSION) {
        return Ok(None);
    }

    let target = detect_update_target()?;
    let asset_name = preferred_asset_name(target);
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| format!("No compatible update asset found for {}", target.label()))?;
    let digest = checked_digest(asset.size, asset.digest.as_deref())?;

    Ok(Some(AvailableUpdate {
        version: normalize_version_label(release.tag_name.as_str()),
        html_url: release.html_url,
        asset_name: asset.name.clone(),
        asset_download_url: asset.browser_download_url.clone(),
        asset_size: asset.size,
        asset_sha256: digest,
        target,
    }))
}

/// Owns the ready helper until the UI is ready to close. Dropping a queued
/// update cancels it without allowing the installer to run.
pub struct PreparedUpdate {
    message: String,
    handoff: Option<UpdateHandoff>,
}

impl PreparedUpdate {
    pub fn commit(self) -> Result<String, String> {
        if let Some(mut handoff) = self.handoff {
            handoff.commit()?;
        }
        Ok(self.message)
    }
}

pub fn start_update(update: &AvailableUpdate) -> Result<PreparedUpdate, String> {
    let current_exe = if cfg!(target_os = "linux") {
        env::var_os("APPIMAGE").map(PathBuf::from).map_or_else(env::current_exe, Ok)
    } else {
        env::current_exe()
    }
    .map_err(|e| format!("Failed to locate current executable: {e}"))?;
    let temp_dir = create_update_temp_dir()?;
    let asset_path = temp_dir.join(&update.asset_name);
    download_asset(update, &asset_path)?;
    let handoff = prepare_handoff(
        &current_exe,
        &temp_dir,
        &update.asset_name,
        &update.version,
        &update.asset_sha256,
        &app_config_dir(),
    )?;
    Ok(PreparedUpdate {
        message: format!(
            "Downloaded {}. Edolview will close, update, and reopen automatically.",
            update.version
        ),
        handoff: Some(handoff),
    })
}

fn http_agent(seconds: u64) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(seconds)))
        .build()
        .new_agent()
}

fn fetch_latest_release() -> Result<ReleaseResponse, String> {
    http_agent(20)
        .get(LATEST_RELEASE_API_URL)
        .header("User-Agent", "edolview")
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("Failed to query GitHub releases: {e}"))?
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("Failed to read GitHub release response: {e}"))
        .and_then(|body| {
            serde_json::from_str::<ReleaseResponse>(&body)
                .map_err(|e| format!("Failed to parse GitHub release response: {e}"))
        })
}

fn checked_digest(size: u64, digest: Option<&str>) -> Result<String, String> {
    if size == 0 || size > 256 * 1024 * 1024 {
        return Err("Invalid update asset size".to_string());
    }
    let digest = digest
        .and_then(|value| value.strip_prefix("sha256:"))
        .filter(|value| value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or("GitHub asset has no valid SHA-256 digest")?;
    Ok(digest.to_ascii_lowercase())
}

fn transfer_verified(
    mut input: impl io::Read,
    output: &mut impl io::Write,
    size: u64,
    digest: &str,
) -> Result<(), String> {
    let mut hash = Sha256::new();
    let mut received = 0;
    let mut buffer = [0; 64 * 1024];
    loop {
        let count = input.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        received += count as u64;
        if received > size {
            return Err("Update asset exceeds its advertised size".into());
        }
        output.write_all(&buffer[..count]).map_err(|e| e.to_string())?;
        hash.update(&buffer[..count]);
    }
    if received != size || format!("{:x}", hash.finalize()) != digest {
        return Err("Update asset size or SHA-256 does not match GitHub metadata".into());
    }
    Ok(())
}

fn download_asset(update: &AvailableUpdate, destination: &Path) -> Result<(), String> {
    let mut response = http_agent(300)
        .get(&update.asset_download_url)
        .header("User-Agent", "edolview")
        .call()
        .map_err(|e| format!("Failed to download update asset: {e}"))?;
    let mut asset = fs::File::create(destination).map_err(|e| format!("Failed to create update asset: {e}"))?;
    let result = transfer_verified(
        response.body_mut().as_reader(),
        &mut asset,
        update.asset_size,
        &update.asset_sha256,
    );
    drop(asset);
    if result.is_err() {
        let _ = fs::remove_file(destination);
    }
    result
}

fn detect_update_target() -> Result<UpdateTarget, String> {
    let exe = env::current_exe().map_err(|e| format!("Failed to locate current executable: {e}"))?;

    #[cfg(target_os = "windows")]
    {
        return Ok(windows_update_target(&exe));
    }

    #[cfg(target_os = "macos")]
    {
        if app_bundle_path(&exe).is_some() {
            return Ok(UpdateTarget::MacDmg);
        }
        return Ok(UpdateTarget::PortableZip);
    }

    #[cfg(target_os = "linux")]
    {
        if env::var_os("APPIMAGE").is_some() || exe.extension().and_then(|ext| ext.to_str()) == Some("AppImage") {
            return Ok(UpdateTarget::LinuxAppImage);
        }
        if exe == Path::new("/usr/bin/edolview") || exe.starts_with(Path::new("/opt/edolview")) {
            return Ok(UpdateTarget::LinuxDeb);
        }
        return Ok(UpdateTarget::PortableZip);
    }

    #[allow(unreachable_code)]
    Err("Automatic updates are not supported on this platform".to_string())
}

#[cfg(windows)]
fn windows_update_target(exe: &Path) -> UpdateTarget {
    if install_scope(exe).is_some() {
        UpdateTarget::WindowsInno
    } else {
        UpdateTarget::PortableZip
    }
}

pub fn take_update_failure() -> Result<Option<String>, String> {
    read_update_failure(&app_config_dir().join(RESULT_FILE))
}

fn read_update_failure(path: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(result) => {
            fs::remove_file(path).map_err(|e| format!("Failed to clear update result: {e}"))?;
            let message = result.strip_prefix("ERROR\n").unwrap_or(&result).trim();
            Ok((!message.is_empty()).then(|| message.chars().take(4096).collect()))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("Failed to read update result: {error}")),
    }
}

fn preferred_asset_name(target: UpdateTarget) -> &'static str {
    match target {
        UpdateTarget::PortableZip => {
            #[cfg(target_os = "windows")]
            {
                "edolview-windows.zip"
            }
            #[cfg(target_os = "macos")]
            {
                "edolview-mac.zip"
            }
            #[cfg(target_os = "linux")]
            {
                "edolview-linux.zip"
            }
        }
        UpdateTarget::WindowsInno => WINDOWS_INSTALLER_ASSET,
        UpdateTarget::MacDmg => "edolview-installer-dmg.dmg",
        UpdateTarget::LinuxDeb => "edolview-installer-deb.deb",
        UpdateTarget::LinuxAppImage => "edolview-installer-appimage.AppImage",
    }
}

fn create_update_temp_dir() -> Result<PathBuf, String> {
    static NEXT_UPDATE_ID: AtomicU64 = AtomicU64::new(0);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to compute update temp timestamp: {e}"))?
        .as_millis();
    // PowerShell supports Unicode paths; keep updates in the user's writable
    // temporary directory rather than falling back to a system-wide ASCII path.
    #[cfg(target_os = "windows")]
    let temp_dir = env::temp_dir();
    #[cfg(not(target_os = "windows"))]
    let temp_dir = safe_temp_dir();
    let id = NEXT_UPDATE_ID.fetch_add(1, Ordering::Relaxed);
    let dir = temp_dir.join(format!("edolview-update-{}-{millis}-{id}", std::process::id()));
    fs::create_dir(&dir).map_err(|e| format!("Failed to create update temp directory: {e}"))?;
    Ok(dir)
}

struct UpdateHandoff {
    directory: PathBuf,
    child: Option<std::process::Child>,
    committed: bool,
}

impl Drop for UpdateHandoff {
    fn drop(&mut self) {
        if !self.committed {
            if let Some(child) = &mut self.child {
                let _ = child.kill();
                let _ = child.wait();
            }
            let _ = fs::remove_dir_all(&self.directory);
        }
    }
}

impl UpdateHandoff {
    fn commit(&mut self) -> Result<(), String> {
        if self.child.as_mut().unwrap().try_wait().map_err(|e| e.to_string())?.is_some() {
            return Err("Update helper exited before Edolview was ready to close".into());
        }
        fs::write(self.directory.join("go.pending"), b"install").map_err(|e| e.to_string())?;
        fs::rename(self.directory.join("go.pending"), self.directory.join("go")).map_err(|e| e.to_string())?;
        self.committed = true;
        Ok(())
    }
}

fn prepare_handoff(
    exe: &Path,
    directory: &Path,
    asset: &str,
    version: &str,
    hash: &str,
    config: &Path,
) -> Result<UpdateHandoff, String> {
    let mut handoff = UpdateHandoff {
        directory: directory.into(),
        child: None,
        committed: false,
    };
    fs::create_dir_all(config).map_err(|e| e.to_string())?;
    let version = version.trim().trim_start_matches(['v', 'V']);
    let package = directory.join(asset);
    fs::write(
        directory.join("context.json"),
        serde_json::to_vec(&serde_json::json!({
            "executable": exe, "package": package, "version": version, "sha256": hash,
            "parent": std::process::id(), "arguments": env::args_os().skip(1).collect::<Vec<_>>(), "config": config,
        }))
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    #[cfg(windows)]
    let mut command =
        {
            fs::write(directory.join("update.ps1"), include_str!("../installer/update.ps1"))
                .map_err(|e| e.to_string())?;
            let shell = PathBuf::from(env::var_os("SystemRoot").ok_or("Cannot find Windows PowerShell")?)
                .join("System32/WindowsPowerShell/v1.0/powershell.exe");
            let mut command = Command::new(shell);
            command.args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command",
            "& ([ScriptBlock]::Create([IO.File]::ReadAllText((Join-Path $env:EDOLVIEW_UPDATE_DIR 'update.ps1'))))"])
            .creation_flags(0x08000000);
            command
        };
    #[cfg(unix)]
    let mut command = {
        fs::write(directory.join("update.sh"), include_str!("../installer/update.sh")).map_err(|e| e.to_string())?;
        fs::write(
            directory.join("restart.sh"),
            format!(
                "#!/bin/sh\nexec {}\n",
                sh_quote(if asset == "edolview-installer-deb.deb" {
                    "/usr/bin/edolview"
                } else {
                    exe.to_str().ok_or("Invalid executable path")?
                })
            ),
        )
        .map_err(|e| e.to_string())?;
        let mut command = Command::new("sh");
        command
            .arg(directory.join("update.sh"))
            .env("EDOLVIEW_UPDATE_EXE", exe)
            .env("EDOLVIEW_UPDATE_PACKAGE", &package)
            .env("EDOLVIEW_UPDATE_VERSION", version)
            .env("EDOLVIEW_UPDATE_HASH", hash)
            .env("EDOLVIEW_UPDATE_PARENT", std::process::id().to_string());
        command
    };
    let child = command
        .env("EDOLVIEW_UPDATE_DIR", directory)
        .env("EDOLVIEW_CONFIG_DIR", config)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start update helper: {e}"))?;
    handoff.child = Some(child);
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if handoff.child.as_mut().unwrap().try_wait().map_err(|e| e.to_string())?.is_some() {
            return Err(read_update_failure(&config.join(RESULT_FILE))?
                .unwrap_or_else(|| "Update helper exited before it was ready".into()));
        }
        if directory.join("ready").is_file() {
            return Ok(handoff);
        }
        if Instant::now() >= deadline {
            return Err("Update helper readiness timed out".into());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn normalize_version_label(version: &str) -> String {
    let trimmed = version.trim();
    if trimmed.starts_with(['v', 'V']) {
        trimmed.to_string()
    } else {
        format!("v{trimmed}")
    }
}

fn is_newer_version(candidate: &str, current: &str) -> bool {
    match (parse_version(candidate), parse_version(current)) {
        (Some(candidate_parts), Some(current_parts)) => compare_versions(&candidate_parts, &current_parts).is_gt(),
        _ => normalize_version_label(candidate) != normalize_version_label(current),
    }
}

fn parse_version(version: &str) -> Option<Vec<u64>> {
    let core = version.trim().trim_start_matches(['v', 'V']).split(['-', '+']).next()?;

    let mut parts = Vec::new();
    for part in core.split('.') {
        if part.is_empty() {
            return None;
        }
        parts.push(part.parse::<u64>().ok()?);
    }
    Some(parts)
}

fn compare_versions(left: &[u64], right: &[u64]) -> std::cmp::Ordering {
    let max_len = left.len().max(right.len());
    for idx in 0..max_len {
        let lhs = *left.get(idx).unwrap_or(&0);
        let rhs = *right.get(idx).unwrap_or(&0);
        match lhs.cmp(&rhs) {
            std::cmp::Ordering::Equal => {}
            non_eq => return non_eq,
        }
    }
    std::cmp::Ordering::Equal
}

#[cfg(unix)]
fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(target_os = "macos")]
fn app_bundle_path(path: &Path) -> Option<PathBuf> {
    path.ancestors().find_map(|ancestor| {
        let ext = ancestor.extension()?.to_str()?;
        if ext.eq_ignore_ascii_case("app") {
            Some(ancestor.to_path_buf())
        } else {
            None
        }
    })
}

#[cfg(not(target_os = "macos"))]
fn app_bundle_path(_path: &Path) -> Option<PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use super::is_newer_version;

    #[test]
    fn update_assets_require_size_and_sha256_and_reject_corruption() {
        use super::*;
        let digest = format!("{:x}", Sha256::digest(b"abc"));
        assert!(checked_digest(3, Some(&format!("sha256:{digest}"))).is_ok());
        assert!(checked_digest(3, None).is_err());
        assert!(checked_digest(0, Some(&format!("sha256:{digest}"))).is_err());
        assert!(checked_digest(3, Some("sha256:bad")).is_err());
        assert!(transfer_verified(&b"abc"[..], &mut Vec::new(), 3, &digest).is_ok());
        for input in [&b"ab"[..], &b"abd"[..], &b"abcd"[..]] {
            assert!(transfer_verified(input, &mut Vec::new(), 3, &digest).is_err());
        }
    }

    #[test]
    fn detects_newer_semver_versions() {
        assert!(is_newer_version("v0.4.20", "0.4.19"));
        assert!(is_newer_version("0.5.0", "0.4.20"));
        assert!(is_newer_version("0.4.20.1", "0.4.20"));
    }

    #[test]
    fn ignores_same_or_older_versions() {
        assert!(!is_newer_version("v0.4.20", "0.4.20"));
        assert!(!is_newer_version("0.4.19", "0.4.20"));
        assert!(!is_newer_version("0.4", "0.4.0"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn inno_helper_waits_for_handoff_and_cancels_cleanly() {
        use super::*;
        let root = create_update_temp_dir().unwrap();
        let unicode = root.join("space 한글 O'Brien");
        let config = unicode.join("config");
        fs::create_dir_all(&config).unwrap();
        let exe = env::current_exe().unwrap();
        let directory = unicode.join("edolview-update-1-2-3");
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("edolview-windows.zip"), b"hello").unwrap();
        // Exercise the same display label that check_for_update passes to start_update.
        let version = normalize_version_label("1.0.2");
        let handoff = prepare_handoff(
            &exe,
            &directory,
            "edolview-windows.zip",
            &version,
            &format!("{:x}", Sha256::digest(b"hello")),
            &config,
        )
        .unwrap();
        // The worker may finish while the UI is blocked in a native dialog.
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(PreparedUpdate {
            message: "ready".into(),
            handoff: Some(handoff),
        })
        .ok()
        .unwrap();
        assert!(directory.join("ready").is_file());
        assert!(!directory.join("go").exists());
        drop(rx); // Closing the app before consuming the result cancels it.
        assert!(!directory.exists());

        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("edolview-windows.zip"), b"hello").unwrap();
        let mut handoff = prepare_handoff(
            &exe,
            &directory,
            "edolview-windows.zip",
            "1.0.2",
            &format!("{:x}", Sha256::digest(b"hello")),
            &config,
        )
        .unwrap();
        let child = handoff.child.as_mut().unwrap();
        child.kill().unwrap();
        child.wait().unwrap();
        let prepared = PreparedUpdate {
            message: "ready".into(),
            handoff: Some(handoff),
        };
        assert!(prepared.commit().is_err()); // A stale result must not close the app.
        assert!(!directory.exists());

        fs::create_dir(&directory).unwrap();
        // A missing download must fail before the app closes.
        assert!(prepare_handoff(
            &exe,
            &directory,
            "edolview-windows.zip",
            "1.0.2",
            &format!("{:x}", Sha256::digest(b"hello")),
            &config
        )
        .is_err());
        assert!(!directory.exists());
        assert!(exe.is_file()); // Parent remains alive; no installer was launched.
        fs::remove_dir_all(root).unwrap();
    }
}
