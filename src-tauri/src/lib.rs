mod commands;
mod persona;

use persona::PersonaState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(PersonaState::default())
        .on_window_event(|window, event| {
            // "Hide to background" close behavior for the manager window.
            if let tauri::WindowEvent::CloseRequested { api } = event {
                if window.label() != "main" {
                    return;
                }
                if let Ok(settings) = commands::load_settings(window.app_handle()) {
                    if settings.close_behavior == commands::CloseBehavior::Hide {
                        api.prevent_close();
                        let _ = window.hide();
                    }
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
            persona::open_persona,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
