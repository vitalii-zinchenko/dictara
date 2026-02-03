use secrecy::SecretString;

use crate::clients::{ApiConfig, Transcriber};
use crate::config::{OpenAIConfig, Provider};
use crate::keychain::{self, ProviderAccount};
use log::error;

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
        api_key: SecretString::from(api_key),
        endpoint: String::new(),
    };

    Transcriber::test_api_key(&config).map_err(|e| {
        let err = format!("Failed to test OpenAI config: {}", e);
        error!("{}", err);
        err
    })
}
