use std::path::{Path, PathBuf};
use std::sync::Arc;

use log::{error, warn};
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

use crate::config::{self, AzureOpenAIConfig, LocalModelConfig, OpenAIConfig, Provider};
use crate::keychain::{self, ProviderAccount};
use crate::models::{is_model_in_catalog, ModelLoader, ModelManager};

use super::api_transcriber::ApiTranscriber;
use super::azure_client::AzureClient;
use super::client::TranscriptionClient;
use super::config::ApiConfig;
use super::error::TranscriptionError;
use super::local_transcriber::LocalTranscriber;
use super::openai_client::OpenAIClient;
use super::service::TranscriptionService;

const MIN_AUDIO_DURATION_MS: u64 = 500; // Minimum 0.5 seconds
const MAX_FILE_SIZE_BYTES: u64 = 25 * 1024 * 1024; // 25MB limit

// Pre-generated 1-second silent WAV file (16kHz, mono) for API testing
static SILENT_WAV: &[u8] = include_bytes!("../../assets/silent_1s.wav");

/// Transcription service that orchestrates audio transcription.
///
/// Abstracts away the transcription implementation details - the caller
/// doesn't need to know whether it's using an API or local model.
pub struct Transcriber {
    service: Box<dyn TranscriptionService>,
}

impl Transcriber {
    /// Create a new Transcriber from application config and app handle.
    ///
    /// The app handle is needed for local provider to access ModelLoader state.
    pub fn from_app(app: &AppHandle) -> Result<Self, TranscriptionError> {
        let store = app
            .store("config.json")
            .map_err(|e| TranscriptionError::ApiError(format!("Failed to open store: {}", e)))?;

        let app_config = config::load_app_config(&store);

        let provider = app_config
            .active_provider
            .as_ref()
            .ok_or(TranscriptionError::ApiKeyMissing)?;

        let service = Self::create_service(provider, app)?;
        Ok(Self { service })
    }

    /// Test API credentials without creating a persistent instance.
    ///
    /// Creates a temporary client and attempts to transcribe the embedded silent audio.
    ///
    /// # Returns
    /// * `Ok(true)` - Credentials are valid
    /// * `Ok(false)` - Credentials are invalid (401 Unauthorized)
    /// * `Err(TranscriptionError)` - Network or other API error
    pub fn test_api_key(config: &ApiConfig) -> Result<bool, TranscriptionError> {
        let client = Self::create_client_from_explicit_config(config);
        let service = ApiTranscriber::new(client);

        // Create temp file for static audio
        let temp_path = std::env::temp_dir().join("dictara_test_audio.wav");
        std::fs::write(&temp_path, SILENT_WAV).map_err(|e| {
            TranscriptionError::IoError(std::io::Error::other(format!(
                "Failed to write test audio: {}",
                e
            )))
        })?;

        let result = match service.transcribe(&temp_path) {
            Ok(_) => Ok(true),
            Err(TranscriptionError::ApiError(msg)) if msg.contains("401") => {
                warn!("API key is invalid (401 Unauthorized)");
                Ok(false)
            }
            Err(e) => Err(e),
        };

        // Clean up temp file, log warning if it fails
        if let Err(e) = std::fs::remove_file(&temp_path) {
            warn!(
                "Failed to clean up temp file '{}': {}. File may need manual cleanup.",
                temp_path.display(),
                e
            );
        }

        result
    }

    /// Transcribe audio file to text.
    ///
    /// # Arguments
    /// * `file_path` - Path to the audio file (WAV, MP3, etc.)
    /// * `duration_ms` - Duration of the recording in milliseconds (for validation)
    ///
    /// # Returns
    /// * `Ok(String)` - Transcribed text
    /// * `Err(TranscriptionError)` - Error details
    pub fn transcribe(
        &self,
        file_path: PathBuf,
        duration_ms: u64,
    ) -> Result<String, TranscriptionError> {
        // Validate minimum duration
        if duration_ms < MIN_AUDIO_DURATION_MS {
            warn!(
                "Audio too short: {}ms < {}ms minimum",
                duration_ms, MIN_AUDIO_DURATION_MS
            );
            return Ok(String::new());
        }

        // Validate file
        self.validate_file(&file_path)?;

        // Transcribe using the appropriate service
        self.service.transcribe(&file_path)
    }

