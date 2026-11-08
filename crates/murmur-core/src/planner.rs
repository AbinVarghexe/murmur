//! Turns a spoken command into a structured plan. Pure text in, plan out: nothing here
//! touches the operating system, so every phrase can be tested.

use crate::keys::KeyCombo;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchEngine {
    Google,
    Youtube,
    Maps,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VolumeOp {
    Up,
    Down,
    Mute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaOp {
    PlayPause,
    Next,
    Previous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowOp {
    Minimize,
    Maximize,
    Close,
    Switch,
    ShowDesktop,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    OpenApp {
        name: String,
    },
    OpenUrl {
        url: String,
    },
    WebSearch {
        query: String,
        engine: SearchEngine,
    },
    PressKeys {
        keys: String,
    },
    TypeText {
        text: String,
    },
    Volume {
        op: VolumeOp,
    },
    Media {
        op: MediaOp,
    },
    ComposeEmail {
        to: Option<String>,
        subject: Option<String>,
        body: Option<String>,
    },
    Whatsapp {
        phone: Option<String>,
        text: String,
    },
    Window {
        op: WindowOp,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub action: Action,
    /// True when the action is disruptive and should be confirmed first.
    pub confirm: bool,
    pub summary: String,
}

fn ci(pattern: &str) -> Regex {
    Regex::new(&format!("(?i){pattern}")).expect("static pattern is valid")
}

static WAKE_PREFIX: LazyLock<Regex> =
    LazyLock::new(|| ci(r"^(?:(?:hey|ok|okay)[,\s]+)?(?:computer|murmur|assistant)[,:]?\s+"));
static POLITE_PREFIX: LazyLock<Regex> =
    LazyLock::new(|| ci(r"^(?:please|can you|could you|would you|go ahead and)[,\s]+"));
static POLITE_SUFFIX: LazyLock<Regex> = LazyLock::new(|| ci(r"[,\s]+please$"));
static TRAILING_NOISE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[,\s]+$").unwrap());
static TRAILING_ALL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[.!?,\s]+$").unwrap());
static TRAILING_EMPHASIS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[!?]+$").unwrap());

/// Command shells and script hosts always need a confirmation before launch.
static SHELL_APPS: LazyLock<Regex> = LazyLock::new(|| {
    ci(r"\b(?:terminal|command prompt|cmd|powershell|pwsh|shell|bash|wsl|regedit|registry)\b")
});

static SHORTCUT_PHRASES: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (ci(r"^copy(?: that| this| it)?$"), "ctrl+c"),
        (ci(r"^paste(?: it| that| here)?$"), "ctrl+v"),
        (ci(r"^cut(?: that| this| it)?$"), "ctrl+x"),
        (ci(r"^select all$"), "ctrl+a"),
        (ci(r"^undo(?: that)?$"), "ctrl+z"),
        (ci(r"^redo(?: that)?$"), "ctrl+y"),
        (ci(r"^save(?: it| this| the file)?$"), "ctrl+s"),
        (ci(r"^(?:open (?:a )?)?new tab$"), "ctrl+t"),
        (ci(r"^close (?:the |this |current )?tab$"), "ctrl+w"),
        (ci(r"^(?:take (?:a )?)?screenshot$"), "printscreen"),
        (ci(r"^scroll down$"), "pagedown"),
        (ci(r"^scroll up$"), "pageup"),
        (ci(r"^go back$"), "alt+left"),
        (ci(r"^go forward$"), "alt+right"),
        (ci(r"^(?:refresh|reload)(?: the)?(?: page)?$"), "f5"),
    ]
});

/// Websites people ask to "open" by name. Apps with their own installs are left out.
const SITES: &[(&str, &str)] = &[
    ("youtube", "https://www.youtube.com/"),
    ("google", "https://www.google.com/"),
    ("gmail", "https://mail.google.com/"),
    ("google drive", "https://drive.google.com/"),
    ("google docs", "https://docs.google.com/"),
    ("google maps", "https://www.google.com/maps"),
    ("maps", "https://www.google.com/maps"),
    ("github", "https://github.com/"),
    ("reddit", "https://www.reddit.com/"),
    ("twitter", "https://x.com/"),
    ("x", "https://x.com/"),
    ("facebook", "https://www.facebook.com/"),
    ("instagram", "https://www.instagram.com/"),
    ("linkedin", "https://www.linkedin.com/"),
    ("amazon", "https://www.amazon.com/"),
    ("netflix", "https://www.netflix.com/"),
    ("chatgpt", "https://chatgpt.com/"),
    ("claude", "https://claude.ai/"),
    ("wikipedia", "https://www.wikipedia.org/"),
    ("whatsapp web", "https://web.whatsapp.com/"),
];

const BROWSER_WORDS: &str = r"(?:chrome|google chrome|edge|microsoft edge|firefox|brave|the browser|a browser|browser|google)";

/// "Open the browser and search for X". The query is the last capture group.
static BROWSER_SEARCH: LazyLock<Regex> = LazyLock::new(|| {
    ci(&format!(
        r"^(?:open|launch|start)\s+(?:up\s+)?{BROWSER_WORDS}\s+(?:and\s+)?(?:search|look up|google)\s+(?:for\s+)?(.+)$"
    ))
});
static GO_TO: LazyLock<Regex> = LazyLock::new(|| ci(r"^(?:go to|visit|browse to)\s+(.+)$"));
static OPEN: LazyLock<Regex> = LazyLock::new(|| {
    ci(
        r"^(?:open|launch|start|run)\s+(?:up\s+)?(?:the\s+)?(.+?)(?:\s+(?:app|application|website|site))?$",
    )
});
static YOUTUBE: LazyLock<Regex> =
    LazyLock::new(|| ci(r"^(?:search\s+youtube\s+for|play)\s+(.+?)(?:\s+on\s+youtube)?$"));
static YOUTUBE_SEARCH_START: LazyLock<Regex> = LazyLock::new(|| ci(r"^search\s+youtube"));
static YOUTUBE_SUFFIX: LazyLock<Regex> = LazyLock::new(|| ci(r"\bon\s+youtube$"));
static MAPS: LazyLock<Regex> =
    LazyLock::new(|| ci(r"^(?:search maps for|navigate to|directions to)\s+(.+)$"));
static WEB_SEARCH: LazyLock<Regex> = LazyLock::new(|| {
    ci(
        r"^(?:(?:search|look up)\s+(?:(?:on|in)\s+)?(?:google|the web|the internet|online)?\s*(?:for\s+)?|google\s+)(.+)$",
    )
});
static EMAIL: LazyLock<Regex> = LazyLock::new(|| {
    ci(
        r"^(?:(?:send|write|compose|draft)\s+(?:an?\s+)?e-?mail\s+to|e-?mail(?:\s+to)?)\s+(.+?)(?:\s+about\s+(.+?))?(?:\s+(?:saying|that says)\s+(.+))?$",
    )
});
static WHATSAPP: LazyLock<Regex> = LazyLock::new(|| {
    ci(
        r"^(?:send\s+)?(?:an?\s+)?whatsapp(?:\s+message)?(?:\s+to\s+([\d\s+-]+?))?\s+(?:saying|that says|message)\s+(.+)$",
    )
});
static PRESS: LazyLock<Regex> = LazyLock::new(|| ci(r"^press\s+(.+)$"));
static TYPE: LazyLock<Regex> = LazyLock::new(|| ci(r"^(?:type|write out|enter text)\s+(.+)$"));

static SPOKEN_DOT: LazyLock<Regex> = LazyLock::new(|| ci(r"\s+dot\s+"));
static SPOKEN_SLASH: LazyLock<Regex> = LazyLock::new(|| ci(r"\s+slash\s+"));
static SPOKEN_AT: LazyLock<Regex> = LazyLock::new(|| ci(r"\s+at\s+"));
static ANY_SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
static HAS_SCHEME: LazyLock<Regex> = LazyLock::new(|| ci(r"^https?://"));
static BARE_DOMAIN: LazyLock<Regex> = LazyLock::new(|| ci(r"^[a-z0-9-]+(\.[a-z0-9-]+)+(/\S*)?$"));
static EMAIL_SHAPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap());

