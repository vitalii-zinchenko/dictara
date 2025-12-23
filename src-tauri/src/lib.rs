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
mod updater;

pub fn run() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| setup::setup_app(app))
        .invoke_handler(tauri::generate_handler![
            tauri_commands::check_accessibility_permission,
            tauri_commands::request_accessibility_permission,
            tauri_commands::restart_app,
            tauri_commands::stop_recording,
            tauri_commands::cancel_recording,
            // App configuration
            tauri_commands::load_app_config,
            tauri_commands::save_app_config,
            // OpenAI provider
            tauri_commands::load_openai_config,
            tauri_commands::save_openai_config,
            tauri_commands::delete_openai_config,
            tauri_commands::test_openai_config,
            // Azure OpenAI provider
            tauri_commands::load_azure_openai_config,
            tauri_commands::save_azure_openai_config,
            tauri_commands::delete_azure_openai_config,
            tauri_commands::test_azure_openai_config,
            // Audio
            tauri_commands::register_audio_level_channel,
            // Error handling
            tauri_commands::retry_transcription,
            tauri_commands::dismiss_error,
            tauri_commands::resize_popup_for_error,
            // Updater
            updater::check_for_updates
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
