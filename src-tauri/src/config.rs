use serde::{Deserialize, Serialize};

/// Provider types supported by the application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAI,
    Azure,
}

/// Configuration for the active AI provider
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    /// Currently enabled provider (only one can be enabled)
    pub enabled_provider: Option<Provider>,

    /// Azure-specific configuration
    pub azure_endpoint: Option<String>,
}

/// Load provider configuration from store
pub fn load_config(store: &tauri_plugin_store::Store<tauri::Wry>) -> ProviderConfig {
    store
        .get("provider_config")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// Save provider configuration to store
pub fn save_config(
    store: &tauri_plugin_store::Store<tauri::Wry>,
    config: &ProviderConfig,
) -> Result<(), String> {
    store.set(
        "provider_config",
        serde_json::to_value(config).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}
