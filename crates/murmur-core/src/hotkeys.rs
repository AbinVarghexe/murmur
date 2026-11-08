//! What counts as a valid hotkey. A hotkey is either a chord ("Control+Alt+Space",
//! registered with the operating system) or one key on its own ("RightAlt", watched by a
//! keyboard hook). The Fn key is not offered: laptop firmware handles it and Windows and
//! macOS never see it.

use regex::Regex;
use std::sync::LazyLock;

pub const INVALID_HOTKEY_MESSAGE: &str =
    "Use Ctrl, Alt or Shift plus a key, or a single key such as Right Alt";
const MAX_HOTKEY_LENGTH: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SingleKey {
    RightAlt,
    RightControl,
    RightShift,
    CapsLock,
    ScrollLock,
    /// F13 to F24.
    F(u8),
}

impl SingleKey {
    pub fn parse(name: &str) -> Option<SingleKey> {
        match name {
            "RightAlt" => Some(SingleKey::RightAlt),
            "RightControl" => Some(SingleKey::RightControl),
            "RightShift" => Some(SingleKey::RightShift),
            "CapsLock" => Some(SingleKey::CapsLock),
            "ScrollLock" => Some(SingleKey::ScrollLock),
            _ => {
                let number = name.strip_prefix('F')?.parse::<u8>().ok()?;
                (13..=24).contains(&number).then_some(SingleKey::F(number))
            }
        }
    }

    /// The name stored in settings.
    pub fn name(&self) -> String {
        match self {
            SingleKey::RightAlt => "RightAlt".into(),
            SingleKey::RightControl => "RightControl".into(),
            SingleKey::RightShift => "RightShift".into(),
            SingleKey::CapsLock => "CapsLock".into(),
            SingleKey::ScrollLock => "ScrollLock".into(),
            SingleKey::F(n) => format!("F{n}"),
        }
    }

    /// Friendly text for the interface.
    pub fn label(&self) -> String {
        match self {
            SingleKey::RightAlt => "Right Alt".into(),
            SingleKey::RightControl => "Right Ctrl".into(),
            SingleKey::RightShift => "Right Shift".into(),
            SingleKey::CapsLock => "Caps Lock".into(),
            SingleKey::ScrollLock => "Scroll Lock".into(),
            SingleKey::F(n) => format!("F{n}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hotkey {
    Chord(String),
    Single(SingleKey),
}

static CHORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?:(?:Control|Ctrl|Alt|Option|Shift|Super|Command|Cmd|CommandOrControl|CmdOrCtrl)\+)+(?:[A-Za-z0-9]|F(?:[1-9]|1[0-9]|2[0-4])|Space|Up|Down|Left|Right|Return|Enter|Tab|Backspace|Delete|Home|End)$",
    )
    .expect("static pattern is valid")
});

/// Parse a stored hotkey, rejecting anything that is not a supported chord or single key.
pub fn parse_hotkey(spec: &str) -> Result<Hotkey, String> {
    if spec.len() > MAX_HOTKEY_LENGTH {
        return Err(INVALID_HOTKEY_MESSAGE.to_string());
    }
    if let Some(single) = SingleKey::parse(spec) {
        return Ok(Hotkey::Single(single));
    }
    if CHORD.is_match(spec) {
        return Ok(Hotkey::Chord(spec.to_string()));
    }
    Err(INVALID_HOTKEY_MESSAGE.to_string())
}

/// A chord in one comparable form: modifiers in a fixed order, then the key, lowercase.
/// `CommandOrControl` becomes `cmdorctrl` so it can be checked against both meanings.
fn canonical(spec: &str) -> String {
    let mut parts: Vec<&str> = spec.split('+').collect();
    let key = parts.pop().unwrap_or("").to_lowercase();
    let has = |names: &[&str]| {
        parts
            .iter()
            .any(|p| names.iter().any(|n| p.eq_ignore_ascii_case(n)))
    };
    let mut out = Vec::new();
    if has(&["control", "ctrl"]) {
        out.push("ctrl");
    }
    if has(&["commandorcontrol", "cmdorctrl"]) {
        out.push("cmdorctrl");
    }
    if has(&["alt", "option"]) {
        out.push("alt");
    }
    if has(&["shift"]) {
        out.push("shift");
    }
    if has(&["command", "cmd", "super"]) {
        out.push("cmd");
    }
    out.push(&key);
    out.join("+")
}

