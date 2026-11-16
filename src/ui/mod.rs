pub mod canvas;
pub mod theme;

use crate::clipboard;
use crate::ipc::{GOOGLE_HOME_URL, NEW_TAB_URL, WELCOME_URL};
use crate::storage::{Bookmark, HistoryEntry};
use crate::ui::canvas::{Canvas, FontStyle, FontSystem};
use crate::ui::theme::{Color, ThemePalette};
use std::time::Instant;
use tao::dpi::PhysicalPosition;
use tao::event::MouseButton;
use tao::keyboard::{KeyCode, ModifiersState};

pub const CHROME_BASE_HEIGHT: f32 = 96.0;
pub const BOOKMARKS_BAR_HEIGHT: f32 = 32.0;
pub const FIND_BAR_HEIGHT: f32 = 42.0;
pub const INSPECTOR_DOCK_HEIGHT: f32 = 188.0;

#[derive(Debug, Clone)]
pub struct TabSnapshot {
    pub id: u64,
    pub title: String,
    pub url: String,
    pub is_loading: bool,
    pub active: bool,
    pub pinned: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PageTelemetry {
    pub dom_nodes: u32,
    pub scripts: u32,
    pub stylesheets: u32,
    pub images: u32,
    pub dcl_ms: u32,
    pub load_ms: u32,
    pub local_keys: u32,
    pub session_keys: u32,
    pub origin: String,
}

pub struct RenderSnapshot<'a> {
    pub tabs: &'a [TabSnapshot],
    pub active_url: &'a str,
    pub active_is_loading: bool,
    pub is_bookmarked: bool,
    pub zoom_percent: u32,
    pub can_reopen_closed: bool,
    pub show_bookmarks_bar: bool,
    pub bookmarks: &'a [Bookmark],
    pub history: &'a [HistoryEntry],
    pub telemetry: Option<&'a PageTelemetry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveInput {
    None,
    Omnibox,
    NewTabSearch,
    Scratchpad,
    FindBar,
    CommandPalette,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitId {
    BrandWelcome,
    Tab(u64),
    CloseTab(u64),
    NewTabBtn,
    ThemeToggle,
    Back,
    Forward,
    ReloadOrStop,
    Home,
    ReopenClosed,
    Omnibox,
    OmniboxClear,
    CopyUrl,
    ZoomOut,
    ZoomReset,
    ZoomIn,
    ToggleBookmark,
    ToggleFind,
    TogglePalette,
    ToggleInspector,
    OpenDevtools,
    BookmarkBarItem(String),
    BookmarkBarToggle,
    FindInput,
    FindPrev,
    FindNext,
    FindClose,
    InspectorTab(usize),
    InspectorRefresh,
    InspectorDevtools,
    InspectorClose,
    // Command Palette
    PaletteOverlayBackdrop,
    PaletteBox,
    PaletteItem(usize),
    // Welcome page
    WelcomeHeroOpen,
    WelcomeHeroXray,
    WelcomeMockTab(usize),
    WelcomeDestPill(usize),
    WelcomeFinish,
    // New tab page
    NewTabWelcomeBtn,
    NewTabSearchInput,
    NewTabSearchSubmit,
    NewTabCategory(usize),
    OpenUrlTile(String),
    RemoveBookmark(String),
    ClearHistory,
    ScratchpadBox,
}

#[derive(Debug, Clone)]
struct HitRegion {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    id: HitId,
    clickable: bool,
}

#[derive(Debug, Clone)]
pub enum UiAction {
    FocusHostWindow,
    OpenWelcome,
    NewTab(Option<String>),
    CloseTab(u64),
    DuplicateTab,
    PinActiveTab,
    SwitchTab(u64),
    PinTab(u64),
    Back,
    Forward,
    Reload,
    ToggleReloadOrStop,
    Home,
    ReopenClosedTab,
    NextTab,
    PrevTab,
    Navigate(String),
    CopyActiveUrl,
    ToggleBookmark,
    RemoveBookmark(String),
    ToggleBookmarksBar,
    ClearHistory,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleTheme,
    ToggleFindBar,
    FindNext(String),
    FindPrev(String),
    CloseFindBar,
    ToggleCommandPalette,
    ToggleInspector,
    OpenDevtools,
    RefreshInspector,
    CompleteOnboarding(String),
    NoteChanged(String),
    RequestRedraw,
}

#[derive(Debug, Clone)]
enum PaletteEntryAction {
    Action(UiAction),
    SwitchTab(u64),
    Navigate(String),
}

#[derive(Debug, Clone)]
struct PaletteEntry {
    title: String,
    subtitle: String,
    badge: String,
    exec: PaletteEntryAction,
}

struct QuickLink {
    title: &'static str,
    url: &'static str,
    badge: &'static str,
    cat: usize, // 1 = Engineering, 2 = Reference
    tag: &'static str,
}

const QUICK_LINKS: &[QuickLink] = &[
    QuickLink {
        title: "Google Search",
        url: GOOGLE_HOME_URL,
        badge: "G",
        cat: 2,
        tag: "Search",
    },
    QuickLink {
        title: "GitHub",
        url: "https://github.com",
        badge: "GH",
        cat: 1,
        tag: "Code",
    },
    QuickLink {
        title: "Rust Lang",
        url: "https://www.rust-lang.org",
        badge: "R",
        cat: 1,
        tag: "Rust",
    },
    QuickLink {
        title: "MDN Web Docs",
        url: "https://developer.mozilla.org",
        badge: "M",
        cat: 1,
        tag: "Docs",
    },
    QuickLink {
        title: "Wikipedia",
        url: "https://en.wikipedia.org",
        badge: "W",
        cat: 2,
        tag: "Wiki",
    },
    QuickLink {
        title: "Hacker News",
        url: "https://news.ycombinator.com",
        badge: "Y",
        cat: 2,
        tag: "Feed",
    },
];

const DESTINATIONS: &[(&str, &str)] = &[
    ("New Tab Workspace", NEW_TAB_URL),
    ("Google Search", GOOGLE_HOME_URL),
    ("GitHub", "https://github.com"),
    ("Rust Docs", "https://www.rust-lang.org"),
];

pub struct NativeUi {
    pub dark_mode: bool,
    pub fonts: FontSystem,
    pub active_input: ActiveInput,
    pub omnibox_text: String,
    pub omnibox_select_all: bool,
    pub newtab_search_text: String,
    pub newtab_category: usize,
    pub scratchpad_text: String,
    pub find_open: bool,
    pub find_text: String,
    pub inspector_open: bool,
    pub inspector_tab: usize,
    pub command_palette_open: bool,
    pub command_palette_query: String,
    pub command_palette_idx: usize,
    pub toast_message: Option<(String, Instant)>,
    pub welcome_scroll_y: f32,
    pub welcome_max_scroll: f32,
    pub newtab_scroll_y: f32,
    pub newtab_max_scroll: f32,
    pub welcome_mock_tab: usize,
    pub welcome_dest_idx: usize,
    pub cursor_pos: (f32, f32),
    pub modifiers: ModifiersState,
    pub last_click: Option<(Instant, HitId)>,
    pub start_instant: Instant,
    pub last_frame_ms: f32,
    hits: Vec<HitRegion>,
    palette_entries: Vec<PaletteEntry>,
    last_synced_url: String,
}

impl NativeUi {
    pub fn new(dark_mode: bool, workspace_note: String) -> Self {
        Self {
            dark_mode,
            fonts: FontSystem::load_system(),
            active_input: ActiveInput::None,
            omnibox_text: String::new(),
            omnibox_select_all: false,
            newtab_search_text: String::new(),
            newtab_category: 0,
            scratchpad_text: workspace_note,
            find_open: false,
            find_text: String::new(),
            inspector_open: false,
            inspector_tab: 0,
            command_palette_open: false,
            command_palette_query: String::new(),
            command_palette_idx: 0,
            toast_message: None,
            welcome_scroll_y: 0.0,
            welcome_max_scroll: 0.0,
            newtab_scroll_y: 0.0,
            newtab_max_scroll: 0.0,
            welcome_mock_tab: 0,
            welcome_dest_idx: 0,
            cursor_pos: (0.0, 0.0),
            modifiers: ModifiersState::empty(),
            last_click: None,
            start_instant: Instant::now(),
            last_frame_ms: 0.0,
            hits: Vec::with_capacity(96),
            palette_entries: Vec::new(),
            last_synced_url: String::new(),
        }
    }

    pub fn is_internal_page(url: &str) -> bool {
        url == WELCOME_URL || url == NEW_TAB_URL
    }

    pub fn top_chrome_height(&self, show_bookmarks_bar: bool, active_url: &str) -> f64 {
        let mut h = CHROME_BASE_HEIGHT;
        if show_bookmarks_bar {
            h += BOOKMARKS_BAR_HEIGHT;
        }
        if self.find_open && !Self::is_internal_page(active_url) {
            h += FIND_BAR_HEIGHT;
        }
        h as f64
    }

    pub fn bottom_dock_height(&self, active_url: &str) -> f64 {
        if self.inspector_open && !Self::is_internal_page(active_url) {
            INSPECTOR_DOCK_HEIGHT as f64
        } else {
            0.0
        }
    }

    /// Distinguishes continuous 60 FPS planetary animations from low-power idle states.
    pub fn fast_animation_needed(&self, active_url: &str, active_loading: bool) -> bool {
        active_url == WELCOME_URL || active_url == NEW_TAB_URL || active_loading || self.toast_message.is_some()
    }

    pub fn caret_blink_needed(&self) -> bool {
        self.active_input != ActiveInput::None
    }

    pub fn show_toast(&mut self, msg: impl Into<String>) {
        self.toast_message = Some((msg.into(), Instant::now()));
    }

    pub fn sync_omnibox_from_url(&mut self, display_url: &str, force: bool) {
        if force || self.active_input != ActiveInput::Omnibox {
            if force || self.last_synced_url != display_url {
                self.omnibox_text = display_url.to_string();
                self.last_synced_url = display_url.to_string();
                if force {
                    self.omnibox_select_all = !self.omnibox_text.is_empty();
                }
            }
        }
    }

    pub fn focus_omnibox(&mut self, current_display_url: &str) {
        self.command_palette_open = false;
        self.active_input = ActiveInput::Omnibox;
        self.omnibox_text = current_display_url.to_string();
        self.omnibox_select_all = !self.omnibox_text.is_empty();
    }

    pub fn open_command_palette(&mut self) {
        self.command_palette_open = true;
        self.command_palette_query.clear();
        self.command_palette_idx = 0;
        self.active_input = ActiveInput::CommandPalette;
    }

    pub fn close_command_palette(&mut self) {
        self.command_palette_open = false;
        if self.active_input == ActiveInput::CommandPalette {
            self.active_input = ActiveInput::None;
        }
    }

    pub fn update_cursor(&mut self, phys: PhysicalPosition<f64>, scale: f64) -> bool {
        let lx = (phys.x / scale) as f32;
        let ly = (phys.y / scale) as f32;
        self.cursor_pos = (lx, ly);
        true
    }

    pub fn handle_wheel(&mut self, active_url: &str, delta_y: f32) -> bool {
        if active_url == WELCOME_URL {
            self.welcome_scroll_y =
                (self.welcome_scroll_y - delta_y).clamp(0.0, self.welcome_max_scroll);
            return true;
        }
        if active_url == NEW_TAB_URL {
            self.newtab_scroll_y =
                (self.newtab_scroll_y - delta_y).clamp(0.0, self.newtab_max_scroll);
            return true;
        }
        false
    }

    fn hit_at(&self, lx: f32, ly: f32) -> Option<HitId> {
        for r in self.hits.iter().rev() {
            if r.clickable && lx >= r.x && lx <= r.x + r.w && ly >= r.y && ly <= r.y + r.h {
                return Some(r.id.clone());
            }
        }
        None
    }

    pub fn pointer_cursor_active(&self) -> bool {
        self.hit_at(self.cursor_pos.0, self.cursor_pos.1).is_some()
    }

    pub fn text_cursor_active(&self) -> bool {
        matches!(
            self.hit_at(self.cursor_pos.0, self.cursor_pos.1),
            Some(
                HitId::Omnibox
                    | HitId::NewTabSearchInput
                    | HitId::ScratchpadBox
                    | HitId::FindInput
                    | HitId::PaletteBox
            )
        )
    }

