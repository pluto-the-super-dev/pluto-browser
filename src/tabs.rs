use crate::ipc::{UserEvent, NEW_TAB_URL, SETTINGS_URL, WELCOME_URL};
use crate::ui::{NativeUi, PageTelemetry};
use anyhow::{Context, Result};
use tao::event_loop::EventLoopProxy;
use tao::window::Window;
use wry::dpi::{LogicalPosition, LogicalSize};
#[cfg(target_os = "windows")]
use wry::{MemoryUsageLevel, WebViewBuilderExtWindows, WebViewExtWindows};
use wry::{NewWindowResponse, PageLoadEvent, Rect, WebContext, WebView, WebViewBuilder};

#[cfg(target_os = "windows")]
pub const BROWSER_PERF_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,Translate --autoplay-policy=no-user-gesture-required --enable-gpu-rasterization --enable-zero-copy --disable-background-networking";

/// Minimal DOM & shortcut bridge injected ONLY into external web pages so keyboard
/// shortcuts, page titles, Pluto Shield ad blocking, Reader Mode, audio mute, and
/// Pluto Inspector telemetry work seamlessly. Contains zero browser UI markup.
const TAB_INIT_SCRIPT: &str = r##"
(function() {
  if (window.__plutoTabInit) return;
  window.__plutoTabInit = true;

  function post(payload) {
    if (window.ipc && typeof window.ipc.postMessage === "function") {
      window.ipc.postMessage(JSON.stringify(payload));
    }
  }

  window.addEventListener("keydown", function(e) {
    const ctrl = e.ctrlKey || e.metaKey;
    const key = (e.key || "").toLowerCase();

    if (e.key === "F12" || (ctrl && e.shiftKey && key === "i")) {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "open_devtools" });
    } else if (ctrl && e.shiftKey && (key === "d" || key === "c")) {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "toggle_inspector" });
    } else if (ctrl && e.shiftKey && key === "b") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "toggle_bookmarks_bar" });
    } else if (ctrl && !e.shiftKey && !e.altKey && key === "b") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "toggle_sidebar_collapse" });
    } else if (ctrl && e.shiftKey && key === "l") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "toggle_sidebar_layout" });
    } else if (ctrl && e.shiftKey && key === "y") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "copy_markdown" });
    } else if (ctrl && !e.shiftKey && !e.altKey && e.key === ",") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "open_settings" });
    } else if (e.altKey && !ctrl && !e.shiftKey && key === "r") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "toggle_reader_mode" });
    } else if (e.altKey && !ctrl && !e.shiftKey && key === "m") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "toggle_mute" });
    } else if (ctrl && e.shiftKey && key === "k") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "duplicate_active_tab" });
    } else if (ctrl && !e.shiftKey && !e.altKey && (key === "k" || key === "p")) {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "toggle_command_palette" });
    } else if (ctrl && !e.shiftKey && !e.altKey && key === "f") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "toggle_find_bar" });
    } else if (ctrl && e.shiftKey && key === "t") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "reopen_closed_tab" });
    } else if (ctrl && !e.altKey && key === "t") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "new_tab" });
    } else if (ctrl && !e.altKey && key === "w") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "close_active_tab" });
    } else if (ctrl && !e.altKey && key === "l") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "focus_address_bar" });
    } else if ((ctrl && !e.altKey && key === "r") || e.key === "F5") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "reload" });
    } else if (ctrl && e.key === "Tab") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: e.shiftKey ? "prev_tab" : "next_tab" });
    } else if (e.altKey && !ctrl && e.key === "ArrowLeft") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "back" });
    } else if (e.altKey && !ctrl && e.key === "ArrowRight") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "forward" });
    } else if (ctrl && !e.altKey && (e.key === "=" || e.key === "+")) {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "zoom_in" });
    } else if (ctrl && !e.altKey && e.key === "-") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "zoom_out" });
    } else if (ctrl && !e.altKey && e.key === "0") {
      e.preventDefault();
      e.stopPropagation();
      post({ cmd: "zoom_reset" });
    }
  }, true);

  if (window !== window.top) return;

  function sendMeta() {
    let favicon = null;
    const iconEl = document.querySelector(
      'link[rel~="icon"], link[rel="shortcut icon"], link[rel="apple-touch-icon"]'
    );
    if (iconEl && iconEl.href) {
      favicon = iconEl.href;
    } else if (location.protocol === "http:" || location.protocol === "https:") {
      favicon = location.origin + "/favicon.ico";
    }
    post({
      cmd: "page_meta",
      title: document.title || null,
      url: location.href || null,
      favicon: favicon
    });
  }

  window.__plutoCollectTelemetry = function() {
    try {
      const nav = performance.getEntriesByType && performance.getEntriesByType("navigation")[0];
      let ls = 0;
      let ss = 0;
      try { ls = window.localStorage.length; } catch (_) {}
      try { ss = window.sessionStorage.length; } catch (_) {}
      post({
        cmd: "inspector_telemetry",
        dom_nodes: document.getElementsByTagName("*").length,
        scripts: document.scripts ? document.scripts.length : 0,
        stylesheets: document.styleSheets ? document.styleSheets.length : 0,
        images: document.images ? document.images.length : 0,
        dcl_ms: nav ? Math.round(nav.domContentLoadedEventEnd) : 0,
        load_ms: nav ? Math.round(nav.loadEventEnd) : 0,
        local_keys: ls,
        session_keys: ss,
        origin: location.origin || location.protocol || ""
      });
    } catch (_) {}
  };

  window.__plutoFind = function(q, backward) {
    if (q) window.find(q, false, Boolean(backward), true);
  };

  window.__plutoSetShield = function(enabled) {
    try {
      const id = "__pluto-shield-style";
      let el = document.getElementById(id);
      if (!enabled) {
        if (el) el.remove();
        return;
      }
      if (!el) {
        el = document.createElement("style");
        el.id = id;
        el.textContent = [
          'ins.adsbygoogle',
          '[id*="google_ads"]',
          '[id*="ad_banner"]',
          '[class*="ad-banner"]',
          '[class*="ad-slot"]',
          '[class*="sponsored-"]',
          '[data-ad]',
          '[aria-label*="Sponsored"]',
          '#onetrust-banner-sdk',
          '.fc-consent-root'
        ].join(",") + " { display: none !important; }";
        (document.head || document.documentElement).appendChild(el);
      }
    } catch (_) {}
  };

  window.__plutoSetMuted = function(muted) {
    try {
      const list = document.querySelectorAll("video, audio");
      for (let i = 0; i < list.length; i++) {
        list[i].muted = Boolean(muted);
      }
    } catch (_) {}
  };

  window.__plutoToggleReader = function(active, isDark) {
    try {
      const id = "__pluto-reader-overlay";
      const existing = document.getElementById(id);
      if (!active) {
        if (existing) existing.remove();
        return;
      }
      if (existing) existing.remove();

      const bg = isDark ? "#08080a" : "#ffffff";
      const surface = isDark ? "#0f0f13" : "#f6f6f7";
      const fg = isDark ? "#f4f4f5" : "#09090b";
      const muted = isDark ? "#a1a1aa" : "#52525b";
      const border = isDark ? "#2c2c35" : "#d4d4d8";

      const overlay = document.createElement("div");
      overlay.id = id;
      overlay.style.cssText =
        "position:fixed;inset:0;z-index:2147483647;overflow-y:auto;padding:48px 24px;" +
        "background:" + bg + ";color:" + fg + ";font-family:system-ui,-apple-system,sans-serif;";

      const box = document.createElement("div");
      box.style.cssText =
        "max-width:740px;margin:0 auto;border:1px solid " + border + ";background:" + surface + ";padding:36px 40px;";

      const topBar = document.createElement("div");
      topBar.style.cssText =
        "display:flex;justify-content:space-between;align-items:center;padding-bottom:16px;" +
        "margin-bottom:24px;border-bottom:1px solid " + border + ";font-size:12px;letter-spacing:1px;color:" + muted + ";";
      const badge = document.createElement("span");
      badge.textContent = "PLUTO READER MODE // " + (location.hostname || "ARTICLE");
      topBar.appendChild(badge);
      box.appendChild(topBar);

      const h1 = document.createElement("h1");
      h1.textContent = document.title || "Reader View";
      h1.style.cssText = "font-size:32px;line-height:1.25;margin:0 0 24px 0;font-weight:700;";
      box.appendChild(h1);

      const source = document.querySelector("article") || document.querySelector("main") || document.body;
      const nodes = source ? source.querySelectorAll("h2,h3,p,li,blockquote,pre") : [];
      let count = 0;
      for (let i = 0; i < nodes.length && count < 120; i++) {
        const txt = (nodes[i].innerText || "").trim();
        if (txt.length < 24 && nodes[i].tagName !== "H2" && nodes[i].tagName !== "H3") continue;
        const p = document.createElement(nodes[i].tagName === "H2" ? "h2" : nodes[i].tagName === "H3" ? "h3" : "p");
        p.textContent = txt;
        p.style.cssText = nodes[i].tagName.startsWith("H")
          ? "font-size:21px;line-height:1.35;margin:28px 0 12px 0;font-weight:600;"
          : "font-size:17px;line-height:1.72;margin:0 0 16px 0;color:" + fg + ";";
        box.appendChild(p);
        count++;
      }
      if (count === 0) {
        const empty = document.createElement("p");
        empty.textContent = "No structured article paragraphs detected on this page.";
        empty.style.cssText = "font-size:16px;color:" + muted + ";";
        box.appendChild(empty);
      }

      overlay.appendChild(box);
      document.documentElement.appendChild(overlay);
    } catch (_) {}
  };

  window.addEventListener("DOMContentLoaded", sendMeta);
  window.addEventListener("load", function() {
    sendMeta();
    window.__plutoCollectTelemetry();
  });
})();
"##;

