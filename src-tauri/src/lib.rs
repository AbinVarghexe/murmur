//! Murmur: offline dictation and voice commands.

mod actions;
mod apps;
mod audio;
mod commands;
mod hotkeys;
mod insert;
mod learning;
mod memory;
mod model;
mod shell;
mod target_observer;
mod windows;

use shell::Shell;
use tauri::{Manager, RunEvent};

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            windows::open_settings(app)
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::model_status,
            commands::download_model,
            commands::confirm_action,
            commands::get_custom_apps,
            commands::save_custom_apps,
            commands::launch_custom_app,
            commands::get_voice_memory,
            commands::learn_voice_correction,
            commands::add_voice_memory_entry,
            commands::delete_voice_memory_entry,
            commands::delete_voice_correction,
            commands::clear_voice_memory,
            commands::start_voice_training,
            commands::stop_voice_training,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let shell = Shell::new(app.handle().clone());
            app.manage(shell.clone());
            windows::create_pill(app.handle())?;
            windows::build_tray(app.handle(), &shell)?;

            let failed_hotkeys = shell.apply_hotkeys();
            let ready = shell.model_ready();
            if ready && shell.settings().warm_on_startup {
                shell.warm_engine();
            }
            if !ready || !failed_hotkeys.is_empty() {
                windows::open_settings(app.handle());
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Murmur");

    app.run(|_app, event| {
        // Closing the settings window must not quit the app: it lives in the tray.
        if let RunEvent::ExitRequested { api, code, .. } = event {
            if code.is_none() {
                api.prevent_exit();
            }
        }
    });
}
