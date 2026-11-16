use crate::clipboard;
use crate::ipc::{resolve_omnibox_input, TabIpcMessage, UserEvent, NEW_TAB_URL, WELCOME_URL};
use crate::storage::{AppSettings, Bookmark, HistoryEntry, SessionData, StorageManager};
use crate::tabs::{derive_title_from_url, display_url, TabManager};
use crate::ui::{ActiveInput, NativeUi, PageTelemetry, RenderSnapshot, TabSnapshot, UiAction};
use anyhow::{anyhow, Context, Result};
use std::num::NonZeroU32;
use std::rc::Rc;
use tao::dpi::PhysicalSize;
use tao::event::{ElementState, MouseScrollDelta, WindowEvent};
use tao::event_loop::EventLoopProxy;
use tao::keyboard::{Key, KeyCode};
use tao::window::{CursorIcon, Theme, Window};
use wry::WebContext;

pub struct AppState {
    window: Rc<Window>,
    proxy: EventLoopProxy<UserEvent>,
    web_context: WebContext,
    _sb_context: softbuffer::Context<Rc<Window>>,
    surface: softbuffer::Surface<Rc<Window>, Rc<Window>>,
    surface_size: (u32, u32),
    ui: NativeUi,
    tabs: TabManager,
    storage: StorageManager,
    bookmarks: Vec<Bookmark>,
    history: Vec<HistoryEntry>,
    settings: AppSettings,
    closed_tabs: Vec<String>,
    needs_redraw: bool,
}

impl AppState {
    pub fn new(window: Rc<Window>, proxy: EventLoopProxy<UserEvent>) -> Result<Self> {
        let storage = StorageManager::new()?;
        let mut web_context = WebContext::new(Some(storage.webview_data_dir()));
        let bookmarks = storage.load_bookmarks().unwrap_or_else(|err| {
            eprintln!("Warning: {err:#}");
            Vec::new()
        });
        let history = storage.load_history().unwrap_or_else(|err| {
            eprintln!("Warning: {err:#}");
            Vec::new()
        });
        let mut settings = storage.load_settings().unwrap_or_else(|err| {
            eprintln!("Warning: {err:#}");
            AppSettings::default()
        });

        let dark_initial = settings.dark_mode || matches!(window.theme(), Theme::Dark);
        settings.dark_mode = dark_initial;

        let sb_context = softbuffer::Context::new(Rc::clone(&window))
            .map_err(|e| anyhow!("Failed to create softbuffer context: {e}"))?;
        let mut surface = softbuffer::Surface::new(&sb_context, Rc::clone(&window))
            .map_err(|e| anyhow!("Failed to create softbuffer surface: {e}"))?;

        let phys = window.inner_size();
        let sw = phys.width.max(1);
        let sh = phys.height.max(1);
        if let (Some(w), Some(h)) = (NonZeroU32::new(sw), NonZeroU32::new(sh)) {
            surface
                .resize(w, h)
                .map_err(|e| anyhow!("Failed to initialize surface size: {e}"))?;
        }

        let ui = NativeUi::new(dark_initial, settings.workspace_note.clone());
        let mut tabs = TabManager::new();
        let show_onboarding = !settings.onboarding_completed;
        let restored = storage.load_session().unwrap_or(None);
        let init_zoom = settings.ui_zoom;

        if let Some(session) = restored.filter(|s| !s.tabs.is_empty()) {
            let active_idx = session.active_index.min(session.tabs.len() - 1);
            for (idx, url) in session.tabs.iter().enumerate() {
                let resolved = resolve_omnibox_input(url);
                let make_active = !show_onboarding && idx == active_idx;
                let top_h = ui.top_chrome_height(settings.show_bookmarks_bar, &resolved);
                let bot_h = ui.bottom_dock_height(&resolved);
                tabs.create_tab(
                    window.as_ref(),
                    &proxy,
                    &mut web_context,
                    &resolved,
                    make_active,
                    init_zoom,
                    top_h,
                    bot_h,
                )?;
            }
            if show_onboarding && !tabs.tabs().iter().any(|t| t.url == WELCOME_URL) {
                let top_h = ui.top_chrome_height(settings.show_bookmarks_bar, WELCOME_URL);
                let bot_h = ui.bottom_dock_height(WELCOME_URL);
                tabs.create_tab(
                    window.as_ref(),
                    &proxy,
                    &mut web_context,
                    WELCOME_URL,
                    true,
                    init_zoom,
                    top_h,
                    bot_h,
                )?;
            } else if show_onboarding {
                if let Some(welcome_id) = tabs
                    .tabs()
                    .iter()
                    .find(|t| t.url == WELCOME_URL)
                    .map(|t| t.id)
                {
                    let top_h = ui.top_chrome_height(settings.show_bookmarks_bar, WELCOME_URL);
                    let bot_h = ui.bottom_dock_height(WELCOME_URL);
                    tabs.switch_to(
                        window.as_ref(),
                        &proxy,
                        &mut web_context,
                        welcome_id,
                        top_h,
                        bot_h,
                    );
                }
            }
        } else {
            let start_url = if show_onboarding {
                WELCOME_URL
            } else {
                NEW_TAB_URL
            };
            let top_h = ui.top_chrome_height(settings.show_bookmarks_bar, start_url);
            let bot_h = ui.bottom_dock_height(start_url);
            tabs.create_tab(
                window.as_ref(),
                &proxy,
                &mut web_context,
                start_url,
                true,
                init_zoom,
                top_h,
                bot_h,
            )?;
        }

        let mut app = Self {
            window,
            proxy,
            web_context,
            _sb_context: sb_context,
            surface,
            surface_size: (sw, sh),
            ui,
            tabs,
            storage,
            bookmarks,
            history,
            settings,
            closed_tabs: Vec::new(),
            needs_redraw: true,
        };

        app.sync_state(false);
        app.render();
        Ok(app)
    }