fn plan(action: Action, summary: impl Into<String>, confirm: bool) -> Plan {
    Plan {
        action,
        confirm,
        summary: summary.into(),
    }
}

fn owned(m: Option<regex::Match<'_>>) -> Option<String> {
    m.map(|m| m.as_str().to_string())
}

/// Strip wake words and politeness. Returns the command and any "?" or "!" that ended the
/// sentence (a lone full stop is only the speech model punctuating the command, so it is
/// dropped; "?" and "!" can be part of text the user wants typed).
fn normalize(utterance: &str) -> (String, String) {
    let trimmed = utterance.trim();
    let without_tail = TRAILING_NOISE.replace(trimmed, "");
    let trailing = TRAILING_EMPHASIS
        .find(&without_tail)
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();

    let mut text = TRAILING_ALL.replace(trimmed, "").into_owned();
    text = WAKE_PREFIX.replace(&text, "").into_owned();
    for _ in 0..3 {
        if !POLITE_PREFIX.is_match(&text) {
            break;
        }
        text = POLITE_PREFIX.replace(&text, "").into_owned();
    }
    text = POLITE_SUFFIX.replace(&text, "").into_owned();
    (text.trim().to_string(), trailing)
}

fn spoken_domain(raw: &str) -> String {
    let dotted = SPOKEN_DOT.replace_all(raw.trim(), ".");
    SPOKEN_SLASH.replace_all(&dotted, "/").into_owned()
}