/// Everyday editing and window shortcuts. As global hotkeys they would break those keys in
/// every app.
const RESERVED: &[&str] = &[
    "ctrl+a",
    "ctrl+c",
    "ctrl+v",
    "ctrl+x",
    "ctrl+z",
    "ctrl+y",
    "ctrl+s",
    "ctrl+w",
    "ctrl+f",
    "ctrl+n",
    "ctrl+t",
    "ctrl+p",
    "ctrl+r",
    "alt+f4",
    "alt+tab",
    "ctrl+shift+escape",
    "cmd+a",
    "cmd+c",
    "cmd+v",
    "cmd+x",
    "cmd+z",
    "cmd+s",
    "cmd+w",
    "cmd+f",
    "cmd+n",
    "cmd+t",
    "cmd+p",
    "cmd+r",
    "cmd+q",
    "cmd+h",
    "cmd+m",
    "cmd+space",
    "cmd+tab",
];

pub fn is_reserved(spec: &str) -> bool {
    let canonical = canonical(spec);
    if canonical.contains("cmdorctrl") {
        let as_ctrl = canonical.replace("cmdorctrl", "ctrl");
        let as_cmd = canonical.replace("cmdorctrl", "cmd");
        return RESERVED.contains(&as_ctrl.as_str()) || RESERVED.contains(&as_cmd.as_str());
    }
    RESERVED.contains(&canonical.as_str())
}

/// True when two hotkey strings mean the same key combination.
pub fn same_hotkey(a: &str, b: &str) -> bool {
    match (SingleKey::parse(a), SingleKey::parse(b)) {
        (Some(x), Some(y)) => x == y,
        (None, None) => canonical(a) == canonical(b),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_chords_with_a_modifier() {
        assert_eq!(
            parse_hotkey("Control+Alt+Space"),
            Ok(Hotkey::Chord("Control+Alt+Space".into()))
        );
        assert!(parse_hotkey("Control+Shift+D").is_ok());
        assert!(parse_hotkey("Alt+F9").is_ok());
        assert!(parse_hotkey("Command+Shift+K").is_ok());
        assert!(parse_hotkey("CommandOrControl+Alt+Space").is_ok());
    }

    #[test]
    fn accepts_single_keys() {
        assert_eq!(
            parse_hotkey("RightAlt"),
            Ok(Hotkey::Single(SingleKey::RightAlt))
        );
        assert_eq!(
            parse_hotkey("CapsLock"),
            Ok(Hotkey::Single(SingleKey::CapsLock))
        );
        assert_eq!(parse_hotkey("F13"), Ok(Hotkey::Single(SingleKey::F(13))));
        assert_eq!(parse_hotkey("F24"), Ok(Hotkey::Single(SingleKey::F(24))));
    }

    #[test]
    fn rejects_a_lone_letter_left_alt_fn_and_made_up_names() {
        for bad in [
            "A", "LeftAlt", "Fn", "RightWin", "F12", "F25", "", "Control", "Control+", "Alt+Foo",
        ] {
            assert!(parse_hotkey(bad).is_err(), "should reject {bad:?}");
        }
    }

    #[test]
    fn rejects_overlong_values() {
        assert!(parse_hotkey(&"Control+".repeat(10)).is_err());
    }

    #[test]
    fn single_key_names_and_labels_round_trip() {
        for name in [
            "RightAlt",
            "RightControl",
            "RightShift",
            "CapsLock",
            "ScrollLock",
            "F13",
            "F19",
            "F24",
        ] {
            assert_eq!(SingleKey::parse(name).unwrap().name(), name);
        }
        assert_eq!(SingleKey::RightAlt.label(), "Right Alt");
        assert_eq!(SingleKey::F(20).label(), "F20");
    }

    #[test]
    fn flags_shortcuts_that_would_break_everyday_editing() {
        for key in [
            "Control+A",
            "Control+W",
            "Control+C",
            "Alt+F4",
            "Ctrl+V",
            "Command+C",
            "Cmd+Q",
        ] {
            assert!(is_reserved(key), "{key} should be reserved");
        }
        assert!(is_reserved("CommandOrControl+C"));
        assert!(!is_reserved("Control+Alt+Space"));
        assert!(!is_reserved("Control+Shift+D"));
    }

    #[test]
    fn compares_hotkeys_by_meaning() {
        assert!(same_hotkey("Control+Alt+A", "Ctrl+Alt+A"));
        assert!(same_hotkey("Alt+Control+A", "Control+Alt+a"));
        assert!(same_hotkey("RightAlt", "RightAlt"));
        assert!(!same_hotkey("RightAlt", "RightShift"));
        assert!(!same_hotkey("Control+Alt+A", "Control+Alt+B"));
        assert!(!same_hotkey("RightAlt", "Control+Alt+A"));
    }
}
