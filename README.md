# Pluto Browser (`pluto-browser`)

A lightweight, orbital-fast desktop web browser written in **Rust** with a **100% Native Rust GUI** (`softbuffer` + `ab_glyph` + pure-Rust SDF vector compositor) and isolated OS WebView tabs (`wry` + `tao`).

No HTML/CSS/JS chrome files, no Electron wrapper, and no Node.js build pipeline. Every pixel of the browser UI, tab bar, omnibox, find bar, Pluto Native Inspector, and animated Planet Pluto orbital system is rendered directly from Rust, while external websites run cleanly inside lazily spawned OS WebView panes.

---

## Architecture Highlights & Measured Footprint

| Metric / Attribute | Pluto Browser (Rust Native UI + Lazy Wry) | Google Chrome | Brave Browser | Arc Desktop |
|---|---|---|---|---|
| **UI Layer** | 100% Pure Rust (`softbuffer` + `ab_glyph` + SDF) | Views + WebUI | Views + WebUI | Swift + Chromium |
| **HTML / CSS / JS UI Files** | **0 files** | Bundled WebUI | Bundled WebUI | Bundled WebUI |
| **Release Binary Size** | ~1.18 MB (`opt-level="s"`, LTO, `strip`) | ~110 MB | ~125 MB | ~240 MB |
| **Startup Internal Route** | ~25.6 MB working set / **0 child WebView processes** | ~320 MB idle RAM | ~290 MB idle RAM | ~480 MB idle RAM |
| **External Tab Engine** | Lazy `Option<WebView>` (`WebView2` / `WebKitGTK` / `WKWebView`) | Bundled Blink | Bundled Blink | Bundled Blink |
| **Background Tab Trimming** | `MemoryUsageLevel::Low` on Windows tab switch | Heavy multi-process | Heavy multi-process | High idle footprint |

---

## Features

- **100% Native Rust Chrome & Pages**:
  - **Direct Zero-Copy Framebuffer Rendering (`src/ui/canvas.rs`)**: Writes anti-aliased SDF rounded rectangles, 3D-shaded spheres, tilted 3D elliptical orbital rings, crisp vector toolbar icons, and glyph-cached OpenType system fonts (`ab_glyph`) straight into the `softbuffer` pixel buffer.
  - **Damage-Rect Partial Presentation**: On `pluto://welcome` and `pluto://newtab`, Pluto Browser presents the entire window from Rust with **zero child WebView processes**. When browsing an external website (`https://...`), `softbuffer::Buffer::present_with_damage` blits only the top bar (and bottom inspector dock if open), preventing any flicker over the child WebView viewport.
- **Planet Pluto Orbital Showcase (`pluto://welcome`)**:
  - Real-time ~60 FPS Planet Pluto & Charon orbital centerpiece with Tombaugh Regio heart basin, twinkling parallax starfield, Notion-inspired Light/Dark toggle, interactive workspace tabs (`Workspace Board`, `Rust Engine`, `Shortcuts`), 4-tier benchmark comparison, and startup destination picker.
- **Native New Tab Workspace (`pluto://newtab`)**:
  - Live Google Search/URL box, category-filterable Quick Links (`All`, `Engineering`, `Reference`), clickable Saved Bookmarks cards with remove buttons, **Recent History** list (`history.json`) with one-click clear, and an editable persistent Workspace Notes scratchpad stored in `settings.json`.
- **Tabbed Workflow, Bookmarks Bar & Command Palette (`Ctrl+K`)**:
  - Open (`Ctrl+T`), close (`Ctrl+W` / middle-click), duplicate (`Ctrl+Shift+K`), switch (`Ctrl+Tab` / `Ctrl+Shift+Tab`), pin (double-click or right-click tab), and reopen closed tabs (`Ctrl+Shift+T`).
  - Toggleable **Bookmarks Bar (`Ctrl+Shift+B`)**, quick URL copy button & native OS clipboard `Ctrl+C` / `Ctrl+V` support (`src/clipboard.rs`).
  - Automatic URL vs. **Google Search** query detection (`https://www.google.com/search?q=...`), HTTPS/Pluto/HTTP protocol badge, interactive `- / 100% / +` zoom controls (`Ctrl+=` / `Ctrl+-` / `Ctrl+0` / `Ctrl+Scroll`) working across both internal Rust views and external tabs, and floating **Command Palette (`Ctrl+K` / `Ctrl+P`)**.
