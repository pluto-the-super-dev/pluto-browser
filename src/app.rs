use crate::ipc::{
    resolve_omnibox_input, RustToUiState, TabUiInfo, UiToRustMessage, UserEvent, NEW_TAB_URL,
    WELCOME_URL,
};
use crate::storage::{AppSettings, Bookmark, SessionData, StorageManager};
use crate::tabs::{
    chrome_bounds, derive_title_from_url, display_url, from_webview_url, TabManager,
};
use anyhow::{Context, Result};
use std::rc::Rc;
use std::sync::OnceLock;
use tao::event_loop::EventLoopProxy;
use tao::window::Window;
#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;
use wry::{WebContext, WebView, WebViewBuilder};

const CHROME_INDEX_HTML: &str = include_str!("ui/index.html");
const CHROME_STYLE_CSS: &str = include_str!("ui/style.css");
const CHROME_APP_JS: &str = include_str!("ui/app.js");

pub struct AppState {
    window: Rc<Window>,
    proxy: EventLoopProxy<UserEvent>,
    web_context: WebContext,
    chrome_webview: WebView,
    tabs: TabManager,
    storage: StorageManager,
    bookmarks: Vec<Bookmark>,
    settings: AppSettings,
    closed_tabs: Vec<String>,
}

impl AppState {
    pub fn new(window: Rc<Window>, proxy: EventLoopProxy<UserEvent>) -> Result<Self> {
        let storage = StorageManager::new()?;
        let mut web_context = WebContext::new(Some(storage.webview_data_dir()));
        let bookmarks = match storage.load_bookmarks() {
            Ok(list) => list,
            Err(err) => {
                eprintln!("Warning: {err:#}");
                Vec::new()
            }
        };
        let settings = match storage.load_settings() {
            Ok(cfg) => cfg,
            Err(err) => {
                eprintln!("Warning: {err:#}");
                AppSettings::default()
            }
        };

        let chrome_html = build_chrome_html();
        let chrome_proxy = proxy.clone();
        let chrome_builder = WebViewBuilder::new_with_web_context(&mut web_context)
            .with_bounds(chrome_bounds(&window))
            .with_devtools(true)
            .with_html(chrome_html)
            .with_ipc_handler(move |req| {
                let _ = chrome_proxy.send_event(UserEvent::ChromeIpc(req.body().clone()));
            });

        #[cfg(target_os = "windows")]
        let chrome_builder = chrome_builder.with_additional_browser_args(crate::tabs::BROWSER_PERF_ARGS);

        let chrome_webview = chrome_builder
            .build_as_child(window.as_ref())
            .context("Failed to create chrome UI webview")?;

        let mut tabs = TabManager::new();

        let restored = match storage.load_session() {
            Ok(session) => session,
            Err(err) => {
                eprintln!("Warning: {err:#}");
                None
            }
        };

        let show_onboarding = !settings.onboarding_completed;

        if let Some(session) = restored.filter(|s| !s.tabs.is_empty()) {
            let active_idx = session.active_index.min(session.tabs.len() - 1);
            for (idx, url) in session.tabs.iter().enumerate() {
                let resolved = resolve_omnibox_input(url);
                let make_active = !show_onboarding && idx == active_idx;
                tabs.create_tab(
                    window.as_ref(),
                    &proxy,
                    &mut web_context,
                    &resolved,
                    make_active,
                )?;
            }
            if show_onboarding && !tabs.tabs().iter().any(|t| t.url == WELCOME_URL) {
                tabs.create_tab(window.as_ref(), &proxy, &mut web_context, WELCOME_URL, true)?;
            } else if show_onboarding {
                if let Some(welcome_id) = tabs
                    .tabs()
                    .iter()
                    .find(|t| t.url == WELCOME_URL)
                    .map(|t| t.id)
                {
                    tabs.switch_to(window.as_ref(), welcome_id);
                }
            }
        } else {
            let start_url = if show_onboarding {
                WELCOME_URL
            } else {
                NEW_TAB_URL
            };
            tabs.create_tab(window.as_ref(), &proxy, &mut web_context, start_url, true)?;
        }

        let app = Self {
            window,
            proxy,
            web_context,
            chrome_webview,
            tabs,
            storage,
            bookmarks,
            settings,
            closed_tabs: Vec::new(),
        };

        app.sync_ui_state();
        Ok(app)
    }

    pub fn handle_resize(&self) {
        let _ = self.chrome_webview.set_bounds(chrome_bounds(&self.window));
        self.tabs.resize_active(&self.window);
    }