fn to_http_url(candidate: &str) -> Option<String> {
    let text = spoken_domain(candidate);
    let has_scheme = HAS_SCHEME.is_match(&text);
    if !has_scheme && !BARE_DOMAIN.is_match(&text) {
        return None;
    }
    let full = if has_scheme {
        text
    } else {
        format!("https://{text}")
    };
    url::Url::parse(&full)
        .ok()
        .filter(|u| matches!(u.scheme(), "http" | "https"))
        .map(|u| u.to_string())
}

/// "control shift t" becomes the normalized combo "ctrl+shift+t".
fn parse_keys(spec: &str) -> Option<String> {
    let lowered = spec
        .to_lowercase()
        .replace('+', " ")
        .replace("page up", "pageup")
        .replace("page down", "pagedown")
        .replace("print screen", "printscreen");
    let tokens: Vec<&str> = lowered
        .split_whitespace()
        .filter(|t| *t != "plus")
        .collect();
    let mapped: Vec<&str> = tokens
        .iter()
        .map(|t| match *t {
            "control" => "ctrl",
            "esc" => "escape",
            "return" => "enter",
            "del" => "delete",
            other => other,
        })
        .collect();
    KeyCombo::parse(&mapped.join("+")).map(|c| c.to_string())
}

fn normalize_email(raw: &str) -> Option<String> {
    let at = SPOKEN_AT.replace_all(raw.trim(), "@");
    let dotted = SPOKEN_DOT.replace_all(&at, ".");
    let joined = ANY_SPACE.replace_all(&dotted, "").into_owned();
    EMAIL_SHAPE.is_match(&joined).then_some(joined)
}

fn plan_email(text: &str) -> Option<Plan> {
    let caps = EMAIL.captures(text)?;
    let recipient = caps.get(1).map_or("", |m| m.as_str());
    let to = normalize_email(recipient);
    let summary = match &to {
        Some(address) => format!("Draft email to {address}"),
        None => format!("Draft email (recipient \"{recipient}\" needs to be entered)"),
    };
    let action = Action::ComposeEmail {
        to,
        subject: owned(caps.get(2)),
        body: owned(caps.get(3)),
    };
    Some(plan(action, summary, false))
}

fn plan_whatsapp(text: &str) -> Option<Plan> {
    let caps = WHATSAPP.captures(text)?;
    let phone = caps
        .get(1)
        .map(|m| {
            m.as_str()
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>()
        })
        .filter(|digits| !digits.is_empty());
    let message = caps.get(2).map_or("", |m| m.as_str()).to_string();
    Some(plan(
        Action::Whatsapp {
            phone,
            text: message,
        },
        "Open WhatsApp with a drafted message",
        false,
    ))
}

