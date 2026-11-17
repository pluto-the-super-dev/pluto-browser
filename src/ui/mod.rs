pub mod canvas;
pub mod theme;

use crate::clipboard;
use crate::ipc::{GOOGLE_HOME_URL, NEW_TAB_URL, SETTINGS_URL, WELCOME_URL};
use crate::storage::{Bookmark, HistoryEntry};
use crate::ui::canvas::{Canvas, FontStyle, FontSystem, FONT_SCALE};
use crate::ui::theme::{Color, ThemePalette};
use std::time::Instant;
use tao::dpi::PhysicalPosition;
use tao::event::MouseButton;
use tao::keyboard::{KeyCode, ModifiersState};

pub const TOP_TAB_ROW_H: f32 = 40.0;
pub const TOOLBAR_ROW_H: f32 = 46.0;
pub const SIDEBAR_W: f32 = 264.0;
pub const BOOKMARKS_BAR_HEIGHT: f32 = 32.0;
pub const WIN_CTRL_W: f32 = 46.0;
pub const FIND_BAR_HEIGHT: f32 = 46.0;
pub const INSPECTOR_DOCK_HEIGHT: f32 = 216.0;

#[derive(Debug, Clone)]
pub struct TabSnapshot {
    pub id: u64,
    pub title: String,
    pub url: String,
    pub is_loading: bool,
    pub active: bool,
    pub pinned: bool,
    pub is_muted: bool,
    pub reader_active: bool,
    pub night_active: bool,
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
    pub active_muted: bool,
    pub active_reader: bool,
    pub active_night: bool,
    pub is_bookmarked: bool,
    pub zoom_percent: u32,
    pub can_reopen_closed: bool,
    pub show_bookmarks_bar: bool,
    pub sidebar_left: bool,
    pub sidebar_collapsed: bool,
    pub adblock_enabled: bool,
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
    DragRegion,
    WinMinimize,
    WinMaximize,
    WinClose,
    BrandWelcome,
    OpenSettings,
    ToggleSidebarLayout,
    ToggleSidebarCollapse,
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
    CopyMarkdown,
    ToggleBookmark,
    ToggleShield,
    ToggleReader,
    ToggleNight,
    ToggleMute,
    ZoomOut,
    ZoomReset,
    ZoomIn,
    BookmarkBarItem(String),
    BookmarkBarToggle,
    ToggleFind,
    TogglePalette,
    ToggleInspector,
    OpenDevtools,
    FindInput,
    FindPrev,
    FindNext,
    FindClose,
    InspectorTab(usize),
    InspectorRefresh,
    InspectorDevtools,
    InspectorClose,
    PaletteOverlayBackdrop,
    PaletteBox,
    PaletteItem(usize),
    WelcomeHeroOpen,
    WelcomeHeroSettings,
    WelcomeHeroXray,
    WelcomeMockTab(usize),
    WelcomeDestPill(usize),
    WelcomeFinish,
    NewTabWelcomeBtn,
    NewTabSettingsBtn,
    NewTabSearchInput,
    NewTabSearchSubmit,
    NewTabCategory(usize),
    OpenUrlTile(String),
    RemoveBookmark(String),
    ClearHistory,
    ScratchpadBox,
    SettingsLayoutTop,
    SettingsLayoutLeft,
    SettingsSidebarCollapse,
    SettingsBookmarksToggle,
    SettingsThemeDark,
    SettingsThemeLight,
    SettingsZoomPreset(u32),
    SettingsShieldToggle,
    SettingsClearHistory,
    SettingsResetOnboarding,
    SettingsOpenInspector,
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
    WindowDrag,
    WindowMinimize,
    WindowToggleMaximize,
    WindowClose,
    FocusHostWindow,
    OpenWelcome,
    OpenSettings,
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
    CopyMarkdown,
    ToggleBookmark,
    RemoveBookmark(String),
    ToggleBookmarksBar,
    ToggleSidebarLayout,
    ToggleSidebarCollapse,
    ToggleShield,
    ToggleReaderMode,
    ToggleNightMode,
    ToggleMute,
    ClearHistory,
    ResetOnboarding,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    SetZoomPercent(u32),
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
    cat: usize,
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
        badge: "RS",
        cat: 1,
        tag: "Rust",
    },
    QuickLink {
        title: "MDN Docs",
        url: "https://developer.mozilla.org",
        badge: "MD",
        cat: 1,
        tag: "Docs",
    },
    QuickLink {
        title: "Wikipedia",
        url: "https://en.wikipedia.org",
        badge: "WK",
        cat: 2,
        tag: "Wiki",
    },
    QuickLink {
        title: "Hacker News",
        url: "https://news.ycombinator.com",
        badge: "HN",
        cat: 2,
        tag: "Feed",
    },
];