    pub fn handle_user_event(&mut self, event: UserEvent) {
        let result = match event {
            UserEvent::ChromeIpc(raw) => self.handle_ipc_message(None, &raw),
            UserEvent::TabIpc { tab_id, raw } => self.handle_ipc_message(Some(tab_id), &raw),
            UserEvent::TabTitleChanged { id, title } => {
                self.on_tab_title_changed(id, title);
                Ok(())
            }
            UserEvent::TabLoadState {
                id,
                is_loading,
                url,
            } => {
                self.on_tab_load_state(id, is_loading, url);
                Ok(())
            }
            UserEvent::TabNewWindowRequested { url } => {
                let resolved = resolve_omnibox_input(&url);
                let res = self.open_new_tab(&resolved);
                res
            }
        };

        if let Err(err) = result {
            eprintln!("Browser error: {err:#}");
        }
    }

    pub fn save_session(&self) {
        let session = SessionData {
            tabs: self.tabs.tabs().iter().map(|t| t.url.clone()).collect(),
            active_index: self.tabs.active_index(),
        };
        if let Err(err) = self.storage.save_session(&session) {
            eprintln!("Warning: {err:#}");
        }
    }

    fn handle_ipc_message(&mut self, source_tab_id: Option<u64>, raw: &str) -> Result<()> {
        let msg: UiToRustMessage = serde_json::from_str(raw)
            .with_context(|| format!("Invalid IPC JSON payload: {raw}"))?;

        match msg {
            UiToRustMessage::UiReady => {
                self.sync_ui_state();
                if let Some(tab_id) = source_tab_id {
                    self.push_bookmarks_to_newtab(tab_id);
                }
            }
            UiToRustMessage::NewTab { url } => {
                let target = url
                    .as_deref()
                    .map(resolve_omnibox_input)
                    .unwrap_or_else(|| NEW_TAB_URL.to_string());
                self.open_new_tab(&target)?;
            }
            UiToRustMessage::CloseTab { id } => {
                if let Some(url) =
                    self.tabs
                        .close_tab(&self.window, &self.proxy, &mut self.web_context, id)?
                {
                    self.record_closed_tab(&url);
                }
                self.save_session();
                self.sync_ui_state();
            }
            UiToRustMessage::CloseActiveTab => {
                if let Some(active_id) = self.tabs.active_tab_id() {
                    if let Some(url) = self.tabs.close_tab(
                        &self.window,
                        &self.proxy,
                        &mut self.web_context,
                        active_id,
                    )? {
                        self.record_closed_tab(&url);
                    }
                    self.save_session();
                    self.sync_ui_state();
                }
            }
            UiToRustMessage::SwitchTab { id } => {
                if self.tabs.switch_to(&self.window, id) {
                    self.save_session();
                    self.sync_ui_state();
                }
            }
            UiToRustMessage::NextTab => {
                if self.tabs.switch_relative(&self.window, 1) {
                    self.save_session();
                    self.sync_ui_state();
                }
            }
            UiToRustMessage::PrevTab => {
                if self.tabs.switch_relative(&self.window, -1) {
                    self.save_session();
                    self.sync_ui_state();
                }
            }
            UiToRustMessage::Navigate { input } => {
                let resolved = resolve_omnibox_input(&input);
                self.tabs.navigate_active(&resolved)?;
                self.save_session();
                self.sync_ui_state();
            }
            UiToRustMessage::Back => {
                self.tabs.back_active()?;
            }
            UiToRustMessage::Forward => {
                self.tabs.forward_active()?;
            }
            UiToRustMessage::Reload => {
                self.tabs.reload_active()?;
            }
            UiToRustMessage::Stop => {
                self.tabs.stop_active()?;
                self.sync_ui_state();
            }
            UiToRustMessage::Home => {
                self.tabs.navigate_active(NEW_TAB_URL)?;
                self.save_session();
                self.sync_ui_state();
            }
            UiToRustMessage::ToggleBookmark => {
                self.toggle_active_bookmark();
            }
            UiToRustMessage::OpenBookmark { url } => {
                let resolved = resolve_omnibox_input(&url);
                self.tabs.navigate_active(&resolved)?;
                self.save_session();
                self.sync_ui_state();
            }
            UiToRustMessage::RemoveBookmark { url } => {
                self.remove_bookmark(&url);
            }
            UiToRustMessage::FocusAddressBar => {
                self.focus_address_bar();
            }
            UiToRustMessage::OpenDevtools => {
                self.tabs.open_devtools_active();
            }
            UiToRustMessage::ToggleInspector => {
                self.tabs.toggle_inspector_active()?;
            }
            UiToRustMessage::ToggleFindBar => {
                self.tabs.toggle_find_active()?;
            }
            UiToRustMessage::ZoomIn => {
                self.tabs.zoom_active(0.1)?;
                self.sync_ui_state();
            }
            UiToRustMessage::ZoomOut => {
                self.tabs.zoom_active(-0.1)?;
                self.sync_ui_state();
            }
            UiToRustMessage::ZoomReset => {
                self.tabs.zoom_reset_active()?;
                self.sync_ui_state();
            }
            UiToRustMessage::ReopenClosedTab => {
                if let Some(url) = self.closed_tabs.pop() {
                    self.open_new_tab(&url)?;
                }
            }
            UiToRustMessage::DuplicateTab { id } => {
                let target_url = id
                    .and_then(|tab_id| self.tabs.get_tab(tab_id))
                    .or_else(|| self.tabs.active_tab())
                    .map(|t| t.url.clone());
                if let Some(url) = target_url {
                    self.open_new_tab(&url)?;
                }
            }
            UiToRustMessage::PinTab { id } => {
                if self.tabs.toggle_pin(id) {
                    self.save_session();
                    self.sync_ui_state();
                }
            }
            UiToRustMessage::CompleteOnboarding { destination } => {
                self.settings.onboarding_completed = true;
                if let Err(err) = self.storage.save_settings(&self.settings) {
                    eprintln!("Warning: {err:#}");
                }
                let target = destination
                    .as_deref()
                    .map(resolve_omnibox_input)
                    .unwrap_or_else(|| NEW_TAB_URL.to_string());
                self.tabs.navigate_active(&target)?;
                self.save_session();
                self.sync_ui_state();
            }
            UiToRustMessage::OpenOnboarding => {
                self.tabs.navigate_active(WELCOME_URL)?;
                self.save_session();
                self.sync_ui_state();
            }
            UiToRustMessage::PageMeta {
                title,
                url,
                favicon,
            } => {
                if let Some(tab_id) = source_tab_id {
                    self.on_tab_page_meta(tab_id, title, url, favicon);
                }
            }
        }

        Ok(())
    }

