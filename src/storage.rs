use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub favicon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionData {
    pub tabs: Vec<String>,
    pub active_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    #[serde(default)]
    pub onboarding_completed: bool,
}

pub struct StorageManager {
    data_dir: PathBuf,
}

impl StorageManager {
    pub fn new() -> Result<Self> {
        let data_dir = ProjectDirs::from("com", "adit", "rust-browser-adit")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".browser-data"));

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