const DESTINATIONS: &[(&str, &str)] = &[
    ("New Tab Workspace", NEW_TAB_URL),
    ("Google Search", GOOGLE_HOME_URL),
    ("GitHub", "https://github.com"),
    ("Rust Lang", "https://www.rust-lang.org"),
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
    pub settings_scroll_y: f32,
    pub settings_max_scroll: f32,
    pub welcome_mock_tab: usize,
    pub welcome_dest_idx: usize,
    pub cursor_pos: (f32, f32),
    pub modifiers: ModifiersState,
    pub last_click: Option<(Instant, HitId)>,
    pub start_instant: Instant,
    pub last_frame_ms: f32,
    pub window_maximized: bool,
    last_drag_click: Option<Instant>,
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
            settings_scroll_y: 0.0,
            settings_max_scroll: 0.0,
            welcome_mock_tab: 0,
            welcome_dest_idx: 0,
            cursor_pos: (0.0, 0.0),
            modifiers: ModifiersState::empty(),
            last_click: None,
            start_instant: Instant::now(),
            last_frame_ms: 0.0,
            window_maximized: false,
            last_drag_click: None,
            hits: Vec::with_capacity(128),
            palette_entries: Vec::new(),
            last_synced_url: String::new(),
        }
    }

    pub fn is_internal_page(url: &str) -> bool {
        url == WELCOME_URL || url == NEW_TAB_URL || url == SETTINGS_URL
    }

    pub fn left_sidebar_width(&self, sidebar_left: bool, sidebar_collapsed: bool) -> f64 {
        if sidebar_left && !sidebar_collapsed {
            SIDEBAR_W as f64
        } else {
            0.0
        }
    }

    pub fn top_chrome_height(
        &self,
        sidebar_left: bool,
        show_bookmarks_bar: bool,
        active_url: &str,
    ) -> f64 {
        let mut h = if sidebar_left {
            TOOLBAR_ROW_H
        } else {
            TOP_TAB_ROW_H + TOOLBAR_ROW_H
        };
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
        active_url == WELCOME_URL
            || active_url == NEW_TAB_URL
            || active_loading
            || self.toast_message.is_some()
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
        if active_url == SETTINGS_URL {
            self.settings_scroll_y =
                (self.settings_scroll_y - delta_y).clamp(0.0, self.settings_max_scroll);
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

    fn in_drag_region(&self, lx: f32, ly: f32) -> bool {
        self.hits.iter().any(|r| {
            r.id == HitId::DragRegion
                && lx >= r.x
                && lx <= r.x + r.w
                && ly >= r.y
                && ly <= r.y + r.h
        })
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
            if button == MouseButton::Left && self.in_drag_region(lx, ly) {
                let now = Instant::now();
                let is_double = self
                    .last_drag_click
                    .is_some_and(|t| now.duration_since(t).as_millis() < 380);
                if is_double {
                    self.last_drag_click = None;
                    actions.push(UiAction::WindowToggleMaximize);
                } else {
                    self.last_drag_click = Some(now);
                    actions.push(UiAction::WindowDrag);
                }
                actions.push(UiAction::RequestRedraw);
                return actions;
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

        actions.push(UiAction::FocusHostWindow);

        match &hit {
            HitId::Omnibox => {
                if self.active_input == ActiveInput::Omnibox {
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
            HitId::DragRegion => actions.push(UiAction::WindowDrag),
            HitId::WinMinimize => actions.push(UiAction::WindowMinimize),
            HitId::WinMaximize => actions.push(UiAction::WindowToggleMaximize),
            HitId::WinClose => actions.push(UiAction::WindowClose),
            HitId::BrandWelcome => actions.push(UiAction::OpenWelcome),
            HitId::OpenSettings | HitId::WelcomeHeroSettings | HitId::NewTabSettingsBtn => {
                actions.push(UiAction::OpenSettings)
            }
            HitId::ToggleSidebarLayout => actions.push(UiAction::ToggleSidebarLayout),
            HitId::ToggleSidebarCollapse | HitId::SettingsSidebarCollapse => {
                actions.push(UiAction::ToggleSidebarCollapse)
            }
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
            HitId::CopyMarkdown => actions.push(UiAction::CopyMarkdown),
            HitId::ToggleBookmark => actions.push(UiAction::ToggleBookmark),
            HitId::ToggleShield | HitId::SettingsShieldToggle => {
                actions.push(UiAction::ToggleShield)
            }
            HitId::ToggleReader => actions.push(UiAction::ToggleReaderMode),
            HitId::ToggleNight => actions.push(UiAction::ToggleNightMode),
            HitId::ToggleMute => actions.push(UiAction::ToggleMute),
            HitId::ZoomOut => actions.push(UiAction::ZoomOut),
            HitId::ZoomReset => actions.push(UiAction::ZoomReset),
            HitId::ZoomIn => actions.push(UiAction::ZoomIn),
            HitId::ToggleFind => {
                if !Self::is_internal_page(active_url) {
                    actions.push(UiAction::ToggleFindBar);
                }
            }
            HitId::TogglePalette => actions.push(UiAction::ToggleCommandPalette),
            HitId::ToggleInspector | HitId::SettingsOpenInspector | HitId::WelcomeHeroXray => {
                actions.push(UiAction::ToggleInspector)
            }
            HitId::OpenDevtools | HitId::InspectorDevtools => actions.push(UiAction::OpenDevtools),
            HitId::BookmarkBarItem(url) => actions.push(UiAction::Navigate(url)),
            HitId::BookmarkBarToggle | HitId::SettingsBookmarksToggle => {
                actions.push(UiAction::ToggleBookmarksBar)
            }
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
            HitId::NewTabWelcomeBtn | HitId::SettingsResetOnboarding => {
                actions.push(UiAction::ResetOnboarding)
            }
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
            HitId::ClearHistory | HitId::SettingsClearHistory => actions.push(UiAction::ClearHistory),
            HitId::SettingsLayoutTop | HitId::SettingsLayoutLeft => {
                actions.push(UiAction::ToggleSidebarLayout)
            }
            HitId::SettingsThemeDark => {
                if !self.dark_mode {
                    self.dark_mode = true;
                    actions.push(UiAction::ToggleTheme);
                }
            }
            HitId::SettingsThemeLight => {
                if self.dark_mode {
                    self.dark_mode = false;
                    actions.push(UiAction::ToggleTheme);
                }
            }
            HitId::SettingsZoomPreset(pct) => actions.push(UiAction::SetZoomPercent(pct)),
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

        if code == KeyCode::F12 || (ctrl && shift && code == KeyCode::KeyI) {
            actions.push(UiAction::OpenDevtools);
            return actions;
        }
        if ctrl && shift && (code == KeyCode::KeyD || code == KeyCode::KeyC) {
            actions.push(UiAction::ToggleInspector);
            return actions;
        }
        if ctrl && !shift && !alt && code == KeyCode::Comma {
            actions.push(UiAction::OpenSettings);
            return actions;
        }
        if ctrl && !shift && !alt && code == KeyCode::KeyB {
            actions.push(UiAction::ToggleSidebarCollapse);
            return actions;
        }
        if ctrl && shift && code == KeyCode::KeyL {
            actions.push(UiAction::ToggleSidebarLayout);
            return actions;
        }
        if ctrl && shift && code == KeyCode::KeyY {
            actions.push(UiAction::CopyMarkdown);
            return actions;
        }
        if alt && !ctrl && !shift && code == KeyCode::KeyR {
            actions.push(UiAction::ToggleReaderMode);
            return actions;
        }
        if alt && !ctrl && !shift && code == KeyCode::KeyN {
            actions.push(UiAction::ToggleNightMode);
            return actions;
        }
        if alt && !ctrl && !shift && code == KeyCode::KeyM {
            actions.push(UiAction::ToggleMute);
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
                        self.command_palette_idx = (self.command_palette_idx
                            + self.palette_entries.len()
                            - 1)
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
            } else if active_url == SETTINGS_URL {
                match code {
                    KeyCode::PageDown | KeyCode::Space => {
                        self.settings_scroll_y =
                            (self.settings_scroll_y + 260.0).clamp(0.0, self.settings_max_scroll);
                        actions.push(UiAction::RequestRedraw);
                    }
                    KeyCode::PageUp => {
                        self.settings_scroll_y =
                            (self.settings_scroll_y - 260.0).clamp(0.0, self.settings_max_scroll);
                        actions.push(UiAction::RequestRedraw);
                    }
                    KeyCode::ArrowDown => {
                        self.settings_scroll_y =
                            (self.settings_scroll_y + 64.0).clamp(0.0, self.settings_max_scroll);
                        actions.push(UiAction::RequestRedraw);
                    }
                    KeyCode::ArrowUp => {
                        self.settings_scroll_y =
                            (self.settings_scroll_y - 64.0).clamp(0.0, self.settings_max_scroll);
                        actions.push(UiAction::RequestRedraw);
                    }
                    _ => {}
                }
            }
            return actions;
        }

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

        let mut hit_buf: Vec<HitRegion> = Vec::with_capacity(128);

        let left_w =
            self.left_sidebar_width(snapshot.sidebar_left, snapshot.sidebar_collapsed) as f32;
        let main_w = (win_w - left_w).max(200.0);
        let top_bar_base = if snapshot.sidebar_left {
            TOOLBAR_ROW_H
        } else {
            TOP_TAB_ROW_H + TOOLBAR_ROW_H
        };
        let top_h = self.top_chrome_height(
            snapshot.sidebar_left,
            snapshot.show_bookmarks_bar,
            snapshot.active_url,
        ) as f32;
        let page_top = top_bar_base
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

        // 1. Pure Rust Internal Page Views with per-tab Zoom scaling & strict clipping
        if is_internal {
            let zoom = (snapshot.zoom_percent as f32 / 100.0).clamp(0.6, 2.0);
            let mut page_hits: Vec<HitRegion> = Vec::with_capacity(64);
            let z_cursor = (cursor.0 / zoom, cursor.1 / zoom);
            let z_vx = left_w / zoom;
            let z_vy = page_top / zoom;
            let z_vw = main_w / zoom;
            let z_vh = page_vh / zoom;

            c.set_scale(scale * zoom as f64);
            c.push_clip(z_vx, z_vy, z_vw, z_vh);
            if snapshot.active_url == WELCOME_URL {
                self.render_welcome_page(
                    &mut c,
                    &pal,
                    time_sec,
                    z_cursor,
                    z_vx,
                    z_vy,
                    z_vw,
                    z_vh,
                    snapshot,
                    &mut page_hits,
                );
            } else if snapshot.active_url == SETTINGS_URL {
                self.render_settings_page(
                    &mut c,
                    &pal,
                    z_cursor,
                    z_vx,
                    z_vy,
                    z_vw,
                    z_vh,
                    snapshot,
                    &mut page_hits,
                );
            } else if snapshot.active_url == NEW_TAB_URL {
                self.render_newtab_page(
                    &mut c,
                    &pal,
                    time_sec,
                    z_cursor,
                    z_vx,
                    z_vy,
                    z_vw,
                    z_vh,
                    snapshot,
                    &mut page_hits,
                );
            }
            c.pop_clip();
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
                left_w,
                dock_y,
                main_w,
                INSPECTOR_DOCK_HEIGHT,
                snapshot,
                &mut hit_buf,
            );
        }

        // 3. Find Bar (when open on external pages)
        if self.find_open && !is_internal {
            self.render_find_bar(
                &mut c,
                &pal,
                time_sec,
                cursor,
                left_w,
                page_top,
                main_w,
                FIND_BAR_HEIGHT,
                &mut hit_buf,
            );
        }

        // 4. Bookmarks Bar
        if snapshot.show_bookmarks_bar {
            self.render_bookmarks_bar(
                &mut c,
                &pal,
                cursor,
                left_w,
                top_bar_base,
                main_w,
                BOOKMARKS_BAR_HEIGHT,
                snapshot.bookmarks,
                &mut hit_buf,
            );
        }

        // 5. Left Arc/Zed Vertical Sidebar (when active & expanded)
        if left_w > 0.0 {
            self.render_left_sidebar(
                &mut c,
                &pal,
                time_sec,
                cursor,
                win_h,
                snapshot,
                &mut hit_buf,
            );
        }

        // 6. Top Chrome Strip (Tabs + Toolbar in Top mode, or single Omnibox Toolbar in Sidebar mode)
        self.render_chrome_bar(
            &mut c,
            &pal,
            time_sec,
            cursor,
            left_w,
            main_w,
            snapshot,
            &mut hit_buf,
        );

        // 7. Floating Command Palette Overlay (Ctrl+K)
        if self.command_palette_open {
            self.render_command_palette(
                &mut c,
                &pal,
                time_sec,
                cursor,
                left_w,
                top_bar_base,
                main_w,
                win_h,
                snapshot,
                &mut hit_buf,
            );
        }

        // 8. Floating Apple-style pill toast notification
        if let Some((msg, _)) = &self.toast_message {
            let tw = self.fonts.measure_text(FontStyle::SemiBold, 10.5, msg) + 32.0;
            let tx = left_w + (main_w - tw) * 0.5;
            let ty = top_h + 12.0;
            c.fill_soft_shadow(tx, ty, tw, 34.0, 17.0, 6.0);
            draw_card(&mut c, tx, ty, tw, 34.0, 17.0, pal.cta_bg, pal.hairline_strong, 1.0);
            c.draw_text(
                &mut self.fonts,
                FontStyle::SemiBold,
                10.5,
                tx + 16.0,
                ty + 9.0,
                pal.cta_fg,
                msg,
            );
        }

        self.hits = hit_buf;
        self.last_frame_ms = frame_start.elapsed().as_secs_f32() * 1000.0;
    }

    #[allow(clippy::too_many_arguments)]
    fn render_window_controls(
        &mut self,
        c: &mut Canvas<'_>,
        pal: &ThemePalette,
        cursor: (f32, f32),
        right: f32,
        y: f32,
        h: f32,
        hits: &mut Vec<HitRegion>,
    ) -> f32 {
        let x0 = right - WIN_CTRL_W * 3.0;
        for (i, hid) in [HitId::WinMinimize, HitId::WinMaximize, HitId::WinClose]
            .into_iter()
            .enumerate()
        {
            let bx = x0 + i as f32 * WIN_CTRL_W;
            let hov = point_in(cursor, bx, y, WIN_CTRL_W, h);
            let mut fg = pal.slate;
            if hov {
                if i == 2 {
                    c.fill_rect(bx, y, WIN_CTRL_W, h, Color::rgb(0xe8, 0x11, 0x23));
                    fg = Color::rgb(0xff, 0xff, 0xff);
                } else {
                    c.fill_rect(bx, y, WIN_CTRL_W, h, pal.btn_hover);
                    fg = pal.ink;
                }
            }
            let cx = bx + WIN_CTRL_W * 0.5;
            let cy = y + h * 0.5;
            match i {
                0 => c.draw_icon_win_minimize(cx, cy, fg),
                1 => c.draw_icon_win_maximize(cx, cy, self.window_maximized, fg),
                _ => c.draw_icon_win_close(cx, cy, fg),
            }
            hits.push(HitRegion {
                x: bx,
                y,
                w: WIN_CTRL_W,
                h,
                id: hid,
                clickable: true,
            });
        }
        x0
    }

    /// Notion-style favicon: tinted letter tile, Pluto squircle for internal pages, orbit spinner while loading.
    #[allow(clippy::too_many_arguments)]
    fn draw_tab_favicon(
        &mut self,
        c: &mut Canvas<'_>,
        pal: &ThemePalette,
        time_sec: f32,
        cx: f32,
        cy: f32,
        tab: &TabSnapshot,
    ) {
        if tab.is_loading {
            for k in 0..3u8 {
                let a = time_sec * 7.0 - k as f32 * 0.6;
                let alpha = 255u8.saturating_sub(k * 75);
                c.fill_circle(
                    cx + a.cos() * 5.0,
                    cy + a.sin() * 5.0,
                    1.7,
                    Color::rgba(pal.primary.r, pal.primary.g, pal.primary.b, alpha),
                );
            }
            return;
        }
        if tab.url.starts_with("pluto://") {
            c.draw_pluto_logo(cx - 8.0, cy - 8.0, 16.0, 0.0);
            return;
        }
        let letter = favicon_letter(&tab.url, &tab.title);
        let tints = [pal.tint_sky, pal.tint_mint, pal.tint_sand, pal.tint_rose];
        c.fill_rounded_rect(cx - 8.0, cy - 8.0, 16.0, 16.0, 4.5, tints[letter as usize % 4]);
        let s = letter.to_string();
        let lw = self.fonts.measure_text(FontStyle::SemiBold, 7.6, &s);
        c.draw_text(
            &mut self.fonts,
            FontStyle::SemiBold,
            7.6,
            cx - lw * 0.5,
            text_top(cy, 7.6),
            pal.charcoal,
            &s,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn render_left_sidebar(
        &mut self,
        c: &mut Canvas<'_>,
        pal: &ThemePalette,
        time_sec: f32,
        cursor: (f32, f32),
        win_h: f32,
        snapshot: &RenderSnapshot<'_>,
        hits: &mut Vec<HitRegion>,
    ) {
        let w = SIDEBAR_W;
        c.fill_rect(0.0, 0.0, w, win_h, pal.strip);
        c.fill_rect(w - 1.0, 0.0, 1.0, win_h, pal.hairline);
        c.push_clip(0.0, 0.0, w - 1.0, win_h);

        // Header: draggable title area with brand + layout switch
        hits.push(HitRegion {
            x: 0.0,
            y: 0.0,
            w: w - 1.0,
            h: TOOLBAR_ROW_H,
            id: HitId::DragRegion,
            clickable: false,
        });
        let hc = TOOLBAR_ROW_H * 0.5;
        let logo_hov = point_in(cursor, 8.0, hc - 15.0, 92.0, 30.0);
        if logo_hov {
            c.fill_rounded_rect(8.0, hc - 15.0, 92.0, 30.0, 8.0, pal.btn_hover);
        }
        c.draw_pluto_logo(14.0, hc - 11.0, 22.0, if logo_hov { 1.0 } else { 0.0 });
        c.draw_text(
            &mut self.fonts,
            FontStyle::SemiBold,
            11.2,
            44.0,
            text_top(hc, 11.2),
            pal.ink,
            "Pluto",
        );
        hits.push(HitRegion {
            x: 8.0,
            y: hc - 15.0,
            w: 92.0,
            h: 30.0,
            id: HitId::BrandWelcome,
            clickable: true,
        });

        let lb = w - 12.0 - 30.0;
        ghost_button(c, pal, cursor, lb, hc - 15.0, 30.0, 30.0, None);
        c.draw_icon_layout_top(lb + 15.0, hc, pal.slate);
        hits.push(HitRegion {
            x: lb,
            y: hc - 15.0,
            w: 30.0,
            h: 30.0,
            id: HitId::ToggleSidebarLayout,
            clickable: true,
        });

        // Search / command field (Floe-like)
        let mut sy = TOOLBAR_ROW_H + 6.0;
        let sp_x = 12.0;
        let sp_w = w - 24.0;
        let sp_h = 34.0;
        let sp_hov = point_in(cursor, sp_x, sy, sp_w, sp_h);
        draw_card(
            c,
            sp_x,
            sy,
            sp_w,
            sp_h,
            9.0,
            if sp_hov { pal.canvas } else { pal.surface },
            pal.hairline,
            1.0,
        );
        let sc = sy + sp_h * 0.5;
        c.draw_icon_search(sp_x + 17.0, sc, pal.steel);
        c.draw_text(
            &mut self.fonts,
            FontStyle::Regular,
            10.0,
            sp_x + 32.0,
            text_top(sc, 10.0),
            pal.steel,
            "Search or command",
        );
        let kbd = "Ctrl K";
        let kw = self.fonts.measure_text(FontStyle::Regular, 8.4, kbd) + 12.0;
        let kx = sp_x + sp_w - kw - 7.0;
        draw_card(c, kx, sc - 10.0, kw, 20.0, 5.0, pal.canvas, pal.hairline, 1.0);
        c.draw_text(
            &mut self.fonts,
            FontStyle::Regular,
            8.4,
            kx + 6.0,
            text_top(sc, 8.4),
            pal.steel,
            kbd,
        );
        hits.push(HitRegion {
            x: sp_x,
            y: sy,
            w: sp_w,
            h: sp_h,
            id: HitId::TogglePalette,
            clickable: true,
        });
        sy += sp_h + 16.0;

        // Favorites
        c.draw_text(
            &mut self.fonts,
            FontStyle::Regular,
            9.2,
            16.0,
            sy,
            pal.steel,
            "Favorites",
        );
        sy += 20.0;
        let cell_w = (w - 36.0) / 3.0;
        let cell_h = 44.0;
        let tints = [
            pal.tint_sky,
            pal.tint_mint,
            pal.tint_sand,
            pal.tint_sky,
            pal.tint_rose,
            pal.tint_sand,
        ];
        for (i, q) in QUICK_LINKS.iter().enumerate() {
            let col = i % 3;
            let row = i / 3;
            let cx = 12.0 + col as f32 * (cell_w + 6.0);
            let cy = sy + row as f32 * (cell_h + 6.0);
            let hov = point_in(cursor, cx, cy, cell_w, cell_h);
            draw_card(
                c,
                cx,
                cy,
                cell_w,
                cell_h,
                10.0,
                if hov { pal.canvas } else { tints[i % tints.len()] },
                if hov { pal.hairline_strong } else { Color::rgba(0, 0, 0, 0) },
                1.0,
            );
            c.draw_text(
                &mut self.fonts,
                FontStyle::SemiBold,
                9.6,
                cx + 10.0,
                cy + 7.0,
                pal.ink,
                q.badge,
            );
            c.draw_text_ellipsis(
                &mut self.fonts,
                FontStyle::Regular,
                8.4,
                cx + 10.0,
                cy + 24.0,
                cell_w - 16.0,
                pal.slate,
                q.tag,
            );
            hits.push(HitRegion {
                x: cx,
                y: cy,
                w: cell_w,
                h: cell_h,
                id: HitId::OpenUrlTile(q.url.to_string()),
                clickable: true,
            });
        }
        sy += 2.0 * (cell_h + 6.0) + 12.0;

        // Tabs header + ghost "+"
        let tabs_hdr = format!("Tabs  Â·  {}", snapshot.tabs.len());
        c.draw_text(
            &mut self.fonts,
            FontStyle::Regular,
            9.2,
            16.0,
            sy + 4.0,
            pal.steel,
            &tabs_hdr,
        );
        let nt_x = w - 12.0 - 26.0;
        ghost_button(c, pal, cursor, nt_x, sy - 2.0, 26.0, 26.0, None);
        c.draw_icon_plus(nt_x + 13.0, sy + 11.0, pal.slate);
        hits.push(HitRegion {
            x: nt_x,
            y: sy - 2.0,
            w: 26.0,
            h: 26.0,
            id: HitId::NewTabBtn,
            clickable: true,
        });
        sy += 30.0;

        // Vertical tab list
        let footer_h = 104.0;
        let footer_y = (win_h - footer_h).max(sy + 40.0);
        for tab in snapshot.tabs {
            let th = 34.0;
            if sy + th > footer_y - 6.0 {
                break;
            }
            let tx = 10.0;
            let tw = w - 20.0;
            let hov = point_in(cursor, tx, sy, tw, th);
            if tab.active {
                draw_card(
                    c,
                    tx,
                    sy,
                    tw,
                    th,
                    8.0,
                    pal.tab_active,
                    if self.dark_mode { Color::rgba(0, 0, 0, 0) } else { pal.hairline },
                    1.0,
                );
            } else if hov {
                c.fill_rounded_rect(tx, sy, tw, th, 8.0, pal.tab_hover);
            }
            let tc = sy + th * 0.5;
            self.draw_tab_favicon(c, pal, time_sec, tx + 18.0, tc, tab);

            let show_close = !tab.pinned && (tab.active || hov);
            let close_w = if show_close { 26.0 } else { 6.0 };
            let flag_w = if tab.is_muted { 18.0 } else { 0.0 };
            c.draw_text_ellipsis(
                &mut self.fonts,
                FontStyle::Regular,
                10.4,
                tx + 34.0,
                text_top(tc, 10.4),
                (tw - 40.0 - close_w - flag_w).max(12.0),
                if tab.active { pal.ink } else { pal.slate },
                &tab.title,
            );
            if tab.is_muted {
                c.draw_icon_volume(tx + tw - close_w - 10.0, tc, true, pal.steel);
            }
            hits.push(HitRegion {
                x: tx,
                y: sy,
                w: tw - if show_close { 26.0 } else { 0.0 },
                h: th,
                id: HitId::Tab(tab.id),
                clickable: true,
            });
            if show_close {
                let cx = tx + tw - 26.0;
                let cy = tc - 10.0;
                let chov = point_in(cursor, cx, cy, 20.0, 20.0);
                if chov {
                    c.fill_rounded_rect(cx, cy, 20.0, 20.0, 5.0, pal.btn_hover);
                }
                c.draw_icon_close(cx + 10.0, tc, if chov { pal.ink } else { pal.steel });
                hits.push(HitRegion {
                    x: cx,
                    y: cy,
                    w: 20.0,
                    h: 20.0,
                    id: HitId::CloseTab(tab.id),
                    clickable: true,
                });
            }
            sy += th + 2.0;
        }

        // Footer: quiet ghost actions
        c.fill_rect(0.0, footer_y, w - 1.0, win_h - footer_y, pal.strip);
        c.fill_rect(12.0, footer_y, w - 24.0, 1.0, pal.hairline);
        let bw = (w - 28.0) / 2.0;
        let bh = 32.0;
        let fx1 = 10.0;
        let fx2 = 18.0 + bw;
        let fy1 = footer_y + 8.0;
        let fy2 = fy1 + bh + 2.0;
        for (bx, by, label, hid, active, icon) in [
            (fx1, fy1, "Bookmarks", HitId::BookmarkBarToggle, snapshot.show_bookmarks_bar, 0u8),
            (fx2, fy1, "Inspector", HitId::ToggleInspector, self.inspector_open, 1),
            (fx1, fy2, "Settings", HitId::OpenSettings, snapshot.active_url == SETTINGS_URL, 2),
            (
                fx2,
                fy2,
                if self.dark_mode { "Midnight" } else { "Daylight" },
                HitId::ThemeToggle,
                false,
                3,
            ),
        ] {
            let hov = point_in(cursor, bx, by, bw, bh);
            if active {
                c.fill_rounded_rect(bx, by, bw, bh, 8.0, pal.tab_active);
            } else if hov {
                c.fill_rounded_rect(bx, by, bw, bh, 8.0, pal.btn_hover);
            }
            let ic = if active || hov { pal.ink } else { pal.slate };
            let icx = bx + 16.0;
            let icy = by + bh * 0.5;
            match icon {
                0 => c.draw_icon_star(icx, icy, active, ic),
                1 => c.draw_icon_history(icx, icy, ic),
                2 => c.draw_icon_settings(icx, icy, ic),
                _ => c.draw_icon_theme(icx, icy, self.dark_mode, ic),
            }
            c.draw_text(
                &mut self.fonts,
                FontStyle::Regular,
                9.8,
                bx + 30.0,
                text_top(icy, 9.8),
                ic,
                label,
            );
            hits.push(HitRegion {
                x: bx,
                y: by,
                w: bw,
                h: bh,
                id: hid,
                clickable: true,
            });
        }
        let status_y = fy2 + bh + 8.0;
        c.fill_circle(
            17.0,
            status_y + 6.0,
            3.0,
            if snapshot.adblock_enabled { pal.brand_green } else { pal.muted },
        );
        c.draw_text(
            &mut self.fonts,
            FontStyle::Regular,
            8.6,
            26.0,
            status_y,
            pal.steel,
            if snapshot.adblock_enabled {
                "Shield on  Â·  Native Rust UI"
            } else {
                "Shield paused  Â·  Native Rust UI"
            },
        );

        c.pop_clip();
    }

    #[allow(clippy::too_many_arguments)]
    fn render_chrome_bar(
        &mut self,
        c: &mut Canvas<'_>,
        pal: &ThemePalette,
        time_sec: f32,
        cursor: (f32, f32),
        left_w: f32,
        main_w: f32,
        snapshot: &RenderSnapshot<'_>,
        hits: &mut Vec<HitRegion>,
    ) {
        let mut toolbar_y = 0.0;
        let right_edge = left_w + main_w;

        // â”€â”€ Row 1: minimalist tab strip + custom window controls (Top layout) â”€â”€
        if !snapshot.sidebar_left {
            let rh = TOP_TAB_ROW_H;
            c.fill_rect(left_w, 0.0, main_w, rh, pal.strip);
            hits.push(HitRegion {
                x: left_w,
                y: 0.0,
                w: main_w,
                h: rh,
                id: HitId::DragRegion,
                clickable: false,
            });

            let lc = rh * 0.5;
            let logo_hov = point_in(cursor, 8.0, lc - 14.0, 30.0, 28.0);
            if logo_hov {
                c.fill_rounded_rect(8.0, lc - 14.0, 30.0, 28.0, 8.0, pal.btn_hover);
            }
            c.draw_pluto_logo(12.0, lc - 11.0, 22.0, if logo_hov { 1.0 } else { 0.0 });
            hits.push(HitRegion {
                x: 8.0,
                y: lc - 14.0,
                w: 30.0,
                h: 28.0,
                id: HitId::BrandWelcome,
                clickable: true,
            });

            let ctrl_x = self.render_window_controls(c, pal, cursor, right_edge, 0.0, rh, hits);

            let mut tx = 46.0;
            let right_limit = (ctrl_x - 72.0).max(tx + 80.0);
            let avail_w = (right_limit - tx).max(60.0);
            let pinned_count = snapshot.tabs.iter().filter(|t| t.pinned).count();
            let reg_count = snapshot.tabs.len().saturating_sub(pinned_count).max(1);
            let pinned_total = pinned_count as f32 * 40.0;
            let reg_w = ((avail_w - pinned_total) / reg_count as f32).clamp(110.0, 236.0);
            let ty = 5.0;
            let th = rh - 10.0;
            let tc = ty + th * 0.5;

            for tab in snapshot.tabs {
                let tw = if tab.pinned { 40.0 } else { reg_w };
                if tx + tw > right_limit + 2.0 {
                    break;
                }
                let bw = tw - 4.0;
                let hov = point_in(cursor, tx, ty, bw, th);
                if tab.active {
                    draw_card(
                        c,
                        tx,
                        ty,
                        bw,
                        th,
                        8.0,
                        pal.tab_active,
                        if self.dark_mode { Color::rgba(0, 0, 0, 0) } else { pal.hairline },
                        1.0,
                    );
                } else if hov {
                    c.fill_rounded_rect(tx, ty, bw, th, 8.0, pal.tab_hover);
                }

                if tab.pinned {
                    self.draw_tab_favicon(c, pal, time_sec, tx + bw * 0.5, tc, tab);
                    hits.push(HitRegion {
                        x: tx,
                        y: ty,
                        w: bw,
                        h: th,
                        id: HitId::Tab(tab.id),
                        clickable: true,
                    });
                    tx += tw;
                    continue;
                }

                self.draw_tab_favicon(c, pal, time_sec, tx + 17.0, tc, tab);
                let show_close = tab.active || hov;
                let close_pad = if show_close { 26.0 } else { 8.0 };
                let flag_w = if tab.is_muted || tab.reader_active || tab.night_active {
                    16.0
                } else {
                    0.0
                };
                c.draw_text_ellipsis(
                    &mut self.fonts,
                    FontStyle::Regular,
                    10.6,
                    tx + 31.0,
                    text_top(tc, 10.6),
                    (bw - 31.0 - close_pad - flag_w).max(12.0),
                    if tab.active { pal.ink } else { pal.slate },
                    &tab.title,
                );
                let fx = tx + bw - close_pad - 9.0;
                if tab.is_muted {
                    c.draw_icon_volume(fx, tc, true, pal.steel);
                } else if tab.reader_active {
                    c.fill_circle(fx, tc, 3.0, pal.primary);
                } else if tab.night_active {
                    c.fill_circle(fx, tc, 3.0, pal.brand_amber);
                }

                hits.push(HitRegion {
                    x: tx,
                    y: ty,
                    w: bw - if show_close { 26.0 } else { 0.0 },
                    h: th,
                    id: HitId::Tab(tab.id),
                    clickable: true,
                });
                if show_close {
                    let cx = tx + bw - 25.0;
                    let cy = tc - 10.0;
                    let chov = point_in(cursor, cx, cy, 20.0, 20.0);
                    if chov {
                        c.fill_rounded_rect(cx, cy, 20.0, 20.0, 5.0, pal.btn_hover);
                    }
                    c.draw_icon_close(cx + 10.0, tc, if chov { pal.ink } else { pal.steel });
                    hits.push(HitRegion {
                        x: cx,
                        y: cy,
                        w: 20.0,
                        h: 20.0,
                        id: HitId::CloseTab(tab.id),
                        clickable: true,
                    });
                }
                tx += tw;
            }

            let ntx = tx + 2.0;
            if ntx + 28.0 <= ctrl_x - 8.0 {
                ghost_button(c, pal, cursor, ntx, tc - 14.0, 28.0, 28.0, None);
                c.draw_icon_plus(ntx + 14.0, tc, pal.slate);
                hits.push(HitRegion {
                    x: ntx,
                    y: tc - 14.0,
                    w: 28.0,
                    h: 28.0,
                    id: HitId::NewTabBtn,
                    clickable: true,
                });
            }
            toolbar_y = rh;
        }

        // â”€â”€ Row 2: quiet toolbar â”€â”€
        let h = TOOLBAR_ROW_H;
        c.fill_rect(left_w, toolbar_y, main_w, h, pal.canvas);
        c.fill_rect(left_w, toolbar_y + h - 1.0, main_w, 1.0, pal.hairline);
        if snapshot.sidebar_left {
            hits.push(HitRegion {
                x: left_w,
                y: toolbar_y,
                w: main_w,
                h,
                id: HitId::DragRegion,
                clickable: false,
            });
        }
        let bh = 32.0;
        let by = toolbar_y + (h - bh) * 0.5;
        let bc = by + bh * 0.5;
        let mut nx = left_w + 10.0;

        // Layout / sidebar toggle
        {
            let collapsed = snapshot.sidebar_left && snapshot.sidebar_collapsed;
            let bw = if collapsed { 50.0 } else { 32.0 };
            let hov = ghost_button(c, pal, cursor, nx, by, bw, bh, None);
            let ic = if hov { pal.ink } else { pal.slate };
            c.draw_icon_sidebar(nx + 16.0, bc, ic);
            if collapsed {
                let cnt = format!("{}", snapshot.tabs.len());
                c.draw_text(
                    &mut self.fonts,
                    FontStyle::Regular,
                    9.6,
                    nx + 29.0,
                    text_top(bc, 9.6),
                    ic,
                    &cnt,
                );
            }
            hits.push(HitRegion {
                x: nx,
                y: by,
                w: bw,
                h: bh,
                id: if snapshot.sidebar_left {
                    HitId::ToggleSidebarCollapse
                } else {
                    HitId::ToggleSidebarLayout
                },
                clickable: true,
            });
            nx += bw + 4.0;
        }

        for (hid, kind) in [(HitId::Back, 0u8), (HitId::Forward, 1), (HitId::ReloadOrStop, 2)] {
            let hov = ghost_button(c, pal, cursor, nx, by, 32.0, bh, None);
            let fg = if hov { pal.ink } else { pal.slate };
            let cx = nx + 16.0;
            match kind {
                0 => c.draw_icon_back(cx, bc, fg),
                1 => c.draw_icon_forward(cx, bc, fg),
                _ => {
                    if snapshot.active_is_loading {
                        c.draw_icon_stop(cx, bc, pal.primary);
                    } else {
                        c.draw_icon_reload(cx, bc, fg);
                    }
                }
            }
            hits.push(HitRegion {
                x: nx,
                y: by,
                w: 32.0,
                h: bh,
                id: hid,
                clickable: true,
            });
            nx += 34.0;
        }
        nx += 6.0;

        // Right cluster (window controls live here in sidebar layout)
        let mut rx = right_edge - 8.0;
        if snapshot.sidebar_left {
            rx = self.render_window_controls(c, pal, cursor, right_edge, toolbar_y, h - 1.0, hits)
                - 6.0;
        }
        // (hit, icon kind, icon color override, tinted bg accent)
        let mut specs: Vec<(HitId, u8, Option<Color>, Option<Color>)> = Vec::with_capacity(6);
        specs.push((
            HitId::ToggleShield,
            0,
            if snapshot.adblock_enabled { Some(pal.brand_green) } else { None },
            None,
        ));
        specs.push((
            HitId::ToggleReader,
            1,
            if snapshot.active_reader { Some(pal.primary) } else { None },
            if snapshot.active_reader { Some(pal.primary) } else { None },
        ));
        if snapshot.active_night {
            specs.push((HitId::ToggleNight, 2, Some(pal.brand_amber), Some(pal.brand_amber)));
        }
        if snapshot.active_muted {
            specs.push((HitId::ToggleMute, 3, Some(pal.primary), Some(pal.primary)));
        }
        specs.push((
            HitId::TogglePalette,
            4,
            None,
            if self.command_palette_open { Some(pal.primary) } else { None },
        ));
        specs.push((
            HitId::OpenSettings,
            5,
            None,
            if snapshot.active_url == SETTINGS_URL { Some(pal.primary) } else { None },
        ));
        let total = specs.len() as f32 * 34.0 - 2.0;
        let actions_x = rx - total;
        let mut ax = actions_x;
        for (hid, kind, icon_col, bg_acc) in specs {
            let hov = ghost_button(c, pal, cursor, ax, by, 32.0, bh, bg_acc);
            let fg = icon_col.unwrap_or(if hov { pal.ink } else { pal.slate });
            let cx = ax + 16.0;
            match kind {
                0 => c.draw_icon_shield(cx, bc, snapshot.adblock_enabled, fg),
                1 => c.draw_icon_reader(cx, bc, fg),
                2 => c.draw_icon_night(cx, bc, true, fg),
                3 => c.draw_icon_volume(cx, bc, true, fg),
                4 => c.draw_icon_more(cx, bc, fg),
                _ => c.draw_icon_settings(cx, bc, fg),
            }
            hits.push(HitRegion {
                x: ax,
                y: by,
                w: 32.0,
                h: bh,
                id: hid,
                clickable: true,
            });
            ax += 34.0;
        }

        // â”€â”€ Omnibox: soft pill, no border until focused â”€â”€
        let omni_x = nx;
        let omni_w = (actions_x - 10.0 - omni_x).max(140.0);
        let focused = self.active_input == ActiveInput::Omnibox;
        let ohov = point_in(cursor, omni_x, by, omni_w, bh);
        if focused {
            let p = pal.primary;
            c.fill_rounded_rect(
                omni_x - 3.0,
                by - 3.0,
                omni_w + 6.0,
                bh + 6.0,
                12.0,
                Color::rgba(p.r, p.g, p.b, 40),
            );
            draw_card(c, omni_x, by, omni_w, bh, 9.0, pal.canvas, pal.primary, 1.2);
        } else {
            c.fill_rounded_rect(
                omni_x,
                by,
                omni_w,
                bh,
                9.0,
                if ohov { pal.surface_soft } else { pal.omnibox_bg },
            );
        }

        let icx = omni_x + 16.0;
        if focused || self.omnibox_text.is_empty() {
            c.draw_icon_search(icx, bc, pal.steel);
        } else if snapshot.active_url.starts_with("pluto://") {
            c.draw_pluto_logo(icx - 7.0, bc - 7.0, 14.0, 0.0);
        } else if snapshot.active_url.starts_with("https://") {
            c.draw_icon_lock(icx, bc, pal.steel);
        } else {
            c.draw_icon_search(icx, bc, pal.steel);
        }

        // Inside right: [clear] [zoom badge] [star]
        let mut irx = omni_x + omni_w - 4.0;
        let star_w = 28.0;
        irx -= star_w;
        let star_x = irx;
        let star_hov = point_in(cursor, star_x, by + 3.0, star_w, bh - 6.0);
        if star_hov {
            c.fill_rounded_rect(star_x, by + 3.0, star_w, bh - 6.0, 6.0, pal.btn_hover);
        }
        c.draw_icon_star(
            star_x + star_w * 0.5,
            bc,
            snapshot.is_bookmarked,
            if snapshot.is_bookmarked {
                pal.star_active
            } else if star_hov {
                pal.ink
            } else {
                pal.steel
            },
        );
        hits.push(HitRegion {
            x: star_x,
            y: by + 3.0,
            w: star_w,
            h: bh - 6.0,
            id: HitId::ToggleBookmark,
            clickable: true,
        });

        if snapshot.zoom_percent != 100 {
            let zl = format!("{}%", snapshot.zoom_percent);
            let zw = self.fonts.measure_text(FontStyle::Regular, 8.6, &zl) + 14.0;
            irx -= zw + 4.0;
            let zx = irx;
            let zhov = point_in(cursor, zx, bc - 10.0, zw, 20.0);
            draw_card(
                c,
                zx,
                bc - 10.0,
                zw,
                20.0,
                6.0,
                if zhov { pal.btn_hover } else { pal.canvas },
                pal.hairline,
                1.0,
            );
            c.draw_text(
                &mut self.fonts,
                FontStyle::Regular,
                8.6,
                zx + 7.0,
                text_top(bc, 8.6),
                pal.slate,
                &zl,
            );
            hits.push(HitRegion {
                x: zx,
                y: bc - 10.0,
                w: zw,
                h: 20.0,
                id: HitId::ZoomReset,
                clickable: true,
            });
        }

        if focused && !self.omnibox_text.is_empty() {
            irx -= 24.0;
            let clr_x = irx;
            c.draw_icon_close(clr_x + 12.0, bc, pal.steel);
            hits.push(HitRegion {
                x: clr_x,
                y: by + 3.0,
                w: 24.0,
                h: bh - 6.0,
                id: HitId::OmniboxClear,
                clickable: true,
            });
        }

        let fs = 11.0;
        let text_x = omni_x + 32.0;
        let text_y = text_top(bc, fs);
        let text_max_w = (irx - text_x - 8.0).max(20.0);
        hits.push(HitRegion {
            x: omni_x,
            y: by,
            w: (irx - omni_x).max(40.0),
            h: bh,
            id: HitId::Omnibox,
            clickable: true,
        });

        let caret_on = focused && (time_sec * 2.0) as u32 % 2 == 0;
        if self.omnibox_text.is_empty() {
            c.draw_text_ellipsis(
                &mut self.fonts,
                FontStyle::Regular,
                fs,
                text_x,
                text_y,
                text_max_w,
                pal.steel,
                "Search Google or type a URL",
            );
            if caret_on {
                c.fill_rect(text_x, by + 8.0, 1.5, bh - 16.0, pal.primary);
            }
        } else if focused {
            let shown = tail_fitting_width(
                &self.fonts,
                FontStyle::Regular,
                fs,
                &self.omnibox_text,
                text_max_w,
            )
            .to_string();
            let tw = self.fonts.measure_text(FontStyle::Regular, fs, &shown);
            if self.omnibox_select_all {
                let p = pal.primary;
                c.fill_rounded_rect(
                    text_x - 2.0,
                    by + 6.0,
                    (tw + 4.0).min(text_max_w),
                    bh - 12.0,
                    4.0,
                    Color::rgba(p.r, p.g, p.b, 60),
                );
            }
            c.draw_text(&mut self.fonts, FontStyle::Regular, fs, text_x, text_y, pal.ink, &shown);
            if caret_on && !self.omnibox_select_all {
                c.fill_rect((text_x + tw + 1.0).min(irx - 4.0), by + 8.0, 1.5, bh - 16.0, pal.primary);
            }
        } else {
            // Calm display: host in ink, path in steel, scheme hidden
            let t = self.omnibox_text.clone();
            let is_web = t.starts_with("https://") || t.starts_with("http://");
            let stripped = t
                .strip_prefix("https://")
                .or_else(|| t.strip_prefix("http://"))
                .unwrap_or(&t);
            let (host, path) = match stripped.find('/') {
                Some(i) if is_web => stripped.split_at(i),
                _ => (stripped, ""),
            };
            let path = if path == "/" { "" } else { path };
            let hw = self
                .fonts
                .measure_text(FontStyle::Regular, fs, host)
                .min(text_max_w);
            c.draw_text_ellipsis(
                &mut self.fonts,
                FontStyle::Regular,
                fs,
                text_x,
                text_y,
                text_max_w,
                pal.ink,
                host,
            );
            if !path.is_empty() && hw < text_max_w - 24.0 {
                c.draw_text_ellipsis(
                    &mut self.fonts,
                    FontStyle::Regular,
                    fs,
                    text_x + hw,
                    text_y,
                    text_max_w - hw,
                    pal.steel,
                    path,
                );
            }
        }
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
        c.fill_rect(x, y, w, h, pal.canvas);
        c.fill_rect(x, y + h - 1.0, w, 1.0, pal.hairline);

        let mid = y + h * 0.5;
        let mut bx = x + 12.0;
        if bookmarks.is_empty() {
            c.draw_text(
                &mut self.fonts,
                FontStyle::Regular,
                9.4,
                bx + 4.0,
                text_top(mid, 9.4),
                pal.steel,
                "No bookmarks yet — press the star in the address bar",
            );
        }

        for bm in bookmarks.iter().take(12) {
            let label = bm.title.as_str();
            let iw = (self.fonts.measure_text(FontStyle::Regular, 9.6, label) + 36.0).clamp(60.0, 180.0);
            if bx + iw > x + w - 16.0 {
                break;
            }
            let by = y + 4.0;
            let bh = h - 8.0;
            let hov = point_in(cursor, bx, by, iw, bh);
            if hov {
                c.fill_rounded_rect(bx, by, iw, bh, 6.0, pal.btn_hover);
            }
            let letter = favicon_letter(&bm.url, label);
            let tints = [pal.tint_sky, pal.tint_mint, pal.tint_sand, pal.tint_rose];
            c.fill_rounded_rect(bx + 7.0, mid - 6.0, 12.0, 12.0, 3.5, tints[letter as usize % 4]);
            let s = letter.to_string();
            let lw = self.fonts.measure_text(FontStyle::SemiBold, 6.4, &s);
            c.draw_text(&mut self.fonts, FontStyle::SemiBold, 6.4, bx + 13.0 - lw * 0.5, text_top(mid, 6.4), pal.charcoal, &s);
            c.draw_text_ellipsis(
                &mut self.fonts,
                FontStyle::Regular,
                9.6,
                bx + 25.0,
                text_top(mid, 9.6),
                iw - 31.0,
                if hov { pal.ink } else { pal.slate },
                label,
            );
            hits.push(HitRegion {
                x: bx,
                y: by,
                w: iw,
                h: bh,
                id: HitId::BookmarkBarItem(bm.url.clone()),
                clickable: true,
            });
            bx += iw + 6.0;
        }
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

        let ix = x + 14.0;
        let iy = y + 7.0;
        let iw = 296.0_f32.min(w - 224.0).max(120.0);
        let ih = h - 14.0;
        let focused = self.active_input == ActiveInput::FindBar;
        draw_card(
            c,
            ix,
            iy,
            iw,
            ih,
            9.0,
            pal.canvas,
            if focused { pal.primary } else { pal.hairline },
            if focused { 1.5 } else { 1.0 },
        );
        hits.push(HitRegion {
            x: ix,
            y: iy,
            w: iw,
            h: ih,
            id: HitId::FindInput,
            clickable: true,
        });

        if self.find_text.is_empty() {
            c.draw_text(&mut self.fonts, FontStyle::Regular, 10.0, ix + 12.0, iy + 8.0, pal.steel, "Find in page...");
            if focused && (time_sec * 2.0) as u32 % 2 == 0 {
                c.fill_rect(ix + 12.0, iy + 7.0, 2.0, ih - 14.0, pal.primary);
            }
        } else {
            let shown = tail_fitting_width(&self.fonts, FontStyle::Regular, 10.0, &self.find_text, iw - 24.0).to_string();
            let tw = self.fonts.measure_text(FontStyle::Regular, 10.0, &shown);
            c.draw_text(&mut self.fonts, FontStyle::Regular, 10.0, ix + 12.0, iy + 8.0, pal.ink, &shown);
            if focused && (time_sec * 2.0) as u32 % 2 == 0 {
                c.fill_rect(ix + 12.0 + tw + 1.0, iy + 7.0, 2.0, ih - 14.0, pal.primary);
            }
        }

        let mut bx = ix + iw + 8.0;
        for (bw, label, hid) in [
            (62.0, "Prev", HitId::FindPrev),
            (62.0, "Next", HitId::FindNext),
            (66.0, "Close", HitId::FindClose),
        ] {
            let hov = point_in(cursor, bx, iy, bw, ih);
            draw_card(
                c,
                bx,
                iy,
                bw,
                ih,
                8.0,
                if hov { pal.btn_hover } else { pal.canvas },
                if hov { pal.hairline_strong } else { pal.hairline },
                1.0,
            );
            c.draw_text(&mut self.fonts, FontStyle::SemiBold, 9.0, bx + 14.0, iy + 8.0, pal.ink, label);
            hits.push(HitRegion {
                x: bx,
                y: iy,
                w: bw,
                h: ih,
                id: hid,
                clickable: true,
            });
            bx += bw + 6.0;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_command_palette(
        &mut self,
        c: &mut Canvas<'_>,
        pal: &ThemePalette,
        time_sec: f32,
        cursor: (f32, f32),
        left_w: f32,
        top_y: f32,
        main_w: f32,
        win_h: f32,
        snapshot: &RenderSnapshot<'_>,
        hits: &mut Vec<HitRegion>,
    ) {
        c.fill_rect(
            left_w,
            top_y,
            main_w,
            win_h - top_y,
            Color::rgba(0, 0, 0, 125),
        );
        hits.push(HitRegion {
            x: left_w,
            y: top_y,
            w: main_w,
            h: win_h - top_y,
            id: HitId::PaletteOverlayBackdrop,
            clickable: true,
        });

        let pw = (main_w - 64.0).clamp(380.0, 680.0);
        let px = left_w + (main_w - pw) * 0.5;
        let py = top_y + 42.0;

        let q = self.command_palette_query.trim().to_ascii_lowercase();
        let mut items: Vec<PaletteEntry> = Vec::new();

        let builtins = [
            ("Open Browser Settings", "Switch Top Bar / Arc Sidebar, Zoom & Shield (Ctrl+,)", "CFG", UiAction::OpenSettings),
            ("Switch Navigation Layout", "Toggle Top Tab Strip vs Arc Left Sidebar (Ctrl+Shift+L)", "UI", UiAction::ToggleSidebarLayout),
            ("Collapse / Expand Sidebar", "Toggle Left Sidebar width (Ctrl+B)", "UI", UiAction::ToggleSidebarCollapse),
            ("Toggle Pluto Shield", "Suppress ads, trackers & cookie banners", "SEC", UiAction::ToggleShield),
            ("Toggle Reader Mode", "Clean Apple/Notion reading view (Alt+R)", "WEB", UiAction::ToggleReaderMode),
            ("Toggle Web Night Eye", "Comfortable dark mode for any website (Alt+N)", "NGT", UiAction::ToggleNightMode),
            ("Mute / Unmute Tab Audio", "Silence video/audio tags (Alt+M)", "AUD", UiAction::ToggleMute),
            ("Copy Clean Markdown Link", "Copy [Title](URL) with trackers stripped (Ctrl+Shift+Y)", "CPY", UiAction::CopyMarkdown),
            ("Copy Page URL", "Copy current address without tracking params", "CPY", UiAction::CopyActiveUrl),
            ("Find in Page", "Search text on this page (Ctrl+F)", "WEB", UiAction::ToggleFindBar),
            ("Reopen Closed Tab", "Restore last closed tab (Ctrl+Shift+T)", "TAB", UiAction::ReopenClosedTab),
            ("Go Home", "Open the start page", "NAV", UiAction::Home),
            ("Open DevTools", "WebView2 developer tools (F12)", "DEV", UiAction::OpenDevtools),
            ("New Tab Workspace", "Open Start Page (Ctrl+T)", "TAB", UiAction::NewTab(None)),
            ("Duplicate Active Tab", "Clone active tab (Ctrl+Shift+K)", "TAB", UiAction::DuplicateTab),
            ("Pin / Unpin Active Tab", "Lock tab slot", "TAB", UiAction::PinActiveTab),
            ("Toggle Bookmarks Bar", "Show/hide bookmarks row (Ctrl+Shift+B)", "UI", UiAction::ToggleBookmarksBar),
            ("Toggle Daylight / Midnight", "Switch Apple/Notion Light vs Dark theme", "THM", UiAction::ToggleTheme),
            ("Zoom In (+10%)", "Increase page scale (Ctrl+=)", "ZOM", UiAction::ZoomIn),
            ("Zoom Out (-10%)", "Decrease page scale (Ctrl+-)", "ZOM", UiAction::ZoomOut),
            ("Reset Zoom (100%)", "Reset scale to 100% (Ctrl+0)", "ZOM", UiAction::ZoomReset),
            ("Pluto Native Inspector", "Bottom runtime & page HUD (Ctrl+Shift+D)", "DEV", UiAction::ToggleInspector),
            ("Welcome Showcase", "Open pluto://welcome product showcase", "PLT", UiAction::OpenWelcome),
            ("Clear Browsing History", "Erase saved local history.json", "PRV", UiAction::ClearHistory),
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
                    title: format!("Tab: {}", t.title),
                    subtitle: t.url.clone(),
                    badge: "TAB".to_string(),
                    exec: PaletteEntryAction::SwitchTab(t.id),
                });
            }
        }
        for bm in snapshot.bookmarks {
            if q.is_empty() || bm.title.to_ascii_lowercase().contains(&q) || bm.url.to_ascii_lowercase().contains(&q) {
                items.push(PaletteEntry {
                    title: bm.title.clone(),
                    subtitle: bm.url.clone(),
                    badge: "BMK".to_string(),
                    exec: PaletteEntryAction::Navigate(bm.url.clone()),
                });
            }
        }
        for h in snapshot.history.iter().take(6) {
            if q.is_empty() || h.title.to_ascii_lowercase().contains(&q) || h.url.to_ascii_lowercase().contains(&q) {
                items.push(PaletteEntry {
                    title: h.title.clone(),
                    subtitle: h.url.clone(),
                    badge: "HST".to_string(),
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
            58.0
        } else {
            self.palette_entries.len() as f32 * 50.0 + 12.0
        };
        let ph = 58.0 + list_h;

        c.fill_soft_shadow(px, py, pw, ph, 14.0, 12.0);
        draw_card(c, px, py, pw, ph, 14.0, pal.canvas, pal.hairline_strong, 1.0);
        c.fill_rounded_rect(px + 1.0, py + 1.0, pw - 2.0, 56.0, 13.0, pal.surface);
        c.fill_rect(px + 1.0, py + 57.0, pw - 2.0, 1.0, pal.hairline);

        c.draw_icon_palette(px + 24.0, py + 29.0, pal.primary);
        hits.push(HitRegion {
            x: px,
            y: py,
            w: pw,
            h: 57.0,
            id: HitId::PaletteBox,
            clickable: true,
        });

        let qt_x = px + 46.0;
        if self.command_palette_query.is_empty() {
            c.draw_text(
                &mut self.fonts,
                FontStyle::Regular,
                11.0,
                qt_x,
                py + 19.0,
                pal.steel,
                "Command Palette: search actions, tabs, settings...",
            );
            if (time_sec * 2.0) as u32 % 2 == 0 {
                c.fill_rect(qt_x, py + 17.0, 2.0, 24.0, pal.primary);
            }
        } else {
            let tw = self.fonts.measure_text(FontStyle::Regular, 11.0, &self.command_palette_query);
            c.draw_text(
                &mut self.fonts,
                FontStyle::Regular,
                11.0,
                qt_x,
                py + 19.0,
                pal.ink,
                &self.command_palette_query,
            );
            if (time_sec * 2.0) as u32 % 2 == 0 {
                c.fill_rect(qt_x + tw + 2.0, py + 17.0, 2.0, 24.0, pal.primary);
            }
        }

        let mut iy = py + 64.0;
        if self.palette_entries.is_empty() {
            c.draw_text(&mut self.fonts, FontStyle::Regular, 10.5, px + 22.0, iy + 14.0, pal.steel, "No matching commands");
        } else {
            for (idx, item) in self.palette_entries.iter().enumerate() {
                let active = idx == self.command_palette_idx;
                let hov = point_in(cursor, px + 8.0, iy, pw - 16.0, 46.0);
                let bg = if active {
                    pal.surface_soft
                } else if hov {
                    pal.btn_hover
                } else {
                    pal.canvas
                };
                draw_card(
                    c,
                    px + 8.0,
                    iy,
                    pw - 16.0,
                    46.0,
                    9.0,
                    bg,
                    if active { pal.hairline_strong } else { Color::rgba(0, 0, 0, 0) },
                    1.0,
                );
                if active {
                    c.fill_rounded_rect(px + 12.0, iy + 10.0, 3.0, 26.0, 1.5, pal.primary);
                }
                draw_card(c, px + 20.0, iy + 11.0, 42.0, 24.0, 6.0, pal.strip, pal.hairline, 1.0);
                c.draw_text(&mut self.fonts, FontStyle::Mono, 8.5, px + 27.0, iy + 17.0, pal.ink, &item.badge);
                c.draw_text_ellipsis(
                    &mut self.fonts,
                    FontStyle::SemiBold,
                    10.5,
                    px + 72.0,
                    iy + 7.0,
                    pw - 96.0,
                    pal.ink,
                    &item.title,
                );
                c.draw_text_ellipsis(
                    &mut self.fonts,
                    FontStyle::Regular,
                    8.5,
                    px + 72.0,
                    iy + 26.0,
                    pw - 96.0,
                    pal.steel,
                    &item.subtitle,
                );
                hits.push(HitRegion {
                    x: px + 8.0,
                    y: iy,
                    w: pw - 16.0,
                    h: 46.0,
                    id: HitId::PaletteItem(idx),
                    clickable: true,
                });
                iy += 50.0;
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
        c.fill_rect(x, y + 1.0, w, 42.0, pal.strip);
        c.fill_rect(x, y + 43.0, w, 1.0, pal.hairline);

        c.draw_text(&mut self.fonts, FontStyle::Bold, 11.0, x + 16.0, y + 14.0, pal.ink, "Pluto Inspector");
        draw_badge(c, &mut self.fonts, x + 150.0, y + 10.0, pal.tint_sky, pal.hairline, pal.primary, "Native Rust");

        let tabs = ["Runtime & Performance", "Page Telemetry"];
        let mut tx = x + 266.0;
        for (idx, label) in tabs.iter().enumerate() {
            let tw = self.fonts.measure_text(FontStyle::SemiBold, 9.5, label) + 22.0;
            let active = self.inspector_tab == idx;
            if active {
                c.fill_rounded_rect(tx, y + 40.0, tw, 3.0, 1.5, pal.primary);
            }
            c.draw_text(
                &mut self.fonts,
                FontStyle::SemiBold,
                9.5,
                tx + 11.0,
                y + 14.0,
                if active { pal.ink } else { pal.steel },
                label,
            );
            hits.push(HitRegion {
                x: tx,
                y: y + 4.0,
                w: tw,
                h: 36.0,
                id: HitId::InspectorTab(idx),
                clickable: true,
            });
            tx += tw + 8.0;
        }

        let close_x = x + w - 78.0;
        let f12_x = close_x - 122.0;
        let ref_x = f12_x - 86.0;
        for (bx, bw, label, primary, hid) in [
            (ref_x, 78.0, "Refresh", false, HitId::InspectorRefresh),
            (f12_x, 114.0, "F12 DevTools", true, HitId::InspectorDevtools),
            (close_x, 66.0, "Close", false, HitId::InspectorClose),
        ] {
            let hov = point_in(cursor, bx, y + 7.0, bw, 30.0);
            let bg = if primary {
                if hov { pal.cta_hover } else { pal.cta_bg }
            } else if hov {
                pal.btn_hover
            } else {
                pal.canvas
            };
            draw_card(c, bx, y + 7.0, bw, 30.0, 8.0, bg, pal.hairline_strong, 1.0);
            c.draw_text(
                &mut self.fonts,
                FontStyle::SemiBold,
                9.0,
                bx + 12.0,
                y + 15.0,
                if primary { pal.cta_fg } else { pal.ink },
                label,
            );
            hits.push(HitRegion {
                x: bx,
                y: y + 7.0,
                w: bw,
                h: 30.0,
                id: hid,
                clickable: true,
            });
        }

        let body_y = y + 58.0;
        let card_w = ((w - 52.0) / 3.0).max(160.0);
        if self.inspector_tab == 0 {
            let layout_name = if snapshot.sidebar_left {
                "Left Sidebar (Arc Style)"
            } else {
                "Top Horizontal Tabs"
            };
            let rows = [
                (
                    "UI Engine",
                    "100% Native Rust (Softbuffer + ab_glyph)",
                    format!("Frame: {:.2} ms  Â·  Zoom: {}%", self.last_frame_ms, snapshot.zoom_percent),
                ),
                (
                    "Workspace Layout",
                    &*layout_name,
                    format!("Open tabs: {}  Â·  Bookmarks: {}", snapshot.tabs.len(), snapshot.bookmarks.len()),
                ),
                (
                    "Privacy & Tools",
                    if snapshot.adblock_enabled { "Pluto Shield Active" } else { "Pluto Shield Paused" },
                    format!("Zero HTML/JS Chrome  Â·  {}", snapshot.active_url),
                ),
            ];
            for (idx, (k, v, sub)) in rows.iter().enumerate() {
                let cx = x + 16.0 + idx as f32 * (card_w + 10.0);
                draw_card(c, cx, body_y, card_w, h - 74.0, 12.0, pal.surface, pal.hairline, 1.0);
                c.draw_text(&mut self.fonts, FontStyle::SemiBold, 9.0, cx + 16.0, body_y + 16.0, pal.steel, k);
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::Bold, 11.5, cx + 16.0, body_y + 44.0, card_w - 32.0, pal.ink, v);
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::Regular, 9.0, cx + 16.0, body_y + 80.0, card_w - 32.0, pal.slate, sub);
            }
        } else {
            let t = snapshot.telemetry.cloned().unwrap_or_default();
            let rows = [
                ("DOM Complexity", format!("{} nodes", t.dom_nodes), format!("Scripts: {}  Â·  Styles: {}", t.scripts, t.stylesheets)),
                ("Load Timing", format!("DCL {} ms  Â·  Load {} ms", t.dcl_ms, t.load_ms), format!("Images: {}", t.images)),
                ("Origin Storage", if t.origin.is_empty() { "Internal Rust".to_string() } else { t.origin }, format!("Local: {}  Â·  Session: {}", t.local_keys, t.session_keys)),
            ];
            for (idx, (k, v, sub)) in rows.iter().enumerate() {
                let cx = x + 16.0 + idx as f32 * (card_w + 10.0);
                draw_card(c, cx, body_y, card_w, h - 74.0, 12.0, pal.surface, pal.hairline, 1.0);
                c.draw_text(&mut self.fonts, FontStyle::SemiBold, 9.0, cx + 16.0, body_y + 16.0, pal.steel, k);
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::Bold, 11.5, cx + 16.0, body_y + 44.0, card_w - 32.0, pal.ink, &v);
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::Regular, 9.0, cx + 16.0, body_y + 80.0, card_w - 32.0, pal.slate, &sub);
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
        snapshot: &RenderSnapshot<'_>,
        hits: &mut Vec<HitRegion>,
    ) {
        c.fill_rect(vx, vy, vw, vh, pal.canvas);

        let scroll = self.welcome_scroll_y;
        let pad_x = ((vw - 1140.0) * 0.5).max(28.0);
        let cw = (vw - pad_x * 2.0).max(340.0);
        let x0 = vx + pad_x;
        let mut y = vy + 28.0 - scroll;

        // 01 // SUBTLE TOP STATUS PILL STRIP
        draw_card(c, x0, y, cw, 40.0, 12.0, pal.strip, pal.hairline, 1.0);
        c.draw_pluto_logo(x0 + 8.0, y + 6.0, 28.0, 0.2);
        c.draw_text(
            &mut self.fonts,
            FontStyle::SemiBold,
            9.5,
            x0 + 44.0,
            y + 13.0,
            pal.ink,
            "Pluto Browser  Â·  Crafted 100% in Native Rust",
        );
        let tele = format!(
            "Instant Frame  {:.1} ms   Â·   Zoom {}%   Â·   Top Tabs + Arc Sidebar",
            self.last_frame_ms, snapshot.zoom_percent
        );
        let tw = self.fonts.measure_text(FontStyle::Regular, 9.0, &tele);
        if cw > 700.0 {
            c.draw_text(
                &mut self.fonts,
                FontStyle::Regular,
                9.0,
                x0 + cw - tw - 16.0,
                y + 13.0,
                pal.steel,
                &tele,
            );
        }
        y += 56.0;

        // 02 // EDITORIAL APPLE + NOTION SPLIT HERO + LIVE ORBITAL SHOWCASE
        let two_col = cw >= 860.0;
        let left_w = if two_col { cw * 0.56 } else { cw };
        let hero_h = if two_col { 426.0 } else { 378.0 };

        c.fill_soft_shadow(x0, y, left_w, hero_h, 16.0, 8.0);
        draw_card(c, x0, y, left_w, hero_h, 16.0, pal.surface, pal.hairline, 1.0);
        draw_badge(
            c,
            &mut self.fonts,
            x0 + 28.0,
            y + 26.0,
            pal.tint_sand,
            pal.hairline,
            pal.charcoal,
            "Meet Pluto 2.0  Â·  A Calmer, Faster Way to Browse",
        );

        let h_size = if cw >= 980.0 { 29.0 } else { 24.0 };
        let h_step = h_size * 1.46;
        c.draw_text(
            &mut self.fonts,
            FontStyle::Bold,
            h_size,
            x0 + 28.0,
            y + 68.0,
            pal.ink,
            "A breath of fresh air",
        );
        c.draw_text(
            &mut self.fonts,
            FontStyle::Bold,
            h_size,
            x0 + 28.0,
            y + 68.0 + h_step,
            pal.ink,
            "for the modern web.",
        );

        let pitch_y = y + 68.0 + h_step * 2.0 + 18.0;
        let (_, pitch_end_y) = draw_wrapped_text(
            c,
            &mut self.fonts,
            FontStyle::Regular,
            11.0,
            x0 + 28.0,
            pitch_y,
            left_w - 56.0,
            25.0,
            pal.slate,
            "No heavy templates or background clutter. Designed with the calm clarity of Notion and the instant feel of native Apple apps â€” with built-in ad shield, clean Reader Mode, Night Eye, and customizable Arc-style sidebars.",
        );

        let cta_y = (pitch_end_y + 24.0).max(y + hero_h - 74.0);
        let mut cx_cta = x0 + 28.0;
        for (bw, label, primary_cta, hid) in [
            (176.0, "Start Browsing  ->", true, HitId::WelcomeHeroOpen),
            (164.0, "Customize Layout", false, HitId::WelcomeHeroSettings),
            (130.0, "Inspector", false, HitId::WelcomeHeroXray),
        ] {
            if cx_cta + bw > x0 + left_w - 16.0 {
                break;
            }
            let hov = point_in(cursor, cx_cta, cta_y, bw, 44.0);
            draw_card(
                c,
                cx_cta,
                cta_y,
                bw,
                44.0,
                12.0,
                if primary_cta {
                    if hov { pal.cta_hover } else { pal.cta_bg }
                } else if hov {
                    pal.btn_hover
                } else {
                    pal.canvas
                },
                if primary_cta {
                    Color::rgba(0, 0, 0, 0)
                } else if hov {
                    pal.hairline_strong
                } else {
                    pal.hairline
                },
                1.0,
            );
            let lw = self.fonts.measure_text(FontStyle::SemiBold, 10.0, label);
            c.draw_text(
                &mut self.fonts,
                FontStyle::SemiBold,
                10.0,
                cx_cta + (bw - lw) * 0.5,
                cta_y + 14.0,
                if primary_cta { pal.cta_fg } else { pal.ink },
                label,
            );
            hits.push(HitRegion {
                x: cx_cta,
                y: cta_y,
                w: bw,
                h: 44.0,
                id: hid,
                clickable: true,
            });
            cx_cta += bw + 10.0;
        }

        // Right Orbital Celestial Card + Interactive Live Switcher
        if two_col {
            let rx = x0 + left_w + 16.0;
            let rw = cw - left_w - 16.0;
            c.fill_soft_shadow(rx, y, rw, hero_h, 16.0, 8.0);
            draw_card(c, rx, y, rw, hero_h, 16.0, pal.strip, pal.hairline, 1.0);

            c.draw_text(
                &mut self.fonts,
                FontStyle::SemiBold,
                9.5,
                rx + 20.0,
                y + 18.0,
                pal.steel,
                "Pluto & Charon  Â·  60 FPS Rust Canvas",
            );
            let orb_pill = "Live Preview";
            draw_badge(c, &mut self.fonts, rx + rw - 110.0, y + 13.0, pal.canvas, pal.hairline, pal.primary, orb_pill);

            let p_cx = rx + rw * 0.5;
            let p_cy = y + hero_h * 0.5 - 10.0;

            // Subtle concentric planetary orbit rings
            for rad in [66.0, 106.0, 146.0] {
                c.stroke_circle(p_cx, p_cy, rad, 1.0, Color::rgba(pal.hairline_strong.r, pal.hairline_strong.g, pal.hairline_strong.b, 52));
            }
            let px_off = ((cursor.0 - p_cx) * 0.045).clamp(-10.0, 10.0);
            let py_off = ((cursor.1 - p_cy) * 0.045).clamp(-10.0, 10.0);
            c.draw_pluto_hero_system(p_cx, p_cy, time_sec, px_off, py_off);

            // Live 1-Click Layout & Appearance Try-On Pills inside the showcase card
            let bar_y = y + hero_h - 62.0;
            c.fill_rect(rx + 16.0, bar_y - 10.0, rw - 32.0, 1.0, pal.hairline);
            let sw_w = ((rw - 48.0) * 0.5).max(120.0);

            let layout_lbl = if snapshot.sidebar_left {
                "Layout: Arc Sidebar"
            } else {
                "Layout: Top Tabs"
            };
            let lay_hov = point_in(cursor, rx + 18.0, bar_y, sw_w, 38.0);
            draw_card(
                c,
                rx + 18.0,
                bar_y,
                sw_w,
                38.0,
                10.0,
                if lay_hov { pal.canvas } else { pal.surface },
                if lay_hov { pal.hairline_strong } else { pal.hairline },
                1.0,
            );
            c.draw_icon_sidebar(rx + 36.0, bar_y + 19.0, pal.ink);
            c.draw_text_ellipsis(
                &mut self.fonts,
                FontStyle::SemiBold,
                9.5,
                rx + 50.0,
                bar_y + 12.0,
                sw_w - 40.0,
                pal.ink,
                layout_lbl,
            );
            hits.push(HitRegion {
                x: rx + 18.0,
                y: bar_y,
                w: sw_w,
                h: 38.0,
                id: HitId::ToggleSidebarLayout,
                clickable: true,
            });

            let thm_x = rx + 28.0 + sw_w;
            let thm_lbl = if self.dark_mode {
                "Theme: Midnight"
            } else {
                "Theme: Daylight"
            };
            let thm_hov = point_in(cursor, thm_x, bar_y, sw_w, 38.0);
            draw_card(
                c,
                thm_x,
                bar_y,
                sw_w,
                38.0,
                10.0,
                if thm_hov { pal.canvas } else { pal.surface },
                if thm_hov { pal.hairline_strong } else { pal.hairline },
                1.0,
            );
            c.draw_icon_theme(thm_x + 18.0, bar_y + 19.0, self.dark_mode, pal.ink);
            c.draw_text_ellipsis(
                &mut self.fonts,
                FontStyle::SemiBold,
                9.5,
                thm_x + 32.0,
                bar_y + 12.0,
                sw_w - 40.0,
                pal.ink,
                thm_lbl,
            );
            hits.push(HitRegion {
                x: thm_x,
                y: bar_y,
                w: sw_w,
                h: 38.0,
                id: HitId::ThemeToggle,
                clickable: true,
            });
        }
        y += hero_h + 34.0;

        // 03 // USER-FRIENDLY BENCHMARKS (PLAIN BENEFIT STORY, ZERO JARGON)
        c.draw_text(&mut self.fonts, FontStyle::Bold, 15.0, x0, y, pal.ink, "Why Pluto feels effortlessly faster");
        y += 28.0;
        let bcols = if cw >= 780.0 { 4 } else { 2 };
        let b_gap = 12.0;
        let bcell_w = (cw - b_gap * (bcols - 1) as f32) / bcols as f32;
        let bcell_h = 148.0;
        let metrics = [
            ("STARTUP SPEED", "0.05s", "Instant Launch", "Opens as fast as a note app â€” up to 8x faster than Chrome.", pal.tint_mint, pal.brand_green, 0.94_f32),
            ("APP FOOTPRINT", "1.4 MB", "99% Smaller", "Tiny native app instead of 250+ MB installers and updaters.", pal.tint_sky, pal.primary, 0.98_f32),
            ("MEMORY USAGE", "80% Less", "Leaves RAM Free", "Runs around 18 MB so coding and design tools stay fast.", pal.tint_sand, pal.brand_amber, 0.84_f32),
            ("PRIVACY & FOCUS", "Built-in", "Zero Extensions", "Native Ad Shield, Safari-clean Reader, and Web Night Eye.", pal.tint_rose, pal.ink, 0.92_f32),
        ];
        for (i, (k, val, badge_txt, desc, tint, accent, bar_fill)) in metrics.iter().enumerate() {
            let col = i % bcols;
            let row = i / bcols;
            let bx = x0 + col as f32 * (bcell_w + b_gap);
            let by = y + row as f32 * (bcell_h + b_gap);
            draw_card(c, bx, by, bcell_w, bcell_h, 14.0, pal.surface, pal.hairline, 1.0);
            draw_badge(c, &mut self.fonts, bx + 16.0, by + 16.0, *tint, Color::rgba(0, 0, 0, 0), *accent, badge_txt);
            c.draw_text(&mut self.fonts, FontStyle::Bold, 20.0, bx + 16.0, by + 48.0, pal.ink, val);
            // Animated smooth meter bar
            let anim = ((time_sec * 1.4 - i as f32 * 0.15).clamp(0.0, 1.0)).powf(0.5);
            let track_w = bcell_w - 32.0;
            c.fill_rounded_rect(bx + 16.0, by + 84.0, track_w, 6.0, 3.0, pal.hairline);
            c.fill_rounded_rect(bx + 16.0, by + 84.0, (track_w * *bar_fill * anim).max(8.0), 6.0, 3.0, *accent);
            draw_wrapped_text(
                c,
                &mut self.fonts,
                FontStyle::Regular,
                8.8,
                bx + 16.0,
                by + 98.0,
                track_w,
                17.0,
                pal.slate,
                desc,
            );
            let _ = k;
        }
        y += ((metrics.len() + bcols - 1) / bcols) as f32 * (bcell_h + b_gap) + 24.0;

        // 04 // PLUTO VS MODERN BROWSERS (FRIENDLY COMPARISON BENTO)
        c.draw_text(&mut self.fonts, FontStyle::Bold, 15.0, x0, y, pal.ink, "How Pluto compares");
        y += 28.0;
        let table_rows = [
            ["Experience", "Pluto Browser", "Google Chrome", "Arc Browser", "Brave"],
            ["Startup Feel", "Instant (~0.05s)", "Heavy (~0.5s+)", "Moderate (~0.4s)", "Moderate (~0.4s)"],
            ["Idle Laptop Battery & RAM", "Featherlight (~18 MB)", "High (~260+ MB)", "Very High (~440+ MB)", "Medium (~230+ MB)"],
            ["Navigation Choices", "Top Bar OR Arc Sidebar", "Top Tabs Only", "Sidebar Only", "Top or Vertical"],
            ["Reading & Focus Tools", "Reader + Night Eye Included", "Needs Extensions", "Boosts + Ext.", "Shields Only"],
            ["Tracker-Clean Link Sharing", "Auto-Cleaned Markdown & URL", "Manual Edit", "Standard Copy", "Clean Links Menu"],
        ];
        let row_h = 42.0;
        let table_h = table_rows.len() as f32 * row_h;
        c.fill_soft_shadow(x0, y, cw, table_h, 14.0, 6.0);
        draw_card(c, x0, y, cw, table_h, 14.0, pal.canvas, pal.hairline, 1.0);
        let col_w = cw / 5.0;
        for (r_idx, row) in table_rows.iter().enumerate() {
            let ry = y + r_idx as f32 * row_h;
            if r_idx > 0 {
                c.fill_rect(x0 + 1.0, ry, cw - 2.0, 1.0, pal.hairline);
            }
            if r_idx == 0 {
                c.fill_rounded_rect(x0 + 1.0, ry + 1.0, cw - 2.0, row_h - 1.0, 13.0, pal.strip);
            }
            for (c_idx, cell) in row.iter().enumerate() {
                let cx = x0 + c_idx as f32 * col_w;
                if c_idx == 1 {
                    c.fill_rect(cx + 2.0, ry + 1.0, col_w - 4.0, row_h - 2.0, pal.tint_sky);
                }
                let fg = if r_idx == 0 {
                    if c_idx == 1 { pal.primary } else { pal.steel }
                } else if c_idx == 1 || c_idx == 0 {
                    pal.ink
                } else {
                    pal.slate
                };
                c.draw_text_ellipsis(
                    &mut self.fonts,
                    if c_idx == 1 || r_idx == 0 { FontStyle::SemiBold } else { FontStyle::Regular },
                    9.5,
                    cx + 14.0,
                    ry + 14.0,
                    col_w - 22.0,
                    fg,
                    cell,
                );
            }
        }
        draw_card(c, x0, y, cw, table_h, 14.0, Color::rgba(0, 0, 0, 0), pal.hairline, 1.0);
        y += table_h + 30.0;

        // 05 // INTERACTIVE FEATURE TOUR DECK (3 PILL TABS)
        c.draw_text(&mut self.fonts, FontStyle::Bold, 15.0, x0, y, pal.ink, "Built-in tools for everyday flow");
        y += 28.0;
        let deck_h = 204.0;
        draw_card(c, x0, y, cw, deck_h, 16.0, pal.surface, pal.hairline, 1.0);

        let d_tabs = [
            "01 Â· Workspaces & Sidebars",
            "02 Â· Reader, Shield & Night Eye",
            "03 Â· Essential Shortcuts",
        ];
        let mut dx = x0 + 14.0;
        let dy = y + 12.0;
        let tab_w = ((cw - 44.0) / 3.0).max(120.0);
        for (idx, lbl) in d_tabs.iter().enumerate() {
            let active = self.welcome_mock_tab == idx;
            let hov = point_in(cursor, dx, dy, tab_w, 34.0);
            draw_card(
                c,
                dx,
                dy,
                tab_w,
                34.0,
                9.0,
                if active { pal.cta_bg } else if hov { pal.btn_hover } else { pal.canvas },
                if active { Color::rgba(0, 0, 0, 0) } else { pal.hairline },
                1.0,
            );
            c.draw_text_ellipsis(
                &mut self.fonts,
                FontStyle::SemiBold,
                9.5,
                dx + 14.0,
                dy + 10.0,
                tab_w - 24.0,
                if active { pal.cta_fg } else { pal.ink },
                lbl,
            );
            hits.push(HitRegion {
                x: dx,
                y: dy,
                w: tab_w,
                h: 34.0,
                id: HitId::WelcomeMockTab(idx),
                clickable: true,
            });
            dx += tab_w + 8.0;
        }
        c.fill_rect(x0 + 14.0, y + 56.0, cw - 28.0, 1.0, pal.hairline);

        let deck_body_y = y + 72.0;
        let deck_items: &[(&str, &str)] = match self.welcome_mock_tab {
            0 => &[
                ("Switch Layout Anytime", "Prefer classic horizontal tabs or an Arc-style vertical sidebar? Flip between them in one click from Settings."),
                ("Quick Notes & Favorites", "Every New Tab gives you fast pinned links and an auto-saving Workspace Scratchpad right beside your bookmarks."),
                ("Command Palette (Ctrl+K)", "Jump to any open tab, bookmark, layout toggle, or setting without lifting your hands off the keyboard."),
            ],
            1 => &[
                ("Pluto Shield Built In", "Automatically hides intrusive ad slots and cookie banners out of the box without slowing pages down."),
                ("Reader & Night Eye", "Press Alt+R for a calm Apple/Notion article layout, or Alt+N to turn bright white websites dark at night."),
                ("Clean Link Sharing", "Copy URLs or formatted Markdown links via Ctrl+Shift+Y with hidden tracking parameters stripped automatically."),
            ],
            _ => &[
                ("Fast Navigation", "Ctrl+L Address Bar   Â·   Ctrl+T New Tab   Â·   Ctrl+W Close Tab   Â·   Ctrl+Shift+T Reopen Closed"),
                ("Sidebar & Appearance", "Ctrl+, Settings   Â·   Ctrl+B Collapse Sidebar   Â·   Ctrl+Shift+L Toggle Top / Arc Layout"),
                ("Smart Web Tools", "Alt+R Reader Mode   Â·   Alt+N Web Night Eye   Â·   Alt+M Mute Audio   Â·   Ctrl+Shift+Y Copy Markdown"),
            ],
        };
        let dcol_w = cw / 3.0;
        for (i, (title, desc)) in deck_items.iter().enumerate() {
            let cx = x0 + i as f32 * dcol_w;
            if i > 0 {
                c.fill_rect(cx, y + 66.0, 1.0, deck_h - 82.0, pal.hairline);
            }
            c.draw_text_ellipsis(&mut self.fonts, FontStyle::Bold, 11.5, cx + 22.0, deck_body_y, dcol_w - 40.0, pal.ink, title);
            draw_wrapped_text(
                c,
                &mut self.fonts,
                FontStyle::Regular,
                10.0,
                cx + 22.0,
                deck_body_y + 30.0,
                dcol_w - 44.0,
                21.0,
                pal.slate,
                desc,
            );
        }
        y += deck_h + 26.0;

        // 06 // QUICK LAUNCH BAR
        draw_card(c, x0, y, cw, 112.0, 16.0, pal.strip, pal.hairline, 1.0);
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 10.0, x0 + 22.0, y + 18.0, pal.steel, "Choose where to begin");

        let mut px_d = x0 + 22.0;
        let py_d = y + 50.0;
        for (idx, (label, _)) in DESTINATIONS.iter().enumerate() {
            let sel = self.welcome_dest_idx == idx;
            let bw = self.fonts.measure_text(FontStyle::SemiBold, 9.5, label) + 28.0;
            let hov = point_in(cursor, px_d, py_d, bw, 38.0);
            draw_card(
                c,
                px_d,
                py_d,
                bw,
                38.0,
                10.0,
                if sel { pal.cta_bg } else if hov { pal.btn_hover } else { pal.canvas },
                if sel { Color::rgba(0, 0, 0, 0) } else if hov { pal.hairline_strong } else { pal.hairline },
                1.0,
            );
            c.draw_text(
                &mut self.fonts,
                FontStyle::SemiBold,
                9.5,
                px_d + 14.0,
                py_d + 12.0,
                if sel { pal.cta_fg } else { pal.ink },
                label,
            );
            hits.push(HitRegion {
                x: px_d,
                y: py_d,
                w: bw,
                h: 38.0,
                id: HitId::WelcomeDestPill(idx),
                clickable: true,
            });
            px_d += bw + 8.0;
        }

        let fin_w = 210.0;
        let fin_x = (x0 + cw - fin_w - 22.0).max(px_d + 8.0);
        let fin_hov = point_in(cursor, fin_x, py_d - 3.0, fin_w, 44.0);
        draw_card(
            c,
            fin_x,
            py_d - 3.0,
            fin_w,
            44.0,
            12.0,
            if fin_hov { pal.cta_hover } else { pal.cta_bg },
            Color::rgba(0, 0, 0, 0),
            0.0,
        );
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 10.5, fin_x + 26.0, py_d + 11.0, pal.cta_fg, "Open Workspace  ->");
        hits.push(HitRegion {
            x: fin_x,
            y: py_d - 3.0,
            w: fin_w,
            h: 44.0,
            id: HitId::WelcomeFinish,
            clickable: true,
        });

        y += 144.0;
        let total_h = (y + scroll) - vy;
        self.welcome_max_scroll = (total_h - vh).max(0.0);
        if self.welcome_scroll_y > self.welcome_max_scroll {
            self.welcome_scroll_y = self.welcome_max_scroll;
        }
        if self.welcome_max_scroll > 4.0 {
            draw_scrollbar(c, pal, vx + vw - 9.0, vy + 6.0, vh - 12.0, scroll, self.welcome_max_scroll, total_h);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_settings_page(
        &mut self,
        c: &mut Canvas<'_>,
        pal: &ThemePalette,
        cursor: (f32, f32),
        vx: f32,
        vy: f32,
        vw: f32,
        vh: f32,
        snapshot: &RenderSnapshot<'_>,
        hits: &mut Vec<HitRegion>,
    ) {
        c.fill_rect(vx, vy, vw, vh, pal.canvas);

        let scroll = self.settings_scroll_y;
        let pad_x = ((vw - 1040.0) * 0.5).max(28.0);
        let cw = (vw - pad_x * 2.0).max(340.0);
        let x0 = vx + pad_x;
        let mut y = vy + 28.0 - scroll;

        // Settings Header Card
        c.fill_soft_shadow(x0, y, cw, 88.0, 16.0, 6.0);
        draw_card(c, x0, y, cw, 88.0, 16.0, pal.strip, pal.hairline, 1.0);
        draw_badge(
            c,
            &mut self.fonts,
            x0 + 22.0,
            y + 16.0,
            pal.tint_sky,
            pal.hairline,
            pal.primary,
            "pluto://settings",
        );
        c.draw_text(
            &mut self.fonts,
            FontStyle::Bold,
            20.0,
            x0 + 22.0,
            y + 44.0,
            pal.ink,
            "Workspace & Appearance",
        );
        y += 112.0;

        // Section 01: Navigation Layout (Top Bar vs Arc Left Sidebar)
        c.draw_text(
            &mut self.fonts,
            FontStyle::Bold,
            13.5,
            x0,
            y,
            pal.ink,
            "Navigation Layout",
        );
        y += 28.0;

        let card_w = (cw - 14.0) * 0.5;
        let card_h = 176.0;

        // Card A: Classic Top Horizontal Bar
        let top_sel = !snapshot.sidebar_left;
        let top_hov = point_in(cursor, x0, y, card_w, card_h);
        draw_card(
            c,
            x0,
            y,
            card_w,
            card_h,
            14.0,
            if top_sel { pal.tint_sky } else if top_hov { pal.btn_hover } else { pal.surface },
            if top_sel { pal.primary } else if top_hov { pal.hairline_strong } else { pal.hairline },
            if top_sel { 1.5 } else { 1.0 },
        );
        // Clean Apple/Safari schematic: Top Bar
        draw_card(c, x0 + 20.0, y + 20.0, 132.0, 68.0, 10.0, pal.canvas, pal.hairline_strong, 1.0);
        c.fill_rounded_rect(x0 + 22.0, y + 22.0, 128.0, 16.0, 6.0, if top_sel { pal.primary } else { pal.steel });
        c.fill_rect(x0 + 22.0, y + 40.0, 128.0, 10.0, pal.strip);
        c.draw_text(&mut self.fonts, FontStyle::Bold, 12.5, x0 + 168.0, y + 20.0, pal.ink, "Classic Top Tabs");
        draw_wrapped_text(
            c,
            &mut self.fonts,
            FontStyle::Regular,
            9.5,
            x0 + 168.0,
            y + 46.0,
            card_w - 186.0,
            20.0,
            pal.slate,
            "Safari & Chrome inspired strip. Full-width canvas with rounded tabs arranged above the search bar.",
        );
        draw_badge(
            c,
            &mut self.fonts,
            x0 + 20.0,
            y + card_h - 38.0,
            if top_sel { pal.cta_bg } else { pal.canvas },
            pal.hairline,
            if top_sel { pal.cta_fg } else { pal.steel },
            if top_sel { "Active Layout" } else { "Use Top Tabs" },
        );
        hits.push(HitRegion {
            x: x0,
            y,
            w: card_w,
            h: card_h,
            id: HitId::SettingsLayoutTop,
            clickable: true,
        });

        // Card B: Arc Left Vertical Sidebar
        let rx = x0 + card_w + 14.0;
        let arc_sel = snapshot.sidebar_left;
        let arc_hov = point_in(cursor, rx, y, card_w, card_h);
        draw_card(
            c,
            rx,
            y,
            card_w,
            card_h,
            14.0,
            if arc_sel { pal.tint_sky } else if arc_hov { pal.btn_hover } else { pal.surface },
            if arc_sel { pal.primary } else if arc_hov { pal.hairline_strong } else { pal.hairline },
            if arc_sel { 1.5 } else { 1.0 },
        );
        draw_card(c, rx + 20.0, y + 20.0, 132.0, 68.0, 10.0, pal.canvas, pal.hairline_strong, 1.0);
        c.fill_rounded_rect(rx + 22.0, y + 22.0, 36.0, 64.0, 6.0, if arc_sel { pal.primary } else { pal.steel });
        c.fill_rect(rx + 60.0, y + 22.0, 90.0, 12.0, pal.strip);
        c.draw_text(&mut self.fonts, FontStyle::Bold, 12.5, rx + 168.0, y + 20.0, pal.ink, "Arc Left Sidebar");
        draw_wrapped_text(
            c,
            &mut self.fonts,
            FontStyle::Regular,
            9.5,
            rx + 168.0,
            y + 46.0,
            card_w - 186.0,
            20.0,
            pal.slate,
            "Notion + Arc style vertical drawer with Favorites grid, vertical tabs list, and compact single-row toolbar.",
        );
        draw_badge(
            c,
            &mut self.fonts,
            rx + 20.0,
            y + card_h - 38.0,
            if arc_sel { pal.cta_bg } else { pal.canvas },
            pal.hairline,
            if arc_sel { pal.cta_fg } else { pal.steel },
            if arc_sel { "Active Layout" } else { "Use Arc Sidebar" },
        );
        hits.push(HitRegion {
            x: rx,
            y,
            w: card_w,
            h: card_h,
            id: HitId::SettingsLayoutLeft,
            clickable: true,
        });
        y += card_h + 14.0;

        // Toggles Row: Sidebar Collapse + Bookmarks Bar
        let row_bw = (cw - 14.0) * 0.5;
        let col_hov = point_in(cursor, x0, y, row_bw, 52.0);
        draw_card(
            c,
            x0,
            y,
            row_bw,
            52.0,
            12.0,
            if col_hov { pal.btn_hover } else { pal.surface },
            pal.hairline,
            1.0,
        );
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 10.5, x0 + 18.0, y + 17.0, pal.ink, "Collapse Sidebar (Ctrl+B)");
        draw_badge(
            c,
            &mut self.fonts,
            x0 + row_bw - 104.0,
            y + 14.0,
            pal.canvas,
            pal.hairline,
            pal.ink,
            if snapshot.sidebar_collapsed { "Collapsed" } else { "Expanded" },
        );
        hits.push(HitRegion {
            x: x0,
            y,
            w: row_bw,
            h: 52.0,
            id: HitId::SettingsSidebarCollapse,
            clickable: true,
        });

        let bm_hov = point_in(cursor, rx, y, row_bw, 52.0);
        draw_card(
            c,
            rx,
            y,
            row_bw,
            52.0,
            12.0,
            if bm_hov { pal.btn_hover } else { pal.surface },
            pal.hairline,
            1.0,
        );
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 10.5, rx + 18.0, y + 17.0, pal.ink, "Bookmarks Bar (Ctrl+Shift+B)");
        draw_badge(
            c,
            &mut self.fonts,
            rx + row_bw - 92.0,
            y + 14.0,
            if snapshot.show_bookmarks_bar { pal.tint_mint } else { pal.canvas },
            pal.hairline,
            if snapshot.show_bookmarks_bar { pal.brand_green } else { pal.steel },
            if snapshot.show_bookmarks_bar { "Visible" } else { "Hidden" },
        );
        hits.push(HitRegion {
            x: rx,
            y,
            w: row_bw,
            h: 52.0,
            id: HitId::SettingsBookmarksToggle,
            clickable: true,
        });
        y += 78.0;

        // Section 02: Appearance & Zoom Scale
        c.draw_text(&mut self.fonts, FontStyle::Bold, 13.5, x0, y, pal.ink, "Appearance & Page Zoom");
        y += 28.0;

        for (tx, label, is_dark_opt, active, hid) in [
            (rx, "Apple Daylight (Notion White)", false, !self.dark_mode, HitId::SettingsThemeLight),
            (x0, "Cupertino Midnight (Dark)", true, self.dark_mode, HitId::SettingsThemeDark),
        ] {
            let hov = point_in(cursor, tx, y, row_bw, 60.0);
            draw_card(
                c,
                tx,
                y,
                row_bw,
                60.0,
                12.0,
                if active { pal.tint_sky } else if hov { pal.btn_hover } else { pal.surface },
                if active { pal.primary } else { pal.hairline },
                if active { 1.5 } else { 1.0 },
            );
            let swatch = if is_dark_opt { Color::rgb(0x16, 0x16, 0x18) } else { Color::rgb(0xff, 0xff, 0xff) };
            draw_card(c, tx + 18.0, y + 15.0, 30.0, 30.0, 9.0, swatch, pal.hairline_strong, 1.0);
            c.draw_text(&mut self.fonts, FontStyle::SemiBold, 10.5, tx + 60.0, y + 21.0, pal.ink, label);
            hits.push(HitRegion {
                x: tx,
                y,
                w: row_bw,
                h: 60.0,
                id: hid,
                clickable: true,
            });
        }
        y += 72.0;

        draw_card(c, x0, y, cw, 66.0, 12.0, pal.surface, pal.hairline, 1.0);
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 10.5, x0 + 18.0, y + 23.0, pal.ink, "Active Tab Zoom Scale:");
        let presets: &[u32] = &[80, 90, 100, 110, 125, 150, 175];
        let mut zx = x0 + 214.0;
        for &pct in presets {
            if zx + 58.0 > x0 + cw - 12.0 {
                break;
            }
            let sel = snapshot.zoom_percent == pct;
            let hov = point_in(cursor, zx, y + 15.0, 56.0, 36.0);
            draw_card(
                c,
                zx,
                y + 15.0,
                56.0,
                36.0,
                9.0,
                if sel { pal.cta_bg } else if hov { pal.btn_hover } else { pal.canvas },
                if sel { Color::rgba(0, 0, 0, 0) } else { pal.hairline },
                1.0,
            );
            let lbl = format!("{pct}%");
            c.draw_text(
                &mut self.fonts,
                FontStyle::SemiBold,
                9.5,
                zx + 11.0,
                y + 25.0,
                if sel { pal.cta_fg } else { pal.ink },
                &lbl,
            );
            hits.push(HitRegion {
                x: zx,
                y: y + 15.0,
                w: 56.0,
                h: 36.0,
                id: HitId::SettingsZoomPreset(pct),
                clickable: true,
            });
            zx += 62.0;
        }
        y += 92.0;

        // Section 03: Smart Web Privacy & Quick Actions
        c.draw_text(&mut self.fonts, FontStyle::Bold, 13.5, x0, y, pal.ink, "Privacy, Shield & Tools");
        y += 28.0;

        let sh_hov = point_in(cursor, x0, y, cw, 64.0);
        draw_card(
            c,
            x0,
            y,
            cw,
            64.0,
            14.0,
            if sh_hov { pal.btn_hover } else { pal.surface },
            pal.hairline,
            1.0,
        );
        c.draw_text(&mut self.fonts, FontStyle::Bold, 11.5, x0 + 20.0, y + 14.0, pal.ink, "Pluto Shield (Built-in Ad & Consent Blocker)");
        c.draw_text(&mut self.fonts, FontStyle::Regular, 9.5, x0 + 20.0, y + 36.0, pal.slate, "Automatically hides common ad slots, sponsored cards, and cookie popups on external websites.");
        draw_badge(
            c,
            &mut self.fonts,
            x0 + cw - 102.0,
            y + 20.0,
            if snapshot.adblock_enabled { pal.tint_mint } else { pal.canvas },
            pal.hairline,
            if snapshot.adblock_enabled { pal.brand_green } else { pal.steel },
            if snapshot.adblock_enabled { "Enabled" } else { "Paused" },
        );
        hits.push(HitRegion {
            x: x0,
            y,
            w: cw,
            h: 64.0,
            id: HitId::SettingsShieldToggle,
            clickable: true,
        });
        y += 76.0;

        let abw = (cw - 20.0) / 3.0;
        for (i, (label, hid)) in [
            ("Clear Browsing History", HitId::SettingsClearHistory),
            ("Replay Welcome Showcase", HitId::SettingsResetOnboarding),
            ("Open Runtime Inspector", HitId::SettingsOpenInspector),
        ]
        .iter()
        .enumerate()
        {
            let bx = x0 + i as f32 * (abw + 10.0);
            let hov = point_in(cursor, bx, y, abw, 44.0);
            draw_card(
                c,
                bx,
                y,
                abw,
                44.0,
                11.0,
                if hov { pal.btn_hover } else { pal.surface },
                if hov { pal.hairline_strong } else { pal.hairline },
                1.0,
            );
            c.draw_text_ellipsis(&mut self.fonts, FontStyle::SemiBold, 9.5, bx + 16.0, y + 14.0, abw - 28.0, pal.ink, label);
            hits.push(HitRegion {
                x: bx,
                y,
                w: abw,
                h: 44.0,
                id: hid.clone(),
                clickable: true,
            });
        }

        y += 72.0;
        let total_h = (y + scroll) - vy;
        self.settings_max_scroll = (total_h - vh).max(0.0);
        if self.settings_scroll_y > self.settings_max_scroll {
            self.settings_scroll_y = self.settings_max_scroll;
        }
        if self.settings_max_scroll > 4.0 {
            draw_scrollbar(c, pal, vx + vw - 9.0, vy + 6.0, vh - 12.0, scroll, self.settings_max_scroll, total_h);
        }
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
        snapshot: &RenderSnapshot<'_>,
        hits: &mut Vec<HitRegion>,
    ) {
        c.fill_rect(vx, vy, vw, vh, pal.canvas);

        let scroll = self.newtab_scroll_y;
        let pad_x = ((vw - 1080.0) * 0.5).max(28.0);
        let cw = (vw - pad_x * 2.0).max(340.0);
        let x0 = vx + pad_x;
        let mut y = vy + 28.0 - scroll;

        // Top Apple + Notion Greeting Header
        c.draw_pluto_logo(x0, y, 44.0, 0.25 + (time_sec * 0.7).sin().abs() * 0.18);
        c.draw_text(&mut self.fonts, FontStyle::Bold, 17.5, x0 + 58.0, y + 3.0, pal.ink, "Pluto Workspace");
        c.draw_text(&mut self.fonts, FontStyle::Regular, 9.5, x0 + 58.0, y + 27.0, pal.steel, "100% Native Rust  Â·  Google Search & Tracker-Clean Links");

        let set_btn_x = x0 + cw - 110.0;
        let wlc_btn_x = set_btn_x - 134.0;
        for (bx, bw, lbl, hid) in [
            (wlc_btn_x, 124.0, "Welcome Tour", HitId::NewTabWelcomeBtn),
            (set_btn_x, 110.0, "Settings", HitId::NewTabSettingsBtn),
        ] {
            let hov = point_in(cursor, bx, y + 4.0, bw, 36.0);
            draw_card(c, bx, y + 4.0, bw, 36.0, 10.0, if hov { pal.btn_hover } else { pal.surface }, if hov { pal.hairline_strong } else { pal.hairline }, 1.0);
            c.draw_text(&mut self.fonts, FontStyle::SemiBold, 9.5, bx + 16.0, y + 14.0, pal.ink, lbl);
            hits.push(HitRegion {
                x: bx,
                y: y + 4.0,
                w: bw,
                h: 36.0,
                id: hid,
                clickable: true,
            });
        }
        y += 66.0;

        // Hero Apple/Notion Search Pill Bar
        let search_h = 56.0;
        let s_focused = self.active_input == ActiveInput::NewTabSearch;
        c.fill_soft_shadow(x0, y, cw, search_h, 14.0, 6.0);
        draw_card(
            c,
            x0,
            y,
            cw,
            search_h,
            14.0,
            pal.surface,
            if s_focused { pal.primary } else { pal.hairline },
            if s_focused { 1.5 } else { 1.0 },
        );
        c.draw_icon_search(x0 + 24.0, y + search_h * 0.5, pal.steel);
        hits.push(HitRegion {
            x: x0,
            y,
            w: cw - 146.0,
            h: search_h,
            id: HitId::NewTabSearchInput,
            clickable: true,
        });

        let sb_x = x0 + cw - 138.0;
        let sb_hov = point_in(cursor, sb_x, y + 7.0, 130.0, search_h - 14.0);
        draw_card(
            c,
            sb_x,
            y + 7.0,
            130.0,
            search_h - 14.0,
            10.0,
            if sb_hov { pal.cta_hover } else { pal.cta_bg },
            Color::rgba(0, 0, 0, 0),
            0.0,
        );
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 10.0, sb_x + 20.0, y + 20.0, pal.cta_fg, "Search Web");
        hits.push(HitRegion {
            x: sb_x,
            y: y + 7.0,
            w: 130.0,
            h: search_h - 14.0,
            id: HitId::NewTabSearchSubmit,
            clickable: true,
        });

        let st_x = x0 + 48.0;
        if self.newtab_search_text.is_empty() {
            c.draw_text(&mut self.fonts, FontStyle::Regular, 11.5, st_x, y + 19.0, pal.steel, "Search Google or type any website URL...");
            if s_focused && (time_sec * 2.0) as u32 % 2 == 0 {
                c.fill_rect(st_x, y + 14.0, 2.0, 28.0, pal.primary);
            }
        } else {
            let shown = tail_fitting_width(&self.fonts, FontStyle::Regular, 11.5, &self.newtab_search_text, cw - 200.0).to_string();
            let tw = self.fonts.measure_text(FontStyle::Regular, 11.5, &shown);
            c.draw_text(&mut self.fonts, FontStyle::Regular, 11.5, st_x, y + 19.0, pal.ink, &shown);
            if s_focused && (time_sec * 2.0) as u32 % 2 == 0 {
                c.fill_rect(st_x + tw + 2.0, y + 14.0, 2.0, 28.0, pal.primary);
            }
        }
        y += search_h + 26.0;

        // Category Filter Pills
        let mut cat_x = x0;
        for (idx, lbl) in ["All Sites", "Engineering", "Reference"].iter().enumerate() {
            let bw = self.fonts.measure_text(FontStyle::SemiBold, 9.5, lbl) + 28.0;
            let sel = self.newtab_category == idx;
            let hov = point_in(cursor, cat_x, y, bw, 32.0);
            draw_card(
                c,
                cat_x,
                y,
                bw,
                32.0,
                16.0,
                if sel { pal.cta_bg } else if hov { pal.btn_hover } else { pal.surface },
                if sel { Color::rgba(0, 0, 0, 0) } else { pal.hairline },
                1.0,
            );
            c.draw_text(&mut self.fonts, FontStyle::SemiBold, 9.5, cat_x + 14.0, y + 9.0, if sel { pal.cta_fg } else { pal.ink }, lbl);
            hits.push(HitRegion {
                x: cat_x,
                y,
                w: bw,
                h: 32.0,
                id: HitId::NewTabCategory(idx),
                clickable: true,
            });
            cat_x += bw + 8.0;
        }
        y += 46.0;

        let filtered: Vec<&QuickLink> = QUICK_LINKS
            .iter()
            .filter(|q| self.newtab_category == 0 || q.cat == self.newtab_category)
            .collect();
        let qcols = if cw >= 780.0 { 3 } else { 2 };
        let qtile_w = (cw - (qcols - 1) as f32 * 12.0) / qcols as f32;
        let qtile_h = 78.0;
        let tints = [
            pal.tint_sky,
            pal.tint_mint,
            pal.tint_sand,
            pal.tint_sky,
            pal.tint_rose,
            pal.tint_sand,
        ];
        for (i, q) in filtered.iter().enumerate() {
            let col = i % qcols;
            let row = i / qcols;
            let qx = x0 + col as f32 * (qtile_w + 12.0);
            let qy = y + row as f32 * (qtile_h + 12.0);
            let hov = point_in(cursor, qx, qy, qtile_w, qtile_h);
            draw_card(
                c,
                qx,
                qy,
                qtile_w,
                qtile_h,
                14.0,
                if hov { pal.surface_soft } else { pal.surface },
                if hov { pal.hairline_strong } else { pal.hairline },
                1.0,
            );
            draw_card(c, qx + 14.0, qy + 17.0, 44.0, 44.0, 11.0, tints[i % tints.len()], Color::rgba(0, 0, 0, 0), 0.0);
            c.draw_text(&mut self.fonts, FontStyle::Bold, 11.5, qx + 24.0, qy + 29.0, pal.ink, q.badge);
            c.draw_text_ellipsis(&mut self.fonts, FontStyle::SemiBold, 11.0, qx + 70.0, qy + 19.0, qtile_w - 84.0, pal.ink, q.title);
            c.draw_text_ellipsis(&mut self.fonts, FontStyle::Regular, 8.8, qx + 70.0, qy + 43.0, qtile_w - 84.0, pal.steel, q.url);
            hits.push(HitRegion {
                x: qx,
                y: qy,
                w: qtile_w,
                h: qtile_h,
                id: HitId::OpenUrlTile(q.url.to_string()),
                clickable: true,
            });
        }
        y += ((filtered.len() + qcols - 1) / qcols) as f32 * (qtile_h + 12.0) + 18.0;

        // Two-Column Lower Notion Workspace: Bookmarks + History vs Native Scratchpad
        let col_w = (cw - 14.0) * 0.5;
        let panel_h = 236.0;

        // Left: Saved Bookmarks & History
        draw_card(c, x0, y, col_w, panel_h, 14.0, pal.surface, pal.hairline, 1.0);
        c.fill_rect(x0 + 16.0, y + 42.0, col_w - 32.0, 1.0, pal.hairline);
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 10.0, x0 + 18.0, y + 14.0, pal.ink, "Bookmarks & Recent History");

        let clr_x = x0 + col_w - 74.0;
        let clr_hov = point_in(cursor, clr_x, y + 8.0, 58.0, 26.0);
        draw_card(c, clr_x, y + 8.0, 58.0, 26.0, 7.0, if clr_hov { pal.btn_hover } else { pal.canvas }, pal.hairline, 1.0);
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 8.5, clr_x + 13.0, y + 14.0, pal.steel, "Clear");
        hits.push(HitRegion {
            x: clr_x,
            y: y + 8.0,
            w: 58.0,
            h: 26.0,
            id: HitId::ClearHistory,
            clickable: true,
        });

        let mut ly = y + 52.0;
        if snapshot.bookmarks.is_empty() && snapshot.history.is_empty() {
            c.draw_text(&mut self.fonts, FontStyle::Regular, 9.5, x0 + 18.0, ly + 18.0, pal.steel, "No saved bookmarks or history yet");
        } else {
            for bm in snapshot.bookmarks.iter().take(3) {
                let row_hov = point_in(cursor, x0 + 10.0, ly, col_w - 20.0, 32.0);
                if row_hov {
                    c.fill_rounded_rect(x0 + 10.0, ly, col_w - 20.0, 32.0, 8.0, pal.btn_hover);
                }
                c.draw_icon_star(x0 + 24.0, ly + 16.0, true, pal.star_active);
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::Regular, 9.5, x0 + 42.0, ly + 9.0, col_w - 90.0, pal.ink, &bm.title);
                hits.push(HitRegion {
                    x: x0 + 10.0,
                    y: ly,
                    w: col_w - 48.0,
                    h: 32.0,
                    id: HitId::OpenUrlTile(bm.url.clone()),
                    clickable: true,
                });
                let rm_x = x0 + col_w - 36.0;
                c.draw_icon_close(rm_x + 10.0, ly + 16.0, pal.steel);
                hits.push(HitRegion {
                    x: rm_x,
                    y: ly + 3.0,
                    w: 24.0,
                    h: 26.0,
                    id: HitId::RemoveBookmark(bm.url.clone()),
                    clickable: true,
                });
                ly += 34.0;
            }
            for h in snapshot.history.iter().take(2) {
                if ly + 32.0 > y + panel_h - 8.0 {
                    break;
                }
                c.draw_icon_history(x0 + 24.0, ly + 16.0, pal.steel);
                c.draw_text_ellipsis(&mut self.fonts, FontStyle::Regular, 9.5, x0 + 42.0, ly + 9.0, col_w - 58.0, pal.slate, &h.title);
                hits.push(HitRegion {
                    x: x0 + 10.0,
                    y: ly,
                    w: col_w - 20.0,
                    h: 32.0,
                    id: HitId::OpenUrlTile(h.url.clone()),
                    clickable: true,
                });
                ly += 34.0;
            }
        }

        // Right: Persistent Notion-Style Rust Scratchpad
        let sx = x0 + col_w + 14.0;
        let sc_focused = self.active_input == ActiveInput::Scratchpad;
        draw_card(
            c,
            sx,
            y,
            col_w,
            panel_h,
            14.0,
            pal.surface,
            if sc_focused { pal.primary } else { pal.hairline },
            if sc_focused { 1.5 } else { 1.0 },
        );
        c.fill_rect(sx + 16.0, y + 42.0, col_w - 32.0, 1.0, pal.hairline);
        c.draw_text(&mut self.fonts, FontStyle::SemiBold, 10.0, sx + 18.0, y + 14.0, pal.ink, "Workspace Notes (Auto-Saved)");
        hits.push(HitRegion {
            x: sx,
            y: y + 44.0,
            w: col_w,
            h: panel_h - 44.0,
            id: HitId::ScratchpadBox,
            clickable: true,
        });

        let ny = y + 54.0;
        if self.scratchpad_text.is_empty() {
            c.draw_text(&mut self.fonts, FontStyle::Regular, 10.0, sx + 18.0, ny, pal.steel, "Click to write quick notes, links, or ideas...");
            if sc_focused && (time_sec * 2.0) as u32 % 2 == 0 {
                c.fill_rect(sx + 18.0, ny, 2.0, 20.0, pal.primary);
            }
        } else {
            let (end_x, end_y) = draw_wrapped_text(
                c,
                &mut self.fonts,
                FontStyle::Regular,
                10.0,
                sx + 18.0,
                ny,
                col_w - 36.0,
                21.0,
                pal.ink,
                &self.scratchpad_text,
            );
            if sc_focused && (time_sec * 2.0) as u32 % 2 == 0 {
                c.fill_rect(end_x + 2.0, end_y, 2.0, 18.0, pal.primary);
            }
        }

        y += panel_h + 26.0;
        let total_h = (y + scroll) - vy;
        self.newtab_max_scroll = (total_h - vh).max(0.0);
        if self.newtab_scroll_y > self.newtab_max_scroll {
            self.newtab_scroll_y = self.newtab_max_scroll;
        }
        if self.newtab_max_scroll > 4.0 {
            draw_scrollbar(c, pal, vx + vw - 9.0, vy + 6.0, vh - 12.0, scroll, self.newtab_max_scroll, total_h);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_card(
    c: &mut Canvas<'_>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    bg: Color,
    border: Color,
    stroke_w: f32,
) {
    c.fill_rounded_rect(x, y, w, h, radius, bg);
    if border.a > 0 && stroke_w > 0.0 {
        c.stroke_rounded_rect(x, y, w, h, radius, stroke_w, border);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_badge(
    c: &mut Canvas<'_>,
    fonts: &mut FontSystem,
    x: f32,
    y: f32,
    bg: Color,
    border: Color,
    fg: Color,
    label: &str,
) {
    let w = fonts.measure_text(FontStyle::SemiBold, 8.5, label) + 18.0;
    draw_card(c, x, y, w, 24.0, 12.0, bg, border, 1.0);
    c.draw_text(fonts, FontStyle::SemiBold, 8.5, x + 9.0, y + 6.0, fg, label);
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

/// Top Y for text so that its x-height sits optically centered on `center`.
#[inline]
fn text_top(center: f32, size: f32) -> f32 {
    center - size * FONT_SCALE * 0.56
}

/// Borderless Notion-style icon button: invisible at rest, soft fill on hover, tinted when active.
#[allow(clippy::too_many_arguments)]
fn ghost_button(
    c: &mut Canvas<'_>,
    pal: &ThemePalette,
    cursor: (f32, f32),
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    accent: Option<Color>,
) -> bool {
    let hov = point_in(cursor, x, y, w, h);
    if let Some(a) = accent {
        c.fill_rounded_rect(x, y, w, h, 8.0, Color::rgba(a.r, a.g, a.b, if hov { 46 } else { 30 }));
    } else if hov {
        c.fill_rounded_rect(x, y, w, h, 8.0, pal.btn_hover);
    }
    hov
}

fn favicon_letter(url: &str, title: &str) -> char {
    let rest = url.split("://").nth(1).unwrap_or(url);
    let host = rest.split('/').next().unwrap_or("");
    let host = host.strip_prefix("www.").unwrap_or(host);
    host.chars()
        .chain(title.chars())
        .find(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or('P')
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
    c.fill_rounded_rect(x, thumb_y, 6.0, thumb_h, 3.0, pal.hairline_strong);
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

        let _ = ui.handle_char_input('r');
        assert!(!ui.omnibox_select_all);
        assert_eq!(ui.omnibox_text, "r");

        for c in "ust-lang.org".chars() {
            let _ = ui.handle_char_input(c);
        }
        assert_eq!(ui.omnibox_text, "rust-lang.org");

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
    fn e2e_render_apple_notion_top_left_settings_and_palette() {
        let mut ui = NativeUi::new(false, "note test".to_string());
        let w = 1280_usize;
        let h = 800_usize;
        let mut buf = vec![0u32; w * h];
        let tabs = vec![TabSnapshot {
            id: 1,
            title: "Welcome".to_string(),
            url: WELCOME_URL.to_string(),
            is_loading: false,
            active: true,
            pinned: false,
            is_muted: false,
            reader_active: false,
            night_active: false,
        }];

        // 1. Render Daylight Top Bar + Apple/Notion Welcome Landing Page
        let snap_top = RenderSnapshot {
            tabs: &tabs,
            active_url: WELCOME_URL,
            active_is_loading: false,
            active_muted: false,
            active_reader: false,
            active_night: false,
            is_bookmarked: false,
            zoom_percent: 125,
            can_reopen_closed: false,
            show_bookmarks_bar: true,
            sidebar_left: false,
            sidebar_collapsed: false,
            adblock_enabled: true,
            bookmarks: &[],
            history: &[],
            telemetry: None,
        };
        ui.render(&mut buf, w, h, 1.0, &snap_top);
        assert!(buf.iter().any(|&px| px != 0));
        assert_eq!(ui.left_sidebar_width(false, false), 0.0);

        // Zoom badge (125%) shows inside omnibox and resets on click
        let z_rst = ui
            .hits
            .iter()
            .find(|r| r.id == HitId::ZoomReset)
            .expect("Zoom badge rendered")
            .clone();
        ui.cursor_pos = (z_rst.x + 4.0, z_rst.y + 4.0);
        let z_acts = ui.handle_mouse_click(MouseButton::Left, WELCOME_URL);
        assert!(z_acts.iter().any(|a| matches!(a, UiAction::ZoomReset)));

        // Custom in-app window controls replace the OS title bar
        let close_btn = ui
            .hits
            .iter()
            .find(|r| r.id == HitId::WinClose)
            .expect("WinClose rendered")
            .clone();
        ui.cursor_pos = (close_btn.x + 6.0, close_btn.y + 6.0);
        let c_acts = ui.handle_mouse_click(MouseButton::Left, WELCOME_URL);
        assert!(c_acts.iter().any(|a| matches!(a, UiAction::WindowClose)));

        // Empty tab-strip space drags the window (tested to the left of window control buttons)
        let min_btn = ui
            .hits
            .iter()
            .find(|r| r.id == HitId::WinMinimize)
            .expect("WinMinimize rendered")
            .clone();
        ui.cursor_pos = (min_btn.x - 30.0, 6.0);
        let d_acts = ui.handle_mouse_click(MouseButton::Left, WELCOME_URL);
        assert!(
            d_acts.iter().any(|a| matches!(a, UiAction::WindowDrag)),
            "Expected WindowDrag, got: {:?}",
            d_acts
        );

        // 2. Render Arc Left Sidebar + pluto://settings
        let snap_side = RenderSnapshot {
            tabs: &tabs,
            active_url: SETTINGS_URL,
            active_is_loading: false,
            active_muted: false,
            active_reader: false,
            active_night: false,
            is_bookmarked: false,
            zoom_percent: 100,
            can_reopen_closed: false,
            show_bookmarks_bar: false,
            sidebar_left: true,
            sidebar_collapsed: false,
            adblock_enabled: true,
            bookmarks: &[],
            history: &[],
            telemetry: None,
        };
        ui.render(&mut buf, w, h, 1.0, &snap_side);
        assert_eq!(ui.left_sidebar_width(true, false), SIDEBAR_W as f64);

        // Clicking Settings Zoom 150% preset emits SetZoomPercent(150)
        let z150 = ui
            .hits
            .iter()
            .find(|r| r.id == HitId::SettingsZoomPreset(150))
            .expect("150% preset in settings")
            .clone();
        ui.cursor_pos = (z150.x + 8.0, z150.y + 8.0);
        let p_acts = ui.handle_mouse_click(MouseButton::Left, SETTINGS_URL);
        assert!(p_acts.iter().any(|a| matches!(a, UiAction::SetZoomPercent(150))));

        // Clicking Top Layout card emits ToggleSidebarLayout
        let top_card = ui
            .hits
            .iter()
            .find(|r| r.id == HitId::SettingsLayoutTop)
            .expect("Top layout card rendered")
            .clone();
        ui.cursor_pos = (top_card.x + 12.0, top_card.y + 12.0);
        let l_acts = ui.handle_mouse_click(MouseButton::Left, SETTINGS_URL);
        assert!(l_acts.iter().any(|a| matches!(a, UiAction::ToggleSidebarLayout)));

        // 3. Render New Tab Workspace + Open Command Palette
        let snap_nt = RenderSnapshot {
            active_url: NEW_TAB_URL,
            ..snap_side
        };
        ui.open_command_palette();
        ui.render(&mut buf, w, h, 1.0, &snap_nt);
        assert!(!ui.palette_entries.is_empty());
    }
}