    fn open_new_tab(&mut self, url: &str) -> Result<()> {
        let id =
            self.tabs
                .create_tab(&self.window, &self.proxy, &mut self.web_context, url, true)?;
        if url == NEW_TAB_URL {
            let _ = self.chrome_webview.focus();
            let _ = self
                .chrome_webview
                .evaluate_script("window.__focusOmnibox && window.__focusOmnibox();");
        } else if let Some(tab) = self.tabs.get_tab(id) {
            let _ = tab.webview.focus();
        }
        self.save_session();
        self.sync_ui_state();
        Ok(())
    }

    fn on_tab_title_changed(&mut self, id: u64, title: String) {
        let Some(tab) = self.tabs.get_tab_mut(id) else {
            return;
        };
        let cleaned = title.trim();
        if tab.url == NEW_TAB_URL || tab.url == WELCOME_URL {
            tab.title = derive_title_from_url(&tab.url);
        } else if !cleaned.is_empty() {
            tab.title = cleaned.to_string();
        }
        self.sync_ui_state();
    }

    fn on_tab_load_state(&mut self, id: u64, is_loading: bool, raw_url: String) {
        let Some(tab) = self.tabs.get_tab_mut(id) else {
            return;
        };
        let logical_url = from_webview_url(&raw_url);
        let url_changed = !logical_url.is_empty() && tab.url != logical_url;
        if !logical_url.is_empty() {
            tab.url = logical_url.clone();
        }
        tab.is_loading = is_loading;

        if is_loading {
            if tab.url == NEW_TAB_URL || tab.url == WELCOME_URL {
                tab.title = derive_title_from_url(&tab.url);
                tab.favicon = None;
            } else if tab.title.is_empty() || url_changed {
                tab.title = derive_title_from_url(&tab.url);
                tab.favicon = None;
            }
        } else if tab.url == NEW_TAB_URL {
            tab.title = "New Tab".to_string();
            tab.favicon = None;
            self.push_bookmarks_to_newtab(id);
        } else if tab.url == WELCOME_URL {
            tab.title = "Welcome".to_string();
            tab.favicon = None;
        }

        if url_changed {
            self.save_session();
        }
        self.sync_ui_state();
    }

    fn on_tab_page_meta(
        &mut self,
        id: u64,
        title: Option<String>,
        raw_url: Option<String>,
        favicon: Option<String>,
    ) {
        let Some(tab) = self.tabs.get_tab_mut(id) else {
            return;
        };

        if let Some(raw) = raw_url {
            let logical = from_webview_url(&raw);
            if !logical.is_empty() && tab.url != logical {
                tab.url = logical;
            }
        }

        if tab.url == NEW_TAB_URL {
            tab.title = "New Tab".to_string();
            tab.favicon = None;
            self.push_bookmarks_to_newtab(id);
        } else if tab.url == WELCOME_URL {
            tab.title = "Welcome".to_string();
            tab.favicon = None;
        } else {
            if let Some(t) = title.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
                tab.title = t;
            }
            if let Some(icon) = favicon.filter(|s| !s.trim().is_empty()) {
                tab.favicon = Some(icon);
            }
        }

