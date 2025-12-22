mod clients;
mod clipboard_paste;
mod config;
mod error;
mod keyboard_listener;
mod keychain;
mod recording;
mod setup;
mod sound_player;
mod tauri_commands;
mod ui;

pub fn run() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| setup::setup_app(app))
        .invoke_handler(tauri::generate_handler![
            tauri_commands::check_accessibility_permission,
            tauri_commands::request_accessibility_permission,
            tauri_commands::restart_app,
            tauri_commands::stop_recording,
            tauri_commands::cancel_recording,
            // Provider configuration
            tauri_commands::load_provider_config,
            tauri_commands::save_provider_config,
            // OpenAI
            tauri_commands::save_openai_key,
            tauri_commands::load_openai_key,
            tauri_commands::delete_openai_key,
            tauri_commands::test_openai_key,
            // Azure
            tauri_commands::save_azure_key,
            tauri_commands::load_azure_key,
            tauri_commands::delete_azure_key,
            tauri_commands::test_azure_key,
            // Audio
            tauri_commands::register_audio_level_channel,
            // Error handling
            tauri_commands::retry_transcription,
            tauri_commands::dismiss_error,
            tauri_commands::resize_popup_for_error
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
