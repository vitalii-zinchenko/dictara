use crate::config::{
    self, AppConfig, AudioFormat, ConfigKey, ConfigStore, Provider, RecordingTrigger,
};
use log::error;
use tauri::State;

// ===== GENERAL APP CONFIGURATION COMMANDS =====

/// Load the entire app configuration
#[tauri::command]
#[specta::specta]
pub fn load_app_config(config_store: State<config::Config>) -> Result<AppConfig, String> {
    Ok(config_store.get(&ConfigKey::APP).unwrap_or_default())
}

/// Save app configuration (general-purpose command that can update multiple fields)
#[tauri::command]
#[specta::specta]
pub fn save_app_config(
    config_store: State<config::Config>,
    active_provider: Option<String>,
    recording_trigger: Option<RecordingTrigger>,
    audio_format: Option<AudioFormat>,
) -> Result<(), String> {
    // Load existing config to preserve fields that aren't being updated
    let mut config = config_store.get(&ConfigKey::APP).unwrap_or_default();

    // Update provider if specified
    if let Some(p) = active_provider {
        config.active_provider = Some(match p.as_str() {
            "open_ai" | "openai" => Provider::OpenAI,
            "azure_open_ai" | "azure_openai" | "azure" => Provider::AzureOpenAI,
            "local" => Provider::Local,
            "open_router" | "openrouter" => Provider::OpenRouter,
            "custom_endpoint" | "custom_url" | "customEndpoint" => Provider::CustomEndpoint,
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

    // Update audio format if specified
    if let Some(format) = audio_format {
        config.audio_format = format;
    }

    config_store.set(&ConfigKey::APP, config)
}
