//! The commands the interface can call. Each one checks which window is asking, so the
//! floating pill cannot change settings and settings cannot answer a confirmation.

use crate::shell::Shell;
use crate::{model, windows};
use murmur_core::memory::{LearningResult, MemoryKind, VoiceMemory};
use murmur_core::settings::Settings;
use serde::Serialize;
use std::sync::Arc;
use tauri::{State, WebviewWindow};

type ShellState<'a> = State<'a, Arc<Shell>>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveResult {
    ok: bool,
    failed: Vec<String>,
    error: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    ready: bool,
    total_bytes: u64,
    downloading: bool,
}

#[derive(Serialize)]
pub struct DownloadResult {
    ok: bool,
    error: Option<String>,
}

fn from_settings(window: &WebviewWindow) -> bool {
    window.label() == windows::SETTINGS
}

#[tauri::command]
pub fn get_settings(window: WebviewWindow, shell: ShellState<'_>) -> Option<Settings> {
    from_settings(&window).then(|| shell.settings())
}

#[tauri::command]
pub fn save_settings(
    window: WebviewWindow,
    shell: ShellState<'_>,
    settings: Settings,
) -> SaveResult {
    let rejected = |error: String| SaveResult {
        ok: false,
        failed: Vec::new(),
        error,
    };
    if !from_settings(&window) {
        return rejected("Unavailable".into());
    }
    if let Err(error) = settings.validate() {
        return rejected(error);
    }

    let previous = shell.settings();
    shell.apply_settings(settings);
    let failed = shell.apply_hotkeys();
    if !failed.is_empty() {
        shell.apply_settings(previous);
        shell.apply_hotkeys();
        let error = format!(
            "Could not register {}. Another app may be using it.",
            failed.join(", ")
        );
        return SaveResult {
            ok: false,
            failed,
            error,
        };
    }
    match shell.persist_settings() {
        Ok(()) => SaveResult {
            ok: true,
            failed: Vec::new(),
            error: String::new(),
        },
        Err(error) => rejected(error),
    }
}

#[tauri::command]
pub fn model_status(window: WebviewWindow, shell: ShellState<'_>) -> Option<ModelStatus> {
    from_settings(&window).then(|| ModelStatus {
        ready: shell.model_ready(),
        total_bytes: model::total_bytes(),
        downloading: shell.is_downloading(),
    })
}

#[tauri::command]
pub async fn download_model(
    window: WebviewWindow,
    shell: ShellState<'_>,
) -> Result<DownloadResult, ()> {
    if !from_settings(&window) {
        return Ok(DownloadResult {
            ok: false,
            error: Some("Unavailable".into()),
        });
    }
    let shell = Arc::clone(&shell);
    let result = tauri::async_runtime::spawn_blocking(move || shell.download_model()).await;
    Ok(match result {
        Ok(Ok(())) => DownloadResult {
            ok: true,
            error: None,
        },
        Ok(Err(error)) => DownloadResult {
            ok: false,
            error: Some(error),
        },
        Err(_) => DownloadResult {
            ok: false,
            error: Some("The download stopped unexpectedly".into()),
        },
    })
}

#[tauri::command]
pub fn confirm_action(window: WebviewWindow, shell: ShellState<'_>, approved: bool) {
    if window.label() == windows::PILL {
        shell.confirm(approved);
    }
}

#[tauri::command]
pub fn get_custom_apps(
    window: WebviewWindow,
    shell: ShellState<'_>,
) -> Vec<crate::apps::CustomApp> {
    if !from_settings(&window) {
        return Vec::new();
    }
    crate::apps::load_custom_apps(shell.data_dir())
}

#[tauri::command]
pub fn save_custom_apps(
    window: WebviewWindow,
    shell: ShellState<'_>,
    apps: Vec<crate::apps::CustomApp>,
) -> Result<(), String> {
    if !from_settings(&window) {
        return Err("Unavailable".into());
    }
    crate::apps::save_custom_apps(shell.data_dir(), &apps)
}

#[tauri::command]
pub fn launch_custom_app(window: WebviewWindow, target: String) -> Result<(), String> {
    if !from_settings(&window) {
        return Err("Unavailable".into());
    }
    crate::apps::platform::launch(&target)
}

fn parse_memory_kind(kind: &str) -> Result<MemoryKind, String> {
    match kind {
        "person" => Ok(MemoryKind::Person),
        "phrase" => Ok(MemoryKind::Phrase),
        "vocabulary" => Ok(MemoryKind::Vocabulary),
        _ => Err("Unknown voice memory category".into()),
    }
}

#[tauri::command]
pub fn get_voice_memory(window: WebviewWindow, shell: ShellState<'_>) -> Option<VoiceMemory> {
    from_settings(&window).then(|| shell.voice_memory())
}

#[tauri::command]
pub fn learn_voice_correction(
    window: WebviewWindow,
    shell: ShellState<'_>,
    original: String,
    corrected: String,
    context: String,
) -> Result<LearningResult, String> {
    if !from_settings(&window) {
        return Err("Unavailable".into());
    }
    if original.trim().is_empty() || corrected.trim().is_empty() {
        return Err("Both the original and corrected transcript are required".into());
    }
    Ok(shell.learn_voice_correction(&original, &corrected, &context))
}

#[tauri::command]
pub fn add_voice_memory_entry(
    window: WebviewWindow,
    shell: ShellState<'_>,
    canonical: String,
    aliases: Vec<String>,
    kind: String,
    id: Option<String>,
) -> Result<(), String> {
    if !from_settings(&window) {
        return Err("Unavailable".into());
    }
    shell.add_voice_memory_entry(
        &canonical,
        &aliases,
        parse_memory_kind(&kind)?,
        id.as_deref(),
    )
}

#[tauri::command]
pub fn delete_voice_memory_entry(
    window: WebviewWindow,
    shell: ShellState<'_>,
    id: String,
) -> Result<bool, String> {
    if !from_settings(&window) {
        return Err("Unavailable".into());
    }
    Ok(shell.delete_voice_memory_entry(&id))
}

#[tauri::command]
pub fn delete_voice_correction(
    window: WebviewWindow,
    shell: ShellState<'_>,
    id: String,
) -> Result<bool, String> {
    if !from_settings(&window) {
        return Err("Unavailable".into());
    }
    Ok(shell.delete_voice_correction(&id))
}

#[tauri::command]
pub fn clear_voice_memory(window: WebviewWindow, shell: ShellState<'_>) -> Result<(), String> {
    if !from_settings(&window) {
        return Err("Unavailable".into());
    }
    shell.clear_voice_memory()
}

#[tauri::command]
pub fn start_voice_training(window: WebviewWindow, shell: ShellState<'_>) -> Result<(), String> {
    if !from_settings(&window) {
        return Err("Unavailable".into());
    }
    shell.start_voice_training()
}

#[tauri::command]
pub fn stop_voice_training(window: WebviewWindow, shell: ShellState<'_>) -> Result<(), String> {
    if !from_settings(&window) {
        return Err("Unavailable".into());
    }
    shell.stop_voice_training()
}
