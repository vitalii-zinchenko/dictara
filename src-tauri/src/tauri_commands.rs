use crate::config::{self, Provider, ProviderConfig};
use crate::keychain::{self, KeychainAccount};
use crate::recording::{LastRecordingState, RecordingCommand};
use crate::setup::{AudioLevelChannel, RecordingCommandSender};
use tauri::ipc::Channel;
use tauri::State;
use tauri_plugin_store::StoreExt;

#[tauri::command]
pub fn check_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos_accessibility_client::accessibility::application_is_trusted()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true // Other platforms don't need this permission
    }
}

#[tauri::command]
pub fn request_accessibility_permission() {
    #[cfg(target_os = "macos")]
    {
        // This will show macOS system dialog and open System Settings
        macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
    }
}

#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

#[tauri::command]
pub fn stop_recording(sender: State<RecordingCommandSender>) -> Result<(), String> {
    sender
        .sender
        .blocking_send(RecordingCommand::FnDown)
        .map_err(|e| format!("Failed to send FnDown command: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn cancel_recording(sender: State<RecordingCommandSender>) -> Result<(), String> {
    sender
        .sender
        .blocking_send(RecordingCommand::Cancel)
        .map_err(|e| format!("Failed to send Cancel command: {}", e))?;

    Ok(())
}

// ===== PROVIDER CONFIGURATION COMMANDS =====

#[tauri::command]
pub fn load_provider_config(app: tauri::AppHandle) -> Result<ProviderConfig, String> {
    println!("[Command] load_provider_config called");

    let store = app.store("config.json").map_err(|e| {
        eprintln!("[Command] Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    Ok(config::load_config(&store))
}

#[tauri::command]
pub fn save_provider_config(
    app: tauri::AppHandle,
    enabled_provider: Option<String>,
    azure_endpoint: Option<String>,
) -> Result<(), String> {
    println!("[Command] save_provider_config called");

    let provider = enabled_provider.map(|p| match p.as_str() {
        "openai" => Provider::OpenAI,
        "azure" => Provider::Azure,
        _ => {
            eprintln!("[Command] Invalid provider: {}", p);
            panic!("Invalid provider")
        }
    });

    let config = ProviderConfig {
        enabled_provider: provider,
        azure_endpoint,
    };

    let store = app.store("config.json").map_err(|e| {
        eprintln!("[Command] Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    config::save_config(&store, &config)
}

// ===== OPENAI KEYCHAIN COMMANDS =====

#[tauri::command]
pub fn save_openai_key(key: String) -> Result<(), String> {
    println!(
        "[Command] save_openai_key called with key length: {}",
        key.len()
    );
    keychain::save_api_key(KeychainAccount::OpenAI, &key).map_err(|e| {
        let error = format!("Failed to save OpenAI API key: {}", e);
        eprintln!("[Command] {}", error);
        error
    })
}

#[tauri::command]
pub fn load_openai_key() -> Result<Option<String>, String> {
    println!("[Command] load_openai_key called");
    keychain::load_api_key(KeychainAccount::OpenAI).map_err(|e| {
        let error = format!("Failed to load OpenAI API key: {}", e);
        eprintln!("[Command] {}", error);
        error
    })
}

#[tauri::command]
pub fn delete_openai_key() -> Result<(), String> {
    println!("[Command] delete_openai_key called");
    keychain::delete_api_key(KeychainAccount::OpenAI).map_err(|e| {
        let error = format!("Failed to delete OpenAI API key: {}", e);
        eprintln!("[Command] {}", error);
        error
    })
}

#[tauri::command]
pub fn test_openai_key(key: String) -> Result<bool, String> {
    println!("[Command] test_openai_key called");

    use crate::clients::openai::OpenAIClient;

    OpenAIClient::test_api_key(Provider::OpenAI, &key, None).map_err(|e| {
        let error = format!("Failed to test OpenAI API key: {}", e);
        eprintln!("[Command] {}", error);
        error
    })
}

// ===== AZURE KEYCHAIN COMMANDS =====

#[tauri::command]
pub fn save_azure_key(key: String) -> Result<(), String> {
    println!(
        "[Command] save_azure_key called with key length: {}",
        key.len()
    );
    keychain::save_api_key(KeychainAccount::Azure, &key).map_err(|e| {
        let error = format!("Failed to save Azure API key: {}", e);
        eprintln!("[Command] {}", error);
        error
    })
}

#[tauri::command]
pub fn load_azure_key() -> Result<Option<String>, String> {
    println!("[Command] load_azure_key called");
    keychain::load_api_key(KeychainAccount::Azure).map_err(|e| {
        let error = format!("Failed to load Azure API key: {}", e);
        eprintln!("[Command] {}", error);
        error
    })
}

#[tauri::command]
pub fn delete_azure_key() -> Result<(), String> {
    println!("[Command] delete_azure_key called");
    keychain::delete_api_key(KeychainAccount::Azure).map_err(|e| {
        let error = format!("Failed to delete Azure API key: {}", e);
        eprintln!("[Command] {}", error);
        error
    })
}

#[tauri::command]
pub fn test_azure_key(key: String, endpoint: String) -> Result<bool, String> {
    println!("[Command] test_azure_key called");

    use crate::clients::openai::OpenAIClient;

    OpenAIClient::test_api_key(Provider::Azure, &key, Some(&endpoint)).map_err(|e| {
        let error = format!("Failed to test Azure API key: {}", e);
        eprintln!("[Command] {}", error);
        error
    })
}

// ===== AUDIO LEVEL CHANNEL =====

#[tauri::command]
pub fn register_audio_level_channel(
    channel: Channel<f32>,
    state: State<AudioLevelChannel>,
) -> Result<(), String> {
    let mut channel_lock = state.channel.lock().unwrap();
    *channel_lock = Some(channel);
    Ok(())
}

// ===== ERROR HANDLING =====

#[tauri::command]
pub fn retry_transcription(sender: State<RecordingCommandSender>) -> Result<(), String> {
    println!("[Command] retry_transcription called");

    sender
        .sender
        .blocking_send(RecordingCommand::RetryTranscription)
        .map_err(|e| format!("Failed to send RetryTranscription command: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn dismiss_error(
    app: tauri::AppHandle,
    last_recording_state: State<LastRecordingState>,
) -> Result<(), String> {
    println!("[Command] dismiss_error called");

    // Delete audio file if exists
    if let Ok(mut last_recording) = last_recording_state.lock() {
        if let Some(path) = last_recording.audio_file_path.take() {
            crate::recording::cleanup_recording_file(&path);
        }
        last_recording.audio_file_path = None;
    }

    // Close popup
    crate::ui::window::close_recording_popup(&app)
        .map_err(|e| format!("Failed to close popup: {}", e))
}

#[tauri::command]
pub fn resize_popup_for_error(app: tauri::AppHandle) -> Result<(), String> {
    println!("[Command] resize_popup_for_error called");

    crate::ui::window::resize_recording_popup_for_error(&app)
        .map_err(|e| format!("Failed to resize popup: {}", e))
}