/// "my downloads folder" becomes "downloads"; "up chrome" becomes "chrome".
fn clean_app_name(raw: &str) -> String {
    let mut name = raw.trim().to_lowercase();
    if let Some(rest) = name.strip_prefix("up ") {
        name = rest.to_string();
    }
    for prefix in ["my ", "the "] {
        if let Some(rest) = name.strip_prefix(prefix) {
            name = rest.to_string();
            break;
        }
    }
    if let Some(rest) = name.strip_suffix(" folder") {
        name = rest.to_string();
    }
    name
}

fn site_for(name: &str) -> Option<&'static str> {
    SITES
        .iter()
        .find(|(site, _)| *site == name)
        .map(|(_, url)| *url)
}

fn search_plan(query: &str, engine: SearchEngine, summary: String) -> Plan {
    plan(
        Action::WebSearch {
            query: query.to_string(),
            engine,
        },
        summary,
        false,
    )
}

fn plan_open_or_search(text: &str) -> Option<Plan> {
    if let Some(caps) = BROWSER_SEARCH.captures(text) {
        let query = caps.get(1).map_or("", |m| m.as_str());
        return Some(search_plan(
            query,
            SearchEngine::Google,
            format!("Search for \"{query}\""),
        ));
    }

    let go = GO_TO.captures(text);
    let open = OPEN.captures(text);
    let target = go
        .as_ref()
        .and_then(|c| c.get(1))
        .or_else(|| open.as_ref().and_then(|c| c.get(1)))
        .map(|m| m.as_str());
    if let Some(target) = target {
        if let Some(url) = to_http_url(target) {
            // An address heard in passing (a video, a call) must not open a page unasked.
            return Some(plan(
                Action::OpenUrl { url: url.clone() },
                format!("Open {url}"),
                true,
            ));
        }
        let name = clean_app_name(target);
        if let Some(site) = site_for(&name) {
            return Some(plan(
                Action::OpenUrl {
                    url: site.to_string(),
                },
                format!("Open {name}"),
                false,
            ));
        }
        if open.is_some() {
            let confirm = SHELL_APPS.is_match(&name);
            return Some(plan(
                Action::OpenApp { name: name.clone() },
                format!("Open {name}"),
                confirm,
            ));
        }
    }

    if let Some(caps) = YOUTUBE.captures(text) {
        if YOUTUBE_SEARCH_START.is_match(text) || YOUTUBE_SUFFIX.is_match(text) {
            let query = caps.get(1).map_or("", |m| m.as_str());
            return Some(search_plan(
                query,
                SearchEngine::Youtube,
                format!("Search YouTube for \"{query}\""),
            ));
        }
    }
    if let Some(caps) = MAPS.captures(text) {
        let query = caps.get(1).map_or("", |m| m.as_str());
        return Some(search_plan(
            query,
            SearchEngine::Maps,
            format!("Find \"{query}\" on Maps"),
        ));
    }
    if let Some(caps) = WEB_SEARCH.captures(text) {
        let query = caps.get(1).map_or("", |m| m.as_str());
        return Some(search_plan(
            query,
            SearchEngine::Google,
            format!("Search for \"{query}\""),
        ));
    }
    None
}

