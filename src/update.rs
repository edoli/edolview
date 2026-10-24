use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::{fs::File, io::Write};

use serde::Deserialize;
use sha2::{Digest, Sha256};

#[cfg(not(target_os = "windows"))]
use crate::util::path_ext::safe_temp_dir;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
use std::time::Instant;

#[cfg(target_os = "windows")]
use crate::{
    util::path_ext::app_config_dir,
    windows_install::{install_scope, InstallScope, RESULT_FILE},
};

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

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
    #[cfg(target_os = "windows")]
    handoff: Option<InnoHandoff>,
}

impl PreparedUpdate {
    pub fn commit(self) -> Result<String, String> {
        #[cfg(target_os = "windows")]
        if let Some(mut handoff) = self.handoff {
            handoff.commit()?;
        }
        Ok(self.message)
    }
}

pub fn start_update(update: &AvailableUpdate) -> Result<PreparedUpdate, String> {
    let current_exe = env::var_os("APPIMAGE")
        .map(PathBuf::from)
        .map_or_else(env::current_exe, Ok)
        .map_err(|e| format!("Failed to locate current executable: {e}"))?;
    #[cfg(target_os = "windows")]
    if is_legacy_machine_install(&current_exe) && install_scope(&current_exe).is_none() {
        return Err("This copy uses the old all-users installation. Uninstall it with administrator permission, then install the new Inno Setup installer from the releases page.".to_string());
    }
    let temp_dir = create_update_temp_dir()?;
    let asset_path = temp_dir.join(if update.target == UpdateTarget::WindowsInno {
        "setup.exe"
    } else {
        update.asset_name.as_str()
    });

    download_asset(update, &asset_path)?;
    #[cfg(target_os = "windows")]
    let handoff = if update.target == UpdateTarget::WindowsInno {
        let scope = install_scope(&current_exe)
            .ok_or("This copy is not registered with Inno Setup. Reinstall it using the new installer.")?;
        Some(prepare_inno_handoff(
            &current_exe,
            &temp_dir,
            scope,
            &update.version,
            &app_config_dir(),
        )?)
    } else {
        launch_update_helper(update.target, &current_exe, &asset_path)?;
        None
    };
    #[cfg(not(target_os = "windows"))]
    launch_update_helper(update.target, &current_exe, &asset_path)?;

    #[cfg(target_os = "windows")]
    let message = format!(
        "Downloaded {}. Edolview will close, update silently, and reopen automatically.",
        update.version
    );

    #[cfg(not(target_os = "windows"))]
    let message = format!(
        "Downloaded {}. Edolview will now close, and a separate updater window will stay visible while the {} is applied.",
        update.version,
        update.target.label()
    );
    Ok(PreparedUpdate {
        message,
        #[cfg(target_os = "windows")]
        handoff,
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
        if exe.starts_with(Path::new("/opt/edolview")) {
            return Ok(UpdateTarget::LinuxDeb);
        }
        return Ok(UpdateTarget::PortableZip);
    }

    #[allow(unreachable_code)]
    Err("Automatic updates are not supported on this platform".to_string())
}

#[cfg(target_os = "windows")]
fn is_legacy_machine_install(exe: &Path) -> bool {
    // Preserve detection of existing releases while requiring a one-time migration.
    let path = exe.to_string_lossy().replace('/', "\\").to_ascii_lowercase();
    path.contains("\\program files\\") || path.contains("\\program files (x86)\\")
}

#[cfg(target_os = "windows")]
fn windows_update_target(exe: &Path) -> UpdateTarget {
    if install_scope(exe).is_some() || is_legacy_machine_install(exe) {
        UpdateTarget::WindowsInno
    } else {
        UpdateTarget::PortableZip
    }
}

pub fn take_update_failure() -> Result<Option<String>, String> {
    #[cfg(target_os = "windows")]
    {
        return read_update_failure(&app_config_dir().join(RESULT_FILE));
    }
    #[cfg(not(target_os = "windows"))]
    Ok(None)
}

#[cfg(target_os = "windows")]
fn read_update_failure(path: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(result) => {
            fs::remove_file(path).map_err(|e| format!("Failed to clear update result: {e}"))?;
            Ok(result.strip_prefix("ERROR\n").map(|error| error.trim().to_string()))
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

#[cfg(target_os = "windows")]
struct InnoHandoff {
    directory: PathBuf,
    child: Option<std::process::Child>,
    committed: bool,
}

#[cfg(target_os = "windows")]
impl Drop for InnoHandoff {
    fn drop(&mut self) {
        if !self.committed {
            if let Some(child) = self.child.as_mut() {
                let _ = child.kill();
                let _ = child.wait();
            }
            for file in ["setup.exe", "update.ps1", "ready", "go.pending", "go"] {
                let _ = fs::remove_file(self.directory.join(file));
            }
            let _ = fs::remove_dir(&self.directory);
        }
    }
}

#[cfg(target_os = "windows")]
impl InnoHandoff {
    fn commit(&mut self) -> Result<(), String> {
        if self
            .child
            .as_mut()
            .ok_or("Update helper was not started")?
            .try_wait()
            .map_err(|e| format!("Failed to read helper status: {e}"))?
            .is_some()
        {
            return Err("Update helper exited before Edolview was ready to close. Try updating again.".into());
        }
        fs::write(self.directory.join("go.pending"), b"install")
            .and_then(|_| fs::rename(self.directory.join("go.pending"), self.directory.join("go")))
            .map_err(|e| format!("Failed to hand off update: {e}"))?;
        self.committed = true;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn prepare_inno_handoff(
    exe: &Path,
    directory: &Path,
    scope: InstallScope,
    version: &str,
    config: &Path,
) -> Result<InnoHandoff, String> {
    let mut handoff = InnoHandoff {
        directory: directory.into(),
        child: None,
        committed: false,
    };
    let script = directory.join("update.ps1");
    fs::write(&script, include_str!("../installer/update.ps1"))
        .map_err(|e| format!("Failed to write update helper: {e}"))?;
    let system = env::var_os("SystemRoot").ok_or("Cannot find Windows PowerShell")?;
    let shell = PathBuf::from(system).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    // The helper is constant; paths and release data are passed as data through
    // environment variables, including paths containing Unicode and quotes.
    let child = Command::new(shell)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            "& ([ScriptBlock]::Create([IO.File]::ReadAllText($env:EDOLVIEW_UPDATE_SCRIPT)))",
        ])
        .env("EDOLVIEW_UPDATE_SCRIPT", &script)
        .env("EDOLVIEW_UPDATE_DIR", directory)
        .env("EDOLVIEW_UPDATE_EXE", exe)
        .env(
            "EDOLVIEW_UPDATE_SCOPE",
            if scope == InstallScope::Machine {
                "machine"
            } else {
                "user"
            },
        )
        // Release labels include `v`; the helper and executable metadata use numeric versions.
        .env("EDOLVIEW_UPDATE_VERSION", version.trim().trim_start_matches(['v', 'V']))
        .env("EDOLVIEW_UPDATE_PARENT", std::process::id().to_string())
        .env("EDOLVIEW_CONFIG_DIR", config)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("Failed to start update helper: {e}"))?;
    handoff.child = Some(child);
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if directory.join("ready").is_file() {
            return Ok(handoff);
        }
        if handoff
            .child
            .as_mut()
            .unwrap()
            .try_wait()
            .map_err(|e| format!("Failed to read helper status: {e}"))?
            .is_some()
        {
            return Err(read_update_failure(&config.join(RESULT_FILE))?
                .unwrap_or_else(|| "Update helper exited before it was ready".into()));
        }
        if Instant::now() >= deadline {
            return Err("Update helper readiness timed out".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn launch_update_helper(target: UpdateTarget, current_exe: &Path, asset_path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let script_path = asset_path.parent().unwrap_or_else(|| Path::new(".")).join("apply_update.ps1");
        write_windows_update_script(&script_path, target, current_exe, asset_path)?;
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Sta",
                "-WindowStyle",
                "Hidden",
                "-File",
            ])
            .arg(&script_path)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Failed to launch update helper: {e}"))?;
        return Ok(());
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let script_path = asset_path.parent().unwrap_or_else(|| Path::new(".")).join("apply_update.sh");
        write_unix_update_script(&script_path, target, current_exe, asset_path)?;
        let launch_cmd = format!(
            "nohup sh {} >/dev/null 2>&1 &",
            sh_quote(script_path.to_string_lossy().as_ref())
        );
        Command::new("sh")
            .arg("-c")
            .arg(launch_cmd)
            .spawn()
            .map_err(|e| format!("Failed to launch update helper: {e}"))?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err("Automatic updates are not supported on this platform".to_string())
}

#[cfg(target_os = "windows")]
fn write_windows_update_script(
    path: &Path,
    target: UpdateTarget,
    current_exe: &Path,
    asset_path: &Path,
) -> Result<(), String> {
    let pid = std::process::id();

    let script = windows_update_script(target, current_exe, asset_path, pid, &app_config_dir().join(RESULT_FILE))?;

    // Windows PowerShell 5 reads .ps1 files as the ANSI code page without a BOM.
    fs::write(path, format!("\u{feff}{script}")).map_err(|e| format!("Failed to write update helper script: {e}"))
}

#[cfg(target_os = "windows")]
fn windows_update_script(
    target: UpdateTarget,
    current_exe: &Path,
    asset_path: &Path,
    pid: u32,
    error_path: &Path,
) -> Result<String, String> {
    let current_dir = current_exe
        .parent()
        .ok_or_else(|| "Current executable has no parent directory".to_string())?;
    let install = match target {
        UpdateTarget::PortableZip => {
            r#"
    $extractDir = Join-Path (Split-Path $assetPath -Parent) 'portable'
    Expand-Archive -LiteralPath $assetPath -DestinationPath $extractDir -Force
    Copy-Item -LiteralPath (Join-Path $extractDir 'edolview.exe') -Destination $targetExe -Force
    Copy-Item -LiteralPath (Join-Path $extractDir 'colormap') -Destination $targetDir -Recurse -Force
"#
        }
        _ => return Err("Unsupported Windows update target".to_string()),
    };
    Ok(format!(
        r#"$ErrorActionPreference = 'Stop'
$assetPath = {asset_path}
$targetExe = {target_exe}
$targetDir = {target_dir}
$errorPath = {error_path}
$logPath = Join-Path (Split-Path $assetPath -Parent) 'installer.log'
$utf8 = New-Object System.Text.UTF8Encoding($false)
try {{
    $pidToWait = {pid}
    while (Get-Process -Id $pidToWait -ErrorAction SilentlyContinue) {{
        Start-Sleep -Milliseconds 300
    }}
    Remove-Item -LiteralPath $errorPath -Force -ErrorAction SilentlyContinue
{install}
    if (-not (Test-Path -LiteralPath $targetExe)) {{
        throw "Updated Edolview executable was not found at $targetExe."
    }}
}} catch {{
    $message = "ERROR`nEdolview update failed: $($_.Exception.Message)"
    [System.IO.File]::WriteAllText($errorPath, $message, $utf8)
    [Console]::Error.WriteLine($message)
}}
# Reopen the existing app on failure too, so its normal toast flow reports the error.
if (Test-Path -LiteralPath $targetExe) {{
    try {{
        Start-Process -FilePath $targetExe
    }} catch {{
        $message = "ERROR`nCould not reopen Edolview after updating: $($_.Exception.Message)"
        [System.IO.File]::WriteAllText($errorPath, $message, $utf8)
        [Console]::Error.WriteLine($message)
    }}
}}
"#,
        asset_path = ps_quote_path(asset_path),
        target_exe = ps_quote_path(current_exe),
        target_dir = ps_quote_string(current_dir.to_string_lossy().trim_end_matches(['\\', '/'])),
        error_path = ps_quote_path(error_path),
    ))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn write_unix_update_script(
    path: &Path,
    target: UpdateTarget,
    current_exe: &Path,
    asset_path: &Path,
) -> Result<(), String> {
    let pid = std::process::id();
    let status_message = sh_quote(status_message(target));
    let script = match target {
        UpdateTarget::PortableZip => {
            let current_dir = current_exe
                .parent()
                .ok_or_else(|| "Current executable has no parent directory".to_string())?;
            format!(
                r#"#!/bin/sh
set -eu
STATUS_PID=""
STATUS_MESSAGE={status_message}
start_status() {{
    if command -v zenity >/dev/null 2>&1; then
        zenity --info --title="Updating Edolview" --text="$STATUS_MESSAGE" --no-wrap --timeout=600 >/dev/null 2>&1 &
        STATUS_PID=$!
    elif command -v osascript >/dev/null 2>&1; then
        osascript -e 'on run argv' -e 'display dialog (item 1 of argv) with title "Updating Edolview" buttons {{}} giving up after 600' -e 'end run' "$STATUS_MESSAGE" >/dev/null 2>&1 &
        STATUS_PID=$!
    elif command -v notify-send >/dev/null 2>&1; then
        notify-send "Updating Edolview" "$STATUS_MESSAGE"
    fi
}}
stop_status() {{
    if [ -n "${{STATUS_PID:-}}" ]; then
        kill "$STATUS_PID" 2>/dev/null || true
    fi
}}
trap 'stop_status' EXIT
start_status
PID={pid}
while kill -0 "$PID" 2>/dev/null; do
    sleep 0.3
done
ASSET={asset}
TARGET_DIR={target_dir}
EXTRACT_DIR="$(dirname "$ASSET")/portable"
PYTHON_BIN="$(command -v python3 || command -v python || true)"
if [ -z "$PYTHON_BIN" ]; then
    exit 1
fi
rm -rf "$EXTRACT_DIR"
mkdir -p "$EXTRACT_DIR"
"$PYTHON_BIN" -c "import sys, zipfile; zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])" "$ASSET" "$EXTRACT_DIR"
rm -f "$TARGET_DIR/edolview"
rm -rf "$TARGET_DIR/colormap"
cp "$EXTRACT_DIR/edolview" "$TARGET_DIR/edolview"
chmod +x "$TARGET_DIR/edolview"
cp -R "$EXTRACT_DIR/colormap" "$TARGET_DIR/colormap"
nohup "$TARGET_DIR/edolview" >/dev/null 2>&1 &
"#,
                status_message = status_message,
                asset = sh_quote(asset_path.to_string_lossy().as_ref()),
                target_dir = sh_quote(current_dir.to_string_lossy().as_ref()),
            )
        }
        UpdateTarget::MacDmg => {
            let app_bundle =
                app_bundle_path(current_exe).ok_or_else(|| "Failed to locate current app bundle".to_string())?;
            format!(
                r#"#!/bin/sh
set -eu
STATUS_PID=""
STATUS_MESSAGE={status_message}
start_status() {{
    if command -v osascript >/dev/null 2>&1; then
        osascript -e 'on run argv' -e 'display dialog (item 1 of argv) with title "Updating Edolview" buttons {{}} giving up after 600' -e 'end run' "$STATUS_MESSAGE" >/dev/null 2>&1 &
        STATUS_PID=$!
    elif command -v zenity >/dev/null 2>&1; then
        zenity --info --title="Updating Edolview" --text="$STATUS_MESSAGE" --no-wrap --timeout=600 >/dev/null 2>&1 &
        STATUS_PID=$!
    elif command -v notify-send >/dev/null 2>&1; then
        notify-send "Updating Edolview" "$STATUS_MESSAGE"
    fi
}}
stop_status() {{
    if [ -n "${{STATUS_PID:-}}" ]; then
        kill "$STATUS_PID" 2>/dev/null || true
    fi
}}
trap 'stop_status' EXIT
start_status
PID={pid}
while kill -0 "$PID" 2>/dev/null; do
    sleep 0.3
done
ASSET={asset}
APP_BUNDLE={app_bundle}
MOUNT_DIR="$(dirname "$ASSET")/mount"
rm -rf "$MOUNT_DIR"
mkdir -p "$MOUNT_DIR"
hdiutil attach -nobrowse -quiet -mountpoint "$MOUNT_DIR" "$ASSET"
SOURCE_APP="$(find "$MOUNT_DIR" -maxdepth 1 -name '*.app' -print -quit)"
if [ -z "$SOURCE_APP" ]; then
    hdiutil detach "$MOUNT_DIR" -quiet || true
    exit 1
fi
rm -rf "$APP_BUNDLE"
ditto "$SOURCE_APP" "$APP_BUNDLE"
hdiutil detach "$MOUNT_DIR" -quiet || hdiutil detach "$MOUNT_DIR" -force -quiet || true
open "$APP_BUNDLE"
"#,
                status_message = status_message,
                asset = sh_quote(asset_path.to_string_lossy().as_ref()),
                app_bundle = sh_quote(app_bundle.to_string_lossy().as_ref()),
            )
        }
        UpdateTarget::LinuxDeb => format!(
            r#"#!/bin/sh
set -eu
STATUS_PID=""
STATUS_MESSAGE={status_message}
start_status() {{
    if command -v zenity >/dev/null 2>&1; then
        zenity --info --title="Updating Edolview" --text="$STATUS_MESSAGE" --no-wrap --timeout=600 >/dev/null 2>&1 &
        STATUS_PID=$!
    elif command -v notify-send >/dev/null 2>&1; then
        notify-send "Updating Edolview" "$STATUS_MESSAGE"
    fi
}}
stop_status() {{
    if [ -n "${{STATUS_PID:-}}" ]; then
        kill "$STATUS_PID" 2>/dev/null || true
    fi
}}
trap 'stop_status' EXIT
start_status
PID={pid}
while kill -0 "$PID" 2>/dev/null; do
    sleep 0.3
done
ASSET={asset}
CURRENT_EXE={current_exe}
if command -v pkexec >/dev/null 2>&1; then
    pkexec /usr/bin/dpkg -i "$ASSET"
    nohup "$CURRENT_EXE" >/dev/null 2>&1 &
elif command -v xdg-open >/dev/null 2>&1; then
    nohup xdg-open "$ASSET" >/dev/null 2>&1 &
else
    exit 1
fi
"#,
            status_message = status_message,
            asset = sh_quote(asset_path.to_string_lossy().as_ref()),
            current_exe = sh_quote(current_exe.to_string_lossy().as_ref()),
        ),
        UpdateTarget::LinuxAppImage => format!(
            r#"#!/bin/sh
set -eu
STATUS_PID=""
STATUS_MESSAGE={status_message}
start_status() {{
    if command -v zenity >/dev/null 2>&1; then
        zenity --info --title="Updating Edolview" --text="$STATUS_MESSAGE" --no-wrap --timeout=600 >/dev/null 2>&1 &
        STATUS_PID=$!
    elif command -v notify-send >/dev/null 2>&1; then
        notify-send "Updating Edolview" "$STATUS_MESSAGE"
    fi
}}
stop_status() {{
    if [ -n "${{STATUS_PID:-}}" ]; then
        kill "$STATUS_PID" 2>/dev/null || true
    fi
}}
trap 'stop_status' EXIT
start_status
PID={pid}
while kill -0 "$PID" 2>/dev/null; do
    sleep 0.3
done
ASSET={asset}
CURRENT_EXE={current_exe}
install -m 755 "$ASSET" "$CURRENT_EXE"
nohup "$CURRENT_EXE" >/dev/null 2>&1 &
"#,
            status_message = status_message,
            asset = sh_quote(asset_path.to_string_lossy().as_ref()),
            current_exe = sh_quote(current_exe.to_string_lossy().as_ref()),
        ),
        _ => return Err("Unsupported Unix update target".to_string()),
    };

    let mut file = File::create(path).map_err(|e| format!("Failed to create update helper script: {e}"))?;
    file.write_all(script.as_bytes())
        .map_err(|e| format!("Failed to write update helper script: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file
            .metadata()
            .map_err(|e| format!("Failed to read update helper metadata: {e}"))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).map_err(|e| format!("Failed to mark update helper executable: {e}"))?;
    }

    Ok(())
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

#[cfg(target_os = "windows")]
fn ps_quote_path(path: &Path) -> String {
    let escaped = path.to_string_lossy().replace('\'', "''");
    format!("'{escaped}'")
}

#[cfg(target_os = "windows")]
fn ps_quote_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn status_message(target: UpdateTarget) -> &'static str {
    match target {
        UpdateTarget::PortableZip => "Edolview is installing the downloaded update and will reopen automatically.",
        UpdateTarget::WindowsInno => {
            "Edolview is installing the update silently and will reopen automatically when it finishes."
        }
        UpdateTarget::MacDmg => {
            "Edolview is replacing the installed app bundle and will reopen automatically when it finishes."
        }
        UpdateTarget::LinuxDeb => {
            "Edolview is handing the update over to the system package installer. When supported by the package-manager flow, it will reopen automatically after the install finishes."
        }
        UpdateTarget::LinuxAppImage => "Edolview is replacing the AppImage and will reopen automatically.",
    }
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
        fs::write(directory.join("setup.exe"), b"hello").unwrap();
        // Exercise the same display label that check_for_update passes to start_update.
        let version = normalize_version_label("1.0.2");
        let handoff = prepare_inno_handoff(&exe, &directory, InstallScope::User, &version, &config).unwrap();
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
        fs::write(directory.join("setup.exe"), b"hello").unwrap();
        let mut handoff = prepare_inno_handoff(&exe, &directory, InstallScope::User, "1.0.2", &config).unwrap();
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
        assert!(prepare_inno_handoff(&exe, &directory, InstallScope::User, "1.0.2", &config).is_err());
        assert!(!directory.exists());
        assert!(exe.is_file()); // Parent remains alive; no installer was launched.
        fs::remove_dir_all(root).unwrap();
    }
}
