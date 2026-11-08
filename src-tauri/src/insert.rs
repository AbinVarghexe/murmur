//! Typing text into whatever app has focus: put it on the clipboard, press Paste, then put
//! the previous clipboard text back. Non-text clipboard contents (images, files) cannot be
//! restored, so they are replaced.

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::thread::sleep;
use std::time::Duration;

/// Time for the target app to read the clipboard before it is restored.
const RESTORE_DELAY: Duration = Duration::from_millis(700);

#[cfg(target_os = "macos")]
const PASTE_MODIFIER: Key = Key::Meta;
#[cfg(not(target_os = "macos"))]
const PASTE_MODIFIER: Key = Key::Control;

fn press_paste() -> Result<(), String> {
    let mut keyboard =
        Enigo::new(&Settings::default()).map_err(|e| format!("Cannot send keys: {e}"))?;
    let fail = |e: enigo::InputError| format!("Could not paste into the active app: {e}");
    keyboard
        .key(PASTE_MODIFIER, Direction::Press)
        .map_err(fail)?;
    let pasted = keyboard.key(Key::Unicode('v'), Direction::Click);
    // Always let go of the modifier, even if the paste key failed.
    let released = keyboard.key(PASTE_MODIFIER, Direction::Release);
    pasted.map_err(fail)?;
    released.map_err(fail)
}

/// Paste `text` at the cursor of the focused app.
pub fn paste_text(text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }
    let mut clipboard = Clipboard::new().map_err(|e| format!("Cannot use the clipboard: {e}"))?;
    let previous = clipboard.get_text().ok();
    set_private(&mut clipboard, text)?;

    let result = press_paste();
    sleep(RESTORE_DELAY);

    // Only touch the clipboard if nothing else has replaced our text in the meantime.
    if clipboard.get_text().ok().as_deref() == Some(text) {
        match previous {
            Some(previous) => {
                let _ = set_private(&mut clipboard, &previous);
            }
            // The old contents could not be read (an image, files): don't leave dictation behind.
            None => {
                let _ = clipboard.clear();
            }
        }
    }
    result
}

/// Put text on the clipboard, keeping it out of Windows clipboard history and cloud sync.
fn set_private(clipboard: &mut Clipboard, text: &str) -> Result<(), String> {
    let request = clipboard.set();
    #[cfg(target_os = "windows")]
    let request = {
        use arboard::SetExtWindows;
        request.exclude_from_history()
    };
    request
        .text(text.to_owned())
        .map_err(|e| format!("Cannot use the clipboard: {e}"))
}