    pub fn needs_fast_animation(&self) -> bool {
        let active_url = self.active_url_str();
        let active_loading = self.tabs.active_tab().is_some_and(|t| t.is_loading);
        self.needs_redraw || self.ui.fast_animation_needed(active_url, active_loading)
    }

    pub fn needs_caret_blink(&self) -> bool {
        self.ui.caret_blink_needed()
    }

    pub fn request_redraw(&mut self) {
        self.needs_redraw = true;
        self.window.request_redraw();
    }

    fn active_url_str(&self) -> &str {
        self.tabs
            .active_tab()
            .map(|t| t.url.as_str())
            .unwrap_or(NEW_TAB_URL)
    }

    fn active_display_url(&self) -> String {
        display_url(self.active_url_str())
    }

    pub fn handle_resize(&mut self, new_size: Option<PhysicalSize<u32>>) {
        let size = new_size.unwrap_or_else(|| self.window.inner_size());
        let w = size.width.max(1);
        let h = size.height.max(1);
        if self.surface_size != (w, h) {
            if let (Some(nw), Some(nh)) = (NonZeroU32::new(w), NonZeroU32::new(h)) {
                let _ = self.surface.resize(nw, nh);
                self.surface_size = (w, h);
            }
        }
        self.update_webview_bounds();
        self.request_redraw();
    }

