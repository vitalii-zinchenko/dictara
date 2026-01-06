use crate::clients::{ApiConfig, Transcriber};
use crate::config::{
    self, AppConfig, AzureOpenAIConfig, OnboardingConfig, OnboardingStep, OpenAIConfig, Provider,
    RecordingTrigger,
};
use crate::keychain::{self, ProviderAccount};
use crate::recording::{LastRecordingState, RecordingCommand};
use crate::setup::{AudioLevelChannel, RecordingCommandSender};
use crate::ui::window;
use log::error;
use tauri::ipc::Channel;
use tauri::State;
use tauri_plugin_store::StoreExt;

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
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
#[specta::specta]
pub fn stop_recording(sender: State<RecordingCommandSender>) -> Result<(), String> {
    sender
        .sender
        .blocking_send(RecordingCommand::StopRecording)
        .map_err(|e| format!("Failed to send StopRecording command: {}", e))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn cancel_recording(sender: State<RecordingCommandSender>) -> Result<(), String> {
    sender
        .sender
        .blocking_send(RecordingCommand::Cancel)
        .map_err(|e| format!("Failed to send Cancel command: {}", e))?;

    Ok(())
}

// ===== APP CONFIGURATION COMMANDS =====

#[tauri::command]
#[specta::specta]
pub fn load_app_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    Ok(config::load_app_config(&store))
}

#[tauri::command]
#[specta::specta]
pub fn save_app_config(
    app: tauri::AppHandle,
    active_provider: Option<String>,
    recording_trigger: Option<RecordingTrigger>,
) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    // Load existing config to preserve fields that aren't being updated
    let mut config = config::load_app_config(&store);

    // Update provider if specified
    if let Some(p) = active_provider {
        config.active_provider = Some(match p.as_str() {
            "open_ai" | "openai" => Provider::OpenAI,
            "azure_open_ai" | "azure_openai" | "azure" => Provider::AzureOpenAI,
            _ => {
                error!("Invalid provider: {}", p);
                return Err(format!("Invalid provider: {}", p));
            }
        });
    }

    // Update recording trigger if specified
    if let Some(trigger) = recording_trigger {
        config.recording_trigger = trigger;
    }

    config::save_app_config(&store, &config)
}

// ===== OPENAI PROVIDER COMMANDS =====

#[tauri::command]
#[specta::specta]
pub fn load_openai_config() -> Result<Option<OpenAIConfig>, String> {
    keychain::load_provider_config::<OpenAIConfig>(ProviderAccount::OpenAI).map_err(|e| {
        let err = format!("Failed to load OpenAI config: {}", e);
        error!("{}", err);
        err
    })
}

#[tauri::command]
#[specta::specta]
pub fn save_openai_config(api_key: String) -> Result<(), String> {
    let config = OpenAIConfig { api_key };

    keychain::save_provider_config(ProviderAccount::OpenAI, &config).map_err(|e| {
        let err = format!("Failed to save OpenAI config: {}", e);
        error!("{}", err);
        err
    })
}

#[tauri::command]
#[specta::specta]
pub fn delete_openai_config() -> Result<(), String> {
    keychain::delete_provider_config(ProviderAccount::OpenAI).map_err(|e| {
        let err = format!("Failed to delete OpenAI config: {}", e);
        error!("{}", err);
        err
    })
}

#[tauri::command]
#[specta::specta]
pub fn test_openai_config(api_key: String) -> Result<bool, String> {
    let config = ApiConfig {
        provider: Provider::OpenAI,
        api_key,
        endpoint: String::new(),
    };

    Transcriber::test_api_key(&config).map_err(|e| {
        let err = format!("Failed to test OpenAI config: {}", e);
        error!("{}", err);
        err
    })
}

// ===== AZURE OPENAI PROVIDER COMMANDS =====

#[tauri::command]
#[specta::specta]
pub fn load_azure_openai_config() -> Result<Option<AzureOpenAIConfig>, String> {
    keychain::load_provider_config::<AzureOpenAIConfig>(ProviderAccount::AzureOpenAI).map_err(|e| {
        let err = format!("Failed to load Azure OpenAI config: {}", e);
        error!("{}", err);
        err
    })
}