pub struct Tab {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub is_loading: bool,
    pub pinned: bool,
    pub is_muted: bool,
    pub reader_active: bool,
    pub zoom: f64,
    pub telemetry: Option<PageTelemetry>,
    pub webview: Option<WebView>,
}

pub struct TabManager {
    tabs: Vec<Tab>,
    active_tab_id: Option<u64>,
    next_id: u64,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_id: None,
            next_id: 1,
        }
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn active_tab_id(&self) -> Option<u64> {
        self.active_tab_id
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        let active_id = self.active_tab_id?;
        self.tabs.iter().find(|t| t.id == active_id)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        let active_id = self.active_tab_id?;
        self.tabs.iter_mut().find(|t| t.id == active_id)
    }

    pub fn get_tab(&self, id: u64) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == id)
    }

    pub fn get_tab_mut(&mut self, id: u64) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    pub fn active_index(&self) -> usize {
        let Some(active_id) = self.active_tab_id else {
            return 0;
        };
        self.tabs
            .iter()
            .position(|t| t.id == active_id)
            .unwrap_or(0)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_tab(
        &mut self,
        window: &Window,
        proxy: &EventLoopProxy<UserEvent>,
        web_context: &mut WebContext,
        initial_url: &str,
        make_active: bool,
        initial_zoom: f64,
        left_offset: f64,
        top_height: f64,
        bottom_height: f64,
    ) -> Result<u64> {
        let id = self.next_id;
        self.next_id += 1;

        let logical_url = if initial_url.trim().is_empty() {
            NEW_TAB_URL.to_string()
        } else {
            initial_url.to_string()
        };
        let zoom = clamp_zoom(initial_zoom);

        // Lazy WebView creation: internal routes render 100% natively in Rust (0 WebView processes)
        // and background restored tabs wait until activated to spawn an OS WebView.
        let webview = if make_active && !NativeUi::is_internal_page(&logical_url) {
            let wv = build_tab_webview(
                id,
                window,
                proxy,
                web_context,
                &logical_url,
                true,
                left_offset,
                top_height,
                bottom_height,
            )?;
            if (zoom - 1.0).abs() > 0.01 {
                let _ = wv.zoom(zoom);
            }
            Some(wv)
        } else {
            None
        };

        let initial_title = derive_title_from_url(&logical_url);
        let is_loading = webview.is_some();

        if make_active {
            if let Some(prev) = self.active_tab_mut() {
                if let Some(prev_wv) = &prev.webview {
                    let _ = prev_wv.set_visible(false);
                    #[cfg(target_os = "windows")]
                    let _ = prev_wv.set_memory_usage_level(MemoryUsageLevel::Low);
                }
            }
            self.active_tab_id = Some(id);
        }

        self.tabs.push(Tab {
            id,
            url: logical_url,
            title: initial_title,
            favicon: None,
            is_loading,
            pinned: false,
            is_muted: false,
            reader_active: false,
            zoom,
            telemetry: None,
            webview,
        });

        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn switch_to(
        &mut self,
        window: &Window,
        proxy: &EventLoopProxy<UserEvent>,
        web_context: &mut WebContext,
        id: u64,
        left_offset: f64,
        top_height: f64,
        bottom_height: f64,
    ) -> bool {
        if !self.tabs.iter().any(|t| t.id == id) {
            return false;
        }
        if self.active_tab_id == Some(id) {
            return false;
        }

        if let Some(prev) = self.active_tab_mut() {
            if let Some(prev_wv) = &prev.webview {
                let _ = prev_wv.set_visible(false);
                #[cfg(target_os = "windows")]
                let _ = prev_wv.set_memory_usage_level(MemoryUsageLevel::Low);
            }
        }

        let bounds = content_bounds(window, left_offset, top_height, bottom_height);
        let Some(idx) = self.tabs.iter().position(|t| t.id == id) else {
            return false;
        };

        let is_internal = NativeUi::is_internal_page(&self.tabs[idx].url);
        if is_internal {
            if let Some(wv) = &self.tabs[idx].webview {
                let _ = wv.set_visible(false);
                #[cfg(target_os = "windows")]
                let _ = wv.set_memory_usage_level(MemoryUsageLevel::Low);
            }
            let _ = window.set_focus();
        } else {
            if self.tabs[idx].webview.is_none() {
                let url_copy = self.tabs[idx].url.clone();
                if let Ok(wv) = build_tab_webview(
                    id,
                    window,
                    proxy,
                    web_context,
                    &url_copy,
                    true,
                    left_offset,
                    top_height,
                    bottom_height,
                ) {
                    if (self.tabs[idx].zoom - 1.0).abs() > 0.01 {
                        let _ = wv.zoom(self.tabs[idx].zoom);
                    }
                    self.tabs[idx].is_loading = true;
                    self.tabs[idx].webview = Some(wv);
                }
            }
            if let Some(wv) = &self.tabs[idx].webview {
                #[cfg(target_os = "windows")]
                let _ = wv.set_memory_usage_level(MemoryUsageLevel::Normal);
                let _ = wv.set_bounds(bounds);
                let _ = wv.set_visible(true);
                let _ = wv.focus();
            }
        }

        self.active_tab_id = Some(id);
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub fn switch_relative(
        &mut self,
        window: &Window,
        proxy: &EventLoopProxy<UserEvent>,
        web_context: &mut WebContext,
        delta: isize,
        left_offset: f64,
        top_height: f64,
        bottom_height: f64,
    ) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        let len = self.tabs.len() as isize;
        let current = self.active_index() as isize;
        let next_idx = (current + delta).rem_euclid(len) as usize;
        let next_id = self.tabs[next_idx].id;
        self.switch_to(
            window,
            proxy,
            web_context,
            next_id,
            left_offset,
            top_height,
            bottom_height,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn close_tab(
        &mut self,
        window: &Window,
        proxy: &EventLoopProxy<UserEvent>,
        web_context: &mut WebContext,
        id: u64,
        left_offset: f64,
        top_height: f64,
        bottom_height: f64,
    ) -> Result<Option<String>> {
        let Some(idx) = self.tabs.iter().position(|t| t.id == id) else {
            return Ok(None);
        };

        let was_active = self.active_tab_id == Some(id);
        let removed = self.tabs.remove(idx);
        let closed_url = removed.url.clone();
        if let Some(wv) = &removed.webview {
            let _ = wv.set_visible(false);
        }
        drop(removed);

        if self.tabs.is_empty() {
            self.active_tab_id = None;
            self.create_tab(
                window,
                proxy,
                web_context,
                NEW_TAB_URL,
                true,
                1.0,
                left_offset,
                top_height,
                bottom_height,
            )?;
            return Ok(Some(closed_url));
        }

        if was_active {
            let new_idx = idx.min(self.tabs.len() - 1);
            let next_id = self.tabs[new_idx].id;
            self.active_tab_id = None;
            self.switch_to(
                window,
                proxy,
                web_context,
                next_id,
                left_offset,
                top_height,
                bottom_height,
            );
        }

        Ok(Some(closed_url))
    }

    pub fn toggle_pin(&mut self, id: u64) -> bool {
        let Some(tab) = self.get_tab_mut(id) else {
            return false;
        };
        tab.pinned = !tab.pinned;
        self.tabs.sort_by_key(|t| !t.pinned);
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub fn navigate_active(
        &mut self,
        window: &Window,
        proxy: &EventLoopProxy<UserEvent>,
        web_context: &mut WebContext,
        url: &str,
        left_offset: f64,
        top_height: f64,
        bottom_height: f64,
    ) -> Result<()> {
        let Some(active_id) = self.active_tab_id else {
            return Ok(());
        };
        let Some(idx) = self.tabs.iter().position(|t| t.id == active_id) else {
            return Ok(());
        };

        self.tabs[idx].url = url.to_string();
        self.tabs[idx].title = derive_title_from_url(url);
        self.tabs[idx].telemetry = None;
        self.tabs[idx].reader_active = false;

        if NativeUi::is_internal_page(url) {
            self.tabs[idx].favicon = None;
            self.tabs[idx].is_loading = false;
            if let Some(wv) = &self.tabs[idx].webview {
                let _ = wv.set_visible(false);
                #[cfg(target_os = "windows")]
                let _ = wv.set_memory_usage_level(MemoryUsageLevel::Low);
            }
            let _ = window.set_focus();
            return Ok(());
        }

        self.tabs[idx].is_loading = true;
        let bounds = content_bounds(window, left_offset, top_height, bottom_height);

        if let Some(wv) = &self.tabs[idx].webview {
            #[cfg(target_os = "windows")]
            let _ = wv.set_memory_usage_level(MemoryUsageLevel::Normal);
            let _ = wv.set_bounds(bounds);
            let _ = wv.set_visible(true);
            wv.load_url(url)
                .with_context(|| format!("Failed to navigate tab {active_id} to {url}"))?;
            let _ = wv.focus();
        } else {
            let wv = build_tab_webview(
                active_id,
                window,
                proxy,
                web_context,
                url,
                true,
                left_offset,
                top_height,
                bottom_height,
            )?;
            if (self.tabs[idx].zoom - 1.0).abs() > 0.01 {
                let _ = wv.zoom(self.tabs[idx].zoom);
            }
            let _ = wv.focus();
            self.tabs[idx].webview = Some(wv);
        }

        Ok(())
    }

    pub fn back_active(&self) -> Result<()> {
        if let Some(tab) = self.active_tab() {
            if let Some(wv) = &tab.webview {
                if !NativeUi::is_internal_page(&tab.url) {
                    wv.go_back().context("Failed to navigate back")?;
                }
            }
        }
        Ok(())
    }

    pub fn forward_active(&self) -> Result<()> {
        if let Some(tab) = self.active_tab() {
            if let Some(wv) = &tab.webview {
                if !NativeUi::is_internal_page(&tab.url) {
                    wv.go_forward().context("Failed to navigate forward")?;
                }
            }
        }
        Ok(())
    }

    pub fn reload_active(&self) -> Result<()> {
        if let Some(tab) = self.active_tab() {
            if let Some(wv) = &tab.webview {
                if !NativeUi::is_internal_page(&tab.url) {
                    wv.reload().context("Failed to reload active tab")?;
                }
            }
        }
        Ok(())
    }

    pub fn stop_active(&mut self) -> Result<()> {
        if let Some(tab) = self.active_tab_mut() {
            tab.is_loading = false;
            if let Some(wv) = &tab.webview {
                wv.evaluate_script("window.stop()")
                    .context("Failed to stop active tab")?;
            }
        }
        Ok(())
    }

    pub fn open_devtools_active(&self) -> bool {
        if let Some(tab) = self.active_tab() {
            if let Some(wv) = &tab.webview {
                if !NativeUi::is_internal_page(&tab.url) {
                    wv.open_devtools();
                    return true;
                }
            }
        }
        false
    }

    pub fn refresh_telemetry_active(&self) {
        if let Some(tab) = self.active_tab() {
            if let Some(wv) = &tab.webview {
                if !NativeUi::is_internal_page(&tab.url) {
                    let _ = wv.evaluate_script(
                        "window.__plutoCollectTelemetry && window.__plutoCollectTelemetry();",
                    );
                }
            }
        }
    }

    pub fn find_active(&self, query: &str, backward: bool) {
        if query.is_empty() {
            return;
        }
        if let Some(tab) = self.active_tab() {
            if let Some(wv) = &tab.webview {
                if !NativeUi::is_internal_page(&tab.url) {
                    if let Ok(q_json) = serde_json::to_string(query) {
                        let script = format!(
                            "window.__plutoFind && window.__plutoFind({q_json}, {backward});"
                        );
                        let _ = wv.evaluate_script(&script);
                    }
                }
            }
        }
    }

    pub fn zoom_active(&mut self, delta: f64) -> Result<f64> {
        if let Some(tab) = self.active_tab_mut() {
            let rounded = clamp_zoom(tab.zoom + delta);
            tab.zoom = rounded;
            if let Some(wv) = &tab.webview {
                wv.zoom(rounded).context("Failed to apply zoom factor")?;
            }
            return Ok(rounded);
        }
        Ok(1.0)
    }

    pub fn set_zoom_active(&mut self, factor: f64) -> Result<f64> {
        if let Some(tab) = self.active_tab_mut() {
            let rounded = clamp_zoom(factor);
            tab.zoom = rounded;
            if let Some(wv) = &tab.webview {
                wv.zoom(rounded).context("Failed to set zoom factor")?;
            }
            return Ok(rounded);
        }
        Ok(clamp_zoom(factor))
    }

    pub fn zoom_reset_active(&mut self) -> Result<()> {
        if let Some(tab) = self.active_tab_mut() {
            tab.zoom = 1.0;
            if let Some(wv) = &tab.webview {
                wv.zoom(1.0).context("Failed to reset zoom factor")?;
            }
        }
        Ok(())
    }

    pub fn toggle_reader_active(&mut self, dark_mode: bool) -> bool {
        let Some(tab) = self.active_tab_mut() else {
            return false;
        };
        if NativeUi::is_internal_page(&tab.url) {
            return false;
        }
        tab.reader_active = !tab.reader_active;
        let active = tab.reader_active;
        if let Some(wv) = &tab.webview {
            let script = format!(
                "window.__plutoToggleReader && window.__plutoToggleReader({active}, {dark_mode});"
            );
            let _ = wv.evaluate_script(&script);
        }
        active
    }

    pub fn toggle_mute_active(&mut self) -> bool {
        let Some(tab) = self.active_tab_mut() else {
            return false;
        };
        if NativeUi::is_internal_page(&tab.url) {
            return false;
        }
        tab.is_muted = !tab.is_muted;
        let muted = tab.is_muted;
        if let Some(wv) = &tab.webview {
            let script = format!("window.__plutoSetMuted && window.__plutoSetMuted({muted});");
            let _ = wv.evaluate_script(&script);
        }
        muted
    }

    pub fn apply_shield_active(&self, enabled: bool) {
        if let Some(tab) = self.active_tab() {
            if let Some(wv) = &tab.webview {
                if !NativeUi::is_internal_page(&tab.url) {
                    let script = format!(
                        "window.__plutoSetShield && window.__plutoSetShield({enabled});"
                    );
                    let _ = wv.evaluate_script(&script);
                }
            }
        }
    }

    pub fn apply_tab_runtime_prefs(&self, id: u64, shield_enabled: bool) {
        if let Some(tab) = self.get_tab(id) {
            if let Some(wv) = &tab.webview {
                if !NativeUi::is_internal_page(&tab.url) {
                    let script = format!(
                        "window.__plutoSetShield && window.__plutoSetShield({shield_enabled}); window.__plutoSetMuted && window.__plutoSetMuted({});",
                        tab.is_muted
                    );
                    let _ = wv.evaluate_script(&script);
                }
            }
        }
    }

    pub fn resize_active(
        &self,
        window: &Window,
        left_offset: f64,
        top_height: f64,
        bottom_height: f64,
        force_hide: bool,
    ) {
        let bounds = content_bounds(window, left_offset, top_height, bottom_height);
        if let Some(tab) = self.active_tab() {
            if let Some(wv) = &tab.webview {
                let visible = !force_hide && !NativeUi::is_internal_page(&tab.url);
                let _ = wv.set_bounds(bounds);
                let _ = wv.set_visible(visible);
            }
        }
    }
}

pub fn clamp_zoom(raw: f64) -> f64 {
    let clamped = raw.clamp(0.6, 2.0);
    (clamped * 10.0).round() / 10.0
}

#[allow(clippy::too_many_arguments)]
fn build_tab_webview(
    id: u64,
    window: &Window,
    proxy: &EventLoopProxy<UserEvent>,
    web_context: &mut WebContext,
    url: &str,
    visible: bool,
    left_offset: f64,
    top_height: f64,
    bottom_height: f64,
) -> Result<WebView> {
    let bounds = content_bounds(window, left_offset, top_height, bottom_height);
    let ipc_proxy = proxy.clone();
    let title_proxy = proxy.clone();
    let load_proxy = proxy.clone();
    let new_win_proxy = proxy.clone();

    let builder = WebViewBuilder::new_with_web_context(web_context)
        .with_bounds(bounds)
        .with_visible(visible)
        .with_devtools(true)
        .with_initialization_script(TAB_INIT_SCRIPT)
        .with_url(url)
        .with_ipc_handler(move |req| {
            let _ = ipc_proxy.send_event(UserEvent::TabIpc {
                tab_id: id,
                raw: req.body().clone(),
            });
        })
        .with_document_title_changed_handler(move |title| {
            let _ = title_proxy.send_event(UserEvent::TabTitleChanged { id, title });
        })
        .with_on_page_load_handler(move |event, loaded_url| {
            let is_loading = matches!(event, PageLoadEvent::Started);
            let _ = load_proxy.send_event(UserEvent::TabLoadState {
                id,
                is_loading,
                url: loaded_url,
            });
        })
        .with_new_window_req_handler(move |target_url, _features| {
            if !target_url.is_empty() {
                let _ = new_win_proxy.send_event(UserEvent::TabNewWindowRequested {
                    url: target_url,
                });
            }
            NewWindowResponse::Deny
        });

    #[cfg(target_os = "windows")]
    let builder = builder.with_additional_browser_args(BROWSER_PERF_ARGS);

    builder
        .build_as_child(window)
        .with_context(|| format!("Failed to create webview for tab {id}"))
}

pub fn content_bounds(
    window: &Window,
    left_offset: f64,
    top_height: f64,
    bottom_height: f64,
) -> Rect {
    let scale = window.scale_factor();
    let inner = window.inner_size().to_logical::<f64>(scale);
    let width = (inner.width - left_offset).max(100.0);
    let height = (inner.height - top_height - bottom_height).max(40.0);
    Rect {
        position: LogicalPosition::new(left_offset, top_height).into(),
        size: LogicalSize::new(width, height).into(),
    }
}

pub fn display_url(logical_url: &str) -> String {
    if logical_url == NEW_TAB_URL {
        String::new()
    } else {
        logical_url.to_string()
    }
}

pub fn derive_title_from_url(raw_url: &str) -> String {
    if raw_url == NEW_TAB_URL {
        return "New Tab".to_string();
    }
    if raw_url == WELCOME_URL {
        return "Welcome".to_string();
    }
    if raw_url == SETTINGS_URL {
        return "Settings".to_string();
    }
    url::Url::parse(raw_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| raw_url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_clamping_stays_within_bounds() {
        assert_eq!(clamp_zoom(0.2), 0.6);
        assert_eq!(clamp_zoom(0.6), 0.6);
        assert_eq!(clamp_zoom(1.1 + 0.1), 1.2);
        assert_eq!(clamp_zoom(2.5), 2.0);
    }

    #[test]
    fn derives_internal_page_titles() {
        assert_eq!(derive_title_from_url(NEW_TAB_URL), "New Tab");
        assert_eq!(derive_title_from_url(WELCOME_URL), "Welcome");
        assert_eq!(derive_title_from_url(SETTINGS_URL), "Settings");
    }
}