    pub fn handle_mouse_click(
        &mut self,
        button: MouseButton,
        active_url: &str,
    ) -> Vec<UiAction> {
        let mut actions = Vec::new();
        let (lx, ly) = self.cursor_pos;
        let Some(hit) = self.hit_at(lx, ly) else {
            self.active_input = ActiveInput::None;
            self.omnibox_select_all = false;
            if self.command_palette_open {
                self.close_command_palette();
                actions.push(UiAction::ToggleCommandPalette);
            }
            actions.push(UiAction::FocusHostWindow);
            actions.push(UiAction::RequestRedraw);
            return actions;
        };

        if button == MouseButton::Middle {
            if let HitId::Tab(id) = hit {
                actions.push(UiAction::CloseTab(id));
            }
            return actions;
        }

        if button == MouseButton::Right {
            if let HitId::Tab(id) = hit {
                actions.push(UiAction::PinTab(id));
            }
            return actions;
        }

        if button != MouseButton::Left {
            return actions;
        }

        // Check double click on Tab for pin toggle
        let now = Instant::now();
        let is_double = self
            .last_click
            .as_ref()
            .is_some_and(|(t, prev_id)| *prev_id == hit && now.duration_since(*t).as_millis() < 360);
        self.last_click = Some((now, hit.clone()));

        if is_double {
            if let HitId::Tab(id) = hit {
                actions.push(UiAction::PinTab(id));
                return actions;
            }
        }

        // Clicking anywhere on our native window should bring OS keyboard focus back from child WebView
        actions.push(UiAction::FocusHostWindow);

        match &hit {
            HitId::Omnibox => {
                if self.active_input == ActiveInput::Omnibox {
                    // Clicking inside while already focused deselects so user can keep typing seamlessly
                    self.omnibox_select_all = false;
                } else {
                    self.active_input = ActiveInput::Omnibox;
                    self.omnibox_select_all = !self.omnibox_text.is_empty();
                }
                actions.push(UiAction::RequestRedraw);
                return actions;
            }
            HitId::OmniboxClear => {
                self.active_input = ActiveInput::Omnibox;
                self.omnibox_text.clear();
                self.omnibox_select_all = false;
                actions.push(UiAction::RequestRedraw);
                return actions;
            }
            HitId::NewTabSearchInput => {
                self.active_input = ActiveInput::NewTabSearch;
                self.omnibox_select_all = false;
                actions.push(UiAction::RequestRedraw);
                return actions;
            }
            HitId::ScratchpadBox => {
                self.active_input = ActiveInput::Scratchpad;
                self.omnibox_select_all = false;
                actions.push(UiAction::RequestRedraw);
                return actions;
            }
            HitId::FindInput => {
                self.active_input = ActiveInput::FindBar;
                self.omnibox_select_all = false;
                actions.push(UiAction::RequestRedraw);
                return actions;
            }
            HitId::PaletteBox => {
                self.active_input = ActiveInput::CommandPalette;
                actions.push(UiAction::RequestRedraw);
                return actions;
            }
            HitId::PaletteOverlayBackdrop => {
                self.close_command_palette();
                actions.push(UiAction::ToggleCommandPalette);
                return actions;
            }
            HitId::PaletteItem(idx) => {
                if let Some(entry) = self.palette_entries.get(*idx).cloned() {
                    self.close_command_palette();
                    actions.push(UiAction::ToggleCommandPalette);
                    match entry.exec {
                        PaletteEntryAction::Action(act) => actions.push(act),
                        PaletteEntryAction::SwitchTab(tid) => actions.push(UiAction::SwitchTab(tid)),
                        PaletteEntryAction::Navigate(url) => actions.push(UiAction::Navigate(url)),
                    }
                }
                return actions;
            }
            _ => {
                self.active_input = ActiveInput::None;
                self.omnibox_select_all = false;
            }
        }

        match hit {
            HitId::BrandWelcome => actions.push(UiAction::OpenWelcome),
            HitId::Tab(id) => actions.push(UiAction::SwitchTab(id)),
            HitId::CloseTab(id) => actions.push(UiAction::CloseTab(id)),
            HitId::NewTabBtn => actions.push(UiAction::NewTab(None)),
            HitId::ThemeToggle => {
                self.dark_mode = !self.dark_mode;
                actions.push(UiAction::ToggleTheme);
            }
            HitId::Back => actions.push(UiAction::Back),
            HitId::Forward => actions.push(UiAction::Forward),
            HitId::ReloadOrStop => actions.push(UiAction::ToggleReloadOrStop),
            HitId::Home => actions.push(UiAction::Home),
            HitId::ReopenClosed => actions.push(UiAction::ReopenClosedTab),
            HitId::CopyUrl => actions.push(UiAction::CopyActiveUrl),
            HitId::ZoomOut => actions.push(UiAction::ZoomOut),
            HitId::ZoomReset => actions.push(UiAction::ZoomReset),
            HitId::ZoomIn => actions.push(UiAction::ZoomIn),
            HitId::ToggleBookmark => actions.push(UiAction::ToggleBookmark),
            HitId::ToggleFind => {
                if !Self::is_internal_page(active_url) {
                    actions.push(UiAction::ToggleFindBar);
                }
            }
            HitId::TogglePalette => actions.push(UiAction::ToggleCommandPalette),
            HitId::ToggleInspector => actions.push(UiAction::ToggleInspector),
            HitId::OpenDevtools | HitId::InspectorDevtools => actions.push(UiAction::OpenDevtools),
            HitId::BookmarkBarItem(url) => actions.push(UiAction::Navigate(url)),
            HitId::BookmarkBarToggle => actions.push(UiAction::ToggleBookmarksBar),
            HitId::FindPrev => actions.push(UiAction::FindPrev(self.find_text.clone())),
            HitId::FindNext => actions.push(UiAction::FindNext(self.find_text.clone())),
            HitId::FindClose => actions.push(UiAction::CloseFindBar),
            HitId::InspectorTab(idx) => {
                self.inspector_tab = idx;
                actions.push(UiAction::RequestRedraw);
            }
            HitId::InspectorRefresh => actions.push(UiAction::RefreshInspector),
            HitId::InspectorClose => {
                self.inspector_open = false;
                actions.push(UiAction::ToggleInspector);
            }
            HitId::WelcomeHeroOpen => actions.push(UiAction::CompleteOnboarding(NEW_TAB_URL.to_string())),
            HitId::WelcomeHeroXray => actions.push(UiAction::ToggleInspector),
            HitId::WelcomeMockTab(idx) => {
                self.welcome_mock_tab = idx;
                actions.push(UiAction::RequestRedraw);
            }
            HitId::WelcomeDestPill(idx) => {
                self.welcome_dest_idx = idx.min(DESTINATIONS.len() - 1);
                actions.push(UiAction::RequestRedraw);
            }
            HitId::WelcomeFinish => {
                let dest = DESTINATIONS
                    .get(self.welcome_dest_idx)
                    .map(|(_, u)| (*u).to_string())
                    .unwrap_or_else(|| NEW_TAB_URL.to_string());
                actions.push(UiAction::CompleteOnboarding(dest));
            }
            HitId::NewTabWelcomeBtn => actions.push(UiAction::OpenWelcome),
            HitId::NewTabSearchSubmit => {
                let q = self.newtab_search_text.trim().to_string();
                if !q.is_empty() {
                    actions.push(UiAction::Navigate(q));
                }
            }
            HitId::NewTabCategory(idx) => {
                self.newtab_category = idx;
                actions.push(UiAction::RequestRedraw);
            }
            HitId::OpenUrlTile(url) => actions.push(UiAction::Navigate(url)),
            HitId::RemoveBookmark(url) => actions.push(UiAction::RemoveBookmark(url)),
            HitId::ClearHistory => actions.push(UiAction::ClearHistory),
            HitId::Omnibox
            | HitId::OmniboxClear
            | HitId::NewTabSearchInput
            | HitId::ScratchpadBox
            | HitId::FindInput
            | HitId::PaletteOverlayBackdrop
            | HitId::PaletteBox
            | HitId::PaletteItem(_) => {}
        }

        actions
    }

    pub fn handle_char_input(&mut self, ch: char) -> Vec<UiAction> {
        if self.active_input == ActiveInput::None {
            return Vec::new();
        }
        if self.modifiers.control_key() || self.modifiers.super_key() || self.modifiers.alt_key() {
            return Vec::new();
        }
        if ch.is_control() {
            return Vec::new();
        }

        let mut actions = Vec::new();
        match self.active_input {
            ActiveInput::Omnibox => {
                if self.omnibox_select_all {
                    self.omnibox_text.clear();
                    self.omnibox_select_all = false;
                }
                self.omnibox_text.push(ch);
                actions.push(UiAction::RequestRedraw);
            }
            ActiveInput::NewTabSearch => {
                self.newtab_search_text.push(ch);
                actions.push(UiAction::RequestRedraw);
            }
            ActiveInput::FindBar => {
                self.find_text.push(ch);
                actions.push(UiAction::RequestRedraw);
            }
            ActiveInput::CommandPalette => {
                self.command_palette_query.push(ch);
                self.command_palette_idx = 0;
                actions.push(UiAction::RequestRedraw);
            }
            ActiveInput::Scratchpad => {
                self.scratchpad_text.push(ch);
                actions.push(UiAction::NoteChanged(self.scratchpad_text.clone()));
                actions.push(UiAction::RequestRedraw);
            }
            ActiveInput::None => {}
        }
        actions
    }

    pub fn handle_key_pressed(
        &mut self,
        code: KeyCode,
        active_url: &str,
        current_display_url: &str,
    ) -> Vec<UiAction> {
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
        let shift = self.modifiers.shift_key();
        let alt = self.modifiers.alt_key();
        let mut actions = Vec::new();

        // Global shortcuts handled directly in Rust
        if code == KeyCode::F12 || (ctrl && shift && code == KeyCode::KeyI) {
            actions.push(UiAction::OpenDevtools);
            return actions;
        }
        if ctrl && shift && (code == KeyCode::KeyD || code == KeyCode::KeyC) {
            actions.push(UiAction::ToggleInspector);
            return actions;
        }
        if ctrl && shift && code == KeyCode::KeyB {
            actions.push(UiAction::ToggleBookmarksBar);
            return actions;
        }
        if ctrl && shift && code == KeyCode::KeyK {
            actions.push(UiAction::DuplicateTab);
            return actions;
        }
        if ctrl && !shift && !alt && (code == KeyCode::KeyK || code == KeyCode::KeyP) {
            actions.push(UiAction::ToggleCommandPalette);
            return actions;
        }
        if ctrl && !shift && !alt && code == KeyCode::KeyF {
            if !Self::is_internal_page(active_url) {
                actions.push(UiAction::ToggleFindBar);
            }
            return actions;
        }
        if ctrl && shift && code == KeyCode::KeyT {
            actions.push(UiAction::ReopenClosedTab);
            return actions;
        }
        if ctrl && !alt && code == KeyCode::KeyT {
            actions.push(UiAction::NewTab(None));
            return actions;
        }
        if ctrl && !alt && code == KeyCode::KeyW {
            actions.push(UiAction::CloseTab(0));
            return actions;
        }
        if ctrl && !alt && code == KeyCode::KeyL {
            self.focus_omnibox(current_display_url);
            actions.push(UiAction::FocusHostWindow);
            actions.push(UiAction::RequestRedraw);
            return actions;
        }
        if (ctrl && !alt && code == KeyCode::KeyR) || code == KeyCode::F5 {
            actions.push(UiAction::Reload);
            return actions;
        }
        if ctrl && code == KeyCode::Tab {
            if shift {
                actions.push(UiAction::PrevTab);
            } else {
                actions.push(UiAction::NextTab);
            }
            return actions;
        }
        if alt && !ctrl && code == KeyCode::ArrowLeft {
            actions.push(UiAction::Back);
            return actions;
        }
        if alt && !ctrl && code == KeyCode::ArrowRight {
            actions.push(UiAction::Forward);
            return actions;
        }
        if ctrl && !alt && (code == KeyCode::Equal || code == KeyCode::NumpadAdd) {
            actions.push(UiAction::ZoomIn);
            return actions;
        }
        if ctrl && !alt && (code == KeyCode::Minus || code == KeyCode::NumpadSubtract) {
            actions.push(UiAction::ZoomOut);
            return actions;
        }
        if ctrl && !alt && code == KeyCode::Digit0 {
            actions.push(UiAction::ZoomReset);
            return actions;
        }

        // Command palette navigation
        if self.command_palette_open {
            match code {
                KeyCode::ArrowDown => {
                    if !self.palette_entries.is_empty() {
                        self.command_palette_idx =
                            (self.command_palette_idx + 1) % self.palette_entries.len();
                    }
                    actions.push(UiAction::RequestRedraw);
                    return actions;
                }
                KeyCode::ArrowUp => {
                    if !self.palette_entries.is_empty() {
                        self.command_palette_idx = (self.command_palette_idx + self.palette_entries.len() - 1)
                            % self.palette_entries.len();
                    }
                    actions.push(UiAction::RequestRedraw);
                    return actions;
                }
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    if let Some(entry) = self.palette_entries.get(self.command_palette_idx).cloned() {
                        self.close_command_palette();
                        actions.push(UiAction::ToggleCommandPalette);
                        match entry.exec {
                            PaletteEntryAction::Action(act) => actions.push(act),
                            PaletteEntryAction::SwitchTab(tid) => actions.push(UiAction::SwitchTab(tid)),
                            PaletteEntryAction::Navigate(url) => actions.push(UiAction::Navigate(url)),
                        }
                    } else {
                        self.close_command_palette();
                        actions.push(UiAction::ToggleCommandPalette);
                    }
                    return actions;
                }
                KeyCode::Escape => {
                    self.close_command_palette();
                    actions.push(UiAction::ToggleCommandPalette);
                    return actions;
                }
                _ => {}
            }
        }

        // Non-input scrolling shortcuts on internal pages
        if self.active_input == ActiveInput::None {
            if active_url == WELCOME_URL {
                match code {
                    KeyCode::PageDown | KeyCode::Space => {
                        self.welcome_scroll_y =
                            (self.welcome_scroll_y + 280.0).clamp(0.0, self.welcome_max_scroll);
                        actions.push(UiAction::RequestRedraw);
                    }
                    KeyCode::PageUp => {
                        self.welcome_scroll_y =
                            (self.welcome_scroll_y - 280.0).clamp(0.0, self.welcome_max_scroll);
                        actions.push(UiAction::RequestRedraw);
                    }
                    KeyCode::ArrowDown => {
                        self.welcome_scroll_y =
                            (self.welcome_scroll_y + 72.0).clamp(0.0, self.welcome_max_scroll);
                        actions.push(UiAction::RequestRedraw);
                    }
                    KeyCode::ArrowUp => {
                        self.welcome_scroll_y =
                            (self.welcome_scroll_y - 72.0).clamp(0.0, self.welcome_max_scroll);
                        actions.push(UiAction::RequestRedraw);
                    }
                    _ => {}
                }
            }
            return actions;
        }

        // Select All / Copy / Paste inside focused Rust inputs
        if ctrl && !shift && !alt {
            if code == KeyCode::KeyA && self.active_input == ActiveInput::Omnibox {
                self.omnibox_select_all = true;
                actions.push(UiAction::RequestRedraw);
                return actions;
            }
            if code == KeyCode::KeyC {
                let clip_src = match self.active_input {
                    ActiveInput::Omnibox => Some(if self.omnibox_text.is_empty() {
                        active_url
                    } else {
                        self.omnibox_text.as_str()
                    }),
                    ActiveInput::NewTabSearch => Some(self.newtab_search_text.as_str()),
                    ActiveInput::FindBar => Some(self.find_text.as_str()),
                    ActiveInput::Scratchpad => Some(self.scratchpad_text.as_str()),
                    ActiveInput::CommandPalette | ActiveInput::None => None,
                };
                if let Some(text) = clip_src {
                    if clipboard::set_clipboard_text(text) {
                        self.show_toast("Copied to clipboard");
                        actions.push(UiAction::RequestRedraw);
                    }
                }
                return actions;
            }
            if code == KeyCode::KeyV {
                if let Some(pasted) = clipboard::get_clipboard_text() {
                    let single_line: String = pasted
                        .chars()
                        .filter(|c| *c != '\r' && *c != '\n')
                        .collect();
                    match self.active_input {
                        ActiveInput::Omnibox => {
                            if self.omnibox_select_all {
                                self.omnibox_text.clear();
                                self.omnibox_select_all = false;
                            }
                            self.omnibox_text.push_str(&single_line);
                        }
                        ActiveInput::NewTabSearch => {
                            self.newtab_search_text.push_str(&single_line);
                        }
                        ActiveInput::FindBar => {
                            self.find_text.push_str(&single_line);
                        }
                        ActiveInput::CommandPalette => {
                            self.command_palette_query.push_str(&single_line);
                            self.command_palette_idx = 0;
                        }
                        ActiveInput::Scratchpad => {
                            self.scratchpad_text.push_str(&pasted);
                            actions.push(UiAction::NoteChanged(self.scratchpad_text.clone()));
                        }
                        ActiveInput::None => {}
                    }
                    actions.push(UiAction::RequestRedraw);
                }
                return actions;
            }
        }

        match code {
            KeyCode::Backspace | KeyCode::Delete => {
                match self.active_input {
                    ActiveInput::Omnibox => {
                        if self.omnibox_select_all {
                            self.omnibox_text.clear();
                            self.omnibox_select_all = false;
                        } else if ctrl {
                            pop_word(&mut self.omnibox_text);
                        } else {
                            self.omnibox_text.pop();
                        }
                    }
                    ActiveInput::NewTabSearch => {
                        if ctrl {
                            pop_word(&mut self.newtab_search_text);
                        } else {
                            self.newtab_search_text.pop();
                        }
                    }
                    ActiveInput::FindBar => {
                        if ctrl {
                            pop_word(&mut self.find_text);
                        } else {
                            self.find_text.pop();
                        }
                    }
                    ActiveInput::CommandPalette => {
                        if ctrl {
                            pop_word(&mut self.command_palette_query);
                        } else {
                            self.command_palette_query.pop();
                        }
                        self.command_palette_idx = 0;
                    }
                    ActiveInput::Scratchpad => {
                        if ctrl {
                            pop_word(&mut self.scratchpad_text);
                        } else {
                            self.scratchpad_text.pop();
                        }
                        actions.push(UiAction::NoteChanged(self.scratchpad_text.clone()));
                    }
                    ActiveInput::None => {}
                }
                actions.push(UiAction::RequestRedraw);
            }
            KeyCode::Enter | KeyCode::NumpadEnter => match self.active_input {
                ActiveInput::Omnibox => {
                    let target = self.omnibox_text.trim().to_string();
                    self.active_input = ActiveInput::None;
                    self.omnibox_select_all = false;
                    actions.push(UiAction::Navigate(target));
                }
                ActiveInput::NewTabSearch => {
                    let target = self.newtab_search_text.trim().to_string();
                    if !target.is_empty() {
                        actions.push(UiAction::Navigate(target));
                    }
                }
                ActiveInput::FindBar => {
                    if shift {
                        actions.push(UiAction::FindPrev(self.find_text.clone()));
                    } else {
                        actions.push(UiAction::FindNext(self.find_text.clone()));
                    }
                }
                ActiveInput::Scratchpad => {
                    self.scratchpad_text.push('\n');
                    actions.push(UiAction::NoteChanged(self.scratchpad_text.clone()));
                    actions.push(UiAction::RequestRedraw);
                }
                ActiveInput::CommandPalette | ActiveInput::None => {}
            },
            KeyCode::Escape => {
                if self.active_input == ActiveInput::FindBar {
                    actions.push(UiAction::CloseFindBar);
                } else {
                    self.active_input = ActiveInput::None;
                    self.omnibox_select_all = false;
                    self.omnibox_text = current_display_url.to_string();
                    actions.push(UiAction::RequestRedraw);
                }
            }
            _ => {}
        }

