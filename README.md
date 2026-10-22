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
