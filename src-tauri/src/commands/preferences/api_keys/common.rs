use crate::config::{self, ConfigKey, ConfigStore, Provider};
use log::error;
use tauri::State;

// ===== PROVIDER SELECTION COMMANDS =====

/// Get the currently active provider
#[tauri::command]
#[specta::specta]
pub fn get_current_provider(
    config_store: State<config::Config>,
) -> Result<Option<Provider>, String> {
    let config = config_store.get(&ConfigKey::APP).unwrap_or_default();
    Ok(config.active_provider)
}

/// Set the currently active provider
#[tauri::command]
#[specta::specta]
pub fn set_current_provider(
    config_store: State<config::Config>,
    provider: String,
) -> Result<(), String> {
    // Load existing config to preserve other fields
    let mut config = config_store.get(&ConfigKey::APP).unwrap_or_default();

    // Parse and set the provider
    config.active_provider = Some(match provider.as_str() {
        "open_ai" | "openai" => Provider::OpenAI,
        "azure_open_ai" | "azure_openai" | "azure" => Provider::AzureOpenAI,
        "local" => Provider::Local,
        "open_router" | "openrouter" => Provider::OpenRouter,
        "custom_endpoint" | "custom_url" | "customEndpoint" => Provider::CustomEndpoint,
        _ => {
            error!("Invalid provider: {}", provider);
            return Err(format!("Invalid provider: {}", provider));
        }
    });

    config_store.set(&ConfigKey::APP, config)
}

/// Clear the currently active provider (set to None)
#[tauri::command]
#[specta::specta]
pub fn clear_current_provider(config_store: State<config::Config>) -> Result<(), String> {
    // Load existing config to preserve other fields
    let mut config = config_store.get(&ConfigKey::APP).unwrap_or_default();
    config.active_provider = None;

    config_store.set(&ConfigKey::APP, config)
}
