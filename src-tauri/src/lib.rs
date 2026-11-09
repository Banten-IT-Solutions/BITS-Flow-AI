mod commands;
mod persona;

use persona::PersonaState;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be the first plugin: a second process forwards here and exits.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Restore path for the "hide to background" close behavior:
            // relaunching the app reveals the existing manager window.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(PersonaState::default())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // "Hide to background" close behavior for the manager window.
                if window.label() == "main"
                    && commands::load_settings(window.app_handle())
                        .is_ok_and(|s| s.close_behavior == commands::CloseBehavior::Hide)
                {
                    api.prevent_close();
                    let _ = window.hide();
                } else if window.label().starts_with("persona-") {
                    // Persona windows close freely; tell the UI so the open
                    // badge and LRU state stay accurate.
                    let _ = window
                        .app_handle()
                        .emit("persona-window-closed", window.label());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_accounts,
            commands::save_accounts,
            commands::list_providers,
            commands::save_providers,
            commands::list_prompts,
            commands::save_prompts,
            commands::get_settings,
            commands::save_settings,
            commands::log_activity,
            commands::list_recent,
            commands::clear_persona_data,
            commands::clear_all_persona_data,
            commands::install_deb_update,
            commands::restart_app,
            commands::export_file,
            commands::import_file,
            persona::open_persona,
            persona::switch_persona_provider,
            persona::persona_state,
            persona::list_open_personas,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
