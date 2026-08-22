use secrecy::SecretString;
use serde::Serialize;

use crate::clients::{ApiConfig, Transcriber};
use crate::config::{AzureOpenAIConfig, Provider};
use crate::keychain::{self, ProviderAccount};
use log::error;

/// Frontend-facing status for Azure OpenAI provider (never exposes API key)
#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AzureOpenAIConfigStatus {
    pub configured: bool,
    pub endpoint: String,
}

// ===== AZURE OPENAI PROVIDER COMMANDS =====

#[tauri::command]
#[specta::specta]
pub fn load_azure_openai_config() -> Result<Option<AzureOpenAIConfigStatus>, String> {
    let config = keychain::load_provider_config::<AzureOpenAIConfig>(ProviderAccount::AzureOpenAI)
        .map_err(|e| {
            let err = format!("Failed to load Azure OpenAI config: {}", e);
            error!("{}", err);
            err
        })?;

    Ok(config.map(|c| AzureOpenAIConfigStatus {
        configured: true,
        endpoint: c.endpoint,
    }))
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
        api_key: SecretString::from(api_key),
        endpoint,
        openai_model: crate::config::OpenAITranscriptionModel::default(),
    };

    Transcriber::test_api_key(&config).map_err(|e| {
        let err = format!("Failed to test Azure OpenAI config: {}", e);
        error!("{}", err);
        err
    })
}