        actions
    }

    pub fn render(
        &mut self,
        buf: &mut [u32],
        phys_w: usize,
        phys_h: usize,
        scale: f64,
        snapshot: &RenderSnapshot<'_>,
    ) {
        let frame_start = Instant::now();
        self.hits.clear();

        if let Some((_, ts)) = &self.toast_message {
            if ts.elapsed().as_secs_f32() > 1.8 {
                self.toast_message = None;
            }
        }

        let pal = if self.dark_mode {
            ThemePalette::dark()
        } else {
            ThemePalette::light()
        };
        let time_sec = self.start_instant.elapsed().as_secs_f32();
        let cursor = self.cursor_pos;
        let is_internal = Self::is_internal_page(snapshot.active_url);

        let mut c = Canvas::new(buf, phys_w, phys_h, scale);
        let win_w = c.logical_width();
        let win_h = c.logical_height();
        c.clear(pal.canvas);

        let mut hit_buf: Vec<HitRegion> = Vec::with_capacity(96);

        let top_h = self.top_chrome_height(snapshot.show_bookmarks_bar, snapshot.active_url) as f32;
        let page_top = CHROME_BASE_HEIGHT
            + if snapshot.show_bookmarks_bar {
                BOOKMARKS_BAR_HEIGHT
            } else {
                0.0
            };
        let page_vh = (win_h
            - page_top
            - if self.inspector_open {
                INSPECTOR_DOCK_HEIGHT
            } else {
                0.0
            })
        .max(60.0);

        // 1. Pure Rust Internal Page Views with per-tab Zoom In / Zoom Out scaling!
        if is_internal {
            let zoom = (snapshot.zoom_percent as f32 / 100.0).clamp(0.6, 2.0);
            let mut page_hits: Vec<HitRegion> = Vec::with_capacity(48);
            let z_cursor = (cursor.0 / zoom, cursor.1 / zoom);
            let z_vy = page_top / zoom;
            let z_vw = win_w / zoom;
            let z_vh = page_vh / zoom;

            c.set_scale(scale * zoom as f64);
            if snapshot.active_url == WELCOME_URL {
                self.render_welcome_page(
                    &mut c,
                    &pal,
                    time_sec,
                    z_cursor,
                    0.0,
                    z_vy,
                    z_vw,
                    z_vh,
                    snapshot.zoom_percent,
                    &mut page_hits,
                );
            } else if snapshot.active_url == NEW_TAB_URL {
                self.render_newtab_page(
                    &mut c,
                    &pal,
                    time_sec,
                    z_cursor,
                    0.0,
                    z_vy,
                    z_vw,
                    z_vh,
                    snapshot.bookmarks,
                    snapshot.history,
                    &mut page_hits,
                );
            }
            c.set_scale(scale);

            for r in page_hits {
                hit_buf.push(HitRegion {
                    x: r.x * zoom,
                    y: r.y * zoom,
                    w: r.w * zoom,
                    h: r.h * zoom,
                    id: r.id,
                    clickable: r.clickable,
                });
            }
        }

        // 2. Bottom Inspector Dock
        if self.inspector_open {
            let dock_y = (win_h - INSPECTOR_DOCK_HEIGHT).max(top_h + 40.0);
            self.render_inspector_dock(
                &mut c,
                &pal,
                cursor,
                0.0,
                dock_y,
                win_w,
                INSPECTOR_DOCK_HEIGHT,
                snapshot,
                &mut hit_buf,
            );
        }

        // 3. Find Bar (when open on external pages)
        if self.find_open && !is_internal {
            let fb_y = CHROME_BASE_HEIGHT
                + if snapshot.show_bookmarks_bar {
                    BOOKMARKS_BAR_HEIGHT
                } else {
                    0.0
                };
            self.render_find_bar(
                &mut c,
                &pal,
                time_sec,
                cursor,
                0.0,
                fb_y,
                win_w,
                FIND_BAR_HEIGHT,
                &mut hit_buf,
            );
        }

        // 4. Bookmarks Bar (optional 32px row below toolbar)
        if snapshot.show_bookmarks_bar {
            self.render_bookmarks_bar(
                &mut c,
                &pal,
                cursor,
                0.0,
                CHROME_BASE_HEIGHT,
                win_w,
                BOOKMARKS_BAR_HEIGHT,
                snapshot.bookmarks,
                &mut hit_buf,
            );
        }

        // 5. Native Rust Top Chrome Bar (96px: 44px Tabs + 52px Toolbar)
        self.render_chrome_bar(
            &mut c,
            &pal,
            time_sec,
            cursor,
            win_w,
            snapshot,
            &mut hit_buf,
        );

        // 6. Floating Command Palette Overlay (Ctrl+K)
        if self.command_palette_open {
            self.render_command_palette(
                &mut c,
                &pal,
                time_sec,
                cursor,
                win_w,
                win_h,
                snapshot,
                &mut hit_buf,
            );
        }

        // 7. Subtle feedback toast pill (e.g. "Copied to clipboard")
        if let Some((msg, _)) = &self.toast_message {
            let tw = self.fonts.measure_text(FontStyle::SemiBold, 12.0, msg) + 28.0;
            let tx = (win_w - tw) * 0.5;
            let ty = top_h + 10.0;
            c.fill_rounded_rect(tx, ty, tw, 30.0, 8.0, pal.brand_navy);
            c.stroke_rounded_rect(tx, ty, tw, 30.0, 8.0, 1.0, pal.primary);
            c.draw_text(
                &mut self.fonts,
                FontStyle::SemiBold,
                12.0,
                tx + 14.0,
                ty + 8.0,
                pal.on_primary,
                msg,
            );
        }

        self.hits = hit_buf;
        self.last_frame_ms = frame_start.elapsed().as_secs_f32() * 1000.0;
    }

