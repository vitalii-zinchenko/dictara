use serde::{Deserialize, Serialize};

/// Provider types supported by the application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
pub enum Provider {
    #[serde(rename = "open_ai", alias = "openai", alias = "open_a_i")]
    OpenAI,
    #[serde(
        rename = "azure_open_ai",
        alias = "azure",
        alias = "azure_openai",
        alias = "azure_open_a_i"
    )]
    AzureOpenAI,
}

/// App configuration (stored locally)
#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
pub struct AppConfig {
    /// Currently active provider (only one can be active)
    pub active_provider: Option<Provider>,
}

/// OpenAI provider configuration (stored in keychain)
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OpenAIConfig {
    pub api_key: String,
}

/// Azure OpenAI provider configuration (stored in keychain)
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AzureOpenAIConfig {
    pub api_key: String,
    pub endpoint: String,
}

/// Load app configuration from store
pub fn load_app_config(store: &tauri_plugin_store::Store<tauri::Wry>) -> AppConfig {
    store
        .get("app_config")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// Save app configuration to store
pub fn save_app_config(
    store: &tauri_plugin_store::Store<tauri::Wry>,
    config: &AppConfig,
) -> Result<(), String> {
    store.set(
        "app_config",
        serde_json::to_value(config).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}
