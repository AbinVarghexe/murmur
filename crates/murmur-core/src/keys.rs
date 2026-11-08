//! Key combinations for voice commands ("press control shift t"). Only a small, safe set
//! is allowed: no Windows or Command key and no free text, so a spoken phrase can never
//! open Run dialogs or inject arbitrary input.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Tab,
    Escape,
    Backspace,
    Delete,
    Space,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    PrintScreen,
    /// Function key F1 to F12.
    F(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyCombo {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub key: Key,
}

impl KeyCombo {
    /// Parse a normalized combo such as `ctrl+shift+t`. Returns `None` for anything
    /// outside the allowed set.
    pub fn parse(spec: &str) -> Option<KeyCombo> {
        let lowered = spec.to_lowercase();
        let mut parts: Vec<&str> = lowered.split('+').collect();
        let key_name = parts.pop().filter(|k| !k.is_empty())?;
        let (mut ctrl, mut alt, mut shift) = (false, false, false);
        for modifier in parts {
            match modifier {
                "ctrl" => ctrl = true,
                "alt" => alt = true,
                "shift" => shift = true,
                _ => return None,
            }
        }
        Some(KeyCombo {
            ctrl,
            alt,
            shift,
            key: parse_key(key_name)?,
        })
    }

    /// Closing the active app or deleting needs a confirmation before it runs.
    pub fn is_disruptive(&self) -> bool {
        let closes_or_cuts = self.ctrl && matches!(self.key, Key::Char('x' | 'w'));
        let task_manager = self.ctrl && self.shift && self.key == Key::Escape;
        matches!(self.key, Key::Delete | Key::Backspace)
            || (self.key == Key::F(4) && self.alt)
            || closes_or_cuts
            || task_manager
    }
}

fn parse_key(name: &str) -> Option<Key> {
    let named = match name {
        "enter" => Some(Key::Enter),
        "tab" => Some(Key::Tab),
        "escape" => Some(Key::Escape),
        "backspace" => Some(Key::Backspace),
        "delete" => Some(Key::Delete),
        "space" => Some(Key::Space),
        "up" => Some(Key::Up),
        "down" => Some(Key::Down),
        "left" => Some(Key::Left),
        "right" => Some(Key::Right),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "pageup" => Some(Key::PageUp),
        "pagedown" => Some(Key::PageDown),
        "printscreen" => Some(Key::PrintScreen),
        _ => None,
    };
    if named.is_some() {
        return named;
    }
    let mut chars = name.chars();
    if let (Some(c), None) = (chars.next(), chars.next()) {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            return Some(Key::Char(c));
        }
    }
    let number = name.strip_prefix('f')?.parse::<u8>().ok()?;
    (1..=12).contains(&number).then_some(Key::F(number))
}

impl fmt::Display for KeyCombo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts: Vec<String> = Vec::new();
        if self.ctrl {
            parts.push("ctrl".into());
        }
        if self.alt {
            parts.push("alt".into());
        }
        if self.shift {
            parts.push("shift".into());
        }
        parts.push(match self.key {
            Key::Char(c) => c.to_string(),
            Key::Enter => "enter".into(),
            Key::Tab => "tab".into(),
            Key::Escape => "escape".into(),
            Key::Backspace => "backspace".into(),
            Key::Delete => "delete".into(),
            Key::Space => "space".into(),
            Key::Up => "up".into(),
            Key::Down => "down".into(),
            Key::Left => "left".into(),
            Key::Right => "right".into(),
            Key::Home => "home".into(),
            Key::End => "end".into(),
            Key::PageUp => "pageup".into(),
            Key::PageDown => "pagedown".into(),
            Key::PrintScreen => "printscreen".into(),
            Key::F(n) => format!("f{n}"),
        });
        write!(f, "{}", parts.join("+"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modifiers_letters_named_and_function_keys() {
        let combo = KeyCombo::parse("ctrl+shift+t").unwrap();
        assert!(combo.ctrl && combo.shift && !combo.alt);
        assert_eq!(combo.key, Key::Char('t'));
        assert_eq!(KeyCombo::parse("alt+f4").unwrap().key, Key::F(4));
        assert_eq!(KeyCombo::parse("enter").unwrap().key, Key::Enter);
        assert_eq!(KeyCombo::parse("ctrl+pagedown").unwrap().key, Key::PageDown);
        assert_eq!(
            KeyCombo::parse("printscreen").unwrap().key,
            Key::PrintScreen
        );
    }

    #[test]
    fn rejects_unknown_modifiers_and_keys() {
        assert!(KeyCombo::parse("win+r").is_none());
        assert!(KeyCombo::parse("cmd+c").is_none());
        assert!(KeyCombo::parse("ctrl+{").is_none());
        assert!(KeyCombo::parse("ctrl+f13").is_none());
        assert!(KeyCombo::parse("").is_none());
        assert!(KeyCombo::parse("ctrl+").is_none());
    }

    #[test]
    fn flags_disruptive_combinations() {
        assert!(KeyCombo::parse("alt+f4").unwrap().is_disruptive());
        assert!(KeyCombo::parse("ctrl+alt+f4").unwrap().is_disruptive());
        assert!(KeyCombo::parse("shift+delete").unwrap().is_disruptive());
        assert!(!KeyCombo::parse("ctrl+c").unwrap().is_disruptive());
    }

    #[test]
    fn displays_in_the_same_normalized_form() {
        assert_eq!(
            KeyCombo::parse("ctrl+shift+t").unwrap().to_string(),
            "ctrl+shift+t"
        );
        assert_eq!(KeyCombo::parse("alt+left").unwrap().to_string(), "alt+left");
    }
}
