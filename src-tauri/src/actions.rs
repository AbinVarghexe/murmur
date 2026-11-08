//! Carries out a planned voice command. The plan comes from `murmur-core`; this is the part
//! that talks to Windows or macOS. User text is never placed in a shell command line: apps
//! are launched with argument lists, links go through the system opener after a scheme check,
//! and keys are sent as key events.

use crate::insert;
use enigo::{Direction, Enigo, Key as EKey, Keyboard, Settings};
use murmur_core::keys::{Key, KeyCombo};
use murmur_core::planner::{Action, MediaOp, VolumeOp, WindowOp};
use murmur_core::urls::{is_safe_external_url, mailto_url, search_url, whatsapp_url};
use std::thread::sleep;
use std::time::Duration;

const VOLUME_STEPS: usize = 5;
const KEY_GAP: Duration = Duration::from_millis(12);

/// How people say app names versus how they are installed.
const SPOKEN_NAMES: &[(&str, &str)] = &[
    ("vs code", "visual studio code"),
    ("vscode", "visual studio code"),
    ("code", "visual studio code"),
    ("word document", "word"),
    ("microsoft word", "word"),
    ("microsoft excel", "excel"),
    ("microsoft edge", "edge"),
    ("google chrome", "chrome"),
    ("chat gpt", "chatgpt"),
    ("chatgpt", "chatgpt"),
    ("chat gbt", "chatgpt"),
    ("chat-gpt", "chatgpt"),
    ("anti gravity", "antigravity"),
    ("antigravity", "antigravity"),
    ("fig ma", "figma"),
    ("figma", "figma"),
];

/// Run one action and describe what happened.
pub fn run(action: &Action) -> Result<String, String> {
    match action {
        Action::OpenApp { name } => {
            crate::apps::open(&resolve_spoken_name(name)).map(|_| format!("Opened {name}"))
        }
        Action::OpenUrl { url } => open_link(url).map(|_| format!("Opened {url}")),
        Action::WebSearch { query, engine } => {
            open_link(&search_url(*engine, query)).map(|_| format!("Searching for \"{query}\""))
        }
        Action::ComposeEmail { to, subject, body } => open_link(&mailto_url(
            to.as_deref(),
            subject.as_deref(),
            body.as_deref(),
        ))
        .map(|_| "Opened an email draft. Review it, then send.".to_string()),
        Action::Whatsapp { phone, text } => open_link(&whatsapp_url(text, phone.as_deref()))
            .map(|_| "Opened WhatsApp. Review it, then send.".to_string()),
        Action::PressKeys { keys } => {
            let combo = KeyCombo::parse(keys)
                .ok_or_else(|| format!("Unsupported key combination: {keys}"))?;
            press_combo(&combo).map(|_| format!("Pressed {keys}"))
        }
        Action::TypeText { text } => insert::paste_text(text).map(|_| "Typed text".to_string()),
        Action::Volume { op } => volume(*op).map(|_| format!("Volume {}", volume_word(*op))),
        Action::Media { op } => media(*op).map(|_| "Media key sent".to_string()),
        Action::Window { op } => window(*op).map(|_| "Done".to_string()),
    }
}

fn resolve_spoken_name(name: &str) -> String {
    SPOKEN_NAMES
        .iter()
        .find(|(spoken, _)| *spoken == name)
        .map_or_else(
            || name.to_string(),
            |(_, installed)| (*installed).to_string(),
        )
}

fn open_link(url: &str) -> Result<(), String> {
    if url.chars().any(char::is_control) || !is_safe_external_url(url) {
        return Err("Blocked an unsafe link".into());
    }
    open::that(url).map_err(|e| format!("Could not open the link: {e}"))
}

fn keyboard() -> Result<Enigo, String> {
    Enigo::new(&Settings::default()).map_err(|e| format!("Cannot send keys: {e}"))
}

fn click(keyboard: &mut Enigo, key: EKey) -> Result<(), String> {
    keyboard
        .key(key, Direction::Click)
        .map_err(|e| format!("Key failed: {e}"))?;
    sleep(KEY_GAP);
    Ok(())
}

fn map_key(key: Key) -> EKey {
    match key {
        Key::Char(c) => EKey::Unicode(c),
        Key::Enter => EKey::Return,
        Key::Tab => EKey::Tab,
        Key::Escape => EKey::Escape,
        Key::Backspace => EKey::Backspace,
        Key::Delete => EKey::Delete,
        Key::Space => EKey::Space,
        Key::Up => EKey::UpArrow,
        Key::Down => EKey::DownArrow,
        Key::Left => EKey::LeftArrow,
        Key::Right => EKey::RightArrow,
        Key::Home => EKey::Home,
        Key::End => EKey::End,
        Key::PageUp => EKey::PageUp,
        Key::PageDown => EKey::PageDown,
        #[cfg(windows)]
        Key::PrintScreen => EKey::PrintScr,
        #[cfg(not(windows))]
        Key::PrintScreen => EKey::F13,
        Key::F(n) => function_key(n),
    }
}

fn function_key(n: u8) -> EKey {
    match n {
        1 => EKey::F1,
        2 => EKey::F2,
        3 => EKey::F3,
        4 => EKey::F4,
        5 => EKey::F5,
        6 => EKey::F6,
        7 => EKey::F7,
        8 => EKey::F8,
        9 => EKey::F9,
        10 => EKey::F10,
        11 => EKey::F11,
        _ => EKey::F12,
    }
}

