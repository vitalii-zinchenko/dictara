mod clients;
mod clipboard_paste;
mod error;
mod events;
mod keyboard_listener;
mod recording;
mod setup;
mod tauri_commands;
mod ui;

pub fn run() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .setup(|app| {
            return setup::setup_app(app);
        })
        .invoke_handler(tauri::generate_handler![
            tauri_commands::check_accessibility_permission,
            tauri_commands::request_accessibility_permission,
            tauri_commands::restart_app,
            tauri_commands::stop_recording,
            tauri_commands::cancel_recording
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