/// Volume, media and window commands, matched against whole phrases.
fn plan_system(text: &str) -> Option<Plan> {
    type Rule = (&'static str, Action, &'static str, bool);
    static RULES: LazyLock<Vec<(Regex, Action, &'static str, bool)>> = LazyLock::new(|| {
        let rules: Vec<Rule> = vec![
            (
                r"^(?:mute|unmute)(?: (?:the )?(?:sound|volume|audio))?$",
                Action::Volume { op: VolumeOp::Mute },
                "Toggle mute",
                false,
            ),
            (
                r"^(?:turn |make )?(?:the )?(?:volume |it )?(?:up|louder)$|^(?:volume up|turn up the volume)$",
                Action::Volume { op: VolumeOp::Up },
                "Volume up",
                false,
            ),
            (
                r"^(?:turn |make )?(?:the )?(?:volume |it )?(?:down|quieter)$|^(?:volume down|turn down the volume)$",
                Action::Volume { op: VolumeOp::Down },
                "Volume down",
                false,
            ),
            (
                r"^(?:pause|play|resume|play/pause)(?: (?:the )?(?:music|video|song))?$",
                Action::Media {
                    op: MediaOp::PlayPause,
                },
                "Play/pause media",
                false,
            ),
            (
                r"^(?:next|skip)(?: (?:this |the )?(?:track|song))?$",
                Action::Media { op: MediaOp::Next },
                "Next track",
                false,
            ),
            (
                r"^(?:previous|last|go back to the previous)(?: (?:track|song))$",
                Action::Media {
                    op: MediaOp::Previous,
                },
                "Previous track",
                false,
            ),
            (
                r"^minimi[sz]e(?: (?:the |this |current )?window)?$",
                Action::Window {
                    op: WindowOp::Minimize,
                },
                "Minimize window",
                false,
            ),
            (
                r"^maximi[sz]e(?: (?:the |this |current )?window)?$",
                Action::Window {
                    op: WindowOp::Maximize,
                },
                "Maximize window",
                false,
            ),
            (
                r"^close (?:the |this |current )?(?:window|app)$",
                Action::Window {
                    op: WindowOp::Close,
                },
                "Close the active window",
                true,
            ),
            (
                r"^switch (?:the )?(?:window|app)s?$",
                Action::Window {
                    op: WindowOp::Switch,
                },
                "Switch window",
                false,
            ),
            (
                r"^show (?:the )?desktop$",
                Action::Window {
                    op: WindowOp::ShowDesktop,
                },
                "Show desktop",
                false,
            ),
        ];
        rules
            .into_iter()
            .map(|(pattern, action, summary, confirm)| (ci(pattern), action, summary, confirm))
            .collect()
    });
    RULES
        .iter()
        .find(|(pattern, ..)| pattern.is_match(text))
        .map(|(_, action, summary, confirm)| plan(action.clone(), *summary, *confirm))
}

fn plan_keys_and_text(text: &str, trailing: &str) -> Option<Plan> {
    for (pattern, keys) in SHORTCUT_PHRASES.iter() {
        if pattern.is_match(text) {
            let confirm = KeyCombo::parse(keys).is_some_and(|c| c.is_disruptive());
            return Some(plan(
                Action::PressKeys {
                    keys: (*keys).to_string(),
                },
                format!("Press {keys}"),
                confirm,
            ));
        }
    }
    if let Some(caps) = PRESS.captures(text) {
        let keys = parse_keys(caps.get(1).map_or("", |m| m.as_str()))?;
        let confirm = KeyCombo::parse(&keys).is_some_and(|c| c.is_disruptive());
        return Some(plan(
            Action::PressKeys { keys: keys.clone() },
            format!("Press {keys}"),
            confirm,
        ));
    }
    let typed = TYPE.captures(text)?;
    let content = format!("{}{trailing}", typed.get(1).map_or("", |m| m.as_str()));
    Some(plan(
        Action::TypeText {
            text: content.clone(),
        },
        format!("Type \"{content}\""),
        false,
    ))
}