#[tauri::command]
#[specta::specta]
pub fn save_azure_openai_config(api_key: String, endpoint: String) -> Result<(), String> {
    let config = AzureOpenAIConfig { api_key, endpoint };

    keychain::save_provider_config(ProviderAccount::AzureOpenAI, &config).map_err(|e| {
        let err = format!("Failed to save Azure OpenAI config: {}", e);
        error!("{}", err);
        err
    })
}

#[tauri::command]
#[specta::specta]
pub fn delete_azure_openai_config() -> Result<(), String> {
    keychain::delete_provider_config(ProviderAccount::AzureOpenAI).map_err(|e| {
        let err = format!("Failed to delete Azure OpenAI config: {}", e);
        error!("{}", err);
        err
    })
}

#[tauri::command]
#[specta::specta]
pub fn test_azure_openai_config(api_key: String, endpoint: String) -> Result<bool, String> {
    let config = ApiConfig {
        provider: Provider::AzureOpenAI,
        api_key,
        endpoint,
    };

    Transcriber::test_api_key(&config).map_err(|e| {
        let err = format!("Failed to test Azure OpenAI config: {}", e);
        error!("{}", err);
        err
    })
}

// ===== AUDIO LEVEL CHANNEL =====

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
pub fn retry_transcription(sender: State<RecordingCommandSender>) -> Result<(), String> {
    sender
        .sender
        .blocking_send(RecordingCommand::RetryTranscription)
        .map_err(|e| format!("Failed to send RetryTranscription command: {}", e))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn dismiss_error(
    app: tauri::AppHandle,
    last_recording_state: State<LastRecordingState>,
) -> Result<(), String> {
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
#[specta::specta]
pub fn resize_popup_for_error(app: tauri::AppHandle) -> Result<(), String> {
    crate::ui::window::resize_recording_popup_for_error(&app)
        .map_err(|e| format!("Failed to resize popup: {}", e))
}

// ===== ONBOARDING COMMANDS =====

#[tauri::command]
#[specta::specta]
pub fn load_onboarding_config(app: tauri::AppHandle) -> Result<OnboardingConfig, String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    Ok(config::load_onboarding_config(&store))
}

#[tauri::command]
#[specta::specta]
pub fn save_onboarding_step(app: tauri::AppHandle, step: OnboardingStep) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    let mut onboarding_config = config::load_onboarding_config(&store);
    onboarding_config.current_step = step;
    config::save_onboarding_config(&store, &onboarding_config)
}

#[tauri::command]
#[specta::specta]
pub fn finish_onboarding(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    let mut onboarding_config = config::load_onboarding_config(&store);
    onboarding_config.finished = true;
    onboarding_config.current_step = OnboardingStep::Complete;
    onboarding_config.pending_restart = false;
    config::save_onboarding_config(&store, &onboarding_config)?;

    // Close the onboarding window
    window::close_onboarding_window(&app).map_err(|e| {
        error!("Failed to close onboarding window: {}", e);
        format!("Failed to close onboarding window: {}", e)
    })
}

#[tauri::command]
#[specta::specta]
pub fn skip_onboarding(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    let mut onboarding_config = config::load_onboarding_config(&store);
    onboarding_config.finished = true;
    config::save_onboarding_config(&store, &onboarding_config)?;

    // Close the onboarding window
    window::close_onboarding_window(&app).map_err(|e| {
        error!("Failed to close onboarding window: {}", e);
        format!("Failed to close onboarding window: {}", e)
    })
}

#[tauri::command]
#[specta::specta]
pub fn set_pending_restart(app: tauri::AppHandle, pending: bool) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    let mut onboarding_config = config::load_onboarding_config(&store);
    onboarding_config.pending_restart = pending;
    config::save_onboarding_config(&store, &onboarding_config)
}

#[tauri::command]
#[specta::specta]
pub fn restart_onboarding(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    // Reset onboarding config to initial state
    let onboarding_config = config::OnboardingConfig::default();
    config::save_onboarding_config(&store, &onboarding_config)?;

    // Open the onboarding window
    crate::ui::window::open_onboarding_window(&app).map_err(|e| {
        error!("Failed to open onboarding window: {}", e);
        format!("Failed to open onboarding window: {}", e)
    })
}
