"""Package the native release-dist binary. Run on macOS or Linux with Python 3.11+."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import plistlib
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import tomllib
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]
DIST = ROOT / "target/dist"
APP = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["package"]["name"]
NAME = {"edolview": "Edolview", "edolwave": "EdolWave", "edot": "Edot"}[APP]
APP_ID = {"edolview": "kr.edoli.edolview", "edolwave": "dev.edoli.edolwave", "edot": "dev.edoli.edot"}[APP]
ICON = ROOT / ("assets/app-icon/edot.png" if APP == "edot" else "icons/icon.png")
LINUXDEPLOY_URL = "https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-x86_64.AppImage"
LINUXDEPLOY_SHA256 = "c20cd71e3a4e3b80c3483cef793cda3f4e990aca14014d23c544ca3ce1270b4d"


def run(*args: object, **kwargs: object) -> subprocess.CompletedProcess:
    print("+", " ".join(map(str, args)), flush=True)
    return subprocess.run(list(map(str, args)), check=True, **kwargs)


def version() -> str:
    value = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]
    if not re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", value) or any(int(x) > 65535 for x in value.split(".")):
        raise ValueError(f"Unsupported package version: {value}")
    return value


def desktop_entry() -> str:
    if APP == "edolview":
        return (ROOT / "packaging/edolview.desktop").read_text(encoding="utf-8")
    categories = "AudioVideo;Audio;Player;" if APP == "edolwave" else "Office;TextEditor;"
    mime = "MimeType=audio/x-wav;audio/wav;audio/vnd.wave;audio/mpeg;audio/flac;audio/ogg;audio/x-vorbis+ogg;audio/aac;audio/x-aac;audio/mp4;audio/x-m4a;audio/x-m4b;audio/x-aiff;audio/x-caf;audio/x-matroska;\n" if APP == "edolwave" else ""
    arguments = " %F" if APP == "edolwave" else ""
    return (f"[Desktop Entry]\nType=Application\nName={NAME}\nExec={APP}{arguments}\n"
            f"Icon={APP}\nTerminal=false\nCategories={categories}\nStartupWMClass={APP_ID}\n{mime}")


def app_plist(build_version: str) -> dict:
    if APP == "edolview":
        text = (ROOT / "packaging/macos/Info.plist.in").read_text(encoding="utf-8")
        return plistlib.loads(text.replace("__SHORT_VERSION__", build_version).replace("__VERSION__", build_version).encode())
    info = {"CFBundleDevelopmentRegion": "en", "CFBundleDisplayName": NAME,
            "CFBundleName": NAME, "CFBundleIdentifier": APP_ID, "CFBundleExecutable": APP,
            "CFBundlePackageType": "APPL", "CFBundleInfoDictionaryVersion": "6.0",
            "CFBundleIconFile": "app.icns", "CFBundleShortVersionString": build_version,
            "CFBundleVersion": build_version, "NSHighResolutionCapable": True}
    if APP == "edolwave":
        info["LSMinimumSystemVersion"] = "12.0"
        extensions = json.loads((ROOT / "installer/audio-formats.json").read_text())["native"]
        info["CFBundleDocumentTypes"] = [{"CFBundleTypeName": "Supported audio files",
            "CFBundleTypeExtensions": extensions, "CFBundleTypeRole": "Viewer", "LSHandlerRank": "Alternate"}]
    return info


def write_executable(path: Path, contents: str) -> None:
    path.write_text(contents, encoding="utf-8", newline="\n")
    path.chmod(0o755)


def copy_binary(source: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
    destination.chmod(0o755)


def copy_assets(destination: Path) -> None:
    if APP == "edolview":
        shutil.copytree(ROOT / "colormap", destination / "colormap")


def zip_tree(source: Path, destination: Path) -> None:
    with zipfile.ZipFile(destination, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for path in sorted(source.rglob("*")):
            name = path.relative_to(source).as_posix()
            if path.is_symlink():
                entry = zipfile.ZipInfo(name)
                entry.create_system = 3
                entry.external_attr = (stat.S_IFLNK | 0o777) << 16
                archive.writestr(entry, os.readlink(path))
            elif path.is_file():
                archive.write(path, name)


def package_mac(binary: Path, stage: Path, build_version: str) -> list[Path]:
    if platform.machine() != "arm64":
        raise ValueError("macOS distributions target Apple Silicon")
    app = stage / f"{NAME}.app"
    contents = app / "Contents"
    resources = contents / "Resources"
    resources.mkdir(parents=True)
    copy_binary(binary, contents / f"MacOS/{APP}")
    copy_assets(contents / "MacOS")
    info = app_plist(build_version)
    info["CFBundleIconFile"] = "app.icns"
    (contents / "Info.plist").write_bytes(plistlib.dumps(info))
    iconset = stage / "app.iconset"
    iconset.mkdir()
    for size in (16, 32, 128, 256, 512):
        for scale in (1, 2):
            suffix = "@2x" if scale == 2 else ""
            run("sips", "-z", size * scale, size * scale, ICON, "--out", iconset / f"icon_{size}x{size}{suffix}.png")
    run("iconutil", "-c", "icns", iconset, "-o", resources / "app.icns")
    run("codesign", "--force", "--deep", "--sign", "-", app)
    run("codesign", "--verify", "--deep", "--strict", app)
    dependencies = run("otool", "-L", binary, capture_output=True, text=True).stdout
    if "/opt/homebrew/" in dependencies or "/usr/local/" in dependencies:
        raise ValueError(f"Non-system dynamic dependencies:\n{dependencies}")
    archive = DIST / f"{APP}-mac.zip"
    run("ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", app, archive)
    image_root = stage / "dmg"
    image_root.mkdir()
    shutil.copytree(app, image_root / app.name, symlinks=True)
    (image_root / "Applications").symlink_to("/Applications")
    image = DIST / f"{APP}-installer-dmg.dmg"
    run("hdiutil", "create", "-volname", NAME, "-srcfolder", image_root, "-ov", "-format", "UDZO", image)
    run("hdiutil", "verify", image)
    return [archive, image]


def linuxdeploy() -> Path:
    tool = ROOT / "target/tools/linuxdeploy-x86_64.AppImage"
    tool.parent.mkdir(parents=True, exist_ok=True)
    if not tool.exists() or hashlib.sha256(tool.read_bytes()).hexdigest() != LINUXDEPLOY_SHA256:
        with urllib.request.urlopen(LINUXDEPLOY_URL) as response:
            data = response.read()
        if hashlib.sha256(data).hexdigest() != LINUXDEPLOY_SHA256:
            raise ValueError("linuxdeploy checksum mismatch")
        tool.write_bytes(data)
    tool.chmod(0o755)
    return tool


def package_deb(binary: Path, stage: Path, desktop: Path, build_version: str) -> Path:
    destination = DIST / f"{APP}-installer-deb.deb"
    if APP == "edolview":
        run("cargo", "deb", "--no-build", "--profile", "release-dist", "--output", destination, cwd=ROOT)
        return destination
    root = stage / "deb"
    copy_binary(binary, root / f"usr/bin/{APP}")
    applications = root / "usr/share/applications"
    applications.mkdir(parents=True)
    shutil.copy2(desktop, applications / desktop.name)
    icon = root / f"usr/share/icons/hicolor/512x512/apps/{APP}.png"
    icon.parent.mkdir(parents=True)
    shutil.copy2(ICON, icon)
    debian = stage / "debian"
    debian.mkdir()
    (debian / "control").write_text(f"Source: {APP}\nSection: utils\nPriority: optional\nMaintainer: Edoli\n\nPackage: {APP}\nArchitecture: any\nDescription: {NAME}\n")
    scan = run("dpkg-shlibdeps", "-O", f"-e{binary}", cwd=stage, capture_output=True, text=True).stdout
    dependencies = next(line.removeprefix("shlibs:Depends=") for line in scan.splitlines() if line.startswith("shlibs:Depends="))
    # GPUI's GPU backend is loaded at runtime rather than present in DT_NEEDED.
    dependencies += ", libvulkan1, libxkbcommon-x11-0, libfontconfig1, libfreetype6"
    if APP == "edot":
        dependencies += ", libcurl4t64"
    control = root / "DEBIAN"
    control.mkdir()
    (control / "control").write_text(f"Package: {APP}\nVersion: {build_version}\nArchitecture: amd64\nSection: utils\nPriority: optional\nMaintainer: Daniel Jeon <edoli.mia@gmail.com>\nDepends: {dependencies}\nRecommends: xdg-desktop-portal, fonts-noto-cjk\nDescription: {NAME}\n Native desktop application.\n")
    for event in ("postinst", "postrm"):
        write_executable(control / event, "#!/bin/sh\nset -e\nif command -v update-desktop-database >/dev/null 2>&1; then update-desktop-database -q /usr/share/applications || true; fi\nif command -v gtk-update-icon-cache >/dev/null 2>&1; then gtk-update-icon-cache -q -t /usr/share/icons/hicolor || true; fi\n")
    run("dpkg-deb", "--root-owner-group", "--build", root, destination)
    return destination


def package_linux(binary: Path, stage: Path, build_version: str) -> list[Path]:
    if platform.machine() != "x86_64":
        raise ValueError("Linux distributions target x86_64")
    dependencies = run("ldd", binary, capture_output=True, text=True).stdout
    print(dependencies)
    if "not found" in dependencies or re.search(r"lib(heif|dav1d|de265|aom)[^ /]*\.so", dependencies):
        raise ValueError("Missing system library or dynamically linked application codec")
    desktop = stage / f"{APP_ID}.desktop"
    desktop.write_text(desktop_entry(), encoding="utf-8", newline="\n")
    run("desktop-file-validate", desktop)
    deb = package_deb(binary, stage, desktop, build_version)
    portable = stage / "portable"
    copy_binary(binary, portable / APP)
    copy_assets(portable)
    archive = DIST / f"{APP}-linux.zip"
    zip_tree(portable, archive)
    appdir = stage / "AppDir"
    copy_binary(binary, appdir / f"usr/bin/{APP}")
    copy_assets(appdir / "usr/bin")
    write_executable(appdir / "AppRun", f'#!/bin/sh\nAPPDIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"\nexport LD_LIBRARY_PATH="$APPDIR/usr/lib${{LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}}"\nexec "$APPDIR/usr/bin/{APP}" "$@"\n')
    icon = stage / f"{APP}.png"
    shutil.copy2(ICON, icon)
    image = DIST / f"{APP}-installer-appimage.AppImage"
    env = dict(os.environ, APPIMAGE_EXTRACT_AND_RUN="1", VERSION=build_version, ARCH="x86_64", LDAI_OUTPUT=str(image))
    run(linuxdeploy(), "--appdir", appdir, "--executable", appdir / f"usr/bin/{APP}", "--desktop-file", desktop, "--icon-file", icon, "--output", "appimage", env=env, cwd=stage)
    if not image.is_file():
        raise FileNotFoundError(image)
    image.chmod(0o755)
    return [archive, deb, image]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--no-build", action="store_true")
    args = parser.parse_args()
    if sys.platform not in ("darwin", "linux"):
        parser.error("Run on macOS or Linux")
    if not args.no_build:
        run("cargo", "build", "--profile", "release-dist", "--locked", cwd=ROOT)
    binary = ROOT / f"target/release-dist/{APP}"
    DIST.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="package-", dir=ROOT / "target") as temporary:
        outputs = (package_mac if sys.platform == "darwin" else package_linux)(binary, Path(temporary), version())
    suffix = "mac" if sys.platform == "darwin" else "linux"
    checksums = DIST / f"SHA256SUMS-{suffix}.txt"
    checksums.write_text("".join(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n" for path in outputs), encoding="ascii", newline="\n")
    for path in [*outputs, checksums]:
        print(f"{path.name}: {path.stat().st_size:,} bytes")


if __name__ == "__main__":
    main()
