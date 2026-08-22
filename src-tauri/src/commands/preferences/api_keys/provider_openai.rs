use secrecy::SecretString;
use serde::Serialize;

use crate::clients::{ApiConfig, Transcriber};
use crate::config::{OpenAIConfig, OpenAITranscriptionModel, Provider};
use crate::keychain::{self, ProviderAccount};
use log::error;

/// Frontend-facing status for OpenAI provider (never exposes API key)
#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIConfigStatus {
    pub configured: bool,
    /// Currently selected transcription model (never includes the API key)
    pub model: OpenAITranscriptionModel,
}

// ===== OPENAI PROVIDER COMMANDS =====

#[tauri::command]
#[specta::specta]
pub fn load_openai_config() -> Result<Option<OpenAIConfigStatus>, String> {
    let config =
        keychain::load_provider_config::<OpenAIConfig>(ProviderAccount::OpenAI).map_err(|e| {
            let err = format!("Failed to load OpenAI config: {}", e);
            error!("{}", err);
            err
        })?;

    Ok(config.map(|c| OpenAIConfigStatus {
        configured: true,
        model: c.model,
    }))
}

/// Save the OpenAI configuration.
///
/// `api_key` is optional: when omitted, the already-stored key is kept so the
/// model can be changed without the user re-entering their key. The stored key
/// is never returned to the frontend.
#[tauri::command]
#[specta::specta]
pub fn save_openai_config(
    api_key: Option<String>,
    model: OpenAITranscriptionModel,
) -> Result<(), String> {
    let api_key = match api_key {
        Some(key) => key,
        None => {
            let existing = keychain::load_provider_config::<OpenAIConfig>(ProviderAccount::OpenAI)
                .map_err(|e| {
                    let err = format!("Failed to load OpenAI config: {}", e);
                    error!("{}", err);
                    err
                })?;

            existing
                .ok_or_else(|| {
                    let err = "No stored OpenAI API key to update".to_string();
                    error!("{}", err);
                    err
                })?
                .api_key
        }
    };

    let config = OpenAIConfig { api_key, model };

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

/// Verify that an API key is accepted by OpenAI.
///
/// This checks the credential, not the model: the probe sends a 1-second
/// silent clip, which the GPT-based transcription models reject outright
/// (and which gives the diarization model's VAD nothing to chunk). Whisper
/// accepts it, so it is always used here regardless of the selected model.
/// Authorization is account-wide, so a key valid for Whisper is valid for
/// every transcription model.
#[tauri::command]
#[specta::specta]
pub fn test_openai_config(api_key: String) -> Result<bool, String> {
    let config = ApiConfig {
        provider: Provider::OpenAI,
        api_key: SecretString::from(api_key),
        endpoint: String::new(),
        openai_model: OpenAITranscriptionModel::Whisper1,
    };

    Transcriber::test_api_key(&config).map_err(|e| {
        let err = format!("Failed to test OpenAI config: {}", e);
        error!("{}", err);
        err
    })
}