    fn update_webview_bounds(&self) {
        let active_url = self.active_url_str();
        let top_h = self
            .ui
            .top_chrome_height(self.settings.show_bookmarks_bar, active_url);
        let bot_h = self.ui.bottom_dock_height(active_url);
        self.tabs
            .resize_active(&self.window, top_h, bot_h, self.ui.command_palette_open);
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent<'_>) {
        let scale = self.window.scale_factor();
        match event {
            WindowEvent::Resized(size) => {
                self.handle_resize(Some(*size));
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                self.handle_resize(None);
            }
            WindowEvent::ThemeChanged(theme) => {
                self.ui.dark_mode = matches!(theme, Theme::Dark);
                self.settings.dark_mode = self.ui.dark_mode;
                self.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.ui.update_cursor(*position, scale);
                let cursor_icon = if self.ui.text_cursor_active() {
                    CursorIcon::Text
                } else if self.ui.pointer_cursor_active() {
                    CursorIcon::Hand
                } else {
                    CursorIcon::Default
                };
                self.window.set_cursor_icon(cursor_icon);
                self.request_redraw();
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => {
                let active_url = self.active_url_str().to_string();
                let actions = self.ui.handle_mouse_click(*button, &active_url);
                self.execute_ui_actions(actions);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y * 42.0,
                    MouseScrollDelta::PixelDelta(pos) => (pos.y / scale) as f32,
                    _ => 0.0,
                };
                if self.ui.modifiers.control_key() || self.ui.modifiers.super_key() {
                    if dy > 0.0 {
                        self.execute_ui_actions(vec![UiAction::ZoomIn]);
                    } else if dy < 0.0 {
                        self.execute_ui_actions(vec![UiAction::ZoomOut]);
                    }
                } else {
                    let active_url = self.active_url_str().to_string();
                    if self.ui.handle_wheel(&active_url, dy) {
                        self.request_redraw();
                    }
                }
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.ui.modifiers = *mods;
            }
            WindowEvent::KeyboardInput { event: key_ev, .. }
                if key_ev.state == ElementState::Pressed =>
            {
                let active_url = self.active_url_str().to_string();
                let disp = self.active_display_url();
                let mut actions =
                    self.ui
                        .handle_key_pressed(key_ev.physical_key, &active_url, &disp);

                let is_special_non_char = matches!(
                    key_ev.physical_key,
                    KeyCode::Enter
                        | KeyCode::NumpadEnter
                        | KeyCode::Escape
                        | KeyCode::Backspace
                        | KeyCode::Delete
                        | KeyCode::Tab
                        | KeyCode::ArrowUp
                        | KeyCode::ArrowDown
                        | KeyCode::ArrowLeft
                        | KeyCode::ArrowRight
                        | KeyCode::PageUp
                        | KeyCode::PageDown
                );

                if !is_special_non_char && self.ui.active_input != ActiveInput::None {
                    if let Some(txt) = key_ev.text {
                        for ch in txt.chars() {
                            if !ch.is_control() {
                                actions.extend(self.ui.handle_char_input(ch));
                            }
                        }
                    } else {
                        match &key_ev.logical_key {
                            Key::Character(s) => {
                                for ch in s.chars() {
                                    if !ch.is_control() {
                                        actions.extend(self.ui.handle_char_input(ch));
                                    }
                                }
                            }
                            Key::Space => {
                                actions.extend(self.ui.handle_char_input(' '));
                            }
                            _ => {}
                        }
                    }
                }
                self.execute_ui_actions(actions);
            }
            WindowEvent::ReceivedImeText(text) => {
                let mut actions = Vec::new();
                for ch in text.chars() {
                    if !ch.is_control() {
                        actions.extend(self.ui.handle_char_input(ch));
                    }
                }
                self.execute_ui_actions(actions);
            }
            _ => {}
        }
    }

