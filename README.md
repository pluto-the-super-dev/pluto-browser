# Pluto Browser (`rust-browser-adit`)

An orbital-fast, lightweight desktop web browser written in **Rust**, built around the OS-native WebView (`wry`) and `tao` windowing engine.

No bundled Chromium fork, no Electron bloat, and no Node.js build pipeline. Pluto Browser compiles down to a ~1 MB stripped binary while keeping an interactive, Notion-inspired UI and a built-in DOM X-Ray Inspector.

---

## Why Pluto Browser

| Metric / Attribute | Pluto Browser (Rust + Wry) | Google Chrome | Brave Browser | Arc Desktop |
|---|---|---|---|---|
| **Shell Language** | 100% Rust (`tao` + `wry`) | C++ | C++ | Swift + C++ |
| **Release Binary Size** | ~1.06 MB (LTO + `strip`) | ~110 MB | ~125 MB | ~240 MB |
| **Memory Safety** | Compile-time borrow checker | Manual C++ memory | Manual C++ memory | Hybrid |
| **Background Tab Memory** | Auto-trimmed (`MemoryUsageLevel::Low` on Windows) | Heavy multi-process | Heavy multi-process | High idle footprint |
| **Background Daemons** | Zero telemetry / auto-updaters | Updater + crash reporter | Updater + telemetry options | Cloud sync service |
| **Engine Strategy** | OS-native WebView (`WebView2` / `WebKitGTK` / `WKWebView`) | Bundled Blink | Bundled Blink | Bundled Blink |

---

## Features

- **Tabbed Workspace**: Open, close, switch, pin (double-click or right-click a tab), and reopen recently closed tabs (`Ctrl+Shift+T`).
- **Orbital Performance Tuning**:
  - **Windows WebView2 GPU & Zero-Copy Flags**: Hardware GPU rasterization, zero-copy compositing, and disabled background networking (`BROWSER_PERF_ARGS`).
  - **Background Tab Hibernation (Windows)**: Switching away from a tab sets `MemoryUsageLevel::Low` (`SetMemoryUsageTargetLevel`) and restores `MemoryUsageLevel::Normal` when activated.
  - **Zero-Allocation Chrome Updates**: Chrome HTML is cached in `OnceLock` and the tab strip performs keyed in-place DOM diffing to prevent favicon flicker.
- **Smart Omnibox**: Automatically detects URLs (adds `https://` for domains, `http://` for `localhost` or IPv4 addresses), supports internal routes (`pluto://welcome`, `pluto://newtab`), and searches via DuckDuckGo otherwise.
- **Built-in Pluto Inspector (`Ctrl+Shift+D`)**:
  - **RAF-Throttled Live Element X-Ray**: Hover any element on any webpage to inspect its selector, box dimensions, typography, colors, padding, and border radius.
  - **Console REPL & Log Capture**: Inspect page console output and run live JavaScript expressions.
  - **DOM & Perf Snapshot**: Inspect live DOM counts, stylesheets, images, and Navigation Timing telemetry.
  - **Storage Inspector**: Check `localStorage`, `sessionStorage`, and cookies at a glance.
  - **Native DevTools (`F12` / `Ctrl+Shift+I`)** and **Find in Page (`Ctrl+F`)** enabled out of the box.
- **Pluto Kuiper Onboarding & Start Page**:
  - `pluto://welcome`: First-run interactive orbital workspace guide, live inspector sandbox, shortcut reference, and benchmark comparison (accessible anytime from the Planet Pluto brand button).
  - `pluto://newtab`: Fast local start page with search bar, filterable quick links, pinned bookmarks, and persistent workspace scratchpad notes.
- **Bookmarks & Session Restore**: Persisted cleanly to local JSON files (`bookmarks.json`, `session.json`, `settings.json`).
- **System Dark / Light Theme**: Adapts automatically via CSS `prefers-color-scheme`.

---

## Cross-Platform Prerequisites & Setup

