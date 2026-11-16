use crate::ipc::{UserEvent, NEW_TAB_URL, WELCOME_URL};
use anyhow::{Context, Result};
use std::borrow::Cow;
use tao::event_loop::EventLoopProxy;
use tao::window::Window;
use wry::dpi::{LogicalPosition, LogicalSize};
#[cfg(target_os = "windows")]
use wry::{MemoryUsageLevel, WebViewBuilderExtWindows, WebViewExtWindows};
use wry::{NewWindowResponse, PageLoadEvent, Rect, WebContext, WebView, WebViewBuilder};

pub const CHROME_HEIGHT: f64 = 84.0;

#[cfg(target_os = "windows")]
pub const BROWSER_PERF_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,Translate --autoplay-policy=no-user-gesture-required --enable-gpu-rasterization --enable-zero-copy --disable-background-networking";

const NEWTAB_HTML: &str = include_str!("ui/newtab.html");
const WELCOME_HTML: &str = include_str!("ui/welcome.html");

const TAB_INIT_SCRIPT: &str = r##"
(function() {
  if (window.__aditTabInit) return;
  window.__aditTabInit = true;

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
      post({ cmd: "new_tab", url: null });
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

  if (window !== window.top) {
    return;
  }

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

  window.addEventListener("DOMContentLoaded", sendMeta);
  window.addEventListener("load", sendMeta);

  // Capture console output for the live in-page Debug Console
  const consoleLogs = [];
  function pushLog(level, args) {
    try {
      const text = Array.from(args).map(function(a) {
        if (typeof a === "string") return a;
        try { return JSON.stringify(a); } catch (_) { return String(a); }
      }).join(" ");
      consoleLogs.push({ level: level, text: text, time: new Date().toLocaleTimeString() });
      if (consoleLogs.length > 120) consoleLogs.shift();
      if (window.__aditRenderLogs) window.__aditRenderLogs();
    } catch (_) {}
  }

  ["log", "warn", "error"].forEach(function(fn) {
    const orig = console[fn];
    if (typeof orig === "function") {
      console[fn] = function() {
        pushLog(fn, arguments);
        return orig.apply(console, arguments);
      };
    }
  });

  window.addEventListener("error", function(ev) {
    pushLog("error", [ev.message || "Uncaught error"]);
  });

  function ensureOverlayHost() {
    let host = document.getElementById("__adit_dev_host");
    if (host && host.shadowRoot) return host.shadowRoot;
    host = document.createElement("div");
    host.id = "__adit_dev_host";
    host.style.cssText = "all:initial;position:fixed;inset:0;pointer-events:none;z-index:2147483646;";
    const shadow = host.attachShadow({ mode: "open" });

    const style = document.createElement("style");
    style.textContent = `
      :host {
        --adit-canvas: #ffffff;
        --adit-surface: #f6f5f4;
        --adit-hairline: #e5e3df;
        --adit-hairline-strong: #c8c4be;
        --adit-ink: #1a1a1a;
        --adit-charcoal: #37352f;
        --adit-steel: #787671;
        --adit-primary: #5645d4;
        --adit-primary-soft: rgba(86, 69, 212, 0.14);
        --adit-tint-lavender: #e6e0f5;
        --adit-tint-mint: #d9f3e1;
        --adit-tint-peach: #ffe8d4;
        --adit-tint-rose: #fde0ec;
        --adit-green: #1aae39;
        --adit-orange: #dd5b00;
        --adit-red: #e03131;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      }
      @media (prefers-color-scheme: dark) {
        :host {
          --adit-canvas: #191919;
          --adit-surface: #202020;
          --adit-hairline: #2e2e2e;
          --adit-hairline-strong: #434343;
          --adit-ink: #f4f4f2;
          --adit-charcoal: #e5e3df;
          --adit-steel: #9b9993;
          --adit-primary: #7b6ef6;
          --adit-primary-soft: rgba(123, 110, 246, 0.22);
          --adit-tint-lavender: #2b2344;
          --adit-tint-mint: #163322;
          --adit-tint-peach: #382515;
          --adit-tint-rose: #3b1c2a;
        }
      }
      * { box-sizing: border-box; margin: 0; padding: 0; }
      .xray-box {
        position: fixed;
        pointer-events: none;
        border: 2px solid var(--adit-primary);
        background: var(--adit-primary-soft);
        border-radius: 4px;
        display: none;
        z-index: 1;
        transition: all 60ms ease-out;
      }
      .xray-tag {
        position: fixed;
        pointer-events: none;
        background: #0a1530;
        color: #ffffff;
        font-size: 11px;
        font-weight: 600;
        padding: 3px 8px;
        border-radius: 6px;
        white-space: nowrap;
        display: none;
        z-index: 2;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
      }
      .find-bar {
        position: fixed;
        top: 12px;
        right: 16px;
        pointer-events: auto;
        display: none;
        align-items: center;
        gap: 6px;
        padding: 6px 8px;
        background: var(--adit-canvas);
        color: var(--adit-ink);
        border: 1px solid var(--adit-hairline-strong);
        border-radius: 10px;
        box-shadow: 0 12px 32px rgba(15, 15, 15, 0.16);
        z-index: 4;
      }
      .find-bar.open { display: flex; }
      .find-input {
        width: 200px;
        height: 28px;
        padding: 0 8px;
        border-radius: 6px;
        border: 1px solid var(--adit-hairline);
        background: var(--adit-surface);
        color: var(--adit-ink);
        font-family: inherit;
        font-size: 12.5px;
        outline: none;
      }
      .find-input:focus { border-color: var(--adit-primary); background: var(--adit-canvas); }
      .btn-sm {
        height: 28px;
        padding: 0 9px;
        border-radius: 6px;
        border: 1px solid var(--adit-hairline);
        background: var(--adit-canvas);
        color: var(--adit-charcoal);
        font-family: inherit;
        font-size: 12px;
        font-weight: 500;
        cursor: pointer;
      }
      .btn-sm:hover { background: var(--adit-surface); }
      .btn-sm:active { transform: scale(0.97); }
      .btn-primary {
        height: 30px;
        padding: 0 12px;
        border: none;
        border-radius: 8px;
        background: var(--adit-primary);
        color: #ffffff;
        font-family: inherit;
        font-size: 12px;
        font-weight: 500;
        cursor: pointer;
      }
      .btn-primary:hover { filter: brightness(1.08); }
      .drawer {
        position: fixed;
        right: 16px;
        bottom: 16px;
        width: 420px;
        max-width: calc(100vw - 32px);
        max-height: 460px;
        pointer-events: auto;
        display: none;
        flex-direction: column;
        background: var(--adit-canvas);
        color: var(--adit-ink);
        border: 1px solid var(--adit-hairline-strong);
        border-radius: 12px;
        box-shadow: 0 24px 48px -8px rgba(15, 15, 15, 0.24);
        overflow: hidden;
        z-index: 3;
      }
      .drawer.open { display: flex; }
      .drawer-head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 10px 12px;
        background: var(--adit-surface);
        border-bottom: 1px solid var(--adit-hairline);
      }
      .drawer-title {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 12.5px;
        font-weight: 600;
        color: var(--adit-ink);
      }
      .badge {
        font-size: 11px;
        font-weight: 600;
        padding: 2px 7px;
        border-radius: 6px;
        background: var(--adit-tint-lavender);
        color: var(--adit-primary);
      }
      .tabs {
        display: flex;
        gap: 4px;
        padding: 6px 12px 0;
        border-bottom: 1px solid var(--adit-hairline);
        background: var(--adit-canvas);
      }
      .tab-btn {
        padding: 7px 9px;
        border: none;
        border-bottom: 2px solid transparent;
        background: transparent;
        color: var(--adit-steel);
        font-family: inherit;
        font-size: 12px;
        font-weight: 500;
        cursor: pointer;
      }
      .tab-btn.active {
        color: var(--adit-ink);
        border-bottom-color: var(--adit-primary);
      }
      .drawer-body {
        padding: 12px;
        overflow-y: auto;
        flex: 1;
        font-size: 12px;
        color: var(--adit-charcoal);
        display: flex;
        flex-direction: column;
        gap: 10px;
      }
      .row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
      }
      .kv-grid {
        display: grid;
        grid-template-columns: 110px 1fr;
        gap: 6px 10px;
        padding: 10px;
        border-radius: 8px;
        background: var(--adit-surface);
        border: 1px solid var(--adit-hairline);
        font-size: 11.5px;
      }
      .kv-k { color: var(--adit-steel); font-weight: 500; }
      .kv-v { color: var(--adit-ink); font-family: ui-monospace, Consolas, monospace; word-break: break-all; }
      .log-list {
        max-height: 220px;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 4px;
      }
      .log-item {
        padding: 6px 8px;
        border-radius: 6px;
        background: var(--adit-surface);
        font-family: ui-monospace, Consolas, monospace;
        font-size: 11px;
        word-break: break-word;
      }
      .log-item.warn { background: var(--adit-tint-peach); color: var(--adit-orange); }
      .log-item.error { background: var(--adit-tint-rose); color: var(--adit-red); }
      .repl-form { display: flex; gap: 6px; }
      .repl-input {
        flex: 1;
        height: 30px;
        padding: 0 9px;
        border-radius: 8px;
        border: 1px solid var(--adit-hairline-strong);
        background: var(--adit-canvas);
        color: var(--adit-ink);
        font-family: ui-monospace, Consolas, monospace;
        font-size: 12px;
        outline: none;
      }
      .repl-input:focus { border-color: var(--adit-primary); }
      .hidden { display: none !important; }
    `;

    const wrap = document.createElement("div");
    wrap.innerHTML = `
      <div class="xray-box" id="xray-box"></div>
      <div class="xray-tag" id="xray-tag"></div>
      <div class="find-bar" id="find-bar">
        <input class="find-input" id="find-input" type="text" placeholder="Find in page..." />
        <button class="btn-sm" id="find-prev" type="button">Prev</button>
        <button class="btn-sm" id="find-next" type="button">Next</button>
        <button class="btn-sm" id="find-close" type="button">Close</button>
      </div>
      <div class="drawer" id="inspector-drawer">
        <div class="drawer-head">
          <div class="drawer-title">
            <span>Pluto Inspector</span>
            <span class="badge">Live</span>
          </div>
          <div style="display:flex;gap:6px;align-items:center;">
            <button class="btn-primary" id="btn-native-dt" type="button">F12 DevTools</button>
            <button class="btn-sm" id="btn-close-insp" type="button">Close</button>
          </div>
        </div>
        <div class="tabs">
          <button class="tab-btn active" data-tab="xray" type="button">X-Ray</button>
          <button class="tab-btn" data-tab="console" type="button">Console</button>
          <button class="tab-btn" data-tab="perf" type="button">DOM & Perf</button>
          <button class="tab-btn" data-tab="storage" type="button">Storage</button>
        </div>
        <div class="drawer-body">
          <div id="pane-xray">
            <div class="row" style="margin-bottom:8px;">
              <button class="btn-sm" id="btn-toggle-pick" type="button">Hover X-Ray: ON</button>
              <button class="btn-sm" id="btn-copy-sel" type="button">Copy Selector</button>
            </div>
            <div class="kv-grid" id="xray-details">
              <span class="kv-k">Selector</span><span class="kv-v">Move cursor over page</span>
              <span class="kv-k">Size</span><span class="kv-v">-</span>
              <span class="kv-k">Font</span><span class="kv-v">-</span>
              <span class="kv-k">Color</span><span class="kv-v">-</span>
              <span class="kv-k">Background</span><span class="kv-v">-</span>
              <span class="kv-k">Padding</span><span class="kv-v">-</span>
              <span class="kv-k">Radius</span><span class="kv-v">-</span>
            </div>
          </div>
          <div id="pane-console" class="hidden">
            <form class="repl-form" id="repl-form" style="margin-bottom:8px;">
              <input class="repl-input" id="repl-input" type="text" placeholder="Run JS expression (e.g. document.title)" />
              <button class="btn-sm" type="submit">Run</button>
            </form>
            <div class="log-list" id="log-list"></div>
          </div>
          <div id="pane-perf" class="hidden">
            <div class="row" style="margin-bottom:8px;">
              <span style="color:var(--adit-steel);">Live DOM & Timing Snapshot</span>
              <button class="btn-sm" id="btn-refresh-perf" type="button">Refresh</button>
            </div>
            <div class="kv-grid" id="perf-grid"></div>
          </div>
          <div id="pane-storage" class="hidden">
            <div class="kv-grid" id="storage-grid"></div>
          </div>
        </div>
      </div>
    `;

    shadow.appendChild(style);
    shadow.appendChild(wrap);
    (document.body || document.documentElement).appendChild(host);

    const xrayBox = shadow.getElementById("xray-box");
    const xrayTag = shadow.getElementById("xray-tag");
    const xrayDetails = shadow.getElementById("xray-details");
    const btnPick = shadow.getElementById("btn-toggle-pick");
    const btnCopySel = shadow.getElementById("btn-copy-sel");
    const drawer = shadow.getElementById("inspector-drawer");
    const findBar = shadow.getElementById("find-bar");
    const findInput = shadow.getElementById("find-input");
    const logList = shadow.getElementById("log-list");
    const perfGrid = shadow.getElementById("perf-grid");
    const storageGrid = shadow.getElementById("storage-grid");

    let pickActive = false;
    let lastSelector = "";

    function cssSelector(el) {
      if (!el || el.nodeType !== 1) return "";
      if (el.id) return "#" + el.id;
      let sel = el.tagName.toLowerCase();
      if (el.className && typeof el.className === "string") {
        const cls = el.className.trim().split(/\s+/).filter(Boolean).slice(0, 2);
        if (cls.length) sel += "." + cls.join(".");
      }
      return sel;
    }

    function updateElementInfo(el) {
      if (!el || el.id === "__adit_dev_host") return;
      const rect = el.getBoundingClientRect();
      const cs = getComputedStyle(el);
      const sel = cssSelector(el);
      lastSelector = sel;

      xrayBox.style.display = "block";
      xrayBox.style.left = rect.left + "px";
      xrayBox.style.top = rect.top + "px";
      xrayBox.style.width = rect.width + "px";
      xrayBox.style.height = rect.height + "px";

      xrayTag.style.display = "block";
      xrayTag.textContent = sel + " - " + Math.round(rect.width) + "x" + Math.round(rect.height) + "px";
      xrayTag.style.left = Math.max(4, rect.left) + "px";
      xrayTag.style.top = Math.max(4, rect.top - 26) + "px";

      xrayDetails.innerHTML = "";
      const rows = [
        ["Selector", sel],
        ["Size", Math.round(rect.width) + " x " + Math.round(rect.height) + " px"],
        ["Font", cs.fontSize + " / " + cs.lineHeight + " " + cs.fontFamily.split(",")[0]],
        ["Color", cs.color],
        ["Background", cs.backgroundColor],
        ["Padding", cs.padding],
        ["Radius", cs.borderRadius]
      ];
      for (const [k, v] of rows) {
        const ke = document.createElement("span");
        ke.className = "kv-k";
        ke.textContent = k;
        const ve = document.createElement("span");
        ve.className = "kv-v";
        ve.textContent = v;
        xrayDetails.appendChild(ke);
        xrayDetails.appendChild(ve);
      }
    }

    function setPickMode(on) {
      pickActive = on;
      btnPick.textContent = "Hover X-Ray: " + (on ? "ON" : "OFF");
      if (!on) {
        xrayBox.style.display = "none";
        xrayTag.style.display = "none";
      }
    }

    let lastMouseX = 0;
    let lastMouseY = 0;
    let mouseRafScheduled = false;

    document.addEventListener("mousemove", function(e) {
      if (!pickActive || !drawer.classList.contains("open")) return;
      lastMouseX = e.clientX;
      lastMouseY = e.clientY;
      if (mouseRafScheduled) return;
      mouseRafScheduled = true;
      requestAnimationFrame(function() {
        mouseRafScheduled = false;
        if (!pickActive || !drawer.classList.contains("open")) return;
        const target = document.elementFromPoint(lastMouseX, lastMouseY);
        if (target && target.id !== "__adit_dev_host") {
          updateElementInfo(target);
        }
      });
    }, { passive: true });

    document.addEventListener("click", function(e) {
      if (!pickActive || !drawer.classList.contains("open")) return;
      if (e.target && e.target.id === "__adit_dev_host") return;
      e.preventDefault();
      e.stopPropagation();
      updateElementInfo(e.target);
      setPickMode(false);
    }, true);

    btnPick.addEventListener("click", function() {
      setPickMode(!pickActive);
    });

    btnCopySel.addEventListener("click", function() {
      if (!lastSelector) return;
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(lastSelector).then(function() {
          btnCopySel.textContent = "Copied";
          setTimeout(function() { btnCopySel.textContent = "Copy Selector"; }, 1200);
        });
      }
    });

    shadow.getElementById("btn-native-dt").addEventListener("click", function() {
      post({ cmd: "open_devtools" });
    });

    shadow.getElementById("btn-close-insp").addEventListener("click", function() {
      drawer.classList.remove("open");
      setPickMode(false);
    });

    window.__aditRenderLogs = function() {
      if (!logList) return;
      logList.innerHTML = "";
      if (consoleLogs.length === 0) {
        const empty = document.createElement("div");
        empty.className = "log-item";
        empty.textContent = "No console messages yet.";
        logList.appendChild(empty);
        return;
      }
      for (let i = consoleLogs.length - 1; i >= 0; i--) {
        const item = consoleLogs[i];
        const row = document.createElement("div");
        row.className = "log-item " + item.level;
        row.textContent = "[" + item.time + "] " + item.text;
        logList.appendChild(row);
      }
    };

    function refreshPerfAndStorage() {
      const nav = performance.getEntriesByType && performance.getEntriesByType("navigation")[0];
      const dcl = nav ? Math.round(nav.domContentLoadedEventEnd) + " ms" : "-";
      const loadMs = nav ? Math.round(nav.loadEventEnd) + " ms" : "-";
      const nodes = document.getElementsByTagName("*").length;
      const imgs = document.images ? document.images.length : 0;
      const scripts = document.scripts ? document.scripts.length : 0;
      const styles = document.styleSheets ? document.styleSheets.length : 0;

      perfGrid.innerHTML = "";
      [
        ["DOM Nodes", String(nodes)],
        ["Images / Scripts", imgs + " / " + scripts],
        ["Stylesheets", String(styles)],
        ["DOMContentLoaded", dcl],
        ["Page Load", loadMs],
        ["Viewport", window.innerWidth + " x " + window.innerHeight + " (@" + window.devicePixelRatio + "x)"]
      ].forEach(function(pair) {
        const k = document.createElement("span");
        k.className = "kv-k";
        k.textContent = pair[0];
        const v = document.createElement("span");
        v.className = "kv-v";
        v.textContent = pair[1];
        perfGrid.appendChild(k);
        perfGrid.appendChild(v);
      });

      let lsCount = "Unavailable";
      let ssCount = "Unavailable";
      try { lsCount = String(window.localStorage.length) + " keys"; } catch (_) {}
      try { ssCount = String(window.sessionStorage.length) + " keys"; } catch (_) {}
      const cookieLen = document.cookie ? document.cookie.split(";").filter(Boolean).length + " entries" : "0 entries";

      storageGrid.innerHTML = "";
      [
        ["Origin", location.origin || location.protocol],
        ["localStorage", lsCount],
        ["sessionStorage", ssCount],
        ["Cookies", cookieLen]
      ].forEach(function(pair) {
        const k = document.createElement("span");
        k.className = "kv-k";
        k.textContent = pair[0];
        const v = document.createElement("span");
        v.className = "kv-v";
        v.textContent = pair[1];
        storageGrid.appendChild(k);
        storageGrid.appendChild(v);
      });
    }

    shadow.getElementById("btn-refresh-perf").addEventListener("click", refreshPerfAndStorage);

    shadow.getElementById("repl-form").addEventListener("submit", function(e) {
      e.preventDefault();
      const input = shadow.getElementById("repl-input");
      const code = input.value.trim();
      if (!code) return;
      try {
        const res = (0, eval)(code);
        pushLog("log", ["> " + code + " =>", res]);
      } catch (err) {
        pushLog("error", ["> " + code + " =>", err && err.message ? err.message : String(err)]);
      }
      input.value = "";
    });

    const tabBtns = shadow.querySelectorAll(".tab-btn");
    const panes = {
      xray: shadow.getElementById("pane-xray"),
      console: shadow.getElementById("pane-console"),
      perf: shadow.getElementById("pane-perf"),
      storage: shadow.getElementById("pane-storage")
    };
    tabBtns.forEach(function(btn) {
      btn.addEventListener("click", function() {
        tabBtns.forEach(function(b) { b.classList.remove("active"); });
        btn.classList.add("active");
        const key = btn.dataset.tab;
        Object.keys(panes).forEach(function(p) {
          panes[p].classList.toggle("hidden", p !== key);
        });
        if (key === "console") window.__aditRenderLogs();
        if (key === "perf" || key === "storage") refreshPerfAndStorage();
      });
    });

    shadow.getElementById("find-next").addEventListener("click", function() {
      if (findInput.value) window.find(findInput.value, false, false, true);
    });
    shadow.getElementById("find-prev").addEventListener("click", function() {
      if (findInput.value) window.find(findInput.value, false, true, true);
    });
    shadow.getElementById("find-close").addEventListener("click", function() {
      findBar.classList.remove("open");
    });
    findInput.addEventListener("keydown", function(e) {
      if (e.key === "Enter") {
        e.preventDefault();
        if (findInput.value) window.find(findInput.value, false, e.shiftKey, true);
      } else if (e.key === "Escape") {
        findBar.classList.remove("open");
      }
    });

    window.__aditToggleInspectorInternal = function() {
      const willOpen = !drawer.classList.contains("open");
      drawer.classList.toggle("open", willOpen);
      setPickMode(willOpen);
      if (willOpen) {
        window.__aditRenderLogs();
        refreshPerfAndStorage();
      }
    };

    window.__aditToggleFindInternal = function() {
      const willOpen = !findBar.classList.contains("open");
      findBar.classList.toggle("open", willOpen);
      if (willOpen) {
        findInput.focus();
        findInput.select();
      }
    };

    return shadow;
  }

  window.__plutoToggleInspector = window.__aditToggleInspector = function() {
    ensureOverlayHost();
    if (window.__aditToggleInspectorInternal) window.__aditToggleInspectorInternal();
  };

  window.__plutoToggleFind = window.__aditToggleFind = function() {
    ensureOverlayHost();
    if (window.__aditToggleFindInternal) window.__aditToggleFindInternal();
  };
})();
"##;