    fn execute_ui_actions(&mut self, actions: Vec<UiAction>) {
        let mut need_sync = false;
        let mut need_draw = false;

        for action in actions {
            let res: Result<()> = (|| -> Result<()> {
                match action {
                    UiAction::FocusHostWindow => {
                        let _ = self.window.set_focus();
                    }
                    UiAction::RequestRedraw => {
                        need_draw = true;
                    }
                    UiAction::OpenWelcome => {
                        self.navigate_to(WELCOME_URL)?;
                        need_sync = true;
                    }
                    UiAction::NewTab(url) => {
                        let target = url
                            .as_deref()
                            .map(resolve_omnibox_input)
                            .unwrap_or_else(|| NEW_TAB_URL.to_string());
                        self.open_new_tab(&target)?;
                        need_draw = true;
                    }
                    UiAction::DuplicateTab => {
                        if let Some(url) = self.tabs.active_tab().map(|t| t.url.clone()) {
                            self.open_new_tab(&url)?;
                            need_draw = true;
                        }
                    }
                    UiAction::PinActiveTab => {
                        if let Some(id) = self.tabs.active_tab_id() {
                            if self.tabs.toggle_pin(id) {
                                self.save_session();
                                need_sync = true;
                            }
                        }
                    }
                    UiAction::CloseTab(id) => {
                        let target_id = if id == 0 {
                            self.tabs.active_tab_id()
                        } else {
                            Some(id)
                        };
                        if let Some(tid) = target_id {
                            let active_url = self.active_url_str().to_string();
                            let top_h = self
                                .ui
                                .top_chrome_height(self.settings.show_bookmarks_bar, &active_url);
                            let bot_h = self.ui.bottom_dock_height(&active_url);
                            if let Some(url) = self.tabs.close_tab(
                                &self.window,
                                &self.proxy,
                                &mut self.web_context,
                                tid,
                                top_h,
                                bot_h,
                            )? {
                                self.record_closed_tab(&url);
                            }
                            self.save_session();
                            need_sync = true;
                        }
                    }
                    UiAction::SwitchTab(id) => {
                        let next_url = self
                            .tabs
                            .get_tab(id)
                            .map(|t| t.url.clone())
                            .unwrap_or_default();
                        let top_h = self
                            .ui
                            .top_chrome_height(self.settings.show_bookmarks_bar, &next_url);
                        let bot_h = self.ui.bottom_dock_height(&next_url);
                        if self.tabs.switch_to(
                            &self.window,
                            &self.proxy,
                            &mut self.web_context,
                            id,
                            top_h,
                            bot_h,
                        ) {
                            self.save_session();
                            need_sync = true;
                        }
                    }
                    UiAction::PinTab(id) => {
                        if self.tabs.toggle_pin(id) {
                            self.save_session();
                            need_sync = true;
                        }
                    }
                    UiAction::NextTab => {
                        let active_url = self.active_url_str().to_string();
                        let top_h = self
                            .ui
                            .top_chrome_height(self.settings.show_bookmarks_bar, &active_url);
                        let bot_h = self.ui.bottom_dock_height(&active_url);
                        if self.tabs.switch_relative(
                            &self.window,
                            &self.proxy,
                            &mut self.web_context,
                            1,
                            top_h,
                            bot_h,
                        ) {
                            self.save_session();
                            need_sync = true;
                        }
                    }
                    UiAction::PrevTab => {
                        let active_url = self.active_url_str().to_string();
                        let top_h = self
                            .ui
                            .top_chrome_height(self.settings.show_bookmarks_bar, &active_url);
                        let bot_h = self.ui.bottom_dock_height(&active_url);
                        if self.tabs.switch_relative(
                            &self.window,
                            &self.proxy,
                            &mut self.web_context,
                            -1,
                            top_h,
                            bot_h,
                        ) {
                            self.save_session();
                            need_sync = true;
                        }
                    }
                    UiAction::Back => self.tabs.back_active()?,
                    UiAction::Forward => self.tabs.forward_active()?,
                    UiAction::Reload => self.tabs.reload_active()?,
                    UiAction::ToggleReloadOrStop => {
                        let loading = self.tabs.active_tab().is_some_and(|t| t.is_loading);
                        if loading {
                            self.tabs.stop_active()?;
                        } else {
                            self.tabs.reload_active()?;
                        }
                        need_sync = true;
                    }
                    UiAction::Home => {
                        self.navigate_to(NEW_TAB_URL)?;
                        need_sync = true;
                    }
                    UiAction::ReopenClosedTab => {
                        if let Some(url) = self.closed_tabs.pop() {
                            self.open_new_tab(&url)?;
                        }
                        need_draw = true;
                    }
                    UiAction::Navigate(input) => {
                        let resolved = resolve_omnibox_input(&input);
                        self.navigate_to(&resolved)?;
                        need_sync = true;
                    }
                    UiAction::CopyActiveUrl => {
                        let url = self.active_url_str().to_string();
                        if clipboard::set_clipboard_text(&url) {
                            self.ui.show_toast("URL copied");
                            need_draw = true;
                        }
                    }
                    UiAction::ToggleBookmark => {
                        self.toggle_active_bookmark();
                        need_draw = true;
                    }
                    UiAction::RemoveBookmark(url) => {
                        self.remove_bookmark(&url);
                        need_draw = true;
                    }
                    UiAction::ToggleBookmarksBar => {
                        self.settings.show_bookmarks_bar = !self.settings.show_bookmarks_bar;
                        let _ = self.storage.save_settings(&self.settings);
                        self.update_webview_bounds();
                        need_draw = true;
                    }
                    UiAction::ClearHistory => {
                        self.history.clear();
                        let _ = self.storage.save_history(&self.history);
                        self.ui.show_toast("History cleared");
                        need_draw = true;
                    }
                    UiAction::ZoomIn => {
                        let z = self.tabs.zoom_active(0.1)?;
                        self.settings.ui_zoom = z;
                        let _ = self.storage.save_settings(&self.settings);
                        need_draw = true;
                    }
                    UiAction::ZoomOut => {
                        let z = self.tabs.zoom_active(-0.1)?;
                        self.settings.ui_zoom = z;
                        let _ = self.storage.save_settings(&self.settings);
                        need_draw = true;
                    }
                    UiAction::ZoomReset => {
                        self.tabs.zoom_reset_active()?;
                        self.settings.ui_zoom = 1.0;
                        let _ = self.storage.save_settings(&self.settings);
                        need_draw = true;
                    }
                    UiAction::ToggleTheme => {
                        self.settings.dark_mode = self.ui.dark_mode;
                        let _ = self.storage.save_settings(&self.settings);
                        need_draw = true;
                    }
                    UiAction::ToggleFindBar => {
                        self.ui.find_open = !self.ui.find_open;
                        if self.ui.find_open {
                            self.ui.active_input = ActiveInput::FindBar;
                            let _ = self.window.set_focus();
                        } else if self.ui.active_input == ActiveInput::FindBar {
                            self.ui.active_input = ActiveInput::None;
                        }
                        self.update_webview_bounds();
                        need_draw = true;
                    }
                    UiAction::CloseFindBar => {
                        self.ui.find_open = false;
                        self.ui.active_input = ActiveInput::None;
                        self.update_webview_bounds();
                        need_draw = true;
                    }
                    UiAction::FindNext(q) => {
                        self.tabs.find_active(&q, false);
                    }
                    UiAction::FindPrev(q) => {
                        self.tabs.find_active(&q, true);
                    }
                    UiAction::ToggleCommandPalette => {
                        if self.ui.command_palette_open {
                            self.ui.close_command_palette();
                        } else {
                            self.ui.open_command_palette();
                            let _ = self.window.set_focus();
                        }
                        self.update_webview_bounds();
                        need_draw = true;
                    }
                    UiAction::ToggleInspector => {
                        self.ui.inspector_open = !self.ui.inspector_open;
                        if self.ui.inspector_open {
                            self.tabs.refresh_telemetry_active();
                        }
                        self.update_webview_bounds();
                        need_draw = true;
                    }
                    UiAction::RefreshInspector => {
                        self.tabs.refresh_telemetry_active();
                        need_draw = true;
                    }
                    UiAction::OpenDevtools => {
                        if !self.tabs.open_devtools_active() {
                            self.ui.inspector_open = !self.ui.inspector_open;
                            self.update_webview_bounds();
                            need_draw = true;
                        }
                    }
                    UiAction::CompleteOnboarding(dest) => {
                        self.settings.onboarding_completed = true;
                        let _ = self.storage.save_settings(&self.settings);
                        let resolved = resolve_omnibox_input(&dest);
                        self.navigate_to(&resolved)?;
                        need_sync = true;
                    }
                    UiAction::NoteChanged(note) => {
                        self.settings.workspace_note = note;
                        let _ = self.storage.save_settings(&self.settings);
                        need_draw = true;
                    }
                }
                Ok(())
            })();
            if let Err(err) = res {
                eprintln!("UI action error: {err:#}");
            }
        }

        if need_sync {
            self.sync_state(false);
        } else if need_draw {
            self.request_redraw();
        }
    }

