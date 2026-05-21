use log::{error, warn};
use secrecy::SecretString;
use serde::Serialize;

use crate::clients::{
    ApiTranscriber, CustomEndpointClient, TranscriptionError, TranscriptionService,
};
use crate::config::CustomEndpointConfig;
use crate::keychain::{self, ProviderAccount};

static SILENT_WAV: &[u8] = include_bytes!("../../../../assets/silent_1s.wav");

/// Frontend-facing status for Custom Endpoint provider (never exposes API key)
#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CustomEndpointConfigStatus {
    pub configured: bool,
    pub base_url: String,
    pub model: String,
    pub has_api_key: bool,
}

// ===== CUSTOM ENDPOINT PROVIDER COMMANDS =====

#[tauri::command]
#[specta::specta]
pub fn load_custom_endpoint_config() -> Result<Option<CustomEndpointConfigStatus>, String> {
    let config =
        keychain::load_provider_config::<CustomEndpointConfig>(ProviderAccount::CustomEndpoint)
            .map_err(|e| {
                let err = format!("Failed to load Custom Endpoint config: {}", e);
                error!("{}", err);
                err
            })?;

    Ok(config.map(|c| CustomEndpointConfigStatus {
        configured: true,
        base_url: c.base_url,
        model: c.model,
        has_api_key: c.api_key.is_some() && !c.api_key.as_ref().unwrap().is_empty(),
    }))
}

#[tauri::command]
#[specta::specta]
pub fn save_custom_endpoint_config(
    api_key: Option<String>,
    base_url: String,
    model: String,
) -> Result<(), String> {
    let final_api_key = if let Some(ref key) = api_key {
        if key == "••••••••••••••••••••••••••••••••••••"
        {
            match keychain::load_provider_config::<CustomEndpointConfig>(
                ProviderAccount::CustomEndpoint,
            ) {
                Ok(Some(c)) => c.api_key,
                _ => return Err("No existing Custom Endpoint API key found to reuse".to_string()),
            }
        } else {
            api_key.clone()
        }
    } else {
        api_key.clone()
    };

    let config = CustomEndpointConfig {
        api_key: final_api_key,
        base_url,
        model,
    };

    keychain::save_provider_config(ProviderAccount::CustomEndpoint, &config).map_err(|e| {
        let err = format!("Failed to save Custom Endpoint config: {}", e);
        error!("{}", err);
        err
    })
}

#[tauri::command]
#[specta::specta]
pub fn delete_custom_endpoint_config() -> Result<(), String> {
    keychain::delete_provider_config(ProviderAccount::CustomEndpoint).map_err(|e| {
        let err = format!("Failed to delete Custom Endpoint config: {}", e);
        error!("{}", err);
        err
    })
}

#[tauri::command]
#[specta::specta]
pub fn test_custom_endpoint_config(
    api_key: Option<String>,
    base_url: String,
    model: String,
) -> Result<bool, String> {
    let final_api_key = if let Some(ref key) = api_key {
        if key == "••••••••••••••••••••••••••••••••••••"
        {
            match keychain::load_provider_config::<CustomEndpointConfig>(
                ProviderAccount::CustomEndpoint,
            ) {
                Ok(Some(c)) => c.api_key,
                _ => return Err("No existing Custom Endpoint API key found to test".to_string()),
            }
        } else {
            api_key.clone()
        }
    } else {
        api_key.clone()
    };

    let client_key = final_api_key.map(SecretString::from);
    let client = CustomEndpointClient::new(client_key, base_url, model);
    let service = ApiTranscriber::new(Box::new(client));

    // Create temp file for static audio
    let temp_path = std::env::temp_dir().join("dictara_test_audio_custom.wav");
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
            warn!("Custom Endpoint credentials invalid (401 Unauthorized)");
            Ok(false)
        }
        Err(e) => {
            let err = format!("Failed to test Custom Endpoint config: {}", e);
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