    // ========== Private methods ==========

    /// Create the appropriate transcription service based on provider.
    fn create_service(
        provider: &Provider,
        app: &AppHandle,
    ) -> Result<Box<dyn TranscriptionService>, TranscriptionError> {
        match provider {
            Provider::OpenAI | Provider::AzureOpenAI => {
                let client = Self::create_api_client(provider)?;
                Ok(Box::new(ApiTranscriber::new(client)))
            }
            Provider::Local => Self::create_local_service(app),
        }
    }

    /// Create API client from keychain credentials.
    fn create_api_client(
        provider: &Provider,
    ) -> Result<Box<dyn TranscriptionClient>, TranscriptionError> {
        match provider {
            Provider::OpenAI => {
                let config: OpenAIConfig = keychain::load_provider_config(ProviderAccount::OpenAI)
                    .map_err(|_| TranscriptionError::ApiKeyMissing)?
                    .ok_or(TranscriptionError::ApiKeyMissing)?;
                Ok(Box::new(OpenAIClient::new(config.api_key)))
            }
            Provider::AzureOpenAI => {
                let config: AzureOpenAIConfig =
                    keychain::load_provider_config(ProviderAccount::AzureOpenAI)
                        .map_err(|_| TranscriptionError::ApiKeyMissing)?
                        .ok_or(TranscriptionError::ApiKeyMissing)?;
                Ok(Box::new(AzureClient::new(config.api_key, config.endpoint)))
            }
            Provider::Local => Err(TranscriptionError::ApiError(
                "Local provider doesn't use API client".to_string(),
            )),
        }
    }

    /// Create local transcription service with validation.
    fn create_local_service(
        app: &AppHandle,
    ) -> Result<Box<dyn TranscriptionService>, TranscriptionError> {
        // Load local model config
        let store = app
            .store("config.json")
            .map_err(|e| TranscriptionError::ApiError(format!("Failed to open store: {}", e)))?;

        let local_config: Option<LocalModelConfig> = config::load_local_model_config(&store);
        let selected_model = local_config
            .and_then(|c| c.selected_model)
            .ok_or(TranscriptionError::NoModelSelected)?;

        // Validate model exists in catalog
        if !is_model_in_catalog(&selected_model) {
            return Err(TranscriptionError::ModelNotFound(selected_model));
        }

        // Validate model is downloaded
        let model_manager = app.state::<Arc<ModelManager>>();
        if !model_manager.is_model_downloaded(&selected_model) {
            return Err(TranscriptionError::ModelNotDownloaded(selected_model));
        }

        // Get ModelLoader from Tauri state
        let loader = app.state::<Arc<ModelLoader>>();

        Ok(Box::new(LocalTranscriber::new(
            loader.inner().clone(),
            selected_model,
        )))
    }

    /// Create client from explicit config (for testing credentials).
    fn create_client_from_explicit_config(config: &ApiConfig) -> Box<dyn TranscriptionClient> {
        match config.provider {
            Provider::OpenAI => Box::new(OpenAIClient::new(config.api_key.clone())),
            Provider::AzureOpenAI => Box::new(AzureClient::new(
                config.api_key.clone(),
                config.endpoint.clone(),
            )),
            Provider::Local => {
                // Local provider doesn't use API testing - just return OpenAI client
                // This code path shouldn't be reached for Local provider
                Box::new(OpenAIClient::new(String::new()))
            }
        }
    }

    /// Validate file exists and is within size limits.
    fn validate_file(&self, file_path: &Path) -> Result<(), TranscriptionError> {
        if !file_path.exists() {
            error!("File not found: {:?}", file_path);
            return Err(TranscriptionError::FileNotFound(
                file_path.to_string_lossy().to_string(),
            ));
        }

        let metadata = std::fs::metadata(file_path)?;
        let file_size = metadata.len();

        if file_size > MAX_FILE_SIZE_BYTES {
            error!(
                "File too large: {} bytes > {} bytes",
                file_size, MAX_FILE_SIZE_BYTES
            );
            return Err(TranscriptionError::FileTooLarge {
                size_bytes: file_size,
            });
        }

        Ok(())
    }
}