pub struct Tab {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub is_loading: bool,
    pub pinned: bool,
    pub zoom: f64,
    pub webview: WebView,
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

    pub fn create_tab(
        &mut self,
        window: &Window,
        proxy: &EventLoopProxy<UserEvent>,
        web_context: &mut WebContext,
        initial_url: &str,
        make_active: bool,
    ) -> Result<u64> {
        let id = self.next_id;
        self.next_id += 1;

        let logical_url = if initial_url.trim().is_empty() {
            NEW_TAB_URL.to_string()
        } else {
            initial_url.to_string()
        };
        let target_url = to_webview_url(&logical_url);
        let bounds = content_bounds(window);

        let ipc_proxy = proxy.clone();
        let title_proxy = proxy.clone();
        let load_proxy = proxy.clone();
        let new_win_proxy = proxy.clone();

        let builder = WebViewBuilder::new_with_web_context(web_context)
            .with_bounds(bounds)
            .with_visible(make_active)
            .with_devtools(true)
            .with_initialization_script(TAB_INIT_SCRIPT)
            .with_custom_protocol("pluto".into(), move |_webview_id, req| {
                handle_internal_protocol(&req.uri().to_string())
            })
            .with_custom_protocol("adit".into(), move |_webview_id, req| {
                handle_internal_protocol(&req.uri().to_string())
            })
            .with_url(&target_url)
            .with_ipc_handler(move |req| {
                let _ = ipc_proxy.send_event(UserEvent::TabIpc {
                    tab_id: id,
                    raw: req.body().clone(),
                });
            })
            .with_document_title_changed_handler(move |title| {
                let _ = title_proxy.send_event(UserEvent::TabTitleChanged { id, title });
            })
            .with_on_page_load_handler(move |event, url| {
                let is_loading = matches!(event, PageLoadEvent::Started);
                let _ = load_proxy.send_event(UserEvent::TabLoadState {
                    id,
                    is_loading,
                    url,
                });
            })
            .with_new_window_req_handler(move |url, _features| {
                if !url.is_empty() {
                    let _ = new_win_proxy.send_event(UserEvent::TabNewWindowRequested { url });
                }
                NewWindowResponse::Deny
            });

        #[cfg(target_os = "windows")]
        let builder = builder.with_additional_browser_args(BROWSER_PERF_ARGS);

        let webview = builder
            .build_as_child(window)
            .with_context(|| format!("Failed to create webview for tab {id}"))?;

        let initial_title = derive_title_from_url(&logical_url);

        if make_active {
            if let Some(prev) = self.active_tab_mut() {
                let _ = prev.webview.set_visible(false);
                #[cfg(target_os = "windows")]
                let _ = prev.webview.set_memory_usage_level(MemoryUsageLevel::Low);
            }
            self.active_tab_id = Some(id);
        } else {
            #[cfg(target_os = "windows")]
            let _ = webview.set_memory_usage_level(MemoryUsageLevel::Low);
        }

        self.tabs.push(Tab {
            id,
            url: logical_url,
            title: initial_title,
            favicon: None,
            is_loading: false,
            pinned: false,
            zoom: 1.0,
            webview,
        });

        Ok(id)
    }