    #[allow(clippy::too_many_arguments)]
    fn render_chrome_bar(
        &mut self,
        c: &mut Canvas<'_>,
        pal: &ThemePalette,
        time_sec: f32,
        cursor: (f32, f32),
        win_w: f32,
        snapshot: &RenderSnapshot<'_>,
        hits: &mut Vec<HitRegion>,
    ) {
        // Tab strip row (0..44)
        c.fill_rect(0.0, 0.0, win_w, 44.0, pal.strip);

        // Brand button (Planet Pluto)
        let brand_x = 10.0;
        let brand_y = 7.0;
        let brand_sz = 30.0;
        let brand_hover = point_in(cursor, brand_x, brand_y, brand_sz, brand_sz);
        c.fill_rounded_rect(brand_x, brand_y, brand_sz, brand_sz, 8.0, pal.canvas);
        c.stroke_rounded_rect(
            brand_x,
            brand_y,
            brand_sz,
            brand_sz,
            8.0,
            1.0,
            if brand_hover { pal.primary } else { pal.hairline },
        );
        c.draw_pluto_logo(brand_x + 4.0, brand_y + 4.0, 22.0, if brand_hover { 1.0 } else { 0.0 });
        hits.push(HitRegion {
            x: brand_x,
            y: brand_y,
            w: brand_sz,
            h: brand_sz,
            id: HitId::BrandWelcome,
            clickable: true,
        });

        // Right-side Theme Toggle button inside strip
        let theme_x = win_w - 40.0;
        let theme_y = 7.0;
        let theme_hover = point_in(cursor, theme_x, theme_y, 30.0, 30.0);
        if theme_hover {
            c.fill_rounded_rect(theme_x, theme_y, 30.0, 30.0, 8.0, pal.btn_hover);
        }
        c.draw_icon_theme(
            theme_x + 15.0,
            theme_y + 15.0,
            self.dark_mode,
            if theme_hover { pal.ink } else { pal.steel },
        );
        hits.push(HitRegion {
            x: theme_x,
            y: theme_y,
            w: 30.0,
            h: 30.0,
            id: HitId::ThemeToggle,
            clickable: true,
        });

        // Tabs layout
        let tabs_start_x = 48.0;
        let tabs_end_x = (theme_x - 42.0).max(tabs_start_x + 90.0);
        let avail_w = tabs_end_x - tabs_start_x;

        let pinned_count = snapshot.tabs.iter().filter(|t| t.pinned).count();
        let unpinned_count = snapshot.tabs.len().saturating_sub(pinned_count);
        let pinned_total = pinned_count as f32 * 48.0;
        let unpinned_w = if unpinned_count > 0 {
            ((avail_w - pinned_total - (snapshot.tabs.len() as f32 * 5.0)) / unpinned_count as f32)
                .clamp(92.0, 230.0)
        } else {
            175.0
        };

        let mut tx = tabs_start_x;
        c.push_clip(tabs_start_x, 0.0, avail_w, 44.0);
        for tab in snapshot.tabs {
            let tw = if tab.pinned { 44.0 } else { unpinned_w };
            if tx > tabs_end_x {
                break;
            }
            let ty = 6.0;
            let th = 38.0;
            let hovered = point_in(cursor, tx, ty, tw, th);

            if tab.active {
                c.fill_rounded_rect(tx, ty, tw, th + 2.0, 9.0, pal.canvas);
                c.stroke_rounded_rect(tx, ty, tw, th + 4.0, 9.0, 1.0, pal.hairline);
                c.fill_rounded_rect(tx + 10.0, ty + 1.0, (tw - 20.0).max(8.0), 2.5, 1.2, pal.primary);
            } else if hovered {
                c.fill_rounded_rect(tx, ty, tw, th, 9.0, pal.tab_hover);
            }

            let icon_cx = if tab.pinned { tx + tw * 0.5 } else { tx + 17.0 };
            let icon_cy = ty + 19.0;
            if tab.is_loading {
                c.draw_spinner(icon_cx, icon_cy, 5.8, time_sec, pal.hairline, pal.primary);
            } else if tab.url == WELCOME_URL || tab.url == NEW_TAB_URL {
                c.fill_circle(icon_cx, icon_cy, 5.2, pal.primary);
                c.fill_circle(icon_cx + 3.5, icon_cy - 3.5, 2.2, pal.brand_purple_soft);
            } else {
                c.draw_icon_globe(icon_cx, icon_cy, if tab.active { pal.ink } else { pal.steel });
            }

            if !tab.pinned {
                let close_x = tx + tw - 24.0;
                let close_y = ty + 9.0;
                let close_hover = point_in(cursor, close_x, close_y, 20.0, 20.0);

                let text_x = tx + 30.0;
                let text_w = (close_x - text_x - 4.0).max(20.0);
                c.draw_text_ellipsis(
                    &mut self.fonts,
                    FontStyle::SemiBold,
                    12.5,
                    text_x,
                    ty + 11.5,
                    text_w,
                    if tab.active { pal.ink } else { pal.slate },
                    &tab.title,
                );

                if close_hover {
                    c.fill_rounded_rect(close_x, close_y, 20.0, 20.0, 5.0, pal.btn_active);
                }
                c.draw_icon_close(
                    close_x + 10.0,
                    close_y + 10.0,
                    if close_hover || tab.active {
                        pal.charcoal
                    } else {
                        pal.steel
                    },
                );
                hits.push(HitRegion {
                    x: tx,
                    y: ty,
                    w: tw - 26.0,
                    h: th,
                    id: HitId::Tab(tab.id),
                    clickable: true,
                });
                hits.push(HitRegion {
                    x: close_x,
                    y: close_y,
                    w: 20.0,
                    h: 20.0,
                    id: HitId::CloseTab(tab.id),
                    clickable: true,
                });
            } else {
                hits.push(HitRegion {
                    x: tx,
                    y: ty,
                    w: tw,
                    h: th,
                    id: HitId::Tab(tab.id),
                    clickable: true,
                });
            }

            tx += tw + 5.0;
        }
        c.pop_clip();

        // New Tab '+' button right after tabs
        let plus_x = (tx + 2.0).min(theme_x - 36.0);
        let plus_y = 7.0;
        let plus_hover = point_in(cursor, plus_x, plus_y, 30.0, 30.0);
        if plus_hover {
            c.fill_rounded_rect(plus_x, plus_y, 30.0, 30.0, 8.0, pal.btn_hover);
        }
        c.draw_icon_plus(plus_x + 15.0, plus_y + 15.0, if plus_hover { pal.ink } else { pal.steel });
        hits.push(HitRegion {
            x: plus_x,
            y: plus_y,
            w: 30.0,
            h: 30.0,
            id: HitId::NewTabBtn,
            clickable: true,
        });

        // Toolbar row (44..96)
        c.fill_rect(0.0, 44.0, win_w, 52.0, pal.canvas);
        c.fill_rect(0.0, 95.0, win_w, 1.0, pal.hairline);

        let tb_y = 54.0;
        let nav_buttons = [
            (10.0, HitId::Back),
            (44.0, HitId::Forward),
            (78.0, HitId::ReloadOrStop),
            (112.0, HitId::Home),
            (146.0, HitId::ReopenClosed),
        ];

        for (bx, hit_id) in nav_buttons {
            let disabled = matches!(hit_id, HitId::ReopenClosed) && !snapshot.can_reopen_closed;
            let hovered = !disabled && point_in(cursor, bx, tb_y, 32.0, 32.0);
            if hovered {
                c.fill_rounded_rect(bx, tb_y, 32.0, 32.0, 8.0, pal.btn_hover);
            }
            let ic = if disabled {
                pal.muted
            } else if hovered {
                pal.ink
            } else {
                pal.slate
            };
            let cx = bx + 16.0;
            let cy = tb_y + 16.0;
            match &hit_id {
                HitId::Back => c.draw_icon_back(cx, cy, ic),
                HitId::Forward => c.draw_icon_forward(cx, cy, ic),
                HitId::ReloadOrStop => {
                    if snapshot.active_is_loading {
                        c.draw_icon_stop(cx, cy, ic);
                    } else {
                        c.draw_icon_reload(cx, cy, ic);
                    }
                }
                HitId::Home => c.draw_icon_home(cx, cy, ic),
                HitId::ReopenClosed => c.draw_icon_reopen(cx, cy, ic),
                _ => {}
            }
            if !disabled {
                hits.push(HitRegion {
                    x: bx,
                    y: tb_y,
                    w: 32.0,
                    h: 32.0,
                    id: hit_id,
                    clickable: true,
                });
            }
        }

        // Right tool controls: Zoom [- 100% +] + Find + CmdPalette + Inspector + F12 Dev
        let f12_w = 78.0;
        let insp_w = 88.0;
        let pal_w = 32.0;
        let find_w = 32.0;
        let zoom_grp_w = 104.0;

        let f12_x = win_w - f12_w - 10.0;
        let insp_x = f12_x - insp_w - 6.0;
        let pal_x = insp_x - pal_w - 6.0;
        let find_x = pal_x - find_w - 6.0;
        let zoom_x = find_x - zoom_grp_w - 10.0;

        // Omnibox fills remaining middle width
        let omni_x = 186.0;
        let omni_w = (zoom_x - omni_x - 12.0).max(160.0);
        let omni_y = 52.0;
        let omni_h = 36.0;
        let omni_focused = self.active_input == ActiveInput::Omnibox;

        c.fill_rounded_rect(
            omni_x,
            omni_y,
            omni_w,
            omni_h,
            9.0,
            if omni_focused { pal.canvas } else { pal.omnibox_bg },
        );
        c.stroke_rounded_rect(
            omni_x,
            omni_y,
            omni_w,
            omni_h,
            9.0,
            if omni_focused { 2.0 } else { 1.0 },
            if omni_focused { pal.primary } else { pal.hairline },
        );

        // Protocol pill badge inside omnibox
        let badge_label = if snapshot.active_url.starts_with("https://") {
            "HTTPS"
        } else if snapshot.active_url == WELCOME_URL || snapshot.active_url == NEW_TAB_URL {
            "Pluto"
        } else {
            "HTTP"
        };
        let (badge_bg, badge_fg) = if badge_label == "HTTPS" {
            (pal.tint_mint, pal.tint_mint_ink)
        } else if badge_label == "Pluto" {
            (pal.tint_lavender, pal.tint_lavender_ink)
        } else {
            (pal.tint_gray, pal.slate)
        };
        let badge_w = self.fonts.measure_text(FontStyle::SemiBold, 11.0, badge_label) + 14.0;
        c.fill_rounded_rect(omni_x + 7.0, omni_y + 7.0, badge_w, 22.0, 6.0, badge_bg);
        c.draw_text(
            &mut self.fonts,
            FontStyle::SemiBold,
            11.0,
            omni_x + 14.0,
            omni_y + 11.5,
            badge_fg,
            badge_label,
        );

        // Right controls inside Omnibox: Bookmark star + Copy URL + Clear (when typing)
        let star_x = omni_x + omni_w - 30.0;
        let star_hover = point_in(cursor, star_x, omni_y + 5.0, 26.0, 26.0);
        if star_hover {
            c.fill_rounded_rect(star_x, omni_y + 5.0, 26.0, 26.0, 6.0, pal.btn_hover);
        }
        c.draw_icon_star(
            star_x + 13.0,
            omni_y + 18.0,
            snapshot.is_bookmarked,
            if snapshot.is_bookmarked {
                pal.star_active
            } else if star_hover {
                pal.ink
            } else {
                pal.steel
            },
        );
        hits.push(HitRegion {
            x: star_x,
            y: omni_y + 5.0,
            w: 26.0,
            h: 26.0,
            id: HitId::ToggleBookmark,
            clickable: true,
        });

        let copy_x = star_x - 28.0;
        let copy_hover = point_in(cursor, copy_x, omni_y + 5.0, 26.0, 26.0);
        if copy_hover {
            c.fill_rounded_rect(copy_x, omni_y + 5.0, 26.0, 26.0, 6.0, pal.btn_hover);
        }
        c.draw_icon_copy(copy_x + 13.0, omni_y + 18.0, if copy_hover { pal.ink } else { pal.steel });
        hits.push(HitRegion {
            x: copy_x,
            y: omni_y + 5.0,
            w: 26.0,
            h: 26.0,
            id: HitId::CopyUrl,
            clickable: true,
        });

        let mut text_right_edge = copy_x - 6.0;
        if omni_focused && !self.omnibox_text.is_empty() {
            let clr_x = copy_x - 26.0;
            let clr_hov = point_in(cursor, clr_x, omni_y + 7.0, 22.0, 22.0);
            if clr_hov {
                c.fill_rounded_rect(clr_x, omni_y + 7.0, 22.0, 22.0, 11.0, pal.btn_active);
            }
            c.draw_icon_close(clr_x + 11.0, omni_y + 18.0, if clr_hov { pal.ink } else { pal.steel });
            hits.push(HitRegion {
                x: clr_x,
                y: omni_y + 7.0,
                w: 22.0,
                h: 22.0,
                id: HitId::OmniboxClear,
                clickable: true,
            });
            text_right_edge = clr_x - 6.0;
        }

        // Omnibox editable text + visible caret + horizontal auto-scroll
        let text_x = omni_x + badge_w + 15.0;
        let text_max_w = (text_right_edge - text_x).max(24.0);
        c.push_clip(text_x, omni_y + 2.0, text_max_w, omni_h - 4.0);
        if self.omnibox_text.is_empty() {
            if omni_focused && ((time_sec * 2.0) as i32) % 2 == 0 {
                c.fill_rect(text_x, omni_y + 8.0, 2.0, 20.0, pal.primary);
            }
            c.draw_text(
                &mut self.fonts,
                FontStyle::Regular,
                13.0,
                text_x + 4.0,
                omni_y + 9.5,
                pal.steel,
                "Search Google or type a URL (Ctrl+L)...",
            );
        } else {
            let visible = tail_fitting_width(&self.fonts, FontStyle::Regular, 13.0, &self.omnibox_text, text_max_w - 6.0);
            let tw = self.fonts.measure_text(FontStyle::Regular, 13.0, visible);
            if omni_focused && self.omnibox_select_all {
                c.fill_rounded_rect(
                    text_x - 2.0,
                    omni_y + 7.0,
                    (tw + 6.0).min(text_max_w),
                    22.0,
                    4.0,
                    Color::rgba(pal.primary.r, pal.primary.g, pal.primary.b, 58),
                );
            }
            c.draw_text(
                &mut self.fonts,
                FontStyle::Regular,
                13.0,
                text_x,
                omni_y + 9.5,
                pal.ink,
                visible,
            );
            if omni_focused && ((time_sec * 2.0) as i32) % 2 == 0 {
                let cx = text_x + tw + 2.0;
                c.fill_rect(cx, omni_y + 8.0, 2.0, 20.0, pal.primary);
            }
        }
        c.pop_clip();
        hits.push(HitRegion {
            x: omni_x,
            y: omni_y,
            w: text_right_edge - omni_x,
            h: omni_h,
            id: HitId::Omnibox,
            clickable: true,
        });

        // Visible Zoom cluster: [- | 100% | +]
        c.fill_rounded_rect(zoom_x, tb_y, zoom_grp_w, 32.0, 8.0, pal.surface);
        c.stroke_rounded_rect(zoom_x, tb_y, zoom_grp_w, 32.0, 8.0, 1.0, pal.hairline);

        let z_out_hov = point_in(cursor, zoom_x, tb_y, 28.0, 32.0);
        if z_out_hov {
            c.fill_rounded_rect(zoom_x + 1.0, tb_y + 1.0, 27.0, 30.0, 7.0, pal.btn_hover);
        }
        c.draw_icon_minus(zoom_x + 14.0, tb_y + 16.0, if z_out_hov { pal.ink } else { pal.slate });
        hits.push(HitRegion {
            x: zoom_x,
            y: tb_y,
            w: 28.0,
            h: 32.0,
            id: HitId::ZoomOut,
            clickable: true,
        });

        let z_mid_x = zoom_x + 28.0;
        let z_mid_w = 48.0;
        let z_mid_hov = point_in(cursor, z_mid_x, tb_y, z_mid_w, 32.0);
        if z_mid_hov {
            c.fill_rect(z_mid_x, tb_y + 1.0, z_mid_w, 30.0, pal.btn_hover);
        }
        let zlabel = format!("{}%", snapshot.zoom_percent);
        let zw = self.fonts.measure_text(FontStyle::SemiBold, 11.5, &zlabel);
        c.draw_text(
            &mut self.fonts,
            FontStyle::SemiBold,
            11.5,
            z_mid_x + (z_mid_w - zw) * 0.5,
            tb_y + 9.0,
            if snapshot.zoom_percent != 100 { pal.primary } else { pal.charcoal },
            &zlabel,
        );
        hits.push(HitRegion {
            x: z_mid_x,
            y: tb_y,
            w: z_mid_w,
            h: 32.0,
            id: HitId::ZoomReset,
            clickable: true,
        });

        let z_in_x = z_mid_x + z_mid_w;
        let z_in_hov = point_in(cursor, z_in_x, tb_y, 28.0, 32.0);
        if z_in_hov {
            c.fill_rounded_rect(z_in_x, tb_y + 1.0, 27.0, 30.0, 7.0, pal.btn_hover);
        }
        c.draw_icon_plus(z_in_x + 14.0, tb_y + 16.0, if z_in_hov { pal.ink } else { pal.slate });
        hits.push(HitRegion {
            x: z_in_x,
            y: tb_y,
            w: 28.0,
            h: 32.0,
            id: HitId::ZoomIn,
            clickable: true,
        });

        // Find button
        let find_hover = point_in(cursor, find_x, tb_y, find_w, 32.0);
        if self.find_open || find_hover {
            c.fill_rounded_rect(
                find_x,
                tb_y,
                find_w,
                32.0,
                8.0,
                if self.find_open { pal.tint_lavender } else { pal.btn_hover },
            );
        }
        c.draw_icon_search(
            find_x + 16.0,
            tb_y + 16.0,
            if self.find_open { pal.primary } else { pal.slate },
        );
        hits.push(HitRegion {
            x: find_x,
            y: tb_y,
            w: find_w,
            h: 32.0,
            id: HitId::ToggleFind,
            clickable: true,
        });

        // Command Palette (Ctrl+K) button
        let pal_hover = point_in(cursor, pal_x, tb_y, pal_w, 32.0);
        if self.command_palette_open || pal_hover {
            c.fill_rounded_rect(
                pal_x,
                tb_y,
                pal_w,
                32.0,
                8.0,
                if self.command_palette_open { pal.tint_lavender } else { pal.btn_hover },
            );
        }
        c.draw_icon_palette(
            pal_x + 16.0,
            tb_y + 16.0,
            if self.command_palette_open { pal.primary } else { pal.slate },
        );
        hits.push(HitRegion {
            x: pal_x,
            y: tb_y,
            w: pal_w,
            h: 32.0,
            id: HitId::TogglePalette,
            clickable: true,
        });

        // Inspector button
        let insp_hover = point_in(cursor, insp_x, tb_y, insp_w, 32.0);
        c.fill_rounded_rect(
            insp_x,
            tb_y,
            insp_w,
            32.0,
            8.0,
            if self.inspector_open {
                pal.tint_lavender
            } else if insp_hover {
                pal.surface
            } else {
                pal.canvas
            },
        );
        c.stroke_rounded_rect(
            insp_x,
            tb_y,
            insp_w,
            32.0,
            8.0,
            1.0,
            if self.inspector_open { pal.primary } else { pal.hairline },
        );
        c.draw_text(
            &mut self.fonts,
            FontStyle::SemiBold,
            12.0,
            insp_x + 12.0,
            tb_y + 8.5,
            if self.inspector_open { pal.primary } else { pal.charcoal },
            "Inspector",
        );
        hits.push(HitRegion {
            x: insp_x,
            y: tb_y,
            w: insp_w,
            h: 32.0,
            id: HitId::ToggleInspector,
            clickable: true,
        });

        // F12 Dev primary purple button
        let f12_hover = point_in(cursor, f12_x, tb_y, f12_w, 32.0);
        c.fill_rounded_rect(
            f12_x,
            tb_y,
            f12_w,
            32.0,
            8.0,
            if f12_hover { pal.primary_pressed } else { pal.primary },
        );
        c.draw_text(
            &mut self.fonts,
            FontStyle::SemiBold,
            12.0,
            f12_x + 13.0,
            tb_y + 8.5,
            pal.on_primary,
            "F12 Dev",
        );
        hits.push(HitRegion {
            x: f12_x,
            y: tb_y,
            w: f12_w,
            h: 32.0,
            id: HitId::OpenDevtools,
            clickable: true,
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn render_bookmarks_bar(
        &mut self,
        c: &mut Canvas<'_>,
        pal: &ThemePalette,
        cursor: (f32, f32),
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        bookmarks: &[Bookmark],
        hits: &mut Vec<HitRegion>,
    ) {
        c.fill_rect(x, y, w, h, pal.surface);
        c.fill_rect(x, y + h - 1.0, w, 1.0, pal.hairline);

        let mut bx = x + 12.0;
        let max_bx = w - 42.0;

        if bookmarks.is_empty() {
            c.draw_text(
                &mut self.fonts,
                FontStyle::Regular,
                11.5,
                bx,
                y + 8.5,
                pal.steel,
                "Bookmarks bar: click the star icon in the address bar to pin favorite sites here (Ctrl+Shift+B to hide)",
            );
        } else {
            c.push_clip(x, y, max_bx - x, h);
            for bm in bookmarks {
                let label = if bm.title.trim().is_empty() {
                    &bm.url
                } else {
                    &bm.title
                };
                let pill_w = (self.fonts.measure_text(FontStyle::SemiBold, 11.5, label) + 24.0).clamp(60.0, 170.0);
                if bx + pill_w > max_bx {
                    break;
                }
                let hov = point_in(cursor, bx, y + 4.0, pill_w, 24.0);
                if hov {
                    c.fill_rounded_rect(bx, y + 4.0, pill_w, 24.0, 6.0, pal.btn_hover);
                }
                c.fill_circle(bx + 10.0, y + 16.0, 3.0, pal.primary);
                c.draw_text_ellipsis(
                    &mut self.fonts,
                    FontStyle::SemiBold,
                    11.5,
                    bx + 18.0,
                    y + 8.5,
                    pill_w - 22.0,
                    pal.charcoal,
                    label,
                );
                hits.push(HitRegion {
                    x: bx,
                    y: y + 4.0,
                    w: pill_w,
                    h: 24.0,
                    id: HitId::BookmarkBarItem(bm.url.clone()),
                    clickable: true,
                });
                bx += pill_w + 6.0;
            }
            c.pop_clip();
        }

        // Right close toggle for bookmarks bar
        let cx = w - 30.0;
        let chov = point_in(cursor, cx, y + 5.0, 22.0, 22.0);
        if chov {
            c.fill_rounded_rect(cx, y + 5.0, 22.0, 22.0, 6.0, pal.btn_hover);
        }
        c.draw_icon_close(cx + 11.0, y + 16.0, if chov { pal.ink } else { pal.steel });
        hits.push(HitRegion {
            x: cx,
            y: y + 5.0,
            w: 22.0,
            h: 22.0,
            id: HitId::BookmarkBarToggle,
            clickable: true,
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn render_find_bar(
        &mut self,
        c: &mut Canvas<'_>,
        pal: &ThemePalette,
        time_sec: f32,
        cursor: (f32, f32),
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        hits: &mut Vec<HitRegion>,
    ) {
        c.fill_rect(x, y, w, h, pal.surface);
        c.fill_rect(x, y + h - 1.0, w, 1.0, pal.hairline);

        let ix = x + 16.0;
        let iy = y + 6.0;
        let iw = 280.0;
        let ih = 30.0;
        let focused = self.active_input == ActiveInput::FindBar;

        c.fill_rounded_rect(ix, iy, iw, ih, 7.0, pal.canvas);
        c.stroke_rounded_rect(
            ix,
            iy,
            iw,
            ih,
            7.0,
            if focused { 2.0 } else { 1.0 },
            if focused { pal.primary } else { pal.hairline_strong },
        );
        c.push_clip(ix + 6.0, iy + 2.0, iw - 12.0, ih - 4.0);
        if self.find_text.is_empty() {
            if focused && ((time_sec * 2.0) as i32) % 2 == 0 {
                c.fill_rect(ix + 8.0, iy + 6.0, 2.0, 18.0, pal.primary);
            }
            c.draw_text(&mut self.fonts, FontStyle::Regular, 12.5, ix + 12.0, iy + 7.5, pal.steel, "Find in page...");
        } else {
            let visible = tail_fitting_width(&self.fonts, FontStyle::Regular, 12.5, &self.find_text, iw - 20.0);
            c.draw_text(&mut self.fonts, FontStyle::Regular, 12.5, ix + 8.0, iy + 7.5, pal.ink, visible);
            if focused && ((time_sec * 2.0) as i32) % 2 == 0 {
                let tw = self.fonts.measure_text(FontStyle::Regular, 12.5, visible);
                c.fill_rect(ix + 10.0 + tw, iy + 6.0, 2.0, 18.0, pal.primary);
            }
        }
        c.pop_clip();
        hits.push(HitRegion {
            x: ix,
            y: iy,
            w: iw,
            h: ih,
            id: HitId::FindInput,
            clickable: true,
        });

        let btns = [
            (ix + iw + 8.0, 56.0, "Prev", HitId::FindPrev),
            (ix + iw + 70.0, 56.0, "Next", HitId::FindNext),
            (ix + iw + 132.0, 62.0, "Close", HitId::FindClose),
        ];
        for (bx, bw, label, hid) in btns {
            let hov = point_in(cursor, bx, iy, bw, ih);
            c.fill_rounded_rect(bx, iy, bw, ih, 7.0, if hov { pal.surface_soft } else { pal.canvas });
            c.stroke_rounded_rect(bx, iy, bw, ih, 7.0, 1.0, pal.hairline);
            c.draw_text(&mut self.fonts, FontStyle::SemiBold, 12.0, bx + 12.0, iy + 7.5, pal.charcoal, label);
            hits.push(HitRegion {
                x: bx,
                y: iy,
                w: bw,
                h: ih,
                id: hid,
                clickable: true,
            });
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_command_palette(
        &mut self,
        c: &mut Canvas<'_>,
        pal: &ThemePalette,
        time_sec: f32,
        cursor: (f32, f32),
        win_w: f32,
        win_h: f32,
        snapshot: &RenderSnapshot<'_>,
        hits: &mut Vec<HitRegion>,
    ) {
        c.fill_rect(0.0, CHROME_BASE_HEIGHT, win_w, win_h - CHROME_BASE_HEIGHT, Color::rgba(10, 15, 36, 125));
        hits.push(HitRegion {
            x: 0.0,
            y: CHROME_BASE_HEIGHT,
            w: win_w,
            h: win_h - CHROME_BASE_HEIGHT,
            id: HitId::PaletteOverlayBackdrop,
            clickable: true,
        });

        let pw = (win_w - 80.0).clamp(380.0, 620.0);
        let px = (win_w - pw) * 0.5;
        let py = CHROME_BASE_HEIGHT + 44.0;

        let q = self.command_palette_query.trim().to_ascii_lowercase();
        let mut items: Vec<PaletteEntry> = Vec::new();

        let builtins = [
            ("New Tab", "Open Start Page (Ctrl+T)", "Action", UiAction::NewTab(None)),
            ("Duplicate Active Tab", "Clone current tab (Ctrl+Shift+K)", "Action", UiAction::DuplicateTab),
            ("Pin / Unpin Active Tab", "Toggle pinned tab in strip", "Action", UiAction::PinActiveTab),
            ("Toggle Bookmarks Bar", "Show or hide quick bookmarks bar (Ctrl+Shift+B)", "View", UiAction::ToggleBookmarksBar),
            ("Toggle Dark / Light Theme", "Switch Pluto Browser theme", "View", UiAction::ToggleTheme),
            ("Zoom In (+10%)", "Increase page & UI zoom (Ctrl+=)", "Zoom", UiAction::ZoomIn),
            ("Zoom Out (-10%)", "Decrease page & UI zoom (Ctrl+-)", "Zoom", UiAction::ZoomOut),
            ("Reset Zoom (100%)", "Reset page zoom to 100% (Ctrl+0)", "Zoom", UiAction::ZoomReset),
            ("Pluto Native Inspector", "Open bottom diagnostics HUD (Ctrl+Shift+D)", "Dev", UiAction::ToggleInspector),
            ("Open Welcome Showcase", "Visit pluto://welcome orbital showcase", "Pluto", UiAction::OpenWelcome),
            ("Clear Browsing History", "Erase saved local history.json", "Privacy", UiAction::ClearHistory),
        ];
        for (title, sub, badge, act) in builtins {
            if q.is_empty() || title.to_ascii_lowercase().contains(&q) || sub.to_ascii_lowercase().contains(&q) {
                items.push(PaletteEntry {
                    title: title.to_string(),
                    subtitle: sub.to_string(),
                    badge: badge.to_string(),
                    exec: PaletteEntryAction::Action(act),
                });
            }
        }
        for t in snapshot.tabs {
            if q.is_empty() || t.title.to_ascii_lowercase().contains(&q) || t.url.to_ascii_lowercase().contains(&q) {
                items.push(PaletteEntry {
                    title: format!("Switch to Tab: {}", t.title),
                    subtitle: t.url.clone(),
                    badge: "Tab".to_string(),
                    exec: PaletteEntryAction::SwitchTab(t.id),
                });
            }
        }
        for bm in snapshot.bookmarks {
            if q.is_empty() || bm.title.to_ascii_lowercase().contains(&q) || bm.url.to_ascii_lowercase().contains(&q) {
                items.push(PaletteEntry {
                    title: bm.title.clone(),
                    subtitle: bm.url.clone(),
                    badge: "Bookmark".to_string(),
                    exec: PaletteEntryAction::Navigate(bm.url.clone()),
                });
            }
        }
        for h in snapshot.history.iter().take(6) {
            if q.is_empty() || h.title.to_ascii_lowercase().contains(&q) || h.url.to_ascii_lowercase().contains(&q) {
                items.push(PaletteEntry {
                    title: h.title.clone(),
                    subtitle: h.url.clone(),
                    badge: "History".to_string(),
                    exec: PaletteEntryAction::Navigate(h.url.clone()),
                });
            }
        }

        items.truncate(8);
        if self.command_palette_idx >= items.len() && !items.is_empty() {
            self.command_palette_idx = 0;
        }
        self.palette_entries = items;

        let list_h = if self.palette_entries.is_empty() {
            54.0
        } else {
            self.palette_entries.len() as f32 * 46.0 + 12.0
        };
        let ph = 54.0 + list_h;

        c.fill_rounded_rect(px, py, pw, ph, 12.0, pal.canvas);
        c.stroke_rounded_rect(px, py, pw, ph, 12.0, 1.5, pal.primary);

        // Search input row inside palette
        c.fill_rect(px + 1.0, py + 52.0, pw - 2.0, 1.0, pal.hairline);
        c.draw_icon_palette(px + 22.0, py + 26.0, pal.primary);
        c.push_clip(px + 40.0, py + 8.0, pw - 56.0, 38.0);
        if self.command_palette_query.is_empty() {
            if ((time_sec * 2.0) as i32) % 2 == 0 {
                c.fill_rect(px + 42.0, py + 14.0, 2.0, 22.0, pal.primary);
            }
            c.draw_text(
                &mut self.fonts,
                FontStyle::Regular,
                14.0,
                px + 47.0,
                py + 16.0,
                pal.steel,
                "Type a command, tab name, bookmark, or URL (Esc to close)...",
            );
        } else {
            let visible = tail_fitting_width(&self.fonts, FontStyle::Regular, 14.0, &self.command_palette_query, pw - 68.0);
            c.draw_text(&mut self.fonts, FontStyle::Regular, 14.0, px + 42.0, py + 16.0, pal.ink, visible);
            if ((time_sec * 2.0) as i32) % 2 == 0 {
                let tw = self.fonts.measure_text(FontStyle::Regular, 14.0, visible);
                c.fill_rect(px + 44.0 + tw, py + 14.0, 2.0, 22.0, pal.primary);
            }
        }
        c.pop_clip();
        hits.push(HitRegion {
            x: px,
            y: py,
            w: pw,
            h: 52.0,
            id: HitId::PaletteBox,
            clickable: true,
        });

        if self.palette_entries.is_empty() {
            c.draw_text(&mut self.fonts, FontStyle::Regular, 13.0, px + 20.0, py + 70.0, pal.steel, "No matching commands or tabs.");
        } else {
            for (idx, entry) in self.palette_entries.iter().enumerate() {
                let iy = py + 58.0 + idx as f32 * 46.0;
                let hov = point_in(cursor, px + 8.0, iy, pw - 16.0, 42.0);
                let selected = idx == self.command_palette_idx || hov;
                if selected {
                    c.fill_rounded_rect(px + 8.0, iy, pw - 16.0, 42.0, 8.0, pal.surface);
                    if idx == self.command_palette_idx {
                        c.stroke_rounded_rect(px + 8.0, iy, pw - 16.0, 42.0, 8.0, 1.0, pal.primary);
                    }
                }
                let bw = self.fonts.measure_text(FontStyle::SemiBold, 11.0, &entry.badge) + 14.0;
                c.fill_rounded_rect(px + pw - bw - 18.0, iy + 10.0, bw, 22.0, 6.0, pal.tint_lavender);
                c.draw_text(&mut self.fonts, FontStyle::SemiBold, 11.0, px + pw - bw - 11.0, iy + 14.5, pal.tint_lavender_ink, &entry.badge);

                c.draw_text_ellipsis(&mut self.fonts, FontStyle::SemiBold, 13.0, px + 18.0, iy + 6.0, pw - bw - 44.0, pal.ink, &entry.title);
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::Regular, 11.5, px + 18.0, iy + 24.0, pw - bw - 44.0, pal.steel, &entry.subtitle);

                hits.push(HitRegion {
                    x: px + 8.0,
                    y: iy,
                    w: pw - 16.0,
                    h: 42.0,
                    id: HitId::PaletteItem(idx),
                    clickable: true,
                });
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_inspector_dock(
        &mut self,
        c: &mut Canvas<'_>,
        pal: &ThemePalette,
        cursor: (f32, f32),
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        snapshot: &RenderSnapshot<'_>,
        hits: &mut Vec<HitRegion>,
    ) {
        c.fill_rect(x, y, w, h, pal.canvas);
        c.fill_rect(x, y, w, 1.0, pal.hairline_strong);
        c.fill_rect(x, y + 1.0, w, 38.0, pal.surface);
        c.fill_rect(x, y + 39.0, w, 1.0, pal.hairline);

        c.draw_text(&mut self.fonts, FontStyle::Bold, 13.0, x + 16.0, y + 12.0, pal.ink, "Pluto Native Inspector");
        c.fill_rounded_rect(x + 196.0, y + 8.0, 96.0, 22.0, 6.0, pal.tint_lavender);
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 11.0, x + 205.0, y + 13.0, pal.tint_lavender_ink, "Pure Rust UI");

        let tabs = ["Runtime & Perf", "Page Telemetry"];
        let mut tx = x + 310.0;
        for (idx, label) in tabs.iter().enumerate() {
            let tw = self.fonts.measure_text(FontStyle::SemiBold, 12.5, label) + 18.0;
            let active = self.inspector_tab == idx;
            if active {
                c.fill_rect(tx, y + 37.0, tw, 2.0, pal.primary);
            }
            c.draw_text(
                &mut self.fonts,
                FontStyle::SemiBold,
                12.5,
                tx + 9.0,
                y + 12.0,
                if active { pal.ink } else { pal.steel },
                label,
            );
            hits.push(HitRegion {
                x: tx,
                y: y + 4.0,
                w: tw,
                h: 34.0,
                id: HitId::InspectorTab(idx),
                clickable: true,
            });
            tx += tw + 8.0;
        }

        let close_x = x + w - 72.0;
        let f12_x = close_x - 114.0;
        let ref_x = f12_x - 82.0;
        for (bx, bw, label, primary, hid) in [
            (ref_x, 74.0, "Refresh", false, HitId::InspectorRefresh),
            (f12_x, 106.0, "F12 DevTools", true, HitId::InspectorDevtools),
            (close_x, 60.0, "Close", false, HitId::InspectorClose),
        ] {
            let hov = point_in(cursor, bx, y + 6.0, bw, 27.0);
            let bg = if primary {
                if hov { pal.primary_pressed } else { pal.primary }
            } else if hov {
                pal.surface_soft
            } else {
                pal.canvas
            };
            c.fill_rounded_rect(bx, y + 6.0, bw, 27.0, 6.0, bg);
            if !primary {
                c.stroke_rounded_rect(bx, y + 6.0, bw, 27.0, 6.0, 1.0, pal.hairline);
            }
            c.draw_text(
                &mut self.fonts,
                FontStyle::SemiBold,
                11.5,
                bx + 11.0,
                y + 13.0,
                if primary { pal.on_primary } else { pal.charcoal },
                label,
            );
            hits.push(HitRegion {
                x: bx,
                y: y + 6.0,
                w: bw,
                h: 27.0,
                id: hid,
                clickable: true,
            });
        }

        let body_y = y + 50.0;
        let card_w = ((w - 56.0) / 3.0).max(160.0);
        if self.inspector_tab == 0 {
            let rows = [
                (
                    "UI Renderer",
                    "100% Native Rust (Softbuffer + ab_glyph)",
                    format!("Frame time: {:.2} ms | Zoom: {}%", self.last_frame_ms, snapshot.zoom_percent),
                    pal.tint_lavender,
                ),
                (
                    "WebView Strategy",
                    if Self::is_internal_page(snapshot.active_url) {
                        "Suspended (Zero WebView Processes)"
                    } else {
                        "Active tab isolated, background trimmed"
                    },
                    format!("Tabs: {} | Bookmarks: {} | History: {}", snapshot.tabs.len(), snapshot.bookmarks.len(), snapshot.history.len()),
                    pal.tint_mint,
                ),
                (
                    "Active Route",
                    snapshot.active_url,
                    format!("Search: Google | Theme: {}", if self.dark_mode { "Dark" } else { "Light" }),
                    pal.tint_peach,
                ),
            ];
            for (i, (k, v, sub, tint)) in rows.iter().enumerate() {
                let cx = x + 16.0 + i as f32 * (card_w + 12.0);
                c.fill_rounded_rect(cx, body_y, card_w, 122.0, 10.0, pal.surface);
                c.stroke_rounded_rect(cx, body_y, card_w, 122.0, 10.0, 1.0, pal.hairline);
                c.fill_rounded_rect(cx + 12.0, body_y + 12.0, 110.0, 22.0, 5.0, *tint);
                let tag_ink = pal.charcoal;
                c.draw_text(&mut self.fonts, FontStyle::SemiBold, 11.0, cx + 18.0, body_y + 17.0, tag_ink, k);
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::SemiBold, 13.0, cx + 12.0, body_y + 46.0, card_w - 24.0, pal.ink, v);
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::Mono, 11.5, cx + 12.0, body_y + 76.0, card_w - 24.0, pal.steel, sub);
            }
        } else {
            let tel = snapshot.telemetry.cloned().unwrap_or_default();
            let stats = [
                (
                    "DOM Nodes",
                    if Self::is_internal_page(snapshot.active_url) {
                        format!("{} native UI hit regions", hits.len())
                    } else {
                        format!("{} elements ({} imgs / {} scripts)", tel.dom_nodes, tel.images, tel.scripts)
                    },
                    format!("Stylesheets: {}", tel.stylesheets),
                ),
                (
                    "Navigation Timing",
                    format!("DOMContentLoaded: {} ms", tel.dcl_ms),
                    format!("Page Load: {} ms", tel.load_ms),
                ),
                (
                    "Storage & Origin",
                    if tel.origin.is_empty() { snapshot.active_url.to_string() } else { tel.origin },
                    format!("localStorage: {} keys | session: {} keys", tel.local_keys, tel.session_keys),
                ),
            ];
            for (i, (k, v, sub)) in stats.iter().enumerate() {
                let cx = x + 16.0 + i as f32 * (card_w + 12.0);
                c.fill_rounded_rect(cx, body_y, card_w, 122.0, 10.0, pal.surface);
                c.stroke_rounded_rect(cx, body_y, card_w, 122.0, 10.0, 1.0, pal.hairline);
                c.draw_text(&mut self.fonts, FontStyle::SemiBold, 12.0, cx + 12.0, body_y + 14.0, pal.steel, k);
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::SemiBold, 13.5, cx + 12.0, body_y + 46.0, card_w - 24.0, pal.ink, v);
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::Mono, 11.5, cx + 12.0, body_y + 78.0, card_w - 24.0, pal.slate, sub);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_welcome_page(
        &mut self,
        c: &mut Canvas<'_>,
        pal: &ThemePalette,
        time_sec: f32,
        cursor: (f32, f32),
        vx: f32,
        vy: f32,
        vw: f32,
        vh: f32,
        zoom_percent: u32,
        hits: &mut Vec<HitRegion>,
    ) {
        c.push_clip(vx, vy, vw, vh);
        let sy = self.welcome_scroll_y;
        let container_w = (vw - 64.0).clamp(400.0, 1140.0);
        let cx = vx + (vw - container_w) * 0.5;

        let nav_h = 56.0;
        let hero_y = vy - sy;
        let hero_h = 740.0;

        c.fill_vertical_gradient(
            vx,
            hero_y + nav_h,
            vw,
            hero_h,
            pal.brand_navy,
            Color::rgb(0x07, 0x0f, 0x24),
        );

        let px_off = ((cursor.0 / vw.max(1.0)) - 0.5) * -14.0;
        let py_off = ((cursor.1 / vh.max(1.0)) - 0.5) * -8.0;
        let stars = [
            (0.12, 0.14, 4.0, Color::rgb(0xff, 0x64, 0xc8)),
            (0.86, 0.18, 4.8, Color::rgb(0xf5, 0xd7, 0x5e)),
            (0.20, 0.36, 3.2, Color::rgb(0x2a, 0x9d, 0x99)),
            (0.81, 0.38, 3.8, Color::rgb(0xd6, 0xb6, 0xf6)),
            (0.08, 0.28, 3.0, Color::rgb(0xdd, 0x5b, 0x00)),
            (0.32, 0.09, 2.2, Color::rgb(0xff, 0xff, 0xff)),
            (0.70, 0.11, 2.4, Color::rgb(0xd6, 0xb6, 0xf6)),
        ];
        for (i, (sx_r, sy_r, sr, col)) in stars.iter().enumerate() {
            let pulse = 0.55 + 0.45 * (time_sec * 1.6 + i as f32 * 1.1).sin();
            let star_x = vx + vw * sx_r + px_off * 0.4;
            let star_y = hero_y + nav_h + hero_h * sy_r + py_off * 0.4;
            c.fill_circle(
                star_x,
                star_y,
                sr * (0.85 + pulse * 0.25),
                col.with_alpha((pulse * 220.0) as u8),
            );
        }

        c.draw_pluto_hero_system(vx + vw * 0.5, hero_y + nav_h + 74.0, time_sec, px_off, py_off);

        let badge_w = 340.0;
        let badge_x = vx + (vw - badge_w) * 0.5;
        let badge_y = hero_y + nav_h + 138.0;
        c.fill_rounded_rect(badge_x, badge_y, badge_w, 26.0, 13.0, Color::rgba(255, 255, 255, 22));
        c.stroke_rounded_rect(badge_x, badge_y, badge_w, 26.0, 13.0, 1.0, Color::rgba(214, 182, 246, 75));
        c.fill_circle(badge_x + 15.0, badge_y + 13.0, 3.5, pal.brand_green);
        c.draw_text(
            &mut self.fonts,
            FontStyle::SemiBold,
            11.0,
            badge_x + 26.0,
            badge_y + 6.5,
            Color::rgb(0xd6, 0xb6, 0xf6),
            "PLUTO ENGINE - 100% NATIVE RUST UI",
        );

        let h1 = "The orbital workspace browser, built in Rust.";
        let h1_size = if vw >= 940.0 { 38.0 } else { 28.0 };
        let h1_w = self.fonts.measure_text(FontStyle::Bold, h1_size, h1);
        c.draw_text(
            &mut self.fonts,
            FontStyle::Bold,
            h1_size,
            vx + (vw - h1_w) * 0.5,
            badge_y + 38.0,
            Color::rgb(255, 255, 255),
            h1,
        );

        let sub = "Zero HTML chrome. Pure Rust UI, Google Search default, Ctrl+K Command Palette & WebView2 tab trimming.";
        let sub_w = self.fonts.measure_text(FontStyle::Regular, 14.5, sub);
        c.draw_text(
            &mut self.fonts,
            FontStyle::Regular,
            14.5,
            vx + (vw - sub_w) * 0.5,
            badge_y + 90.0,
            Color::rgba(255, 255, 255, 208),
            sub,
        );

        let cta_y = badge_y + 128.0;
        let btn1_w = 166.0;
        let btn2_w = 184.0;
        let cta_start = vx + (vw - (btn1_w + btn2_w + 14.0)) * 0.5;
        let hov1 = point_in(cursor, cta_start, cta_y, btn1_w, 40.0);
        c.fill_rounded_rect(
            cta_start,
            cta_y,
            btn1_w,
            40.0,
            8.0,
            if hov1 { pal.primary_pressed } else { pal.primary },
        );
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 14.0, cta_start + 20.0, cta_y + 11.0, pal.on_primary, "Open Workspace");
        hits.push(HitRegion {
            x: cta_start,
            y: cta_y,
            w: btn1_w,
            h: 40.0,
            id: HitId::WelcomeHeroOpen,
            clickable: true,
        });

        let btn2_x = cta_start + btn1_w + 14.0;
        let hov2 = point_in(cursor, btn2_x, cta_y, btn2_w, 40.0);
        if hov2 {
            c.fill_rounded_rect(btn2_x, cta_y, btn2_w, 40.0, 8.0, Color::rgba(255, 255, 255, 24));
        }
        c.stroke_rounded_rect(btn2_x, cta_y, btn2_w, 40.0, 8.0, 1.0, Color::rgba(214, 182, 246, 150));
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 14.0, btn2_x + 20.0, cta_y + 11.0, Color::rgb(255, 255, 255), "Try Live Inspector");
        hits.push(HitRegion {
            x: btn2_x,
            y: cta_y,
            w: btn2_w,
            h: 40.0,
            id: HitId::WelcomeHeroXray,
            clickable: true,
        });

        // Workspace Mockup Card
        let mock_y = cta_y + 60.0;
        let mock_h = 282.0;
        c.fill_rounded_rect(cx, mock_y, container_w, mock_h, 12.0, pal.canvas);
        c.stroke_rounded_rect(cx, mock_y, container_w, mock_h, 12.0, 1.0, pal.hairline);

        c.fill_rounded_rect(cx + 1.0, mock_y + 1.0, container_w - 2.0, 46.0, 11.0, pal.surface);
        c.fill_rect(cx + 1.0, mock_y + 34.0, container_w - 2.0, 12.0, pal.surface);
        c.fill_rect(cx, mock_y + 46.0, container_w, 1.0, pal.hairline);
        c.draw_pluto_logo(cx + 16.0, mock_y + 12.0, 22.0, 0.0);
        c.draw_text(&mut self.fonts, FontStyle::Bold, 14.0, cx + 46.0, mock_y + 14.5, pal.ink, "Pluto HQ");

        let mock_tabs = ["Workspace Board", "Rust Engine", "Shortcuts"];
        let mut mtx = cx + container_w - 390.0;
        for (idx, label) in mock_tabs.iter().enumerate() {
            let tw = self.fonts.measure_text(FontStyle::SemiBold, 13.0, label) + 18.0;
            let active = self.welcome_mock_tab == idx;
            if active {
                c.fill_rect(mtx + 6.0, mock_y + 44.0, tw - 12.0, 2.0, pal.ink);
            }
            c.draw_text(
                &mut self.fonts,
                FontStyle::SemiBold,
                13.0,
                mtx + 9.0,
                mock_y + 15.0,
                if active { pal.ink } else { pal.steel },
                label,
            );
            hits.push(HitRegion {
                x: mtx,
                y: mock_y + 6.0,
                w: tw,
                h: 38.0,
                id: HitId::WelcomeMockTab(idx),
                clickable: true,
            });
            mtx += tw + 8.0;
        }

        let mbody_y = mock_y + 62.0;
        if self.welcome_mock_tab == 0 {
            let col_w = (container_w - 64.0) / 3.0;
            let cols = [
                (
                    "Core Architecture",
                    "Rust",
                    pal.tint_lavender,
                    pal.tint_lavender_ink,
                    "Native Softbuffer + SDF",
                    "Every pixel of the chrome, icons, zoom, and Pluto orbit renders natively from Rust.",
                ),
                (
                    "Modern Productivity",
                    "Ctrl+K",
                    pal.tint_mint,
                    pal.tint_mint_ink,
                    "Command Palette + History",
                    "Jump between tabs, bookmarks, Google searches, and recent history in one keypress.",
                ),
                (
                    "Built-in Tooling",
                    "Fast",
                    pal.tint_peach,
                    pal.tint_peach_ink,
                    "Pluto Native Inspector",
                    "Live Rust HUD + DOM telemetry and native DevTools without extension bloat.",
                ),
            ];
            for (i, (head, tag, tbg, tfg, title, desc)) in cols.iter().enumerate() {
                let bx = cx + 18.0 + i as f32 * (col_w + 14.0);
                c.fill_rounded_rect(bx, mbody_y, col_w, 198.0, 10.0, pal.surface_soft);
                c.stroke_rounded_rect(bx, mbody_y, col_w, 198.0, 10.0, 1.0, pal.hairline);
                c.draw_text(&mut self.fonts, FontStyle::SemiBold, 13.0, bx + 14.0, mbody_y + 14.0, pal.charcoal, head);
                let tag_w = self.fonts.measure_text(FontStyle::SemiBold, 11.0, tag) + 14.0;
                c.fill_rounded_rect(bx + col_w - tag_w - 14.0, mbody_y + 11.0, tag_w, 22.0, 5.0, *tbg);
                c.draw_text(&mut self.fonts, FontStyle::SemiBold, 11.0, bx + col_w - tag_w - 7.0, mbody_y + 15.5, *tfg, tag);

                c.fill_rounded_rect(bx + 12.0, mbody_y + 44.0, col_w - 24.0, 138.0, 8.0, pal.canvas);
                c.stroke_rounded_rect(bx + 12.0, mbody_y + 44.0, col_w - 24.0, 138.0, 8.0, 1.0, pal.hairline);
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::SemiBold, 14.0, bx + 22.0, mbody_y + 58.0, col_w - 44.0, pal.ink, title);
                draw_wrapped_text(c, &mut self.fonts, FontStyle::Regular, 12.5, bx + 22.0, mbody_y + 88.0, col_w - 44.0, 19.0, pal.slate, desc);
            }
        } else if self.welcome_mock_tab == 1 {
            c.fill_rounded_rect(cx + 20.0, mbody_y, container_w - 40.0, 198.0, 10.0, pal.brand_navy);
            let lines = [
                ("Pluto Pure-Rust Render Telemetry", Color::rgb(0xa4, 0xa0, 0x97)),
                (&format!("frame_budget: {:.2} ms | active_zoom: {}%", self.last_frame_ms, zoom_percent), Color::rgb(0xd6, 0xb6, 0xf6)),
                ("html_files_in_chrome: 0 | css_files: 0 | js_files: 0", Color::rgb(0x4c, 0xd4, 0x68)),
                ("default_search: https://www.google.com/search?q=...", Color::rgb(255, 255, 255)),
                ("tab_engine: Lazy Option<WebView> (0 child processes on internal routes)", Color::rgb(210, 220, 240)),
            ];
            for (i, (line, col)) in lines.iter().enumerate() {
                c.draw_text(&mut self.fonts, FontStyle::Mono, 12.5, cx + 40.0, mbody_y + 24.0 + i as f32 * 32.0, *col, line);
            }
        } else {
            let keys = [
                ("Focus Omnibox", "Ctrl+L"),
                ("Command Palette", "Ctrl+K"),
                ("Bookmarks Bar", "Ctrl+Shift+B"),
                ("Pluto Inspector", "Ctrl+Shift+D"),
                ("Zoom In / Out / Reset", "Ctrl+ / - / 0"),
                ("Duplicate Tab", "Ctrl+Shift+K"),
            ];
            let kw = (container_w - 64.0) / 3.0;
            for (i, (label, kbd)) in keys.iter().enumerate() {
                let col = i % 3;
                let row = i / 3;
                let kx = cx + 18.0 + col as f32 * (kw + 14.0);
                let ky = mbody_y + row as f32 * 76.0 + 12.0;
                c.fill_rounded_rect(kx, ky, kw, 58.0, 8.0, pal.surface);
                c.stroke_rounded_rect(kx, ky, kw, 58.0, 8.0, 1.0, pal.hairline);
                c.draw_text(&mut self.fonts, FontStyle::SemiBold, 13.0, kx + 14.0, ky + 20.0, pal.charcoal, label);
                let kbw = self.fonts.measure_text(FontStyle::SemiBold, 11.0, kbd) + 16.0;
                c.fill_rounded_rect(kx + kw - kbw - 10.0, ky + 15.0, kbw, 28.0, 6.0, pal.canvas);
                c.stroke_rounded_rect(kx + kw - kbw - 10.0, ky + 15.0, kbw, 28.0, 6.0, 1.0, pal.hairline_strong);
                c.draw_text(&mut self.fonts, FontStyle::SemiBold, 11.0, kx + kw - kbw - 2.0, ky + 22.0, pal.ink, kbd);
            }
        }

        // Comparison Section
        let cmp_y = hero_y + nav_h + hero_h + 44.0;
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 12.0, cx, cmp_y, pal.primary, "COMPARE ARCHITECTURE");
        c.draw_text(&mut self.fonts, FontStyle::Bold, 26.0, cx, cmp_y + 22.0, pal.ink, "Native Rust chrome by design, not HTML wrapper.");

        let tiers = [
            (
                "Pluto Browser",
                "Pure Rust UI",
                "~1.2 MB",
                "Zero HTML/CSS/JS chrome",
                &["Softbuffer + SDF UI", "Background tab trimming", "Google Search + Ctrl+K", "Local JSON state"][..],
                true,
            ),
            (
                "Google Chrome",
                "",
                "~110 MB",
                "Bundled Blink runtime",
                &["~320 MB idle RAM", "~1,400 ms cold start", "Full DevTools suite", "Background updaters"],
                false,
            ),
            (
                "Brave Browser",
                "",
                "~125 MB",
                "Chromium fork",
                &["~290 MB idle RAM", "~1,150 ms cold start", "Built-in DevTools", "Bundled extras"],
                false,
            ),
            (
                "Arc Desktop",
                "",
                "~240 MB",
                "Swift + Chromium",
                &["~480 MB idle RAM", "~2,100 ms cold start", "Chromium DevTools", "Account sign-in"],
                false,
            ),
        ];

        let card_w = (container_w - 48.0) / 4.0;
        let card_y = cmp_y + 70.0;
        for (i, (name, badge, metric, sub, facts, featured)) in tiers.iter().enumerate() {
            let tx = cx + i as f32 * (card_w + 16.0);
            c.fill_rounded_rect(tx, card_y, card_w, 246.0, 12.0, if *featured { pal.surface } else { pal.canvas });
            c.stroke_rounded_rect(
                tx,
                card_y,
                card_w,
                246.0,
                12.0,
                if *featured { 2.0 } else { 1.0 },
                if *featured { pal.primary } else { pal.hairline },
            );
            c.draw_text(&mut self.fonts, FontStyle::Bold, 15.5, tx + 18.0, card_y + 18.0, pal.ink, name);
            if !badge.is_empty() {
                let bw = self.fonts.measure_text(FontStyle::SemiBold, 10.5, badge) + 14.0;
                c.fill_rounded_rect(tx + card_w - bw - 14.0, card_y + 16.0, bw, 22.0, 11.0, pal.primary);
                c.draw_text(&mut self.fonts, FontStyle::SemiBold, 10.5, tx + card_w - bw - 7.0, card_y + 21.0, pal.on_primary, badge);
            }
            c.draw_text(&mut self.fonts, FontStyle::Bold, 27.0, tx + 18.0, card_y + 54.0, pal.ink, metric);
            c.draw_text_ellipsis(&mut self.fonts, FontStyle::Regular, 12.5, tx + 18.0, card_y + 92.0, card_w - 36.0, pal.slate, sub);
            c.fill_rect(tx + 18.0, card_y + 120.0, card_w - 36.0, 1.0, pal.hairline);
            for (fi, f) in facts.iter().enumerate() {
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::Regular, 12.5, tx + 18.0, card_y + 134.0 + fi as f32 * 24.0, card_w - 36.0, pal.charcoal, f);
            }
        }

        // Bottom Setup Destination Banner
        let setup_y = card_y + 282.0;
        c.fill_rounded_rect(cx, setup_y, container_w, 206.0, 12.0, pal.surface);
        c.stroke_rounded_rect(cx, setup_y, container_w, 206.0, 12.0, 1.0, pal.hairline);
        c.draw_text(&mut self.fonts, FontStyle::Bold, 23.0, cx + 28.0, setup_y + 26.0, pal.ink, "Choose where Pluto Browser opens.");
        c.draw_text(
            &mut self.fonts,
            FontStyle::Regular,
            13.5,
            cx + 28.0,
            setup_y + 62.0,
            pal.slate,
            "Select your starting page below. Click the Planet Pluto button anytime to reopen this showcase.",
        );

        let mut px = cx + 28.0;
        let py = setup_y + 98.0;
        for (idx, (label, _)) in DESTINATIONS.iter().enumerate() {
            let pw = self.fonts.measure_text(FontStyle::SemiBold, 13.0, label) + 28.0;
            let active = self.welcome_dest_idx == idx;
            c.fill_rounded_rect(px, py, pw, 34.0, 17.0, if active { pal.ink_deep } else { pal.canvas });
            if !active {
                c.stroke_rounded_rect(px, py, pw, 34.0, 17.0, 1.0, pal.hairline_strong);
            }
            c.draw_text(
                &mut self.fonts,
                FontStyle::SemiBold,
                13.0,
                px + 14.0,
                py + 9.0,
                if active { Color::rgb(255, 255, 255) } else { pal.charcoal },
                label,
            );
            hits.push(HitRegion {
                x: px,
                y: py,
                w: pw,
                h: 34.0,
                id: HitId::WelcomeDestPill(idx),
                clickable: true,
            });
            px += pw + 10.0;
        }

        let fin_y = setup_y + 148.0;
        let fin_hov = point_in(cursor, cx + 28.0, fin_y, 184.0, 38.0);
        c.fill_rounded_rect(
            cx + 28.0,
            fin_y,
            184.0,
            38.0,
            8.0,
            if fin_hov { pal.primary_pressed } else { pal.primary },
        );
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 14.0, cx + 48.0, fin_y + 10.0, pal.on_primary, "Launch Workspace");
        hits.push(HitRegion {
            x: cx + 28.0,
            y: fin_y,
            w: 184.0,
            h: 38.0,
            id: HitId::WelcomeFinish,
            clickable: true,
        });

        // Sticky Top Nav inside Welcome view
        c.fill_rect(vx, vy, vw, nav_h, pal.canvas);
        c.fill_rect(vx, vy + nav_h - 1.0, vw, 1.0, pal.hairline);
        c.draw_pluto_logo(cx, vy + 15.0, 26.0, 0.0);
        c.draw_text(&mut self.fonts, FontStyle::Bold, 16.0, cx + 36.0, vy + 18.0, pal.ink, "Pluto Browser");

        let nav_cta_x = cx + container_w - 156.0;
        let nhov = point_in(cursor, nav_cta_x, vy + 10.0, 156.0, 36.0);
        c.fill_rounded_rect(
            nav_cta_x,
            vy + 10.0,
            156.0,
            36.0,
            8.0,
            if nhov { pal.primary_pressed } else { pal.primary },
        );
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 13.5, nav_cta_x + 17.0, vy + 19.0, pal.on_primary, "Open Workspace");
        hits.push(HitRegion {
            x: nav_cta_x,
            y: vy + 10.0,
            w: 156.0,
            h: 36.0,
            id: HitId::WelcomeHeroOpen,
            clickable: true,
        });

        let total_content_h = (setup_y + 236.0) - hero_y;
        self.welcome_max_scroll = (total_content_h - vh).max(0.0);
        self.welcome_scroll_y = self.welcome_scroll_y.clamp(0.0, self.welcome_max_scroll);

        if self.welcome_max_scroll > 0.0 {
            draw_scrollbar(c, pal, vx + vw - 9.0, vy + 4.0, vh - 8.0, self.welcome_scroll_y, self.welcome_max_scroll, total_content_h);
        }
        c.pop_clip();
    }

    #[allow(clippy::too_many_arguments)]
    fn render_newtab_page(
        &mut self,
        c: &mut Canvas<'_>,
        pal: &ThemePalette,
        time_sec: f32,
        cursor: (f32, f32),
        vx: f32,
        vy: f32,
        vw: f32,
        vh: f32,
        bookmarks: &[Bookmark],
        history: &[HistoryEntry],
        hits: &mut Vec<HitRegion>,
    ) {
        c.push_clip(vx, vy, vw, vh);
        let sy = self.newtab_scroll_y;
        let container_w = (vw - 64.0).clamp(400.0, 1040.0);
        let cx = vx + (vw - container_w) * 0.5;
        let mut y = vy + 28.0 - sy;

        // Top Workspace Strip
        c.draw_pluto_logo(cx, y, 28.0, 0.0);
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 14.5, cx + 38.0, y + 5.0, pal.charcoal, "Pluto Browser / Workspace");

        let wbtn_w = 190.0;
        let wbtn_x = cx + container_w - wbtn_w;
        let whov = point_in(cursor, wbtn_x, y - 2.0, wbtn_w, 32.0);
        c.fill_rounded_rect(wbtn_x, y - 2.0, wbtn_w, 32.0, 7.0, if whov { pal.surface } else { pal.canvas });
        c.stroke_rounded_rect(wbtn_x, y - 2.0, wbtn_w, 32.0, 7.0, 1.0, pal.hairline);
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 12.5, wbtn_x + 14.0, y + 6.5, pal.charcoal, "Welcome & Benchmarks");
        hits.push(HitRegion {
            x: wbtn_x,
            y: y - 2.0,
            w: wbtn_w,
            h: 32.0,
            id: HitId::NewTabWelcomeBtn,
            clickable: true,
        });

