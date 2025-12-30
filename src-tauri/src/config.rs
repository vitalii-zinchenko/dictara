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
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// Currently active provider (only one can be active)
    #[serde(alias = "active_provider")]
    pub active_provider: Option<Provider>,
}

/// OpenAI provider configuration (stored in keychain)
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIConfig {
    pub api_key: String,
}

/// Azure OpenAI provider configuration (stored in keychain)
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AzureOpenAIConfig {
    pub api_key: String,
    pub endpoint: String,
}

/// Onboarding step enum - tracks current position in the wizard
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type, Default)]
pub enum OnboardingStep {
    #[default]
    #[serde(rename = "welcome")]
    Welcome,
    #[serde(rename = "accessibility")]
    Accessibility,
    #[serde(rename = "api_keys")]
    ApiKeys,
    #[serde(rename = "fn_hold")]
    FnHold,
    #[serde(rename = "fn_space")]
    FnSpace,
    #[serde(rename = "complete")]
    Complete,
}

/// Onboarding configuration (stored locally)
#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingConfig {
    /// Whether the user has completed or skipped onboarding
    pub finished: bool,
    /// Current step in the onboarding flow
    #[serde(alias = "current_step")]
    pub current_step: OnboardingStep,
    /// Flag to track if we're resuming after an accessibility restart
    #[serde(alias = "pending_restart")]
    pub pending_restart: bool,
}

/// Load app configuration from store
pub fn load_app_config(store: &tauri_plugin_store::Store<tauri::Wry>) -> AppConfig {
    // Try camelCase first, fall back to legacy snake_case
    store
        .get("appConfig")
        .or_else(|| store.get("app_config"))
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// Save app configuration to store
pub fn save_app_config(
    store: &tauri_plugin_store::Store<tauri::Wry>,
    config: &AppConfig,
) -> Result<(), String> {
    store.set(
        "appConfig",
        serde_json::to_value(config).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Load onboarding configuration from store
pub fn load_onboarding_config(store: &tauri_plugin_store::Store<tauri::Wry>) -> OnboardingConfig {
    // Try camelCase first, fall back to legacy snake_case
    store
        .get("onboardingConfig")
        .or_else(|| store.get("onboarding_config"))
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// Save onboarding configuration to store
pub fn save_onboarding_config(
    store: &tauri_plugin_store::Store<tauri::Wry>,
    config: &OnboardingConfig,
) -> Result<(), String> {
    store.set(
        "onboardingConfig",
        serde_json::to_value(config).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}