/// Hold the modifiers, click the key, then always release the modifiers.
fn press_combo(combo: &KeyCombo) -> Result<(), String> {
    let mut keyboard = keyboard()?;
    // The combos are written with Ctrl (copy = ctrl+c); on a Mac the same shortcuts use Command.
    let control = if cfg!(target_os = "macos") {
        EKey::Meta
    } else {
        EKey::Control
    };
    let mut held: Vec<EKey> = Vec::new();
    if combo.ctrl {
        held.push(control);
    }
    if combo.alt {
        held.push(EKey::Alt);
    }
    if combo.shift {
        held.push(EKey::Shift);
    }
    for modifier in &held {
        keyboard
            .key(*modifier, Direction::Press)
            .map_err(|e| format!("Key failed: {e}"))?;
    }
    let result = keyboard
        .key(map_key(combo.key), Direction::Click)
        .map_err(|e| format!("Key failed: {e}"));
    for modifier in held.iter().rev() {
        let _ = keyboard.key(*modifier, Direction::Release);
    }
    result
}

fn volume_word(op: VolumeOp) -> &'static str {
    match op {
        VolumeOp::Up => "up",
        VolumeOp::Down => "down",
        VolumeOp::Mute => "muted",
    }
}

fn volume(op: VolumeOp) -> Result<(), String> {
    let mut keyboard = keyboard()?;
    match op {
        VolumeOp::Mute => click(&mut keyboard, EKey::VolumeMute),
        VolumeOp::Up => (0..VOLUME_STEPS).try_for_each(|_| click(&mut keyboard, EKey::VolumeUp)),
        VolumeOp::Down => {
            (0..VOLUME_STEPS).try_for_each(|_| click(&mut keyboard, EKey::VolumeDown))
        }
    }
}

fn media(op: MediaOp) -> Result<(), String> {
    let mut keyboard = keyboard()?;
    let key = match op {
        MediaOp::PlayPause => EKey::MediaPlayPause,
        MediaOp::Next => EKey::MediaNextTrack,
        MediaOp::Previous => EKey::MediaPrevTrack,
    };
    click(&mut keyboard, key)
}

/// Window commands as the keyboard shortcuts each system already has.
fn window(op: WindowOp) -> Result<(), String> {
    let mut keyboard = keyboard()?;
    let mac = cfg!(target_os = "macos");
    let chord = |keyboard: &mut Enigo, modifiers: &[EKey], key: EKey| -> Result<(), String> {
        for modifier in modifiers {
            keyboard
                .key(*modifier, Direction::Press)
                .map_err(|e| format!("Key failed: {e}"))?;
        }
        let result = keyboard
            .key(key, Direction::Click)
            .map_err(|e| format!("Key failed: {e}"));
        for modifier in modifiers.iter().rev() {
            let _ = keyboard.key(*modifier, Direction::Release);
        }
        result
    };
    match (op, mac) {
        (WindowOp::Minimize, true) => chord(&mut keyboard, &[EKey::Meta], EKey::Unicode('m')),
        (WindowOp::Maximize, true) => chord(
            &mut keyboard,
            &[EKey::Control, EKey::Meta],
            EKey::Unicode('f'),
        ),
        (WindowOp::Close, true) => chord(&mut keyboard, &[EKey::Meta], EKey::Unicode('w')),
        (WindowOp::Switch, true) => chord(&mut keyboard, &[EKey::Meta], EKey::Tab),
        (WindowOp::ShowDesktop, true) => click(&mut keyboard, EKey::F11),
        (WindowOp::Minimize, false) => {
            chord(&mut keyboard, &[EKey::Alt], EKey::Space)?;
            sleep(Duration::from_millis(120));
            click(&mut keyboard, EKey::Unicode('n'))
        }
        (WindowOp::Maximize, false) => {
            chord(&mut keyboard, &[EKey::Alt], EKey::Space)?;
            sleep(Duration::from_millis(120));
            click(&mut keyboard, EKey::Unicode('x'))
        }
        (WindowOp::Close, false) => chord(&mut keyboard, &[EKey::Alt], EKey::F4),
        (WindowOp::Switch, false) => chord(&mut keyboard, &[EKey::Alt], EKey::Tab),
        (WindowOp::ShowDesktop, false) => chord(&mut keyboard, &[EKey::Meta], EKey::Unicode('d')),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_spoken_app_names_to_installed_names() {
        assert_eq!(resolve_spoken_name("vs code"), "visual studio code");
        assert_eq!(resolve_spoken_name("google chrome"), "chrome");
        assert_eq!(resolve_spoken_name("notepad"), "notepad");
    }

    #[test]
    fn refuses_unsafe_links() {
        assert!(open_link("file:///C:/Windows/System32/calc.exe").is_err());
        assert!(open_link("javascript:alert(1)").is_err());
    }

    #[test]
    fn rejects_unsupported_key_combinations_without_sending_anything() {
        let action = Action::PressKeys {
            keys: "win+r".into(),
        };
        assert!(run(&action).unwrap_err().contains("Unsupported"));
    }
}