- **Pluto Native Inspector (`Ctrl+Shift+D`) & Find Bar (`Ctrl+F`)**:
  - Pure Rust bottom dock showing real frame render times (`ms`), active WebView isolation status, live DOM/Timing telemetry from external pages, plus `F12` / `Ctrl+Shift+I` for native OS WebView DevTools.
- **Windows Performance & Hibernation Tuning**:
  - GPU rasterization & zero-copy flags (`BROWSER_PERF_ARGS`) and automatic `MemoryUsageLevel::Low` / `MemoryUsageLevel::Normal` memory trimming when switching tabs or visiting internal routes.

---

## Cross-Platform Prerequisites & Setup

Pluto Browser targets **Windows 10/11** as the primary verified platform while keeping UI rendering (`tao` + `softbuffer` + `ab_glyph`) and WebView integration (`wry`) cross-platform friendly for **Linux** and **macOS**.

### 1. Windows 10 / 11 (Verified Target)

- **Rust Toolchain**:
  - **GNU toolchain (no Visual Studio Build Tools required)**:
    ```powershell
    rustup-init.exe -y --default-host x86_64-pc-windows-gnu --default-toolchain stable
    ```
  - **MSVC toolchain**:
    Install Visual Studio 2022 Build Tools with the **Desktop development with C++** workload.
- **WebView2 Runtime**:
  - Pre-installed on Windows 11. On Windows 10, install the [Microsoft Edge WebView2 Evergreen Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) if absent.
- **Automatic DLL Packaging (`build.rs`)**:
  For `x86_64-pc-windows-gnu` builds, `build.rs` automatically locates `WebView2Loader.dll` in your Cargo registry and copies it into `target/debug/` and `target/release/`.

### 2. Linux (Ubuntu / Debian, Fedora, Arch)

On Linux, `tao` + `softbuffer` draw via X11/Wayland and `wry` uses **WebKitGTK (`webkit2gtk-4.1`)**.

- **Ubuntu / Debian**:
  ```bash
  sudo apt update
  sudo apt install -y \
    build-essential \
    pkg-config \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libsoup-3.0-dev \
    libjavascriptcoregtk-4.1-dev
  ```
- **Fedora**:
  ```bash
  sudo dnf install -y \
    gcc-c++ \
    pkg-config \
    gtk3-devel \
    webkit2gtk4.1-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel
  ```
- **Arch Linux**:
  ```bash
  sudo pacman -S --needed \
    base-devel \
    pkgconf \
    gtk3 \
    webkit2gtk-4.1 \
    libayatana-appindicator \
    librsvg
  ```

> **Wayland Note**: If testing child WebView geometry on Wayland, you can compare `GDK_BACKEND=x11 cargo run` against native Wayland compositors.

### 3. macOS (Apple Silicon & Intel)

On macOS, `tao` uses Cocoa/AppKit, `softbuffer` draws via CoreGraphics, and `wry` uses the built-in **WKWebView**.

```bash
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## Build, Run & Test

```bash
# Clone the repository
git clone https://github.com/<your-username>/pluto-browser.git
cd pluto-browser

# Run in debug mode
cargo run

# Run unit tests
cargo test