    pub fn switch_to(&mut self, window: &Window, id: u64) -> bool {
        if !self.tabs.iter().any(|t| t.id == id) {
            return false;
        }
        if self.active_tab_id == Some(id) {
            return false;
        }

        if let Some(prev) = self.active_tab_mut() {
            let _ = prev.webview.set_visible(false);
            #[cfg(target_os = "windows")]
            let _ = prev.webview.set_memory_usage_level(MemoryUsageLevel::Low);
        }

        let bounds = content_bounds(window);
        if let Some(next) = self.get_tab_mut(id) {
            #[cfg(target_os = "windows")]
            let _ = next.webview.set_memory_usage_level(MemoryUsageLevel::Normal);
            let _ = next.webview.set_bounds(bounds);
            let _ = next.webview.set_visible(true);
            let _ = next.webview.focus();
            self.active_tab_id = Some(id);
            true
        } else {
            false
        }
    }

    pub fn switch_relative(&mut self, window: &Window, delta: isize) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        let len = self.tabs.len() as isize;
        let current = self.active_index() as isize;
        let next_idx = (current + delta).rem_euclid(len) as usize;
        let next_id = self.tabs[next_idx].id;
        self.switch_to(window, next_id)
    }

    pub fn close_tab(
        &mut self,
        window: &Window,
        proxy: &EventLoopProxy<UserEvent>,
        web_context: &mut WebContext,
        id: u64,
    ) -> Result<Option<String>> {
        let Some(idx) = self.tabs.iter().position(|t| t.id == id) else {
            return Ok(None);
        };

        let was_active = self.active_tab_id == Some(id);
        let removed = self.tabs.remove(idx);
        let closed_url = removed.url.clone();
        let _ = removed.webview.set_visible(false);
        drop(removed);

        if self.tabs.is_empty() {
            self.active_tab_id = None;
            self.create_tab(window, proxy, web_context, NEW_TAB_URL, true)?;
            return Ok(Some(closed_url));
        }

        if was_active {
            let new_idx = idx.min(self.tabs.len() - 1);
            let next_id = self.tabs[new_idx].id;
            self.active_tab_id = None;
            self.switch_to(window, next_id);
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

    pub fn navigate_active(&mut self, url: &str) -> Result<()> {
        let Some(tab) = self.active_tab_mut() else {
            return Ok(());
        };
        tab.url = url.to_string();
        tab.title = derive_title_from_url(url);
        if url == NEW_TAB_URL || url == WELCOME_URL {
            tab.favicon = None;
        }
        tab.is_loading = true;
        let target = to_webview_url(url);
        tab.webview
            .load_url(&target)
            .with_context(|| format!("Failed to navigate tab {} to {target}", tab.id))?;
        Ok(())
    }

    pub fn back_active(&self) -> Result<()> {
        if let Some(tab) = self.active_tab() {
            tab.webview
                .go_back()
                .context("Failed to navigate back in active tab")?;
        }
        Ok(())
    }

    pub fn forward_active(&self) -> Result<()> {
        if let Some(tab) = self.active_tab() {
            tab.webview
                .go_forward()
                .context("Failed to navigate forward in active tab")?;
        }
        Ok(())
    }

    pub fn reload_active(&self) -> Result<()> {
        if let Some(tab) = self.active_tab() {
            tab.webview
                .reload()
                .context("Failed to reload active tab")?;
        }
        Ok(())
    }

    pub fn stop_active(&mut self) -> Result<()> {
        if let Some(tab) = self.active_tab_mut() {
            tab.is_loading = false;
            tab.webview
                .evaluate_script("window.stop()")
                .context("Failed to stop active tab")?;
        }
        Ok(())
    }

    pub fn open_devtools_active(&self) {
        if let Some(tab) = self.active_tab() {
            tab.webview.open_devtools();
        }
    }

    pub fn toggle_inspector_active(&self) -> Result<()> {
        if let Some(tab) = self.active_tab() {
            tab.webview
                .evaluate_script("window.__aditToggleInspector && window.__aditToggleInspector();")
                .context("Failed to toggle live inspector")?;
        }
        Ok(())
    }

    pub fn toggle_find_active(&self) -> Result<()> {
        if let Some(tab) = self.active_tab() {
            tab.webview
                .evaluate_script("window.__aditToggleFind && window.__aditToggleFind();")
                .context("Failed to toggle find bar")?;
        }
        Ok(())
    }

    pub fn zoom_active(&mut self, delta: f64) -> Result<()> {
        if let Some(tab) = self.active_tab_mut() {
            let next = (tab.zoom + delta).clamp(0.5, 2.0);
            let rounded = (next * 10.0).round() / 10.0;
            tab.zoom = rounded;
            tab.webview
                .zoom(rounded)
                .context("Failed to apply zoom factor")?;
        }
        Ok(())
    }

    pub fn zoom_reset_active(&mut self) -> Result<()> {
        if let Some(tab) = self.active_tab_mut() {
            tab.zoom = 1.0;
            tab.webview
                .zoom(1.0)
                .context("Failed to reset zoom factor")?;
        }
        Ok(())
    }

    pub fn resize_active(&self, window: &Window) {
        let bounds = content_bounds(window);
        if let Some(tab) = self.active_tab() {
            let _ = tab.webview.set_bounds(bounds);
        }
    }
}

pub fn chrome_bounds(window: &Window) -> Rect {
    let scale = window.scale_factor();
    let inner = window.inner_size().to_logical::<f64>(scale);
    Rect {
        position: LogicalPosition::new(0.0, 0.0).into(),
        size: LogicalSize::new(inner.width.max(100.0), CHROME_HEIGHT).into(),
    }
}

pub fn content_bounds(window: &Window) -> Rect {
    let scale = window.scale_factor();
    let inner = window.inner_size().to_logical::<f64>(scale);
    Rect {
        position: LogicalPosition::new(0.0, CHROME_HEIGHT).into(),
        size: LogicalSize::new(
            inner.width.max(100.0),
            (inner.height - CHROME_HEIGHT).max(50.0),
        )
        .into(),
    }
}

fn handle_internal_protocol(uri: &str) -> wry::http::Response<Cow<'static, [u8]>> {
    let body = if uri.contains("welcome") {
        WELCOME_HTML.as_bytes()
    } else {
        NEWTAB_HTML.as_bytes()
    };
    wry::http::Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Cow::Borrowed(body))
        .unwrap_or_else(|_| wry::http::Response::new(Cow::Borrowed(&[])))
}

