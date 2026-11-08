//! Opening an installed app by the name a person said.
//!
//! The name is matched against custom registered applications, what is actually installed
//! (Start Menu shortcuts & WindowsApps on Windows, application folders on macOS), using
//! `murmur_core::app_match`. The user's words are only ever passed as a program argument,
//! never through a shell.

use murmur_core::app_match::best_match;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomApp {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub target: String,
}

pub fn default_custom_apps() -> Vec<CustomApp> {
    vec![
        CustomApp {
            id: "chatgpt".into(),
            name: "ChatGPT".into(),
            aliases: vec![
                "chat gpt".into(),
                "chatgpt".into(),
                "chat-gpt".into(),
                "chat gbt".into(),
                "gpt".into(),
                "chat gpt app".into(),
            ],
            target: "chatgpt-desktop.exe".into(),
        },
        CustomApp {
            id: "antigravity".into(),
            name: "Antigravity".into(),
            aliases: vec![
                "anti gravity".into(),
                "antigravity".into(),
                "antigravity ide".into(),
            ],
            target: "antigravity.exe".into(),
        },
        CustomApp {
            id: "figma".into(),
            name: "Figma".into(),
            aliases: vec!["fig ma".into(), "figma".into()],
            target: "figma.exe".into(),
        },
    ]
}

pub fn custom_apps_path(data_dir: &Path) -> PathBuf {
    data_dir.join("custom_apps.json")
}

pub fn load_custom_apps(data_dir: &Path) -> Vec<CustomApp> {
    let path = custom_apps_path(data_dir);
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(apps) = serde_json::from_str::<Vec<CustomApp>>(&content) {
            return apps;
        }
    }
    let defaults = default_custom_apps();
    let _ = save_custom_apps(data_dir, &defaults);
    defaults
}

pub fn save_custom_apps(data_dir: &Path, apps: &[CustomApp]) -> Result<(), String> {
    let _ = std::fs::create_dir_all(data_dir);
    let path = custom_apps_path(data_dir);
    let json = serde_json::to_string_pretty(apps).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| format!("Cannot save custom apps: {e}"))
}

/// Open the app best matching `name`, checking custom registered apps first.
pub fn open_with_data_dir(name: &str, data_dir: &Path) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() || name.starts_with('-') {
        return Err("No app name given".into());
    }

    // 1. Check custom registered apps
    let custom_apps = load_custom_apps(data_dir);
    for app in &custom_apps {
        let mut candidates = vec![app.name.clone()];
        candidates.extend(app.aliases.clone());
        if best_match(name, &candidates).is_some() {
            return platform::launch(&app.target);
        }
    }

    // 2. Check platform shortcuts & aliases
    platform::open(name)
}

/// Open the app best matching `name`.
pub fn open(name: &str) -> Result<(), String> {
    let data_dir = std::env::var_os("MURMUR_DATA_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            #[cfg(windows)]
            {
                std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("Murmur"))
            }
            #[cfg(not(windows))]
            {
                std::env::var_os("HOME").map(|d| PathBuf::from(d).join(".config/Murmur"))
            }
        })
        .unwrap_or_else(|| std::env::temp_dir().join("Murmur"));
    open_with_data_dir(name, &data_dir)
}

#[cfg(windows)]
pub mod platform {
    use super::*;
    use std::os::windows::process::CommandExt;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const MAX_DEPTH: usize = 4;

    /// Things that open directly: a program, a settings page or a known folder.
    const ALIASES: &[(&str, &str)] = &[
        ("notepad", "notepad.exe"),
        ("calculator", "calc.exe"),
        ("calc", "calc.exe"),
        ("paint", "mspaint.exe"),
        ("explorer", "explorer.exe"),
        ("file explorer", "explorer.exe"),
        ("files", "explorer.exe"),
        ("task manager", "taskmgr.exe"),
        ("terminal", "wt.exe"),
        ("command prompt", "cmd.exe"),
        ("settings", "ms-settings:"),
        ("downloads", "shell:Downloads"),
        ("documents", "shell:Personal"),
        ("desktop", "shell:Desktop"),
        ("pictures", "shell:My Pictures"),
        ("chatgpt", "chatgpt-desktop.exe"),
        ("chat gpt", "chatgpt-desktop.exe"),
        ("chat-gpt", "chatgpt-desktop.exe"),
        ("chat gbt", "chatgpt-desktop.exe"),
        ("chatgpt app", "chatgpt-desktop.exe"),
        ("antigravity", "antigravity.exe"),
        ("anti gravity", "antigravity.exe"),
        ("figma", "figma.exe"),
        ("fig ma", "figma.exe"),
    ];

    pub fn open(name: &str) -> Result<(), String> {
        let lower = name.to_lowercase();
        if let Some((_, target)) = ALIASES.iter().find(|(spoken, _)| *spoken == lower) {
            return launch(target);
        }
        if let Some(shortcut) = find_shortcut(&lower) {
            return launch(&shortcut.to_string_lossy());
        }
        if let Some(app_id) = find_store_app(&lower) {
            return launch(&format!("shell:AppsFolder\\{app_id}"));
        }
        // Fallback for ChatGPT URL protocol
        if lower.contains("chat") && (lower.contains("gpt") || lower.contains("gbt")) {
            return launch("chatgpt:");
        }
        Err(format!("No app named \"{name}\" found"))
    }

