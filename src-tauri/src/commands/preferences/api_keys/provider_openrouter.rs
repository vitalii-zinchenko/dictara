use log::{error, warn};
use secrecy::SecretString;
use serde::Serialize;

use crate::clients::{OpenRouterTranscriber, TranscriptionError, TranscriptionService};
use crate::config::OpenRouterConfig;
use crate::keychain::{self, ProviderAccount};

static SILENT_WAV: &[u8] = include_bytes!("../../../../assets/silent_1s.wav");

/// Frontend-facing status for OpenRouter provider (never exposes API key)
#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenRouterConfigStatus {
    pub configured: bool,
    pub model: String,
}

// ===== OPENROUTER PROVIDER COMMANDS =====

#[tauri::command]
#[specta::specta]
pub fn load_openrouter_config() -> Result<Option<OpenRouterConfigStatus>, String> {
    let config = keychain::load_provider_config::<OpenRouterConfig>(ProviderAccount::OpenRouter)
        .map_err(|e| {
            let err = format!("Failed to load OpenRouter config: {}", e);
            error!("{}", err);
            err
        })?;

    Ok(config.map(|c| OpenRouterConfigStatus {
        configured: true,
        model: c.model,
    }))
}

#[tauri::command]
#[specta::specta]
pub fn save_openrouter_config(api_key: String, model: String) -> Result<(), String> {
    let final_api_key = if api_key.is_empty() || api_key == "••••••••••••••••••••••••••••••••••••"
    {
        keychain::load_provider_config::<OpenRouterConfig>(ProviderAccount::OpenRouter)
            .map_err(|e| format!("Failed to load existing OpenRouter config: {}", e))?
            .map(|c| c.api_key)
            .ok_or_else(|| "No existing OpenRouter API key found to reuse".to_string())?
    } else {
        api_key
    };

    let config = OpenRouterConfig {
        api_key: final_api_key,
        model,
    };

    keychain::save_provider_config(ProviderAccount::OpenRouter, &config).map_err(|e| {
        let err = format!("Failed to save OpenRouter config: {}", e);
        error!("{}", err);
        err
    })
}

#[tauri::command]
#[specta::specta]
pub fn delete_openrouter_config() -> Result<(), String> {
    keychain::delete_provider_config(ProviderAccount::OpenRouter).map_err(|e| {
        let err = format!("Failed to delete OpenRouter config: {}", e);
        error!("{}", err);
        err
    })
}

#[tauri::command]
#[specta::specta]
pub fn test_openrouter_config(api_key: String, model: String) -> Result<bool, String> {
    let final_api_key = if api_key.is_empty() || api_key == "••••••••••••••••••••••••••••••••••••"
    {
        keychain::load_provider_config::<OpenRouterConfig>(ProviderAccount::OpenRouter)
            .map_err(|e| {
                format!(
                    "Failed to load existing OpenRouter config for testing: {}",
                    e
                )
            })?
            .map(|c| c.api_key)
            .ok_or_else(|| "No existing OpenRouter API key found to test".to_string())?
    } else {
        api_key
    };

    let service = OpenRouterTranscriber::new(SecretString::from(final_api_key), model);

    // Create temp file for static audio
    let temp_path = std::env::temp_dir().join("dictara_test_audio_openrouter.wav");
    std::fs::write(&temp_path, SILENT_WAV).map_err(|e| {
        let err = format!("Failed to write test audio: {}", e);
        error!("{}", err);
        err
    })?;

    let result = match service.transcribe(&temp_path) {
        Ok(_) => Ok(true),
        Err(TranscriptionError::ApiError(msg))
            if msg.contains("401") || msg.contains("Unauthorized") =>
        {
            warn!("OpenRouter API key is invalid (401 Unauthorized)");
            Ok(false)
        }
        Err(e) => {
            let err = format!("Failed to test OpenRouter config: {}", e);
            error!("{}", err);
            Err(err)
        }
    };

    // Clean up temp file
    if let Err(e) = std::fs::remove_file(&temp_path) {
        warn!(
            "Failed to clean up temp file '{}': {}",
            temp_path.display(),
            e
        );
    }

    result
}
