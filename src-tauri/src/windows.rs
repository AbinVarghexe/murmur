//! The two windows (the floating pill and the settings window) and the tray icon.
//!
//! The pill never takes focus, so text is typed into the app you were using. Settings is built
//! when you open it and destroyed when you close it, so it costs no memory while you dictate.

use crate::shell::Shell;
use murmur_core::controller::{Mode, Phase, PillState};
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder,
};

pub const PILL: &str = "pill";
pub const SETTINGS: &str = "settings";

const PILL_WIDTH: f64 = 280.0;
const PILL_HEIGHT: f64 = 48.0;
/// Tall enough for the approval card.
const CONFIRM_HEIGHT: f64 = 210.0;
/// Distance from the bottom edge of the screen, above the taskbar or dock.
const PILL_BOTTOM_MARGIN: f64 = 60.0;
const RESULT_VISIBLE: Duration = Duration::from_millis(2600);
const NOTICE_VISIBLE: Duration = Duration::from_millis(2200);

/// Bumped on every pill update so a late "hide" timer cannot hide a newer message.
static PILL_GENERATION: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize, Clone)]
struct ConfirmRequest<'a> {
    summary: &'a str,
}

pub fn create_pill(app: &AppHandle) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, PILL, WebviewUrl::App("pill.html".into()))
        .title("Murmur")
        .inner_size(PILL_WIDTH, PILL_HEIGHT)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focusable(false)
        .focused(false)
        .visible(false)
        .build()?;
    Ok(())
}

fn place_pill(app: &AppHandle, height: f64) {
    let Some(pill) = app.get_webview_window(PILL) else {
        return;
    };
    let monitor = pill
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten());
    if let Some(monitor) = monitor {
        let scale = monitor.scale_factor();
        let size = monitor.size().to_logical::<f64>(scale);
        let origin = monitor.position().to_logical::<f64>(scale);
        let x = origin.x + (size.width - PILL_WIDTH) / 2.0;
        let y = origin.y + size.height - height - PILL_BOTTOM_MARGIN;
        let _ = pill.set_position(LogicalPosition::new(x, y));
    }
}

/// Show or hide the pill and send it the new state.
pub fn publish_pill(app: &AppHandle, state: &PillState) {
    let Some(pill) = app.get_webview_window(PILL) else {
        return;
    };
    let generation = PILL_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let _ = app.emit_to(PILL, "pill-state", state);

    if state.phase == Phase::Idle {
        let _ = pill.hide();
        return;
    }
    let _ = pill.set_size(LogicalSize::new(PILL_WIDTH, PILL_HEIGHT));
    place_pill(app, PILL_HEIGHT);
    let _ = pill.show();

    if state.phase == Phase::Result || state.phase == Phase::Notice {
        let app = app.clone();
        let timeout = if state.phase == Phase::Notice {
            NOTICE_VISIBLE
        } else {
            RESULT_VISIBLE
        };
        let _ = std::thread::Builder::new()
            .name("murmur-pill-hide".into())
            .spawn(move || {
                std::thread::sleep(timeout);
                if PILL_GENERATION.load(Ordering::SeqCst) == generation {
                    if let Some(pill) = app.get_webview_window(PILL) {
                        let _ = pill.hide();
                    }
                }
            });
    }
}

/// Ask the user before a disruptive command. The pill briefly becomes clickable.
pub fn show_confirm(app: &AppHandle, summary: &str) {
    let Some(pill) = app.get_webview_window(PILL) else {
        return;
    };
    PILL_GENERATION.fetch_add(1, Ordering::SeqCst);
    let _ = pill.set_focusable(true);
    let _ = pill.set_size(LogicalSize::new(PILL_WIDTH, CONFIRM_HEIGHT));
    place_pill(app, CONFIRM_HEIGHT);
    let _ = pill.show();
    let _ = pill.set_focus();
    let _ = app.emit_to(PILL, "pill-confirm", ConfirmRequest { summary });
}

/// Put the pill back to its never-steals-focus state after a confirmation. It is hidden so
/// keyboard focus returns to the app the user was in before the command runs.
pub fn end_confirm(app: &AppHandle) {
    if let Some(pill) = app.get_webview_window(PILL) {
        let _ = pill.set_focusable(false);
        let _ = pill.hide();
    }
}

pub fn open_settings(app: &AppHandle) {
    if let Some(existing) = app.get_webview_window(SETTINGS) {
        let _ = existing.show();
        let _ = existing.unminimize();
        let _ = existing.set_focus();
        return;
    }
    let built = WebviewWindowBuilder::new(app, SETTINGS, WebviewUrl::App("settings.html".into()))
        .title("Murmur")
        .inner_size(820.0, 780.0)
        .min_inner_size(560.0, 520.0)
        .center()
        .build();
    if let Err(error) = built {
        eprintln!("[murmur] could not open settings: {error}");
    }
}

pub fn build_tray(app: &AppHandle, shell: &Arc<Shell>) -> tauri::Result<()> {
    let dictate = MenuItemBuilder::with_id("dictate", "Start dictation").build(app)?;
    let command = MenuItemBuilder::with_id("command", "Voice command").build(app)?;
    let cancel = MenuItemBuilder::with_id("cancel", "Cancel recording").build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit Murmur").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&dictate, &command, &cancel])
        .item(&PredefinedMenuItem::separator(app)?)
        .items(&[&settings, &quit])
        .build()?;

    let menu_shell = Arc::clone(shell);
    let click_app = app.clone();
    let mut tray = TrayIconBuilder::new()
        .tooltip("Murmur")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "dictate" => menu_shell.toggle(Mode::Dictation),
            "command" => menu_shell.toggle(Mode::Agent),
            "cancel" => menu_shell.cancel(),
            "settings" => open_settings(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(move |_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                open_settings(&click_app);
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}