    pub fn handle_user_event(&mut self, event: UserEvent) {
        let res = match event {
            UserEvent::TabIpc { tab_id, raw } => self.handle_tab_ipc(tab_id, &raw),
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
                self.open_new_tab(&resolved)
            }
        };
        if let Err(err) = res {
            eprintln!("Browser error: {err:#}");
        }
    }

    fn handle_tab_ipc(&mut self, tab_id: u64, raw: &str) -> Result<()> {
        let msg: TabIpcMessage =
            serde_json::from_str(raw).with_context(|| format!("Invalid IPC JSON: {raw}"))?;

        match msg {
            TabIpcMessage::NewTab => self.execute_ui_actions(vec![UiAction::NewTab(None)]),
            TabIpcMessage::CloseActiveTab => {
                self.execute_ui_actions(vec![UiAction::CloseTab(0)])
            }
            TabIpcMessage::DuplicateActiveTab => {
                self.execute_ui_actions(vec![UiAction::DuplicateTab])
            }
            TabIpcMessage::NextTab => self.execute_ui_actions(vec![UiAction::NextTab]),
            TabIpcMessage::PrevTab => self.execute_ui_actions(vec![UiAction::PrevTab]),
            TabIpcMessage::Back => self.execute_ui_actions(vec![UiAction::Back]),
            TabIpcMessage::Forward => self.execute_ui_actions(vec![UiAction::Forward]),
            TabIpcMessage::Reload => self.execute_ui_actions(vec![UiAction::Reload]),
            TabIpcMessage::ReopenClosedTab => {
                self.execute_ui_actions(vec![UiAction::ReopenClosedTab])
            }
            TabIpcMessage::FocusAddressBar => {
                let disp = self.active_display_url();
                let _ = self.window.set_focus();
                self.ui.focus_omnibox(&disp);
                self.request_redraw();
            }
            TabIpcMessage::ToggleCommandPalette => {
                self.execute_ui_actions(vec![UiAction::ToggleCommandPalette])
            }
            TabIpcMessage::ToggleBookmarksBar => {
                self.execute_ui_actions(vec![UiAction::ToggleBookmarksBar])
            }
            TabIpcMessage::OpenDevtools => {
                self.execute_ui_actions(vec![UiAction::OpenDevtools])
            }
            TabIpcMessage::ToggleInspector => {
                self.execute_ui_actions(vec![UiAction::ToggleInspector])
            }
            TabIpcMessage::ToggleFindBar => {
                let _ = self.window.set_focus();
                self.ui.find_open = !self.ui.find_open;
                if self.ui.find_open {
                    self.ui.active_input = ActiveInput::FindBar;
                }
                self.update_webview_bounds();
                self.request_redraw();
            }
            TabIpcMessage::ZoomIn => self.execute_ui_actions(vec![UiAction::ZoomIn]),
            TabIpcMessage::ZoomOut => self.execute_ui_actions(vec![UiAction::ZoomOut]),
            TabIpcMessage::ZoomReset => self.execute_ui_actions(vec![UiAction::ZoomReset]),
            TabIpcMessage::PageMeta {
                title,
                url,
                favicon,
            } => {
                if let Some(tab) = self.tabs.get_tab_mut(tab_id) {
                    if let Some(u) = url.filter(|s| !s.is_empty() && s != "about:blank") {
                        tab.url = u;
                    }
                    if !NativeUi::is_internal_page(&tab.url) {
                        if let Some(t) = title.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
                            tab.title = t;
                        }
                        if let Some(icon) = favicon.filter(|s| !s.trim().is_empty()) {
                            tab.favicon = Some(icon);
                        }
                        StorageManager::record_history_entry(&mut self.history, &tab.title, &tab.url);
                        let _ = self.storage.save_history(&self.history);
                    }
                    self.sync_state(false);
                }
            }
            TabIpcMessage::InspectorTelemetry {
                dom_nodes,
                scripts,
                stylesheets,
                images,
                dcl_ms,
                load_ms,
                local_keys,
                session_keys,
                origin,
            } => {
                if let Some(tab) = self.tabs.get_tab_mut(tab_id) {
                    tab.telemetry = Some(PageTelemetry {
                        dom_nodes,
                        scripts,
                        stylesheets,
                        images,
                        dcl_ms,
                        load_ms,
                        local_keys,
                        session_keys,
                        origin,
                    });
                    self.request_redraw();
                }
            }
        }
        Ok(())
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

    fn navigate_to(&mut self, url: &str) -> Result<()> {
        let top_h = self
            .ui
            .top_chrome_height(self.settings.show_bookmarks_bar, url);
        let bot_h = self.ui.bottom_dock_height(url);
        self.tabs.navigate_active(
            &self.window,
            &self.proxy,
            &mut self.web_context,
            url,
            top_h,
            bot_h,
        )?;
        self.ui.newtab_search_text.clear();
        self.save_session();
        Ok(())
    }

    fn open_new_tab(&mut self, url: &str) -> Result<()> {
        let top_h = self
            .ui
            .top_chrome_height(self.settings.show_bookmarks_bar, url);
        let bot_h = self.ui.bottom_dock_height(url);
        let _id = self.tabs.create_tab(
            &self.window,
            &self.proxy,
            &mut self.web_context,
            url,
            true,
            self.settings.ui_zoom,
            top_h,
            bot_h,
        )?;
        if url == NEW_TAB_URL {
            let _ = self.window.set_focus();
            self.ui.focus_omnibox("");
        }
        self.save_session();
        self.sync_state(false);
        Ok(())
    }

    fn on_tab_title_changed(&mut self, id: u64, title: String) {
        let Some(tab) = self.tabs.get_tab_mut(id) else {
            return;
        };
        let cleaned = title.trim();
        if NativeUi::is_internal_page(&tab.url) {
            tab.title = derive_title_from_url(&tab.url);
        } else if !cleaned.is_empty() {
            tab.title = cleaned.to_string();
        }
        self.sync_state(false);
    }

    fn on_tab_load_state(&mut self, id: u64, is_loading: bool, raw_url: String) {
        let Some(tab) = self.tabs.get_tab_mut(id) else {
            return;
        };
        let url_changed = !raw_url.is_empty() && raw_url != "about:blank" && tab.url != raw_url;
        if !raw_url.is_empty() && raw_url != "about:blank" {
            tab.url = raw_url;
        }
        tab.is_loading = is_loading;
        if is_loading && (tab.title.is_empty() || url_changed) {
            tab.title = derive_title_from_url(&tab.url);
            tab.favicon = None;
        }
        if !is_loading && !NativeUi::is_internal_page(&tab.url) {
            StorageManager::record_history_entry(&mut self.history, &tab.title, &tab.url);
            let _ = self.storage.save_history(&self.history);
        }
        if url_changed {
            self.save_session();
        }
        self.sync_state(false);
    }

    fn toggle_active_bookmark(&mut self) {
        let Some(active) = self.tabs.active_tab() else {
            return;
        };
        if NativeUi::is_internal_page(&active.url) || active.url.is_empty() {
            return;
        }
        let url = active.url.clone();
        let title = active.title.clone();
        let favicon = active.favicon.clone();

        if let Some(pos) = self.bookmarks.iter().position(|b| b.url == url) {
            self.bookmarks.remove(pos);
            self.ui.show_toast("Bookmark removed");
        } else {
            self.bookmarks.push(Bookmark {
                title,
                url,
                favicon,
            });
            self.ui.show_toast("Bookmark saved");
        }

        if let Err(err) = self.storage.save_bookmarks(&self.bookmarks) {
            eprintln!("Warning: {err:#}");
        }
        self.sync_state(false);
    }

    fn remove_bookmark(&mut self, url: &str) {
        let before = self.bookmarks.len();
        self.bookmarks.retain(|b| b.url != url);
        if self.bookmarks.len() != before {
            let _ = self.storage.save_bookmarks(&self.bookmarks);
            self.sync_state(false);
        }
    }

    fn record_closed_tab(&mut self, url: &str) {
        if !url.is_empty() && !NativeUi::is_internal_page(url) {
            self.closed_tabs.push(url.to_string());
            if self.closed_tabs.len() > 20 {
                self.closed_tabs.remove(0);
            }
        }
    }

    fn sync_state(&mut self, force_omnibox: bool) {
        let window_title = self
            .tabs
            .active_tab()
            .map(|t| format!("{} - Pluto Browser", t.title))
            .unwrap_or_else(|| "Pluto Browser".to_string());
        self.window.set_title(&window_title);
        let disp = self.active_display_url();
        self.ui.sync_omnibox_from_url(&disp, force_omnibox);
        self.update_webview_bounds();
        self.request_redraw();
    }

    pub fn render(&mut self) {
        self.needs_redraw = false;
        let phys = self.window.inner_size();
        let w = phys.width.max(1);
        let h = phys.height.max(1);
        if self.surface_size != (w, h) {
            if let (Some(nw), Some(nh)) = (NonZeroU32::new(w), NonZeroU32::new(h)) {
                let _ = self.surface.resize(nw, nh);
                self.surface_size = (w, h);
            }
        }

        let active_id = self.tabs.active_tab_id();
        let active_url = self.active_url_str().to_string();
        let active_is_loading = self.tabs.active_tab().is_some_and(|t| t.is_loading);
        let zoom_percent = self
            .tabs
            .active_tab()
            .map(|t| (t.zoom * 100.0).round() as u32)
            .unwrap_or(100);
        let is_bookmarked = self
            .tabs
            .active_tab()
            .map(|t| {
                !NativeUi::is_internal_page(&t.url)
                    && self.bookmarks.iter().any(|b| b.url == t.url)
            })
            .unwrap_or(false);
        let telemetry = self.tabs.active_tab().and_then(|t| t.telemetry.as_ref());

        let tabs_snap: Vec<TabSnapshot> = self
            .tabs
            .tabs()
            .iter()
            .map(|t| TabSnapshot {
                id: t.id,
                title: t.title.clone(),
                url: t.url.clone(),
                is_loading: t.is_loading,
                active: Some(t.id) == active_id,
                pinned: t.pinned,
            })
            .collect();

        let snapshot = RenderSnapshot {
            tabs: &tabs_snap,
            active_url: &active_url,
            active_is_loading,
            is_bookmarked,
            zoom_percent,
            can_reopen_closed: !self.closed_tabs.is_empty(),
            show_bookmarks_bar: self.settings.show_bookmarks_bar,
            bookmarks: &self.bookmarks,
            history: &self.history,
            telemetry,
        };

        let scale = self.window.scale_factor();
        let webview_visible = !self.ui.command_palette_open
            && !NativeUi::is_internal_page(&active_url)
            && self
                .tabs
                .active_tab()
                .is_some_and(|t| t.webview.is_some());

        let top_h_px = ((self
            .ui
            .top_chrome_height(self.settings.show_bookmarks_bar, &active_url)
            * scale)
            .ceil() as u32)
            .clamp(1, h);
        let bot_h_logical = self.ui.bottom_dock_height(&active_url);

        if let Ok(mut buf) = self.surface.buffer_mut() {
            let buf_w = buf.width().get() as usize;
            let buf_h = buf.height().get() as usize;
            self.ui.render(&mut buf, buf_w, buf_h, scale, &snapshot);

            if !webview_visible {
                let _ = buf.present();
            } else {
                let mut damage = Vec::with_capacity(2);
                if let (Some(nw), Some(nh)) =
                    (NonZeroU32::new(buf_w as u32), NonZeroU32::new(top_h_px.min(buf_h as u32)))
                {
                    damage.push(softbuffer::Rect {
                        x: 0,
                        y: 0,
                        width: nw,
                        height: nh,
                    });
                }
                if bot_h_logical > 0.0 {
                    let bot_h_px = ((bot_h_logical * scale).ceil() as u32).clamp(1, buf_h as u32);
                    let bot_y_px = (buf_h as u32).saturating_sub(bot_h_px);
                    if let (Some(nw), Some(nh)) =
                        (NonZeroU32::new(buf_w as u32), NonZeroU32::new(bot_h_px))
                    {
                        damage.push(softbuffer::Rect {
                            x: 0,
                            y: bot_y_px,
                            width: nw,
                            height: nh,
                        });
                    }
                }
                let _ = buf.present_with_damage(&damage);
            }
        }
    }
}