# Build size-optimized stripped release binary (~1.18 MB)
cargo build --release
```

### Binary Output Paths

| OS | Release Executable |
|---|---|
| **Windows** | `target\release\pluto-browser.exe` |
| **Linux** | `target/release/pluto-browser` |
| **macOS** | `target/release/pluto-browser` |

### Persisted App Data Locations

Managed by `StorageManager` in [src/storage.rs](src/storage.rs) via `ProjectDirs::from("app", "pluto", "pluto-browser")`:

| OS | Default Data Directory |
|---|---|
| **Windows** | `%APPDATA%\pluto\pluto-browser\data\` |
| **Linux** | `$XDG_DATA_HOME/pluto-browser/` or `~/.local/share/pluto-browser/` |
| **macOS** | `~/Library/Application Support/app.pluto.pluto-browser/` |

---

## Keyboard Shortcuts

| Windows / Linux | macOS | Action |
|---|---|---|
| `Ctrl+T` | `Cmd+T` | Open a new tab (`pluto://newtab`) |
| `Ctrl+W` | `Cmd+W` | Close the active tab |
| `Ctrl+Shift+K` | `Cmd+Shift+K` | Duplicate the active tab |
| `Ctrl+Shift+T` | `Cmd+Shift+T` | Reopen the most recently closed tab |
| `Ctrl+K` / `Ctrl+P` | `Cmd+K` / `Cmd+P` | Open the Command Palette |
| `Ctrl+Shift+B` | `Cmd+Shift+B` | Toggle the Bookmarks Bar |
| `Ctrl+L` | `Cmd+L` | Focus and select the omnibox |
| `Ctrl+R` / `F5` | `Cmd+R` / `F5` | Reload the active tab |
| `Ctrl+Tab` | `Ctrl+Tab` | Switch to the next tab |
| `Ctrl+Shift+Tab` | `Ctrl+Shift+Tab` | Switch to the previous tab |
| `Alt+Left` | `Alt+Left` | Navigate back |
| `Alt+Right` | `Alt+Right` | Navigate forward |
| `Ctrl+F` | `Cmd+F` | Toggle the Find in Page bar |
| `Ctrl+Shift+D` / `Ctrl+Shift+C` | `Cmd+Shift+D` / `Cmd+Shift+C` | Toggle Pluto Native Inspector Dock |
| `F12` / `Ctrl+Shift+I` | `F12` / `Cmd+Shift+I` | Open OS WebView DevTools |
| `Ctrl+=` / `Ctrl+-` / `Ctrl+0` | `Cmd+=` / `Cmd+-` / `Cmd+0` | Zoom in / Zoom out / Reset zoom (also `Ctrl+Scroll`) |

---

## Project Structure

```text
pluto-browser/
├── Cargo.toml           # Crate metadata (pluto-browser) & LTO/strip release profile
├── Cargo.lock           # Pinned dependencies for reproducible builds
├── build.rs             # Auto-copies WebView2Loader.dll for Windows GNU targets
├── .gitignore           # Cross-platform Rust, OS, IDE, and browser state ignores
├── README.md
└── src/
    ├── main.rs          # tao EventLoop, adaptive ~60 FPS / idle loop, procedural Pluto RGBA icon
    ├── app.rs           # AppState coordinator, softbuffer Surface + present_with_damage, IPC/UI dispatch
    ├── clipboard.rs     # Zero-dependency native OS clipboard (Win32 UTF-16 + macOS/Linux tools)
    ├── tabs.rs          # TabManager owning lazy Option<WebView>, zoom clamping, memory trimming & bridge
    ├── ipc.rs           # Typed serde messages (TabIpcMessage) & Google/URL omnibox resolver
    ├── storage.rs       # BOM-tolerant JSON persistence (bookmarks, history, session, settings)
    └── ui/
        ├── mod.rs       # 100% Pure Rust NativeUi: Chrome, Find Bar, Inspector, Welcome & New Tab views
        ├── canvas.rs    # Zero-copy pixel Canvas, cached ab_glyph fonts, SDF vector icons & Pluto hero system
        └── theme.rs     # Notion-inspired Light & Dark ThemePalette color tokens
```

---

## Multi-Platform Roadmap & Contributing

1. **Linux Verification**: Test `WebKitGTK 4.1` child WebView resize bounds and Wayland/X11 DPI scaling on Ubuntu/Fedora/Arch.
2. **macOS Integration**: Add Cocoa native app menu shortcuts (`Cmd+Q`, `Cmd+C`, `Cmd+V`, `Cmd+H`) and `.app` bundling.
3. **Pure Rust Rendering Upgrades**: Add clipboard support (`arboard`) and continue migrating browser capabilities into pure Rust modules without HTML chrome.

### First-Time Push to GitHub

```bash
git init
git add .
git commit -m "chore: initial Pluto Browser release with 100% native Rust UI"
git branch -M main
git remote add origin https://github.com/<your-username>/pluto-browser.git
git push -u origin main
```
