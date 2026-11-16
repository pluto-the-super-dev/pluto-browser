use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub favicon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub timestamp_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionData {
    pub tabs: Vec<String>,
    pub active_index: usize,
}

fn default_true() -> bool {
    true
}

fn default_zoom_one() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub onboarding_completed: bool,
    /// Pure-Black Zed architectural dark grid is enabled by default.
    #[serde(default = "default_true")]
    pub dark_mode: bool,
    #[serde(default)]
    pub workspace_note: String,
    #[serde(default = "default_true")]
    pub show_bookmarks_bar: bool,
    #[serde(default = "default_zoom_one")]
    pub ui_zoom: f64,
    /// false = Top horizontal tab bar, true = Arc/Zed-style left vertical sidebar.
    #[serde(default)]
    pub sidebar_left: bool,
    #[serde(default)]
    pub sidebar_collapsed: bool,
    #[serde(default = "default_true")]
    pub adblock_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            onboarding_completed: false,
            dark_mode: true,
            workspace_note: String::new(),
            show_bookmarks_bar: true,
            ui_zoom: 1.0,
            sidebar_left: false,
            sidebar_collapsed: false,
            adblock_enabled: true,
        }
    }
}

pub struct StorageManager {
    data_dir: PathBuf,
}

impl StorageManager {
    pub fn new() -> Result<Self> {
        let data_dir = ProjectDirs::from("app", "pluto", "pluto-browser")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".pluto-browser-data"));

        fs::create_dir_all(&data_dir).with_context(|| {
            format!(
                "Failed to create browser data directory at {}",
                data_dir.display()
            )
        })?;

        Ok(Self { data_dir })
    }

    fn bookmarks_path(&self) -> PathBuf {
        self.data_dir.join("bookmarks.json")
    }

    fn history_path(&self) -> PathBuf {
        self.data_dir.join("history.json")
    }

    fn session_path(&self) -> PathBuf {
        self.data_dir.join("session.json")
    }

    fn settings_path(&self) -> PathBuf {
        self.data_dir.join("settings.json")
    }

    pub fn webview_data_dir(&self) -> PathBuf {
        self.data_dir.join("webview2_pluto")
    }

    pub fn load_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let path = self.bookmarks_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read bookmarks from {}", path.display()))?;
        let cleaned = raw.trim_start_matches('\u{feff}');
        let bookmarks: Vec<Bookmark> = serde_json::from_str(cleaned)
            .with_context(|| format!("Failed to parse bookmarks JSON at {}", path.display()))?;
        Ok(bookmarks)
    }

    pub fn save_bookmarks(&self, bookmarks: &[Bookmark]) -> Result<()> {
        let path = self.bookmarks_path();
        let json = serde_json::to_string_pretty(bookmarks)
            .context("Failed to serialize bookmarks")?;
        fs::write(&path, json)
            .with_context(|| format!("Failed to write bookmarks to {}", path.display()))?;
        Ok(())
    }

    pub fn load_history(&self) -> Result<Vec<HistoryEntry>> {
        let path = self.history_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read history from {}", path.display()))?;
        let cleaned = raw.trim_start_matches('\u{feff}');
        let history: Vec<HistoryEntry> = serde_json::from_str(cleaned)
            .with_context(|| format!("Failed to parse history JSON at {}", path.display()))?;
        Ok(history)
    }

    pub fn save_history(&self, history: &[HistoryEntry]) -> Result<()> {
        let path = self.history_path();
        let json =
            serde_json::to_string_pretty(history).context("Failed to serialize history")?;
        fs::write(&path, json)
            .with_context(|| format!("Failed to write history to {}", path.display()))?;
        Ok(())
    }

    pub fn record_history_entry(history: &mut Vec<HistoryEntry>, title: &str, url: &str) {
        if url.is_empty() || url.starts_with("pluto://") || url == "about:blank" {
            return;
        }
        let clean_title = if title.trim().is_empty() {
            url.to_string()
        } else {
            title.trim().to_string()
        };
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        history.retain(|h| h.url != url);
        history.insert(
            0,
            HistoryEntry {
                title: clean_title,
                url: url.to_string(),
                timestamp_secs: ts,
            },
        );
        if history.len() > 100 {
            history.truncate(100);
        }
    }

    pub fn load_session(&self) -> Result<Option<SessionData>> {
        let path = self.session_path();
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read session from {}", path.display()))?;
        let cleaned = raw.trim_start_matches('\u{feff}');
        let session: SessionData = serde_json::from_str(cleaned)
            .with_context(|| format!("Failed to parse session JSON at {}", path.display()))?;
        Ok(Some(session))
    }

    pub fn save_session(&self, session: &SessionData) -> Result<()> {
        let path = self.session_path();
        let json =
            serde_json::to_string_pretty(session).context("Failed to serialize session")?;
        fs::write(&path, json)
            .with_context(|| format!("Failed to write session to {}", path.display()))?;
        Ok(())
    }

    pub fn load_settings(&self) -> Result<AppSettings> {
        let path = self.settings_path();
        if !path.exists() {
            return Ok(AppSettings::default());
        }
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read settings from {}", path.display()))?;
        let cleaned = raw.trim_start_matches('\u{feff}');
        let settings: AppSettings = serde_json::from_str(cleaned)
            .with_context(|| format!("Failed to parse settings JSON at {}", path.display()))?;
        Ok(settings)
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let path = self.settings_path();
        let json =
            serde_json::to_string_pretty(settings).context("Failed to serialize settings")?;
        fs::write(&path, json)
            .with_context(|| format!("Failed to write settings to {}", path.display()))?;
        Ok(())
    }
}
