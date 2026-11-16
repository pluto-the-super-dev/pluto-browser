use crate::storage::Bookmark;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use url::Url;

pub const NEW_TAB_URL: &str = "pluto://newtab";
pub const WELCOME_URL: &str = "pluto://welcome";

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum UiToRustMessage {
    NewTab {
        #[serde(default)]
        url: Option<String>,
    },
    CloseTab {
        id: u64,
    },
    CloseActiveTab,
    SwitchTab {
        id: u64,
    },
    NextTab,
    PrevTab,
    Navigate {
        input: String,
    },
    Back,
    Forward,
    Reload,
    Stop,
    Home,
    ToggleBookmark,
    OpenBookmark {
        url: String,
    },
    RemoveBookmark {
        url: String,
    },
    FocusAddressBar,
    OpenDevtools,
    ToggleInspector,
    ToggleFindBar,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ReopenClosedTab,
    DuplicateTab {
        #[serde(default)]
        id: Option<u64>,
    },
    PinTab {
        id: u64,
    },
    CompleteOnboarding {
        #[serde(default)]
        destination: Option<String>,
    },
    OpenOnboarding,
    PageMeta {
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        favicon: Option<String>,
    },
    UiReady,
}

#[derive(Debug, Clone, Serialize)]
pub struct TabUiInfo {
    pub id: u64,
    pub title: String,
    pub url: String,
    pub display_url: String,
    pub favicon: Option<String>,
    pub is_loading: bool,
    pub active: bool,
    pub pinned: bool,
    pub zoom_percent: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RustToUiState {
    pub tabs: Vec<TabUiInfo>,
    pub active_tab_id: Option<u64>,
    pub active_url: String,
    pub is_loading: bool,
    pub is_bookmarked: bool,
    pub zoom_percent: u32,
    pub can_reopen_closed: bool,
    pub bookmarks: Vec<Bookmark>,
}

#[derive(Debug)]
pub enum UserEvent {
    ChromeIpc(String),
    TabIpc { tab_id: u64, raw: String },
    TabTitleChanged { id: u64, title: String },
    TabLoadState { id: u64, is_loading: bool, url: String },
    TabNewWindowRequested { url: String },
}

/// Converts raw omnibox input into a navigable URL, adding a scheme when the input
/// looks like a host/URL or falling back to a DuckDuckGo search query.
pub fn resolve_omnibox_input(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("about:newtab")
        || trimmed.eq_ignore_ascii_case("about:blank")
        || trimmed.eq_ignore_ascii_case("adit://newtab")
        || trimmed.eq_ignore_ascii_case(NEW_TAB_URL)
    {
        return NEW_TAB_URL.to_string();
    }

    if trimmed.eq_ignore_ascii_case(WELCOME_URL)
        || trimmed.eq_ignore_ascii_case("pluto://onboarding")
        || trimmed.eq_ignore_ascii_case("adit://welcome")
        || trimmed.eq_ignore_ascii_case("adit://onboarding")
        || trimmed.eq_ignore_ascii_case("about:welcome")
    {
        return WELCOME_URL.to_string();
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("file://")
        || lower.starts_with("data:")
        || lower.starts_with("pluto://")
        || lower.starts_with("adit://")
    {
        if let Ok(parsed) = Url::parse(trimmed) {
            return parsed.to_string();
        }
    }

    if !trimmed.chars().any(char::is_whitespace) {
        let authority_end = trimmed.find(['/', '?', '#']).unwrap_or(trimmed.len());
        let authority = &trimmed[..authority_end];
        let host = split_host_port(authority);

        if let Some(scheme) = infer_scheme_for_host(host) {
            let candidate = format!("{scheme}://{trimmed}");
            if let Ok(parsed) = Url::parse(&candidate) {
                return parsed.to_string();
            }
        }
    }

    Url::parse_with_params("https://duckduckgo.com/", &[("q", trimmed)])
        .map(|u| u.to_string())
        .unwrap_or_else(|_| "https://duckduckgo.com/".to_string())
}

fn split_host_port(authority: &str) -> &str {
    match authority.rsplit_once(':') {
        Some((host, port)) if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) => host,
        _ => authority,
    }
}

fn infer_scheme_for_host(host: &str) -> Option<&'static str> {
    if host.is_empty() {
        return None;
    }
    if host.eq_ignore_ascii_case("localhost") {
        return Some("http");
    }
    if host.parse::<Ipv4Addr>().is_ok() {
        return Some("http");
    }
    if !host.contains('.') || host.starts_with('.') || host.ends_with('.') {
        return None;
    }

    let labels: Vec<&str> = host.split('.').collect();
    if labels.len() < 2 {
        return None;
    }
    let valid_labels = labels.iter().all(|label| {
        !label.is_empty()
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    });
    if !valid_labels {
        return None;
    }

    let tld = labels.last().copied().unwrap_or_default();
    if tld.len() >= 2 && tld.chars().any(|c| c.is_ascii_alphabetic()) {
        Some("https")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_new_tab_aliases() {
        assert_eq!(resolve_omnibox_input(""), NEW_TAB_URL);
        assert_eq!(resolve_omnibox_input("   "), NEW_TAB_URL);
        assert_eq!(resolve_omnibox_input("pluto://newtab"), NEW_TAB_URL);
        assert_eq!(resolve_omnibox_input("adit://newtab"), NEW_TAB_URL);
        assert_eq!(resolve_omnibox_input("about:newtab"), NEW_TAB_URL);
        assert_eq!(resolve_omnibox_input("about:blank"), NEW_TAB_URL);
    }

    #[test]
    fn resolves_welcome_aliases() {
        assert_eq!(resolve_omnibox_input("pluto://welcome"), WELCOME_URL);
        assert_eq!(resolve_omnibox_input("pluto://onboarding"), WELCOME_URL);
        assert_eq!(resolve_omnibox_input("adit://welcome"), WELCOME_URL);
        assert_eq!(resolve_omnibox_input("about:welcome"), WELCOME_URL);
        assert_eq!(resolve_omnibox_input("adit://onboarding"), WELCOME_URL);
    }

    #[test]
    fn resolves_urls_and_adds_https() {
        assert_eq!(resolve_omnibox_input("example.com"), "https://example.com/");
        assert_eq!(
            resolve_omnibox_input("https://rust-lang.org/learn"),
            "https://rust-lang.org/learn"
        );
        assert_eq!(
            resolve_omnibox_input("localhost:3000/test"),
            "http://localhost:3000/test"
        );
        assert_eq!(
            resolve_omnibox_input("127.0.0.1:8080"),
            "http://127.0.0.1:8080/"
        );
    }

    #[test]
    fn resolves_search_queries_to_duckduckgo() {
        assert_eq!(
            resolve_omnibox_input("rust programming language"),
            "https://duckduckgo.com/?q=rust+programming+language"
        );
        assert_eq!(
            resolve_omnibox_input("hello"),
            "https://duckduckgo.com/?q=hello"
        );
    }
}