Pluto Browser uses [`wry`](https://github.com/tauri-apps/wry) and [`tao`](https://github.com/tauri-apps/tao) so the core Rust architecture targets Windows, Linux, and macOS. Currently **Windows 10/11** is the primary verified target, and contributors are actively invited to build and tune the Linux and macOS targets.

### 1. Windows 10 / 11 (Primary Verified Target)

- **Rust Toolchain**:
  - **GNU toolchain (no Visual Studio Build Tools required)**:
    ```powershell
    rustup-init.exe -y --default-host x86_64-pc-windows-gnu --default-toolchain stable
    ```
  - **MSVC toolchain**:
    Install [Visual Studio 2022 Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **Desktop development with C++** workload.
- **WebView2 Runtime**:
  - Pre-installed on Windows 11.
  - On Windows 10, install the [Microsoft Edge WebView2 Evergreen Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) if missing.
- **Automatic DLL Packaging (`build.rs`)**:
  For `x86_64-pc-windows-gnu` builds, `build.rs` automatically locates `WebView2Loader.dll` in your Cargo registry and copies it into `target/debug/` or `target/release/`.

### 2. Linux (Ubuntu / Debian, Fedora, Arch)

On Linux, `tao` renders with GTK3 and `wry` uses **WebKitGTK (`webkit2gtk-4.1`)**.

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

> **Wayland Note**: If testing on a Wayland compositor and child WebView bounds behave unexpectedly, try launching with `GDK_BACKEND=x11 cargo run` to compare XWayland vs native Wayland behavior.

### 3. macOS (Apple Silicon & Intel)

On macOS, `tao` uses Cocoa/AppKit and `wry` uses the system **WKWebView** (no external WebView runtime installation needed).

1. Install **Xcode Command Line Tools**:
   ```bash
   xcode-select --install
   ```
2. Install Rust via `rustup`:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

> **macOS Keyboard Note**: Chrome UI and page shortcuts check `e.ctrlKey || e.metaKey`, so `Cmd+T`, `Cmd+W`, `Cmd+L`, `Cmd+R`, and `Cmd+F` map naturally on macOS alongside Linux/Windows `Ctrl` shortcuts.

---

## Build, Run & Test

```bash
# Clone the repository
git clone https://github.com/<your-username>/<your-repo>.git
cd <your-repo>

# Run in debug mode (dev build)
cargo run

# Run unit tests
cargo test

# Run linter checks
cargo clippy

# Build size-optimized stripped release binary
cargo build --release
```

### Output Executable Locations

| OS | Release Binary Path |
|---|---|
| **Windows** | `target\release\rust-browser-adit.exe` |
| **Linux** | `target/release/rust-browser-adit` |
| **macOS** | `target/release/rust-browser-adit` |

### Persisted App Data Locations

Pluto Browser uses [`directories::ProjectDirs::from("com", "adit", "rust-browser-adit")`](src/storage.rs) to store `bookmarks.json`, `session.json`, `settings.json`, and the `webview2_pluto/` profile directory:

| OS | Default Data Directory |
|---|---|
| **Windows** | `%APPDATA%\adit\rust-browser-adit\data\` |
| **Linux** | `$XDG_DATA_HOME/rust-browser-adit/` or `~/.local/share/rust-browser-adit/` |
| **macOS** | `~/Library/Application Support/com.adit.rust-browser-adit/` |

*(Tip: Deleting `settings.json` and `session.json` lets you test the first-run `pluto://welcome` onboarding flow on next launch.)*

---

## Keyboard Shortcuts

| Windows / Linux | macOS | Action |
|---|---|---|
| `Ctrl+T` | `Cmd+T` | Open a new tab (`pluto://newtab`) |
| `Ctrl+W` | `Cmd+W` | Close the active tab |
| `Ctrl+Shift+T` | `Cmd+Shift+T` | Reopen the most recently closed tab |
| `Ctrl+L` | `Cmd+L` | Focus and select the omnibox |
| `Ctrl+R` / `F5` | `Cmd+R` / `F5` | Reload the active tab |
| `Ctrl+Tab` | `Ctrl+Tab` | Switch to the next tab |
| `Ctrl+Shift+Tab` | `Ctrl+Shift+Tab` | Switch to the previous tab |
| `Alt+Left` | `Alt+Left` | Navigate back |
| `Alt+Right` | `Alt+Right` | Navigate forward |
| `Ctrl+F` | `Cmd+F` | Open in-page Find Bar |
| `Ctrl+Shift+D` / `Ctrl+Shift+C` | `Cmd+Shift+D` / `Cmd+Shift+C` | Toggle Pluto Live Element X-Ray & Inspector |
| `F12` / `Ctrl+Shift+I` | `F12` / `Cmd+Shift+I` | Open native OS WebView DevTools |
| `Ctrl+=` / `Ctrl+-` / `Ctrl+0` | `Cmd+=` / `Cmd+-` / `Cmd+0` | Zoom in / Zoom out / Reset zoom |

---

## Architecture & Contributor Guide

Pluto Browser intentionally avoids global mutable statics and complex multi-process IPC frameworks. A single `AppState` owns the window, chrome WebView, `TabManager`, bookmarks, settings, and closed-tab history on the `tao` event loop thread.

```text
rust-browser-adit/
├── Cargo.toml           # Dependencies & release profile (opt-level="s", lto=true, strip=true)
├── Cargo.lock           # Pinned dependency versions for repeatable builds
├── build.rs             # Auto-copies WebView2Loader.dll on Windows GNU targets
├── .gitignore           # Cross-platform Rust, OS, IDE, and browser state ignores
├── README.md
└── src/
    ├── main.rs          # tao EventLoop, window lifecycle, procedural Planet Pluto RGBA icon
    ├── app.rs           # AppState coordinator, chrome WebView builder, OnceLock HTML cache, session sync
    ├── tabs.rs          # TabManager owning child WebViews, custom protocols, Windows memory trimming, X-Ray injection
    ├── ipc.rs           # Typed serde IPC commands/events (UiToRustMessage, RustToUiState) & omnibox parser
    ├── storage.rs       # BOM-tolerant JSON persistence (bookmarks, session, settings)
    └── ui/
        ├── index.html   # Chrome shell (Pluto brand button, tabs row, omnibox, inspector controls)
        ├── style.css    # Compact 84px Notion-styled top bar CSS (dark/light system theme)
        ├── app.js       # Keyed in-place tab DOM diffing, omnibox state sync & shortcuts
        ├── newtab.html  # Built-in Start Page (pluto://newtab)
        └── welcome.html # Interactive Kuiper onboarding & benchmark showcase (pluto://welcome)
```

### How Key Subsystems Work

1. **Custom Internal Protocols Across OSes (`src/tabs.rs`)**:
   - Both `pluto://` and backward-compatible `adit://` custom schemes are registered on tab WebViews.
   - Because Windows WebView2 maps custom protocols to `http://<scheme>.localhost/<path>`, `to_webview_url()` and `from_webview_url()` automatically normalize `pluto://newtab` <-> `http://pluto.localhost/newtab` on `#[cfg(target_os = "windows")]` while keeping `pluto://newtab` directly on `#[cfg(not(target_os = "windows"))]` (Linux & macOS).
2. **IPC Communication (`src/ipc.rs`, `src/app.rs`, `src/ui/app.js`)**:
   - **UI / Page -> Rust**: WebViews call `window.ipc.postMessage(JSON.stringify({ cmd: ... }))`, forwarded through `tao::event_loop::EventLoopProxy<UserEvent>`.
   - **Rust -> Chrome UI**: `AppState::sync_ui_state()` serializes `RustToUiState` and pushes it via `evaluate_script("window.__onBrowserState(...)")`.
3. **Platform-Conditional Hooks (`#[cfg(target_os = "windows")]`)**:
   - Windows-specific tuning (`with_additional_browser_args` and `set_memory_usage_level(MemoryUsageLevel::{Low, Normal})`) is gated cleanly under `#[cfg(target_os = "windows")]` so Linux (`WebKitGTK`) and macOS (`WKWebView`) builds compile without Windows API leakage.

---

## Multi-Platform Roadmap

Planned improvements for upcoming open-source milestones across Windows, Linux, and macOS:

- [ ] **Linux CI & Verification**: Test and polish WebKitGTK (`4.1`) child WebView bounds, DPI scaling, and Wayland/X11 focus handling.
- [ ] **macOS App Bundle & Menu Bar**: Add macOS native application menu integration (`tao` menu bar for Copy/Paste/Cmd+Q/Cmd+W) and `.app` packaging.
- [ ] **Workspace Profiles & Bookmarks Bar**: Optional collapsible bookmarks bar and named workspaces.
- [ ] **Download Prompt & Basic History Drawer**: Lightweight download notifications and local searchable history without background daemons.
- [ ] **GitHub Actions Matrix CI**: Automated `cargo test`, `cargo clippy`, and release builds across `windows-latest`, `ubuntu-latest`, and `macos-latest`.

---

## Contributing

Contributions from friends and external contributors are welcome!

1. Fork or clone the repository and create a feature branch:
   ```bash
   git checkout -b feat/linux-webkitgtk-polish
   ```
2. Keep changes focused inside existing module boundaries (`ipc.rs`, `tabs.rs`, `app.rs`, `storage.rs`, or `ui/`).
3. Test locally before opening a PR:
   ```bash
   cargo test
   cargo build
   cargo build --release
   ```
4. Document any platform-specific dependencies or runtime flags tested on Linux, macOS, or Windows in your Pull Request description.

### First-Time Push to GitHub

```bash
git init
git add .
git commit -m "chore: initial Pluto Browser release with cross-platform docs"
git branch -M main
git remote add origin https://github.com/<your-username>/<your-repo>.git
git push -u origin main
```
