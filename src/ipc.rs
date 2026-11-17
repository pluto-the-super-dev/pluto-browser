use serde::Deserialize;
use std::net::Ipv4Addr;
use url::Url;

pub const NEW_TAB_URL: &str = "pluto://newtab";
pub const WELCOME_URL: &str = "pluto://welcome";
pub const SETTINGS_URL: &str = "pluto://settings";
pub const GOOGLE_HOME_URL: &str = "https://www.google.com/";

/// IPC messages sent from external web pages (via minimal init bridge) to Rust.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum TabIpcMessage {
    NewTab,
    CloseActiveTab,
    NextTab,
    PrevTab,
    Back,
    Forward,
    Reload,
    FocusAddressBar,
    OpenSettings,
    ToggleSidebarLayout,
    ToggleSidebarCollapse,
    ToggleCommandPalette,
    ToggleBookmarksBar,
    DuplicateActiveTab,
    ToggleReaderMode,
    ToggleNightMode,
    ToggleMute,
    CopyMarkdown,
    OpenDevtools,
    ToggleInspector,
    ToggleFindBar,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ReopenClosedTab,
    PageMeta {
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        favicon: Option<String>,
    },
    InspectorTelemetry {
        #[serde(default)]
        dom_nodes: u32,
        #[serde(default)]
        scripts: u32,
        #[serde(default)]
        stylesheets: u32,
        #[serde(default)]
        images: u32,
        #[serde(default)]
        dcl_ms: u32,
        #[serde(default)]
        load_ms: u32,
        #[serde(default)]
        local_keys: u32,
        #[serde(default)]
        session_keys: u32,
        #[serde(default)]
        origin: String,
    },
}

#[derive(Debug)]
pub enum UserEvent {
    TabIpc { tab_id: u64, raw: String },
    TabTitleChanged { id: u64, title: String },
    TabLoadState { id: u64, is_loading: bool, url: String },
    TabNewWindowRequested { url: String },
}

/// Converts raw omnibox input into a navigable URL, adding a scheme when the input
/// looks like a host/URL or falling back to a Google Search query.
pub fn resolve_omnibox_input(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("about:newtab")
        || trimmed.eq_ignore_ascii_case("about:blank")
        || trimmed.eq_ignore_ascii_case(NEW_TAB_URL)
    {
        return NEW_TAB_URL.to_string();
    }

    if trimmed.eq_ignore_ascii_case(WELCOME_URL)
        || trimmed.eq_ignore_ascii_case("pluto://onboarding")
        || trimmed.eq_ignore_ascii_case("about:welcome")
    {
        return WELCOME_URL.to_string();
    }

    if trimmed.eq_ignore_ascii_case(SETTINGS_URL)
        || trimmed.eq_ignore_ascii_case("about:settings")
    {
        return SETTINGS_URL.to_string();
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("file://")
        || lower.starts_with("data:")
        || lower.starts_with("pluto://")
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

    Url::parse_with_params("https://www.google.com/search", &[("q", trimmed)])
        .map(|u| u.to_string())
        .unwrap_or_else(|_| GOOGLE_HOME_URL.to_string())
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

/// Strips common cross-site tracking query parameters (`utm_*`, `fbclid`, `gclid`, etc.)
/// purely in Rust while preserving search & content query parameters.
pub fn strip_tracking_params(raw_url: &str) -> (String, usize) {
    let Ok(mut parsed) = Url::parse(raw_url) else {
        return (raw_url.to_string(), 0);
    };
    if parsed.query().is_none() {
        return (raw_url.to_string(), 0);
    }

    const TRACKING_KEYS: &[&str] = &[
        "fbclid", "gclid", "dclid", "gbraid", "wbraid", "msclkid", "yclid", "mc_cid", "mc_eid",
        "igshid", "si", "_hsenc", "_hsmi", "ref_src", "vero_id",
    ];

    let mut kept: Vec<(String, String)> = Vec::new();
    let mut removed = 0usize;
    for (k, v) in parsed.query_pairs() {
        let lower = k.to_ascii_lowercase();
        let is_utm = lower.starts_with("utm_");
        if is_utm || TRACKING_KEYS.contains(&lower.as_str()) {
            removed += 1;
        } else {
            kept.push((k.into_owned(), v.into_owned()));
        }
    }

    if removed == 0 {
        return (raw_url.to_string(), 0);
    }

    if kept.is_empty() {
        parsed.set_query(None);
    } else {
        let mut pairs = parsed.query_pairs_mut();
        pairs.clear();
        for (k, v) in kept {
            pairs.append_pair(&k, &v);
        }
    }
    (parsed.to_string(), removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_new_tab_aliases() {
        assert_eq!(resolve_omnibox_input(""), NEW_TAB_URL);
        assert_eq!(resolve_omnibox_input("   "), NEW_TAB_URL);
        assert_eq!(resolve_omnibox_input("pluto://newtab"), NEW_TAB_URL);
        assert_eq!(resolve_omnibox_input("about:newtab"), NEW_TAB_URL);
        assert_eq!(resolve_omnibox_input("about:blank"), NEW_TAB_URL);
    }

    #[test]
    fn resolves_welcome_and_settings_aliases() {
        assert_eq!(resolve_omnibox_input("pluto://welcome"), WELCOME_URL);
        assert_eq!(resolve_omnibox_input("pluto://onboarding"), WELCOME_URL);
        assert_eq!(resolve_omnibox_input("about:welcome"), WELCOME_URL);
        assert_eq!(resolve_omnibox_input("pluto://settings"), SETTINGS_URL);
        assert_eq!(resolve_omnibox_input("about:settings"), SETTINGS_URL);
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
    fn resolves_search_queries_to_google() {
        assert_eq!(
            resolve_omnibox_input("rust programming language"),
            "https://www.google.com/search?q=rust+programming+language"
        );
        assert_eq!(
            resolve_omnibox_input("hello"),
            "https://www.google.com/search?q=hello"
        );
    }

    #[test]
    fn strips_tracking_query_params_cleanly() {
        let (clean, count) = strip_tracking_params(
            "https://example.com/article?id=42&utm_source=twitter&utm_medium=social&fbclid=abc123",
        );
        assert_eq!(count, 3);
        assert_eq!(clean, "https://example.com/article?id=42");

        let (plain, removed) = strip_tracking_params("https://www.google.com/search?q=pluto+rust");
        assert_eq!(removed, 0);
        assert_eq!(plain, "https://www.google.com/search?q=pluto+rust");
    }
}