        c.fill_rect(cx, y + 42.0, container_w, 1.0, pal.hairline);
        y += 64.0;

        // Orbital active status pill + greeting + compact planet mark
        c.fill_rounded_rect(cx, y, 174.0, 26.0, 13.0, pal.tint_lavender);
        let dot_pulse = 0.65 + 0.35 * (time_sec * 2.4).sin();
        c.fill_circle(cx + 15.0, y + 13.0, 3.6 + dot_pulse * 0.7, pal.primary);
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 12.0, cx + 28.0, y + 6.0, pal.tint_lavender_ink, "Kuiper Orbit Active");

        c.draw_pluto_hero_system(cx + container_w - 88.0, y + 42.0, time_sec, 0.0, 0.0);

        c.draw_text(&mut self.fonts, FontStyle::Bold, 32.0, cx, y + 38.0, pal.ink, "Pluto Workspace");
        c.draw_text(
            &mut self.fonts,
            FontStyle::Regular,
            14.5,
            cx,
            y + 82.0,
            pal.slate,
            "Orbiting natively in pure Rust - search with Google or jump into your workspace.",
        );
        y += 120.0;

        // Interactive Google Search Box
        let sbox_w = (container_w * 0.70).clamp(340.0, 700.0);
        let sbox_h = 48.0;
        let sfocused = self.active_input == ActiveInput::NewTabSearch;
        c.fill_rounded_rect(cx, y, sbox_w, sbox_h, 9.0, if sfocused { pal.canvas } else { pal.surface });
        c.stroke_rounded_rect(
            cx,
            y,
            sbox_w,
            sbox_h,
            9.0,
            if sfocused { 2.0 } else { 1.0 },
            if sfocused { pal.primary } else { pal.hairline },
        );
        c.draw_icon_search(cx + 22.0, y + 24.0, pal.steel);

        let go_x = cx + sbox_w - 72.0;
        c.push_clip(cx + 38.0, y + 4.0, go_x - (cx + 46.0), sbox_h - 8.0);
        if self.newtab_search_text.is_empty() {
            if sfocused && ((time_sec * 2.0) as i32) % 2 == 0 {
                c.fill_rect(cx + 42.0, y + 12.0, 2.0, 24.0, pal.primary);
            }
            c.draw_text(&mut self.fonts, FontStyle::Regular, 14.5, cx + 47.0, y + 14.5, pal.steel, "Search Google or type a URL...");
        } else {
            let visible = tail_fitting_width(&self.fonts, FontStyle::Regular, 14.5, &self.newtab_search_text, (go_x - cx - 56.0).max(40.0));
            c.draw_text(&mut self.fonts, FontStyle::Regular, 14.5, cx + 42.0, y + 14.5, pal.ink, visible);
            if sfocused && ((time_sec * 2.0) as i32) % 2 == 0 {
                let tw = self.fonts.measure_text(FontStyle::Regular, 14.5, visible);
                c.fill_rect(cx + 44.0 + tw, y + 12.0, 2.0, 24.0, pal.primary);
            }
        }
        c.pop_clip();

        let go_hov = point_in(cursor, go_x, y + 6.0, 64.0, 36.0);
        c.fill_rounded_rect(go_x, y + 6.0, 64.0, 36.0, 8.0, if go_hov { pal.primary_pressed } else { pal.primary });
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 14.0, go_x + 16.0, y + 14.0, pal.on_primary, "Search");
        hits.push(HitRegion {
            x: cx,
            y,
            w: sbox_w - 76.0,
            h: sbox_h,
            id: HitId::NewTabSearchInput,
            clickable: true,
        });
        hits.push(HitRegion {
            x: go_x,
            y: y + 6.0,
            w: 64.0,
            h: 36.0,
            id: HitId::NewTabSearchSubmit,
            clickable: true,
        });
        y += 76.0;

        // Quick Links header + Category pill filter tabs
        c.draw_text(&mut self.fonts, FontStyle::Bold, 16.0, cx, y + 4.0, pal.ink, "Quick Links");
        let cats = ["All", "Engineering", "Reference"];
        let mut cat_x = cx + container_w - 280.0;
        for (idx, label) in cats.iter().enumerate() {
            let cw = self.fonts.measure_text(FontStyle::SemiBold, 13.0, label) + 26.0;
            let active = self.newtab_category == idx;
            c.fill_rounded_rect(cat_x, y, cw, 30.0, 15.0, if active { pal.ink_deep } else { pal.canvas });
            if !active {
                c.stroke_rounded_rect(cat_x, y, cw, 30.0, 15.0, 1.0, pal.hairline);
            }
            c.draw_text(
                &mut self.fonts,
                FontStyle::SemiBold,
                13.0,
                cat_x + 13.0,
                y + 7.5,
                if active { Color::rgb(255, 255, 255) } else { pal.steel },
                label,
            );
            hits.push(HitRegion {
                x: cat_x,
                y,
                w: cw,
                h: 30.0,
                id: HitId::NewTabCategory(idx),
                clickable: true,
            });
            cat_x += cw + 8.0;
        }
        y += 44.0;

        let filtered: Vec<&QuickLink> = QUICK_LINKS
            .iter()
            .filter(|q| self.newtab_category == 0 || q.cat == self.newtab_category)
            .collect();
        let tile_w = (container_w - 28.0) / 3.0;
        for (i, item) in filtered.iter().enumerate() {
            let col = i % 3;
            let row = i / 3;
            let tx = cx + col as f32 * (tile_w + 14.0);
            let ty = y + row as f32 * 108.0;
            let hov = point_in(cursor, tx, ty, tile_w, 94.0);
            c.fill_rounded_rect(tx, ty, tile_w, 94.0, 12.0, pal.canvas);
            c.stroke_rounded_rect(tx, ty, tile_w, 94.0, 12.0, 1.0, if hov { pal.primary } else { pal.hairline });

            c.fill_rounded_rect(tx + 16.0, ty + 14.0, 28.0, 28.0, 6.0, pal.surface);
            c.stroke_rounded_rect(tx + 16.0, ty + 14.0, 28.0, 28.0, 6.0, 1.0, pal.hairline);
            c.draw_text(&mut self.fonts, FontStyle::Bold, 12.5, tx + 22.0, ty + 20.5, pal.charcoal, item.badge);

            let (tbg, tfg) = if item.cat == 1 {
                (pal.tint_lavender, pal.tint_lavender_ink)
            } else {
                (pal.tint_sky, pal.tint_sky_ink)
            };
            let tw = self.fonts.measure_text(FontStyle::SemiBold, 11.5, item.tag) + 14.0;
            c.fill_rounded_rect(tx + tile_w - tw - 14.0, ty + 15.0, tw, 22.0, 6.0, tbg);
            c.draw_text(&mut self.fonts, FontStyle::SemiBold, 11.5, tx + tile_w - tw - 7.0, ty + 20.0, tfg, item.tag);

            c.draw_text(&mut self.fonts, FontStyle::SemiBold, 15.0, tx + 16.0, ty + 49.0, pal.ink, item.title);
            let host = item.url.trim_start_matches("https://").trim_start_matches("www.").trim_end_matches('/');
            c.draw_text(&mut self.fonts, FontStyle::Regular, 12.5, tx + 16.0, ty + 70.0, pal.steel, host);

            hits.push(HitRegion {
                x: tx,
                y: ty,
                w: tile_w,
                h: 94.0,
                id: HitId::OpenUrlTile(item.url.to_string()),
                clickable: true,
            });
        }
        let ql_rows = (filtered.len().div_ceil(3)).max(1);
        y += ql_rows as f32 * 108.0 + 18.0;

        // Middle Split Workspace: Saved Bookmarks + Scratchpad
        let left_w = (container_w - 20.0) * 0.58;
        let right_x = cx + left_w + 20.0;
        let right_w = container_w - left_w - 20.0;

        c.draw_text(&mut self.fonts, FontStyle::Bold, 16.0, cx, y, pal.ink, "Saved Bookmarks");
        let mut bm_bottom = y + 106.0;
        if bookmarks.is_empty() {
            c.fill_rounded_rect(cx, y + 30.0, left_w, 76.0, 12.0, pal.surface);
            c.stroke_rounded_rect(cx, y + 30.0, left_w, 76.0, 12.0, 1.0, pal.hairline);
            draw_wrapped_text(
                c,
                &mut self.fonts,
                FontStyle::Regular,
                13.5,
                cx + 18.0,
                y + 46.0,
                left_w - 36.0,
                21.0,
                pal.slate,
                "No bookmarks saved yet. Click the star button in the omnibox on any site to pin it here.",
            );
        } else {
            let bcol_w = (left_w - 12.0) / 2.0;
            for (i, bm) in bookmarks.iter().enumerate() {
                let col = i % 2;
                let row = i / 2;
                let bx = cx + col as f32 * (bcol_w + 12.0);
                let by = y + 30.0 + row as f32 * 86.0;
                let hov = point_in(cursor, bx, by, bcol_w, 76.0);
                c.fill_rounded_rect(bx, by, bcol_w, 76.0, 12.0, pal.canvas);
                c.stroke_rounded_rect(bx, by, bcol_w, 76.0, 12.0, 1.0, if hov { pal.primary } else { pal.hairline });
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::SemiBold, 14.0, bx + 14.0, by + 16.0, bcol_w - 76.0, pal.ink, &bm.title);
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::Regular, 12.0, bx + 14.0, by + 44.0, bcol_w - 28.0, pal.steel, &bm.url);

                let rx = bx + bcol_w - 64.0;
                let rhov = point_in(cursor, rx, by + 12.0, 54.0, 24.0);
                if hov || rhov {
                    c.fill_rounded_rect(rx, by + 12.0, 54.0, 24.0, 6.0, if rhov { pal.surface } else { pal.canvas });
                    c.stroke_rounded_rect(rx, by + 12.0, 54.0, 24.0, 6.0, 1.0, pal.hairline);
                    c.draw_text(&mut self.fonts, FontStyle::Regular, 11.0, rx + 7.0, by + 17.5, pal.steel, "Remove");
                    hits.push(HitRegion {
                        x: rx,
                        y: by + 12.0,
                        w: 54.0,
                        h: 24.0,
                        id: HitId::RemoveBookmark(bm.url.clone()),
                        clickable: true,
                    });
                }
                hits.push(HitRegion {
                    x: bx,
                    y: by,
                    w: bcol_w - 66.0,
                    h: 76.0,
                    id: HitId::OpenUrlTile(bm.url.clone()),
                    clickable: true,
                });
                bm_bottom = bm_bottom.max(by + 86.0);
            }
        }

        // Scratchpad Card
        let sp_h = 198.0;
        c.fill_rounded_rect(right_x, y, right_w, sp_h, 12.0, pal.tint_cream);
        c.stroke_rounded_rect(right_x, y, right_w, sp_h, 12.0, 1.0, pal.hairline);
        c.draw_text(&mut self.fonts, FontStyle::Bold, 14.0, right_x + 16.0, y + 16.0, pal.charcoal, "Workspace Notes");
        c.draw_text(&mut self.fonts, FontStyle::Regular, 12.0, right_x + right_w - 102.0, y + 17.0, pal.steel, "Saved locally");

        let box_x = right_x + 14.0;
        let box_y = y + 46.0;
        let box_w = right_w - 28.0;
        let box_h = sp_h - 60.0;
        let sp_focused = self.active_input == ActiveInput::Scratchpad;
        c.fill_rounded_rect(box_x, box_y, box_w, box_h, 8.0, pal.canvas);
        c.stroke_rounded_rect(
            box_x,
            box_y,
            box_w,
            box_h,
            8.0,
            if sp_focused { 2.0 } else { 1.0 },
            if sp_focused { pal.primary } else { pal.hairline },
        );

        c.push_clip(box_x + 6.0, box_y + 6.0, box_w - 12.0, box_h - 12.0);
        if self.scratchpad_text.is_empty() {
            if sp_focused && ((time_sec * 2.0) as i32) % 2 == 0 {
                c.fill_rect(box_x + 12.0, box_y + 12.0, 2.0, 18.0, pal.primary);
            }
            c.draw_text(
                &mut self.fonts,
                FontStyle::Regular,
                13.5,
                box_x + 18.0,
                box_y + 12.0,
                pal.steel,
                if sp_focused { "Type notes..." } else { "Click to jot down URLs or notes..." },
            );
        } else {
            let end_pos = draw_wrapped_text(
                c,
                &mut self.fonts,
                FontStyle::Regular,
                13.5,
                box_x + 12.0,
                box_y + 12.0,
                box_w - 24.0,
                21.0,
                pal.ink,
                &self.scratchpad_text,
            );
            if sp_focused && ((time_sec * 2.0) as i32) % 2 == 0 {
                c.fill_rect(end_pos.0 + 2.0, end_pos.1, 2.0, 18.0, pal.primary);
            }
        }
        c.pop_clip();
        hits.push(HitRegion {
            x: box_x,
            y: box_y,
            w: box_w,
            h: box_h,
            id: HitId::ScratchpadBox,
            clickable: true,
        });

        // Recent History strip below bookmarks & scratchpad
        let hist_y = bm_bottom.max(y + sp_h + 24.0);
        c.draw_icon_history(cx + 8.0, hist_y + 10.0, pal.steel);
        c.draw_text(&mut self.fonts, FontStyle::Bold, 16.0, cx + 24.0, hist_y, pal.ink, "Recent History");
        if !history.is_empty() {
            let clr_x = cx + container_w - 102.0;
            let clrh = point_in(cursor, clr_x, hist_y - 2.0, 102.0, 26.0);
            c.fill_rounded_rect(clr_x, hist_y - 2.0, 102.0, 26.0, 6.0, if clrh { pal.surface } else { pal.canvas });
            c.stroke_rounded_rect(clr_x, hist_y - 2.0, 102.0, 26.0, 6.0, 1.0, pal.hairline);
            c.draw_text(&mut self.fonts, FontStyle::SemiBold, 11.5, clr_x + 12.0, hist_y + 4.5, pal.steel, "Clear History");
            hits.push(HitRegion {
                x: clr_x,
                y: hist_y - 2.0,
                w: 102.0,
                h: 26.0,
                id: HitId::ClearHistory,
                clickable: true,
            });
        }

        let mut end_y = hist_y + 72.0;
        if history.is_empty() {
            c.draw_text(&mut self.fonts, FontStyle::Regular, 13.0, cx, hist_y + 32.0, pal.steel, "Visited websites will appear here automatically.");
        } else {
            let hcol_w = (container_w - 28.0) / 3.0;
            for (i, item) in history.iter().take(6).enumerate() {
                let col = i % 3;
                let row = i / 3;
                let hx = cx + col as f32 * (hcol_w + 14.0);
                let hy = hist_y + 32.0 + row as f32 * 70.0;
                let hov = point_in(cursor, hx, hy, hcol_w, 58.0);
                c.fill_rounded_rect(hx, hy, hcol_w, 58.0, 10.0, pal.surface);
                c.stroke_rounded_rect(hx, hy, hcol_w, 58.0, 10.0, 1.0, if hov { pal.primary } else { pal.hairline });
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::SemiBold, 13.0, hx + 14.0, hy + 10.0, hcol_w - 28.0, pal.ink, &item.title);
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::Regular, 11.5, hx + 14.0, hy + 32.0, hcol_w - 28.0, pal.steel, &item.url);
                hits.push(HitRegion {
                    x: hx,
                    y: hy,
                    w: hcol_w,
                    h: 58.0,
                    id: HitId::OpenUrlTile(item.url.clone()),
                    clickable: true,
                });
                end_y = end_y.max(hy + 70.0);
            }
        }

        let total_h = end_y - (vy + 28.0 - sy);
        self.newtab_max_scroll = (total_h + 28.0 - vh).max(0.0);
        self.newtab_scroll_y = self.newtab_scroll_y.clamp(0.0, self.newtab_max_scroll);

        if self.newtab_max_scroll > 0.0 {
            draw_scrollbar(c, pal, vx + vw - 9.0, vy + 4.0, vh - 8.0, self.newtab_scroll_y, self.newtab_max_scroll, total_h + 28.0);
        }
        c.pop_clip();
    }
}

