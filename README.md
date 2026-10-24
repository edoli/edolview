# EdolView - Image Viewer for Visualization and Analysis

A lightning-fast, versatile image viewer 🚀
Designed for researchers, engineers, and graphics professionals who need **speed, flexibility, and powerful visualization**.


* ⚡ **Blazing Fast Startup & I/O**: Instant startup and rapid image loading without delays
* 📂 **Wide Format Support**: Open and explore all major research and [graphics formats](#supported-file-formats)
* 🎨 **Colormap Visualization**: Go beyond raw images with rich colormap-based data visualization
* 🔧 **Flexible Display Controls**: Fine-tune normalization, exposure, gamma, and offset with ease

![Screenshot](images/screenshot.jpg)

## Installation

- __Windows__:
  - [edolview-windows.zip](https://github.com/edoli/edolview/releases/latest/download/edolview-windows.zip)
  - [edolview-installer-exe.exe](https://github.com/edoli/edolview/releases/latest/download/edolview-installer-exe.exe)
- __macOS__:
  - [edolview-mac.zip](https://github.com/edoli/edolview/releases/latest/download/edolview-mac.zip)
  - [edolview-installer-dmg.dmg](https://github.com/edoli/edolview/releases/latest/download/edolview-installer-dmg.dmg)
- __Linux__:
  - [edolview-linux.zip](https://github.com/edoli/edolview/releases/latest/download/edolview-linux.zip)
  - [edolview-installer-appimage.AppImage](https://github.com/edoli/edolview/releases/latest/download/edolview-installer-appimage.AppImage)
  - [edolview-installer-deb.deb](https://github.com/edoli/edolview/releases/latest/download/edolview-installer-deb.deb)

## Usage

### Shortcuts

* **Navigation**
  * `←` / `→` : navigate image files in current directory
  * `page up` / `page down` : navigate loaded assets
  * `+` / `-`, `scroll` : zoom in / out
  * `Home` : reset view
  * `f11` : fullscreen mode

* **Selection**
  * `shift` + `drag` : select rectangle region
  * `ctrl` / `cmd` + `a` : select full image
  * `esc` : deselect region
  * `ctrl` / `cmd` + `c` : copy image or selected region
  * `ctrl` / `cmd` + `s` : save image or selected region

* **ROIs**
  * `r` : toggle ROIs window
  * `ctrl` / `cmd` + `r` : add or remove the current selection as an ROI
  * `[` / `]` : jump to previous / next ROI

* **Image List**
  * Rows selected for batch deletion have a purple fill and outline.
  * `shift` + click selects a range; `ctrl` / `cmd` + `shift` + click adds a range.
  * Right-click a selected row and choose **Delete** to remove the selected images from the list.
  * Click an image name to view it and clear the batch selection.
  * `ctrl` / `cmd` + click an image name selects the secondary image for comparison.

* **View Presets**
  * `ctrl` / `cmd` + `1` .. `9` : save current view preset to slot 1 .. 9
  * `alt` + `1` .. `9` : apply view preset from slot 1 .. 9

### Features

* Color maps: **Edolview** supports various color maps. To get more information of color maps, see the wiki page: https://github.com/edoli/edolview/wiki/Color-Map
 
* **Remote Viewer**: **Edolview** can also be controlled remotely over the network. The current protocol is implemented in the [Python package](https://pypi.org/project/edolview/) and the [VS Code extension](https://marketplace.visualstudio.com/items?itemName=edolview-vscode.edolview). See [Remote-Viewer](https://github.com/edoli/edolview/wiki/Remote%E2%80%90Viewer).

* **Copy-Paste Selected Region**

https://github.com/user-attachments/assets/fd76159c-da77-4f5d-946c-667cda5c1482

* **Plot Selected Region**

https://github.com/user-attachments/assets/4a219f8b-39f3-48a8-a5ea-b9d610bb3f40


## How to Build

The project is Rust-based and uses **image-rs 0.25.10**, the pure-Rust decoder.
Optional HEIF/HEIC support can be enabled with `--features heif` after preparing `libheif`.
Optional AVIF decoding can be enabled with `--features avif` after preparing the native `dav1d` library required by image-rs.

### 0) Prerequisites (all platforms)
* Rust


### 1) Install Rust

**Linux / macOS**

```bash
# installs rustup + stable toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# then reload shell or:
source "$HOME/.cargo/env"

# Check installation
rustc -V && cargo -V
```

**Windows (PowerShell)**

```powershell
# Option A: winget
winget install Rustlang.Rustup

# Option B: official installer
# https://win.rustup.rs/x86_64 (downloads rustup-init.exe)

# Check installation
rustc -V; cargo -V
```

### 2) Optional HEIF/HEIC support

Install and build the vcpkg dependencies before using `--features heif`:

```bash
cargo install cargo-vcpkg
cargo vcpkg -v build
```

On Linux and macOS, add the vcpkg pkg-config directory before building with `--features heif`:

```bash
# Linux x86_64
export PKG_CONFIG_PATH="$(pwd)/target/vcpkg/installed/x64-linux-release/lib/pkgconfig:${PKG_CONFIG_PATH:-}"

# macOS arm64
export PKG_CONFIG_PATH="$(pwd)/target/vcpkg/installed/arm64-osx-release/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
```

Windows uses the vcpkg triplet from `Cargo.toml` and does not need an extra `PKG_CONFIG_PATH` setting.

### 3) Build the viewer

```bash
# generate resources for the app
cargo run -p xtask -- icons
# from project root
cargo build --release
# enable optional HEIF/HEIC support when libheif is available
# cargo run -p xtask --features heif -- icons
# cargo build --release --features heif
# or run directly
cargo run
```

**Built executable files path**

* Linux/macOS: `target/release/edolview`
* Windows: `target/release/edolview.exe`

---

### Troubleshooting

* HEIF build errors → verify that the vcpkg `libheif` installation for the current target is available.

## Supported file formats
EdolView accepts the following file extensions in the current build:

* Common image formats: `png`, `apng`, `jpg`, `jpeg`, `jpe`, `jfif`, `jp2`, `j2k`, `j2c`, `jpc`, `jpf`, `bmp`, `dib`, `tif`, `tiff`, `webp`, `gif`, `tga`, `ico`, `qoi`, `ff`
* HDR and analysis formats: `exr`, `hdr`, `pic`, `pfm`, `flo`
* Netpbm family: `pbm`, `pgm`, `ppm`, `pnm`, `pxm`, `pam`
* Optional AVIF support: `avif`
* Optional HEIF support: `heic`, `heif`

## License
EdolView is available under the MIT license.

## 배포

GitHub Actions의 `.github/workflows/ci.yml`은 main push, main 대상 PR, 수동 실행과 `v*` 태그 push에서 동작합니다. CI가 Cargo 버전의 숫자 세 부분(각 0~65535, 선행 0 없음)을 검사하고 태그가 있으면 정확히 `v<Cargo 버전>`인지 확인합니다. 버전 검사 → stable Rust의 `release-dist` 빌드 → OS별 패키징을 수행하며 태그 실행만 GitHub Release를 공개합니다.

Windows x64, macOS Apple Silicon, Linux x64를 배포합니다. Windows CRT는 정적으로 링크하고 Linux 앱 라이브러리는 정적으로 묶되 GUI·GPU·libc 등 시스템 라이브러리는 사용합니다. macOS 앱은 인증서 없이 ad-hoc 서명하며 Apple 공증은 수행하지 않습니다.

파일명은 `edolview-windows.zip`, `edolview-mac.zip`, `edolview-linux.zip`, `edolview-installer-exe.exe`, `edolview-installer-dmg.dmg`, `edolview-installer-deb.deb`, `edolview-installer-appimage.AppImage`으로 고정합니다. Windows/Linux ZIP은 실행 파일과 필수 asset만, macOS ZIP은 `.app` 번들을 포함합니다. 최종 배포 파일의 `SHA256SUMS.txt`를 발행하며 모든 OS의 필수 파일이 준비되어야 Release를 공개합니다. 이전 버전을 재배포해도 더 높은 버전의 latest를 대체하지 않습니다.

앱 업데이트는 GitHub Releases API에서 대상 파일과 SHA-256·크기를 읽어 검증합니다. 설치형은 기존 설치 경로·범위를 유지하고 포터블은 ZIP의 파일을 교체한 뒤 다시 실행합니다. 검증 정보가 없거나 다운로드 검증에 실패하면 업데이트하지 않습니다.

릴리스 준비는 Cargo.toml 버전 변경 → Cargo.lock 갱신 → 검증 → 두 파일을 포함한 커밋 → 같은 버전의 태그 생성 → main과 태그 push 순서로 수행합니다. 별도 버전 변경·커밋·태그 자동화는 없습니다.

로컬 Windows 패키징은 `tools/install_inno.ps1` 후 `tools/package.ps1`로 실행하며 기본적으로 HEIF·AVIF 추가 셋업을 요구하지 않습니다. CI 배포 빌드는 네이티브 코덱을 준비하고 `--features heif,avif`를 사용합니다.
