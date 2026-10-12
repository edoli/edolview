use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::{fs::File, io::Write};

use serde::Deserialize;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(target_os = "windows")]
use std::{
    os::windows::process::CommandExt,
    time::{Duration, Instant},
};
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(not(target_os = "windows"))]
use crate::util::path_ext::safe_temp_dir;

pub const RELEASES_PAGE_URL: &str = "https://github.com/edoli/edolview-rs/releases";
const LATEST_RELEASE_API_URL: &str = "https://api.github.com/repos/edoli/edolview-rs/releases?per_page=100";
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
    pub target: UpdateTarget,
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
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

#[derive(Clone, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[cfg(target_os = "windows")]
const RESULT_FILE: &str = "update-result.txt";
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

pub fn check_for_update() -> Result<Option<AvailableUpdate>, String> {
    #[cfg(target_os = "windows")]
    if let Some(error) = read_update_failure(&crate::util::path_ext::app_config_dir().join(RESULT_FILE))? {
        return Err(error);
    }
    let release = fetch_latest_release()?;
    if !is_newer_version(release.tag_name.as_str(), CURRENT_VERSION) {
        return Ok(None);
    }

    let target = detect_update_target()?;
    let asset_name = preferred_asset_name(target);
    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| format!("No compatible update asset found for {}", target.label()))?;

    Ok(Some(AvailableUpdate {
        version: normalize_version_label(release.tag_name.as_str()),
        html_url: release.html_url,
        asset_name: asset.name,
        asset_download_url: asset.browser_download_url,
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
    let current_exe = env::current_exe().map_err(|e| format!("Failed to locate current executable: {e}"))?;
    let temp_dir = create_update_temp_dir()?;
    let asset_path = temp_dir.join(if update.target == UpdateTarget::WindowsInno {
        "setup.exe"
    } else {
        update.asset_name.as_str()
    });

    download_asset(update.asset_download_url.as_str(), &asset_path)?;
    #[cfg(target_os = "windows")]
    let handoff = if update.target == UpdateTarget::WindowsInno {
        Some(prepare_inno_handoff(
            &current_exe,
            &temp_dir,
            &update.version,
            &crate::util::path_ext::app_config_dir(),
        )?)
    } else {
        launch_update_helper(update.target, &current_exe, &asset_path)?;
        None
    };
    #[cfg(not(target_os = "windows"))]
    launch_update_helper(update.target, &current_exe, &asset_path)?;

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

fn fetch_latest_release() -> Result<ReleaseResponse, String> {
    ureq::get(LATEST_RELEASE_API_URL)
        .header("User-Agent", "edolview-rs")
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("Failed to query GitHub releases: {e}"))?
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("Failed to read GitHub release response: {e}"))
        .and_then(|body| {
            serde_json::from_str::<Vec<ReleaseResponse>>(&body)
                .map_err(|e| format!("Failed to parse GitHub release response: {e}"))
                .and_then(select_latest_release)
        })
}

// The legacy v1.0.2 updater needs v1.0.3 marked Latest, while the bridge
// discovers v1.1.0 using published versions rather than that label.
fn select_latest_release(releases: Vec<ReleaseResponse>) -> Result<ReleaseResponse, String> {
    releases
        .into_iter()
        .filter(|r| !r.draft && !r.prerelease && parse_version(&r.tag_name).is_some())
        .max_by(|a, b| compare_versions(&parse_version(&a.tag_name).unwrap(), &parse_version(&b.tag_name).unwrap()))
        .ok_or_else(|| "No published stable release was found".to_string())
}

fn download_asset(url: &str, destination: &Path) -> Result<(), String> {
    let mut response = ureq::get(url)
        .header("User-Agent", "edolview-rs")
        .call()
        .map_err(|e| format!("Failed to download update asset: {e}"))?;
    let mut asset = fs::File::create(destination).map_err(|e| format!("Failed to create update asset: {e}"))?;
    let mut reader = response.body_mut().as_reader();

    io::copy(&mut reader, &mut asset)
        .map(|_| ())
        .map_err(|e| format!("Failed to save update asset: {e}"))
}

fn detect_update_target() -> Result<UpdateTarget, String> {
    let exe = env::current_exe().map_err(|e| format!("Failed to locate current executable: {e}"))?;

    #[cfg(target_os = "windows")]
    {
        let exe_lower = exe.to_string_lossy().to_ascii_lowercase();
        if exe_lower.contains("\\program files\\") {
            return Ok(UpdateTarget::WindowsInno);
        }
        return Ok(UpdateTarget::PortableZip);
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
        if exe.extension().and_then(|ext| ext.to_str()) == Some("AppImage") {
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
        UpdateTarget::WindowsInno => "edolview-installer-exe.exe",
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
fn prepare_inno_handoff(exe: &Path, directory: &Path, version: &str, config: &Path) -> Result<InnoHandoff, String> {
    let mut handoff = InnoHandoff {
        directory: directory.into(),
        child: None,
        committed: false,
    };
    let script = directory.join("update.ps1");
    fs::write(&script, include_str!("../installer/msi-to-inno.ps1"))
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
        .env("EDOLVIEW_UPDATE_VERSION", version.trim_start_matches(['v', 'V']))
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
                "-ExecutionPolicy",
                "Bypass",
                "-Sta",
                "-WindowStyle",
                "Hidden",
                "-File",
            ])
            .arg(&script_path)
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
    let current_dir = current_exe
        .parent()
        .ok_or_else(|| "Current executable has no parent directory".to_string())?;
    let pid = std::process::id();

    let script = match target {
        UpdateTarget::PortableZip => format!(
            r#"$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$form = New-Object System.Windows.Forms.Form
$form.Text = 'Updating Edolview'
$form.Width = 420
$form.Height = 150
$form.StartPosition = 'CenterScreen'
$form.FormBorderStyle = 'FixedDialog'
$form.ControlBox = $false
$form.TopMost = $true
$label = New-Object System.Windows.Forms.Label
$label.Left = 20
$label.Top = 16
$label.Width = 360
$label.Height = 44
$label.TextAlign = 'MiddleLeft'
$progress = New-Object System.Windows.Forms.ProgressBar
$progress.Left = 20
$progress.Top = 74
$progress.Width = 360
$progress.Height = 18
$progress.Style = 'Marquee'
$progress.MarqueeAnimationSpeed = 30
$form.Controls.Add($label)
$form.Controls.Add($progress)
$form.Show()
function Set-Status([string] $text) {{
    $label.Text = $text
    [System.Windows.Forms.Application]::DoEvents()
}}
Set-Status({status_waiting})
$pidToWait = {pid}
while (Get-Process -Id $pidToWait -ErrorAction SilentlyContinue) {{
    [System.Windows.Forms.Application]::DoEvents()
    Start-Sleep -Milliseconds 300
}}
Set-Status({status_installing})
$assetPath = {asset_path}
$targetDir = {target_dir}
$extractDir = Join-Path (Split-Path $assetPath -Parent) 'portable'
if (Test-Path $extractDir) {{
    Remove-Item $extractDir -Recurse -Force
}}
Expand-Archive -Path $assetPath -DestinationPath $extractDir -Force
Copy-Item (Join-Path $extractDir 'edolview.exe') (Join-Path $targetDir 'edolview.exe') -Force
if (Test-Path (Join-Path $targetDir 'colormap')) {{
    Remove-Item (Join-Path $targetDir 'colormap') -Recurse -Force
}}
Copy-Item (Join-Path $extractDir 'colormap') (Join-Path $targetDir 'colormap') -Recurse -Force
Set-Status({status_restarting})
Start-Process -FilePath (Join-Path $targetDir 'edolview.exe')
$form.Close()
"#,
            status_waiting = ps_quote_string("Waiting for Edolview to close so the update can start..."),
            status_installing = ps_quote_string("Installing the new portable build..."),
            status_restarting = ps_quote_string("Update installed. Restarting Edolview..."),
            asset_path = ps_quote_path(asset_path),
            target_dir = ps_quote_path(current_dir),
        ),
        _ => return Err("Unsupported Windows update target".to_string()),
    };

    fs::write(path, script).map_err(|e| format!("Failed to write update helper script: {e}"))
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
            "Edolview is migrating to the Inno Setup installation and will reopen automatically."
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
    fn bridge_finds_newer_stable_release_even_when_msi_bridge_is_latest() {
        let releases = serde_json::from_str(r#"[
            {"tag_name":"v1.0.3","html_url":"bridge","assets":[]},
            {"tag_name":"v1.1.0","html_url":"inno","assets":[{"name":"edolview-installer-exe.exe","browser_download_url":"download"}]},
            {"tag_name":"v2.0.0","html_url":"draft","assets":[],"draft":true},
            {"tag_name":"v3.0.0-beta","html_url":"pre","assets":[],"prerelease":true}
        ]"#).unwrap();
        let release = super::select_latest_release(releases).unwrap();
        assert_eq!(release.tag_name, "v1.1.0");
        assert_eq!(
            release.assets[0].name,
            super::preferred_asset_name(super::UpdateTarget::WindowsInno)
        );
        assert!(super::is_newer_version(&release.tag_name, "1.0.3"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn migration_refuses_unregistered_parent_before_commit() {
        let directory = super::create_update_temp_dir().unwrap();
        let config = directory.join("config");
        std::fs::create_dir(&config).unwrap();
        std::fs::write(directory.join("setup.exe"), b"not an installer").unwrap();
        let result = super::prepare_inno_handoff(&std::env::current_exe().unwrap(), &directory, "v1.1.0", &config);
        assert!(result
            .err()
            .unwrap()
            .contains("Could not identify this legacy MSI installation"));
        assert!(!directory.join("go").exists());
        assert!(!directory.join("setup.exe").exists());
        std::fs::remove_dir(config).unwrap();
        std::fs::remove_dir(directory).unwrap();
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
}
