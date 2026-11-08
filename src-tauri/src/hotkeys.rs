//! Registers the two hotkeys. Chords ("Control+Alt+Space") go through the operating system's
//! global-shortcut API. Single keys ("RightAlt") are watched by a keyboard listener that only
//! looks for the configured keys: nothing else you type is read, stored or logged.

use crate::shell::Shell;
use murmur_core::controller::Mode;
use murmur_core::hotkeys::{parse_hotkey, Hotkey, SingleKey};
use murmur_core::settings::Settings;
use rdev::{EventType, Key};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

type Bindings = Vec<(SingleKey, Mode)>;

#[derive(Default)]
pub struct Hotkeys {
    bindings: Arc<Mutex<Bindings>>,
    listener_started: AtomicBool,
}

/// Map a key reported by the listener to one of the supported single keys.
fn to_single(key: Key) -> Option<SingleKey> {
    match key {
        Key::AltGr => Some(SingleKey::RightAlt),
        Key::ControlRight => Some(SingleKey::RightControl),
        Key::ShiftRight => Some(SingleKey::RightShift),
        Key::CapsLock => Some(SingleKey::CapsLock),
        Key::ScrollLock => Some(SingleKey::ScrollLock),
        Key::Unknown(code) => function_key(code),
        _ => None,
    }
}

/// F13 to F24 are reported as raw key codes, which differ per system.
#[cfg(windows)]
fn function_key(code: u32) -> Option<SingleKey> {
    (124..=135)
        .contains(&code)
        .then(|| SingleKey::F((code - 124 + 13) as u8))
}

#[cfg(target_os = "macos")]
fn function_key(code: u32) -> Option<SingleKey> {
    const CODES: [(u32, u8); 8] = [
        (105, 13),
        (107, 14),
        (113, 15),
        (106, 16),
        (64, 17),
        (79, 18),
        (80, 19),
        (90, 20),
    ];
    CODES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, n)| SingleKey::F(*n))
}

#[cfg(not(any(windows, target_os = "macos")))]
fn function_key(_code: u32) -> Option<SingleKey> {
    None
}

/// Pressing Alt on its own makes Windows focus the app's menu bar when it is released, which
/// would swallow the paste that follows. Any other key pressed while Alt is held cancels that,
/// so an unused F24 is tapped.
#[cfg(windows)]
fn guard_alt_menu() {
    let _ = rdev::simulate(&EventType::KeyPress(Key::Unknown(135)));
    let _ = rdev::simulate(&EventType::KeyRelease(Key::Unknown(135)));
}

#[cfg(not(windows))]
fn guard_alt_menu() {}

impl Hotkeys {
    /// Apply the hotkeys from `settings`. Returns the ones that could not be registered.
    pub fn register(&self, shell: &Arc<Shell>, settings: &Settings) -> Vec<String> {
        let shortcuts = shell.app.global_shortcut();
        let _ = shortcuts.unregister_all();

        let mut failed = Vec::new();
        let mut singles: Bindings = Vec::new();
        let entries = [
            (Mode::Dictation, &settings.dictation_hotkey),
            (Mode::Agent, &settings.agent_hotkey),
        ];
        for (mode, spec) in entries {
            match parse_hotkey(spec) {
                Ok(Hotkey::Single(key)) => singles.push((key, mode)),
                Ok(Hotkey::Chord(chord)) => {
                    let target = Arc::clone(shell);
                    let registered =
                        shortcuts.on_shortcut(chord.as_str(), move |_app, _shortcut, event| {
                            if event.state() == ShortcutState::Pressed {
                                target.toggle(mode);
                            }
                        });
                    if registered.is_err() {
                        failed.push(spec.clone());
                    }
                }
                Err(_) => failed.push(spec.clone()),
            }
        }

        if !singles.is_empty() && !self.ensure_listener(shell) {
            failed.extend(singles.iter().map(|(key, _)| key.name()));
            singles.clear();
        }
        *self.bindings.lock().unwrap_or_else(|p| p.into_inner()) = singles;
        failed
    }

    /// Start the keyboard listener once. Returns false if it cannot run.
    fn ensure_listener(&self, shell: &Arc<Shell>) -> bool {
        if self.listener_started.swap(true, Ordering::SeqCst) {
            return true;
        }
        let (tx, rx) = mpsc::channel::<(SingleKey, bool)>();
        let spawned = std::thread::Builder::new()
            .name("murmur-key-listener".into())
            .spawn(move || {
                // Blocks for the life of the app. Only the supported single keys are forwarded.
                let _ = rdev::listen(move |event| {
                    let (key, down) = match event.event_type {
                        EventType::KeyPress(key) => (key, true),
                        EventType::KeyRelease(key) => (key, false),
                        _ => return,
                    };
                    if let Some(single) = to_single(key) {
                        let _ = tx.send((single, down));
                    }
                });
            });
        if spawned.is_err() {
            self.listener_started.store(false, Ordering::SeqCst);
            return false;
        }

        let bindings = Arc::clone(&self.bindings);
        let shell = Arc::clone(shell);
        let dispatcher = std::thread::Builder::new()
            .name("murmur-key-dispatch".into())
            .spawn(move || {
                let mut held: HashSet<SingleKey> = HashSet::new();
                for (key, down) in rx {
                    let snapshot: Bindings =
                        bindings.lock().unwrap_or_else(|p| p.into_inner()).clone();
                    let mode = snapshot
                        .iter()
                        .find(|(bound, _)| *bound == key)
                        .map(|(_, mode)| *mode);
                    if down {
                        let Some(mode) = mode else { continue };
                        if !held.insert(key) {
                            continue; // key auto-repeat
                        }
                        shell.key_held(true);
                        if key == SingleKey::RightAlt
                            && !snapshot.iter().any(|(k, _)| *k == SingleKey::F(24))
                        {
                            guard_alt_menu();
                        }
                        shell.key_down(mode);
                    } else if held.remove(&key) {
                        // Always balance the held count, even if the key was unbound while held.
                        shell.key_held(false);
                        if let Some(mode) = mode {
                            shell.key_up(mode);
                        }
                    }
                }
            });
        if dispatcher.is_err() {
            self.listener_started.store(false, Ordering::SeqCst);
            return false;
        }
        true
    }
}