pub fn to_webview_url(logical_url: &str) -> String {
    if logical_url == NEW_TAB_URL || logical_url.eq_ignore_ascii_case("adit://newtab") {
        #[cfg(target_os = "windows")]
        return "http://pluto.localhost/newtab".to_string();
        #[cfg(not(target_os = "windows"))]
        return NEW_TAB_URL.to_string();
    }
    if logical_url == WELCOME_URL || logical_url.eq_ignore_ascii_case("adit://welcome") {
        #[cfg(target_os = "windows")]
        return "http://pluto.localhost/welcome".to_string();
        #[cfg(not(target_os = "windows"))]
        return WELCOME_URL.to_string();
    }
    logical_url.to_string()
}

pub fn from_webview_url(raw_url: &str) -> String {
    let trimmed = raw_url.trim_end_matches('/');
    if trimmed.eq_ignore_ascii_case("http://pluto.localhost/newtab")
        || trimmed.eq_ignore_ascii_case("https://pluto.localhost/newtab")
        || trimmed.eq_ignore_ascii_case("http://adit.localhost/newtab")
        || trimmed.eq_ignore_ascii_case("https://adit.localhost/newtab")
        || trimmed.eq_ignore_ascii_case("adit://newtab")
        || trimmed.eq_ignore_ascii_case(NEW_TAB_URL)
    {
        NEW_TAB_URL.to_string()
    } else if trimmed.eq_ignore_ascii_case("http://pluto.localhost/welcome")
        || trimmed.eq_ignore_ascii_case("https://pluto.localhost/welcome")
        || trimmed.eq_ignore_ascii_case("http://adit.localhost/welcome")
        || trimmed.eq_ignore_ascii_case("https://adit.localhost/welcome")
        || trimmed.eq_ignore_ascii_case("adit://welcome")
        || trimmed.eq_ignore_ascii_case(WELCOME_URL)
    {
        WELCOME_URL.to_string()
    } else {
        raw_url.to_string()
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
    url::Url::parse(raw_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| raw_url.to_string())
}