/// Turn a spoken command into a structured plan, or `None` if it is not recognized.
pub fn plan_command(utterance: &str) -> Option<Plan> {
    let (text, trailing) = normalize(utterance);
    if text.is_empty() {
        return None;
    }
    plan_email(&text)
        .or_else(|| plan_whatsapp(&text))
        .or_else(|| plan_keys_and_text(&text, &trailing))
        .or_else(|| plan_system(&text))
        .or_else(|| plan_open_or_search(&text))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(said: &str) -> Option<Action> {
        plan_command(said).map(|p| p.action)
    }

    fn confirms(said: &str) -> Option<bool> {
        plan_command(said).map(|p| p.confirm)
    }

    fn app(name: &str) -> Option<Action> {
        Some(Action::OpenApp { name: name.into() })
    }

    fn url(link: &str) -> Option<Action> {
        Some(Action::OpenUrl { url: link.into() })
    }

    fn search(query: &str, engine: SearchEngine) -> Option<Action> {
        Some(Action::WebSearch {
            query: query.into(),
            engine,
        })
    }

    fn keys(combo: &str) -> Option<Action> {
        Some(Action::PressKeys { keys: combo.into() })
    }

    #[test]
    fn opens_apps_and_strips_wake_words_and_politeness() {
        assert_eq!(action("Hey computer, please open Notepad."), app("notepad"));
        assert_eq!(action("launch chrome"), app("chrome"));
        assert_eq!(action("Hey Murmur, open calculator"), app("calculator"));
    }

    #[test]
    fn opens_urls_including_spoken_dots() {
        assert_eq!(action("open github dot com"), url("https://github.com/"));
        assert_eq!(action("go to youtube.com"), url("https://youtube.com/"));
    }

    #[test]
    fn plans_web_youtube_and_maps_searches() {
        assert_eq!(
            action("search for best pizza near me"),
            search("best pizza near me", SearchEngine::Google)
        );
        assert_eq!(
            action("search youtube for lo-fi beats"),
            search("lo-fi beats", SearchEngine::Youtube)
        );
        assert_eq!(
            action("directions to the airport"),
            search("the airport", SearchEngine::Maps)
        );
    }

    #[test]
    fn handles_volume_and_media() {
        assert_eq!(
            action("turn the volume up"),
            Some(Action::Volume { op: VolumeOp::Up })
        );
        assert_eq!(
            action("make it quieter"),
            Some(Action::Volume { op: VolumeOp::Down })
        );
        assert_eq!(action("mute"), Some(Action::Volume { op: VolumeOp::Mute }));
        assert_eq!(
            action("pause"),
            Some(Action::Media {
                op: MediaOp::PlayPause
            })
        );
        assert_eq!(
            action("next track"),
            Some(Action::Media { op: MediaOp::Next })
        );
    }

    #[test]
    fn parses_key_presses_and_shortcut_phrases() {
        assert_eq!(action("press control shift t"), keys("ctrl+shift+t"));
        assert_eq!(action("select all"), keys("ctrl+a"));
        assert_eq!(action("press page down"), keys("pagedown"));
        assert_eq!(confirms("press alt f4"), Some(true));
    }

    #[test]
    fn rejects_the_windows_key_and_unknown_keys() {
        assert_eq!(action("press windows r"), None);
        assert_eq!(action("press ctrl banana"), None);
    }

    #[test]
    fn preserves_casing_for_typed_text() {
        assert_eq!(
            action("type Hello World"),
            Some(Action::TypeText {
                text: "Hello World".into()
            })
        );
    }

    #[test]
    fn typed_text_drops_the_commands_full_stop_but_keeps_question_marks() {
        assert_eq!(
            action("Type hello world."),
            Some(Action::TypeText {
                text: "hello world".into()
            })
        );
        assert_eq!(
            action("Type hello, how are you?"),
            Some(Action::TypeText {
                text: "hello, how are you?".into()
            })
        );
    }

    #[test]
    fn drafts_an_email_with_spoken_address_subject_and_body() {
        let plan = plan_command(
            "send an email to john at example dot com about lunch saying let's meet at noon",
        )
        .unwrap();
        assert_eq!(
            plan.action,
            Action::ComposeEmail {
                to: Some("john@example.com".into()),
                subject: Some("lunch".into()),
                body: Some("let's meet at noon".into()),
            }
        );
        assert!(!plan.confirm);
    }

    #[test]
    fn leaves_the_recipient_empty_when_no_address_was_spoken() {
        match action("write an email to Sarah saying hi") {
            Some(Action::ComposeEmail { to, body, .. }) => {
                assert_eq!(to, None);
                assert_eq!(body.as_deref(), Some("hi"));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn drafts_a_whatsapp_message() {
        assert_eq!(
            action("send a whatsapp to 555 123 4567 saying running late"),
            Some(Action::Whatsapp {
                phone: Some("5551234567".into()),
                text: "running late".into()
            })
        );
    }

    #[test]
    fn asks_for_confirmation_before_closing_windows() {
        assert_eq!(confirms("close window"), Some(true));
    }

    #[test]
    fn returns_none_for_unrecognized_or_empty_input() {
        assert_eq!(action("the quick brown fox"), None);
        assert_eq!(action("   "), None);
    }

    #[test]
    fn handles_punctuation_the_speech_model_adds_around_commands() {
        assert_eq!(action("Open Notepad, please."), app("notepad"));
        assert_eq!(action("Please, open Notepad."), app("notepad"));
        assert_eq!(action("Okay, computer, open Notepad."), app("notepad"));
    }

    #[test]
    fn confirms_before_launching_shells() {
        assert_eq!(confirms("open terminal"), Some(true));
        assert_eq!(confirms("open powershell"), Some(true));
        assert_eq!(confirms("open notepad"), Some(false));
    }

    #[test]
    fn confirms_every_alt_f4_variant_and_shift_delete() {
        assert_eq!(confirms("press control alt f4"), Some(true));
        assert_eq!(confirms("press shift delete"), Some(true));
        assert_eq!(confirms("press control c"), Some(false));
    }

    #[test]
    fn understands_more_volume_phrasings_and_not_moon_youtube() {
        assert_eq!(
            action("turn up the volume"),
            Some(Action::Volume { op: VolumeOp::Up })
        );
        assert_eq!(
            action("turn down the volume"),
            Some(Action::Volume { op: VolumeOp::Down })
        );
        assert_eq!(action("play moon youtube"), None);
    }

    #[test]
    fn understands_natural_phrasing() {
        let cases: Vec<(&str, Option<Action>)> = vec![
            ("Open up Chrome.", app("chrome")),
            ("Launch Google Chrome.", app("google chrome")),
            ("Open the calculator.", app("calculator")),
            ("Open my downloads folder.", app("downloads")),
            ("Can you open YouTube?", url("https://www.youtube.com/")),
            ("Open Gmail.", url("https://mail.google.com/")),
            (
                "Open Chrome and search for cats.",
                search("cats", SearchEngine::Google),
            ),
            (
                "Search Google for the weather in Paris.",
                search("the weather in Paris", SearchEngine::Google),
            ),
            (
                "Google the weather.",
                search("the weather", SearchEngine::Google),
            ),
            ("Open a new tab.", keys("ctrl+t")),
            ("Close this tab.", keys("ctrl+w")),
            ("Take a screenshot.", keys("printscreen")),
            ("Scroll down.", keys("pagedown")),
            ("Skip this song.", Some(Action::Media { op: MediaOp::Next })),
            (
                "Minimize this window.",
                Some(Action::Window {
                    op: WindowOp::Minimize,
                }),
            ),
            (
                "Email john at example dot com saying hi.",
                Some(Action::ComposeEmail {
                    to: Some("john@example.com".into()),
                    subject: None,
                    body: Some("hi".into()),
                }),
            ),
        ];
        for (said, expected) in cases {
            assert_eq!(action(said), expected, "phrase: {said}");
        }
    }

    #[test]
    fn handles_what_the_speech_model_really_writes() {
        // Measured output of the model on spoken test sentences.
        assert_eq!(
            action("Open Chrome and search for Best Pizza Near Me."),
            search("Best Pizza Near Me", SearchEngine::Google)
        );
        assert_eq!(action("Open GitHub.com"), url("https://github.com/"));
        assert_eq!(
            action("Turn the volume up."),
            Some(Action::Volume { op: VolumeOp::Up })
        );
        assert_eq!(
            action("Send an email to john at example.com about lunch saying let's meet at noon."),
            Some(Action::ComposeEmail {
                to: Some("john@example.com".into()),
                subject: Some("lunch".into()),
                body: Some("let's meet at noon".into()),
            })
        );
    }

    #[test]
    fn summaries_describe_what_will_happen() {
        assert_eq!(
            plan_command("open notepad").unwrap().summary,
            "Open notepad"
        );
        assert_eq!(
            plan_command("press control shift t").unwrap().summary,
            "Press ctrl+shift+t"
        );
    }
}