    pub fn launch(target: &str) -> Result<(), String> {
        if target.starts_with("http://") || target.starts_with("https://") || target.ends_with(':')
        {
            return open::that(target).map_err(|e| format!("Could not open {target}: {e}"));
        }

        let is_program = target.ends_with(".exe") && !target.contains(['\\', '/']);
        if is_program && target != "explorer.exe" {
            // First check if target is in %LOCALAPPDATA%\Microsoft\WindowsApps
            if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
                let windows_apps_exe = PathBuf::from(local_app_data)
                    .join("Microsoft")
                    .join("WindowsApps")
                    .join(target);
                if windows_apps_exe.exists() {
                    let mut cmd = Command::new(&windows_apps_exe);
                    cmd.creation_flags(CREATE_NO_WINDOW);
                    if cmd.spawn().is_ok() {
                        return Ok(());
                    }
                }
            }

            // Next try direct spawn
            let mut cmd = Command::new(target);
            if cmd.spawn().is_ok() {
                return Ok(());
            }
        }

        // Check if file path directly exists
        let path = Path::new(target);
        if path.exists() {
            if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("lnk"))
            {
                return open::that(path).map_err(|e| format!("Could not open shortcut: {e}"));
            }
            let mut cmd = Command::new(target);
            if cmd.spawn().is_ok() {
                return Ok(());
            }
        }

        // Fallback to explorer.exe
        let mut explorer = Command::new("explorer.exe");
        explorer.arg(target);
        explorer
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Could not start {target}: {e}"))
    }

    fn shortcut_and_app_folders() -> Vec<PathBuf> {
        let mut folders = Vec::new();
        for variable in ["ProgramData", "APPDATA"] {
            if let Some(base) = std::env::var_os(variable) {
                folders.push(PathBuf::from(base).join(r"Microsoft\Windows\Start Menu\Programs"));
            }
        }
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let base = PathBuf::from(local);
            folders.push(base.join(r"Microsoft\WindowsApps"));
            folders.push(base.join(r"Programs"));
        }
        folders
    }

    fn collect_shortcuts(folder: &Path, depth: usize, found: &mut Vec<(String, PathBuf)>) {
        let Ok(entries) = std::fs::read_dir(folder) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if depth < MAX_DEPTH {
                    collect_shortcuts(&path, depth + 1, found);
                }
            } else if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("lnk") || e.eq_ignore_ascii_case("exe"))
            {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    found.push((stem.to_string(), path.clone()));
                }
            }
        }
    }

    fn find_shortcut(spoken: &str) -> Option<PathBuf> {
        let mut shortcuts = Vec::new();
        for folder in shortcut_and_app_folders() {
            collect_shortcuts(&folder, 0, &mut shortcuts);
        }
        let names: Vec<String> = shortcuts.iter().map(|(name, _)| name.clone()).collect();
        best_match(spoken, &names).map(|index| shortcuts[index].1.clone())
    }

    /// Store apps (Calculator, Photos…) have no shortcut file, so ask Windows for its list.
    fn find_store_app(spoken: &str) -> Option<String> {
        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command"])
            .arg("Get-StartApps | ForEach-Object { $_.Name + \"`t\" + $_.AppID }")
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        let apps: Vec<(String, String)> = text
            .lines()
            .filter_map(|line| line.split_once('\t'))
            .map(|(name, id)| (name.trim().to_string(), id.trim().to_string()))
            .collect();
        let names: Vec<String> = apps.iter().map(|(name, _)| name.clone()).collect();
        best_match(spoken, &names).map(|index| apps[index].1.clone())
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    const ALIASES: &[(&str, &str)] = &[
        ("calculator", "Calculator"),
        ("calc", "Calculator"),
        ("notepad", "TextEdit"),
        ("notes", "Notes"),
        ("settings", "System Settings"),
        ("terminal", "Terminal"),
        ("finder", "Finder"),
        ("files", "Finder"),
        ("explorer", "Finder"),
        ("file explorer", "Finder"),
        ("task manager", "Activity Monitor"),
    ];

    fn application_folders() -> Vec<PathBuf> {
        let mut folders = vec![
            PathBuf::from("/Applications"),
            PathBuf::from("/System/Applications"),
        ];
        if let Some(home) = std::env::var_os("HOME") {
            folders.push(PathBuf::from(home).join("Applications"));
        }
        folders
    }

    fn collect_apps(folder: &Path, depth: usize, found: &mut Vec<(String, PathBuf)>) {
        let Ok(entries) = std::fs::read_dir(folder) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "app") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    found.push((stem.to_string(), path.clone()));
                }
            } else if path.is_dir() && depth < 1 {
                collect_apps(&path, depth + 1, found);
            }
        }
    }

    fn run_open(target: &str) -> Result<(), String> {
        let status = Command::new("open")
            .arg("-a")
            .arg(target)
            .status()
            .map_err(|e| format!("Could not run open: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("No app named \"{target}\" found"))
        }
    }

    pub fn open(name: &str) -> Result<(), String> {
        let lower = name.to_lowercase();
        if let Some((_, app)) = ALIASES.iter().find(|(spoken, _)| *spoken == lower) {
            return run_open(app);
        }
        let mut apps = Vec::new();
        for folder in application_folders() {
            collect_apps(&folder, 0, &mut apps);
        }
        let names: Vec<String> = apps.iter().map(|(name, _)| name.clone()).collect();
        match best_match(&lower, &names) {
            Some(index) => run_open(&apps[index].1.to_string_lossy()),
            None => Err(format!("No app named \"{name}\" found")),
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod platform {
    pub fn open(name: &str) -> Result<(), String> {
        Err(format!(
            "Opening apps is not supported on this system: {name}"
        ))
    }
}