        self.sync_ui_state();
    }

    fn toggle_active_bookmark(&mut self) {
        let Some(active) = self.tabs.active_tab() else {
            return;
        };
        if active.url == NEW_TAB_URL || active.url == WELCOME_URL || active.url.is_empty() {
            return;
        }
        let url = active.url.clone();
        let title = active.title.clone();
        let favicon = active.favicon.clone();

        if let Some(pos) = self.bookmarks.iter().position(|b| b.url == url) {
            self.bookmarks.remove(pos);
        } else {
            self.bookmarks.push(Bookmark {
                title,
                url,
                favicon,
            });
        }

        if let Err(err) = self.storage.save_bookmarks(&self.bookmarks) {
            eprintln!("Warning: {err:#}");
        }
        self.push_bookmarks_to_all_newtabs();
        self.sync_ui_state();
    }

    fn remove_bookmark(&mut self, url: &str) {
        let before = self.bookmarks.len();
        self.bookmarks.retain(|b| b.url != url);
        if self.bookmarks.len() != before {
            if let Err(err) = self.storage.save_bookmarks(&self.bookmarks) {
                eprintln!("Warning: {err:#}");
            }
            self.push_bookmarks_to_all_newtabs();
            self.sync_ui_state();
        }
    }

    fn focus_address_bar(&self) {
        let _ = self.chrome_webview.focus();
        let _ = self
            .chrome_webview
            .evaluate_script("window.__focusOmnibox && window.__focusOmnibox();");
    }

    fn push_bookmarks_to_newtab(&self, tab_id: u64) {
        let Some(tab) = self.tabs.get_tab(tab_id) else {
            return;
        };
        if tab.url != NEW_TAB_URL {
            return;
        }
        if let Ok(json) = serde_json::to_string(&self.bookmarks) {
            let script = format!("window.__setBookmarks && window.__setBookmarks({json});");
            let _ = tab.webview.evaluate_script(&script);
        }
    }

    fn push_bookmarks_to_all_newtabs(&self) {
        for tab in self.tabs.tabs() {
            if tab.url == NEW_TAB_URL {
                self.push_bookmarks_to_newtab(tab.id);
            }
        }
    }

    fn record_closed_tab(&mut self, url: &str) {
        if !url.is_empty() && url != NEW_TAB_URL && url != WELCOME_URL {
            self.closed_tabs.push(url.to_string());
            if self.closed_tabs.len() > 20 {
                self.closed_tabs.remove(0);
            }
        }
    }

    fn sync_ui_state(&self) {
        let active_id = self.tabs.active_tab_id();
        let active_tab = self.tabs.active_tab();

        let active_url = active_tab
            .map(|t| display_url(&t.url))
            .unwrap_or_default();
        let is_loading = active_tab.map(|t| t.is_loading).unwrap_or(false);
        let zoom_percent = active_tab
            .map(|t| (t.zoom * 100.0).round() as u32)
            .unwrap_or(100);
        let is_bookmarked = active_tab
            .map(|t| {
                t.url != NEW_TAB_URL
                    && t.url != WELCOME_URL
                    && self.bookmarks.iter().any(|b| b.url == t.url)
            })
            .unwrap_or(false);

        let window_title = active_tab
            .map(|t| format!("{} - Pluto Browser", t.title))
            .unwrap_or_else(|| "Pluto Browser".to_string());
        self.window.set_title(&window_title);

        let tabs = self
            .tabs
            .tabs()
            .iter()
            .map(|t| TabUiInfo {
                id: t.id,
                title: t.title.clone(),
                url: t.url.clone(),
                display_url: display_url(&t.url),
                favicon: t.favicon.clone(),
                is_loading: t.is_loading,
                active: Some(t.id) == active_id,
                pinned: t.pinned,
                zoom_percent: (t.zoom * 100.0).round() as u32,
            })
            .collect();

        let state = RustToUiState {
            tabs,
            active_tab_id: active_id,
            active_url,
            is_loading,
            is_bookmarked,
            zoom_percent,
            can_reopen_closed: !self.closed_tabs.is_empty(),
            bookmarks: self.bookmarks.clone(),
        };

        if let Ok(json) = serde_json::to_string(&state) {
            let script = format!("window.__onBrowserState && window.__onBrowserState({json});");
            let _ = self.chrome_webview.evaluate_script(&script);
        }
    }
}

fn build_chrome_html() -> &'static str {
    static CHROME_HTML_CACHE: OnceLock<String> = OnceLock::new();
    CHROME_HTML_CACHE
        .get_or_init(|| {
            let style_tag = format!("<style>{CHROME_STYLE_CSS}</style>");
            let script_tag = format!("<script>{CHROME_APP_JS}</script>");
            CHROME_INDEX_HTML
                .replace("<!-- __STYLE__ -->", &style_tag)
                .replace("<!-- __SCRIPT__ -->", &script_tag)
        })
        .as_str()
}