fn tail_fitting_width<'a>(
    fonts: &FontSystem,
    style: FontStyle,
    px_size: f32,
    text: &'a str,
    max_w: f32,
) -> &'a str {
    if fonts.measure_text(style, px_size, text) <= max_w {
        return text;
    }
    for (idx, _) in text.char_indices() {
        let slice = &text[idx..];
        if fonts.measure_text(style, px_size, slice) <= max_w {
            return slice;
        }
    }
    text.char_indices()
        .last()
        .map(|(i, _)| &text[i..])
        .unwrap_or("")
}

#[inline]
fn point_in(p: (f32, f32), x: f32, y: f32, w: f32, h: f32) -> bool {
    p.0 >= x && p.0 <= x + w && p.1 >= y && p.1 <= y + h
}

fn pop_word(s: &mut String) {
    while s.ends_with(char::is_whitespace) {
        s.pop();
    }
    while let Some(ch) = s.chars().last() {
        if ch.is_whitespace() || ch == '/' || ch == '.' {
            break;
        }
        s.pop();
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_wrapped_text(
    c: &mut Canvas<'_>,
    fonts: &mut FontSystem,
    style: FontStyle,
    px_size: f32,
    x: f32,
    y: f32,
    max_w: f32,
    line_h: f32,
    color: Color,
    text: &str,
) -> (f32, f32) {
    let mut cur_y = y;
    let mut last_end = (x, y);
    for para in text.split('\n') {
        let mut line = String::new();
        for word in para.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };
            if fonts.measure_text(style, px_size, &candidate) > max_w && !line.is_empty() {
                c.draw_text(fonts, style, px_size, x, cur_y, color, &line);
                cur_y += line_h;
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
        if !line.is_empty() {
            c.draw_text(fonts, style, px_size, x, cur_y, color, &line);
            let w = fonts.measure_text(style, px_size, &line);
            last_end = (x + w, cur_y);
        } else {
            last_end = (x, cur_y);
        }
        cur_y += line_h;
    }
    last_end
}

#[allow(clippy::too_many_arguments)]
fn draw_scrollbar(
    c: &mut Canvas<'_>,
    pal: &ThemePalette,
    x: f32,
    y: f32,
    track_h: f32,
    scroll: f32,
    max_scroll: f32,
    total_h: f32,
) {
    let ratio = (track_h / total_h.max(track_h)).clamp(0.08, 1.0);
    let thumb_h = (track_h * ratio).max(28.0);
    let t = if max_scroll > 0.0 {
        scroll / max_scroll
    } else {
        0.0
    };
    let thumb_y = y + t * (track_h - thumb_h);
    c.fill_rounded_rect(x, thumb_y, 5.0, thumb_h, 2.5, pal.hairline_strong);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omnibox_focus_and_typing_lifecycle() {
        let mut ui = NativeUi::new(false, String::new());
        ui.focus_omnibox("https://example.com/");
        assert_eq!(ui.active_input, ActiveInput::Omnibox);
        assert!(ui.omnibox_select_all);

        // First typed char clears initial selection and starts fresh
        let _ = ui.handle_char_input('r');
        assert!(!ui.omnibox_select_all);
        assert_eq!(ui.omnibox_text, "r");

        for c in "ust-lang.org".chars() {
            let _ = ui.handle_char_input(c);
        }
        assert_eq!(ui.omnibox_text, "rust-lang.org");

        // Pressing Enter submits Navigate("rust-lang.org") and clears focus
        let actions = ui.handle_key_pressed(KeyCode::Enter, NEW_TAB_URL, "");
        assert!(matches!(actions.as_slice(), [UiAction::Navigate(u)] if u == "rust-lang.org"));
        assert_eq!(ui.active_input, ActiveInput::None);
    }

    #[test]
    fn omnibox_ctrl_backspace_and_escape() {
        let mut ui = NativeUi::new(false, String::new());
        ui.focus_omnibox("pluto rocks");
        ui.omnibox_select_all = false;
        ui.modifiers.set(ModifiersState::CONTROL, true);
        let _ = ui.handle_key_pressed(KeyCode::Backspace, WELCOME_URL, WELCOME_URL);
        assert_eq!(ui.omnibox_text, "pluto ");
        ui.modifiers.set(ModifiersState::CONTROL, false);
        let _ = ui.handle_key_pressed(KeyCode::Escape, WELCOME_URL, WELCOME_URL);
        assert_eq!(ui.active_input, ActiveInput::None);
        assert_eq!(ui.omnibox_text, WELCOME_URL);
    }

    #[test]
    fn e2e_render_zoom_omnibox_and_palette() {
        let mut ui = NativeUi::new(false, "note test".to_string());
        let w = 1000_usize;
        let h = 680_usize;
        let mut buf = vec![0u32; w * h];
        let tabs = vec![TabSnapshot {
            id: 1,
            title: "Welcome".to_string(),
            url: WELCOME_URL.to_string(),
            is_loading: false,
            active: true,
            pinned: false,
        }];
        let snap = RenderSnapshot {
            tabs: &tabs,
            active_url: WELCOME_URL,
            active_is_loading: false,
            is_bookmarked: false,
            zoom_percent: 150,
            can_reopen_closed: false,
            show_bookmarks_bar: true,
            bookmarks: &[],
            history: &[],
            telemetry: None,
        };
        ui.render(&mut buf, w, h, 1.0, &snap);
        assert!(buf.iter().any(|&px| px != 0), "Frame buffer must contain rendered pixels");

        // Click ZoomOut and ZoomIn toolbar buttons discovered from rendered layout hits
        let z_in = ui
            .hits
            .iter()
            .find(|r| r.id == HitId::ZoomIn)
            .expect("ZoomIn button hit region rendered")
            .clone();
        ui.cursor_pos = (z_in.x + 5.0, z_in.y + 5.0);
        let z_acts = ui.handle_mouse_click(MouseButton::Left, WELCOME_URL);
        assert!(z_acts.iter().any(|a| matches!(a, UiAction::ZoomIn)));

        // Click Omnibox hit region -> must focus Omnibox & request host window focus
        let omni = ui
            .hits
            .iter()
            .find(|r| r.id == HitId::Omnibox)
            .expect("Omnibox hit region rendered")
            .clone();
        ui.cursor_pos = (omni.x + 40.0, omni.y + 12.0);
        let o_acts = ui.handle_mouse_click(MouseButton::Left, WELCOME_URL);
        assert!(o_acts.iter().any(|a| matches!(a, UiAction::FocusHostWindow)));
        assert_eq!(ui.active_input, ActiveInput::Omnibox);

        // Open Command Palette & verify render generates palette entries cleanly
        ui.open_command_palette();
        ui.render(&mut buf, w, h, 1.0, &snap);
        assert!(!ui.palette_entries.is_empty());
    }
}
