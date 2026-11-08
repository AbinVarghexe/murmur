//! Link builders. Every user-supplied part is percent-encoded, and only a short list of
//! URL schemes is ever opened.

use crate::planner::SearchEngine;
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};

/// The characters JavaScript's `encodeURIComponent` leaves alone.
const COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'!')
    .remove(b'~')
    .remove(b'*')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')');

pub fn encode_component(value: &str) -> String {
    utf8_percent_encode(value, COMPONENT).to_string()
}

pub fn search_url(engine: SearchEngine, query: &str) -> String {
    let base = match engine {
        SearchEngine::Google => "https://www.google.com/search?q=",
        SearchEngine::Youtube => "https://www.youtube.com/results?search_query=",
        SearchEngine::Maps => "https://www.google.com/maps/search/",
    };
    format!("{base}{}", encode_component(query))
}

/// A `mailto:` link that opens a draft in the default mail app. Nothing is sent.
pub fn mailto_url(to: Option<&str>, subject: Option<&str>, body: Option<&str>) -> String {
    let mut params = Vec::new();
    if let Some(subject) = subject.filter(|s| !s.is_empty()) {
        params.push(format!("subject={}", encode_component(subject)));
    }
    if let Some(body) = body.filter(|s| !s.is_empty()) {
        params.push(format!("body={}", encode_component(body)));
    }
    let recipient = to
        .filter(|s| !s.is_empty())
        .map(encode_component)
        .unwrap_or_default();
    if params.is_empty() {
        format!("mailto:{recipient}")
    } else {
        format!("mailto:{recipient}?{}", params.join("&"))
    }
}

/// A WhatsApp click-to-chat link. The user still presses send.
pub fn whatsapp_url(text: &str, phone: Option<&str>) -> String {
    let digits: String = phone
        .unwrap_or("")
        .chars()
        .filter(char::is_ascii_digit)
        .collect();
    format!("https://wa.me/{digits}?text={}", encode_component(text))
}

/// True only for http, https and mailto links. Everything else is refused.
pub fn is_safe_external_url(raw: &str) -> bool {
    match url::Url::parse(raw) {
        Ok(parsed) => matches!(parsed.scheme(), "http" | "https" | "mailto"),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_search_queries() {
        assert_eq!(
            search_url(SearchEngine::Google, "a&b c"),
            "https://www.google.com/search?q=a%26b%20c"
        );
        assert!(search_url(SearchEngine::Youtube, "lo-fi").contains("search_query=lo-fi"));
        assert_eq!(
            search_url(SearchEngine::Maps, "the airport"),
            "https://www.google.com/maps/search/the%20airport"
        );
    }

    #[test]
    fn builds_mailto_drafts_with_encoded_fields() {
        assert_eq!(
            mailto_url(Some("a@b.co"), Some("hi there"), Some("x&y")),
            "mailto:a%40b.co?subject=hi%20there&body=x%26y"
        );
        assert_eq!(mailto_url(None, None, None), "mailto:");
        assert_eq!(
            mailto_url(Some("a@b.co"), None, Some("hi")),
            "mailto:a%40b.co?body=hi"
        );
    }

    #[test]
    fn builds_whatsapp_links_and_strips_non_digits() {
        assert_eq!(
            whatsapp_url("late", Some("+1 (555) 123-4567")),
            "https://wa.me/15551234567?text=late"
        );
        assert_eq!(whatsapp_url("hi", None), "https://wa.me/?text=hi");
    }

    #[test]
    fn only_allows_http_https_and_mailto() {
        assert!(is_safe_external_url("https://example.com"));
        assert!(is_safe_external_url("mailto:a@b.co"));
        assert!(!is_safe_external_url("file:///C:/Windows/System32/cmd.exe"));
        assert!(!is_safe_external_url("javascript:alert(1)"));
        assert!(!is_safe_external_url("not a url"));
    }
}
