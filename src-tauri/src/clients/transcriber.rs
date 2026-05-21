use std::path::{Path, PathBuf};
use std::sync::Arc;

use log::{error, warn};
use secrecy::{ExposeSecret, SecretString};
use tauri::{AppHandle, Manager};

use crate::config::{
    self, AudioFormat, AzureOpenAIConfig, ConfigKey, ConfigStore, CustomEndpointConfig,
    LocalModelConfig, OpenAIConfig, OpenRouterConfig, Provider,
};
use crate::keychain::{self, ProviderAccount};
use crate::models::{is_model_in_catalog, ModelLoader, ModelManager};

use super::api_transcriber::ApiTranscriber;
use super::azure_client::AzureClient;
use super::client::TranscriptionClient;
use super::config::ApiConfig;
use super::custom_endpoint_client::CustomEndpointClient;
use super::error::TranscriptionError;
use super::local_transcriber::LocalTranscriber;
use super::openai_client::OpenAIClient;
use super::openrouter_transcriber::OpenRouterTranscriber;
use super::service::TranscriptionService;

const MIN_AUDIO_DURATION_MS: u64 = 500; // Minimum 0.5 seconds
const MAX_FILE_SIZE_BYTES: u64 = 25 * 1024 * 1024; // 25MB limit

/// Timeout for transcription requests in seconds (applies to all providers)
pub const TRANSCRIPTION_TIMEOUT_SECS: u64 = 120;

// Pre-generated 1-second silent WAV file (16kHz, mono) for API testing
static SILENT_WAV: &[u8] = include_bytes!("../../assets/silent_1s.wav");

/// Transcription service that orchestrates audio transcription.
///
/// Abstracts away the transcription implementation details - the caller
/// doesn't need to know whether it's using an API or local model.
pub struct Transcriber {
    service: Box<dyn TranscriptionService>,
    audio_format: AudioFormat,
    provider: Provider,
}

impl Transcriber {
    /// Create a new Transcriber from application config and app handle.
    ///
    /// The app handle is needed for local provider to access ModelLoader state.
    pub fn from_app(app: &AppHandle) -> Result<Self, TranscriptionError> {
        let config_store = app.state::<config::Config>();

        let app_config = config_store.get(&ConfigKey::APP).unwrap_or_default();

        let provider = app_config
            .active_provider
            .as_ref()
            .ok_or(TranscriptionError::ApiKeyMissing)?;

        let service = Self::create_service(provider, app)?;
        let audio_format = app_config.audio_format;
        Ok(Self {
            service,
            audio_format,
            provider: provider.clone(),
        })
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

        // If the format is set to Mp3 and the provider is an API-based provider (not Local),
        // we convert the file to MP3 first and transcribe that instead!
        if self.audio_format == AudioFormat::Mp3 && self.provider != Provider::Local {
            log::info!(
                "Converting WAV to MP3 for API transmission: {:?}",
                file_path
            );
            let mp3_path = convert_wav_to_mp3(&file_path)?;

            // Perform transcription on the MP3 file
            let result = self.service.transcribe(&mp3_path);

            // Clean up the temporary MP3 file
            if let Err(e) = std::fs::remove_file(&mp3_path) {
                warn!(
                    "Failed to delete temporary MP3 file '{:?}': {}",
                    mp3_path, e
                );
            }

            result
        } else {
            // Transcribe using the appropriate service
            self.service.transcribe(&file_path)
        }
    }

    // ========== Private methods ==========

    /// Create the appropriate transcription service based on provider.
    fn create_service(
        provider: &Provider,
        app: &AppHandle,
    ) -> Result<Box<dyn TranscriptionService>, TranscriptionError> {
        match provider {
            Provider::OpenAI | Provider::AzureOpenAI | Provider::CustomEndpoint => {
                let client = Self::create_api_client(provider)?;
                Ok(Box::new(ApiTranscriber::new(client)))
            }
            Provider::OpenRouter => {
                let config: OpenRouterConfig =
                    keychain::load_provider_config(ProviderAccount::OpenRouter)
                        .map_err(|_| TranscriptionError::ApiKeyMissing)?
                        .ok_or(TranscriptionError::ApiKeyMissing)?;
                Ok(Box::new(OpenRouterTranscriber::new(
                    SecretString::from(config.api_key),
                    config.model,
                )))
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
                Ok(Box::new(OpenAIClient::new(SecretString::from(
                    config.api_key,
                ))))
            }
            Provider::AzureOpenAI => {
                let config: AzureOpenAIConfig =
                    keychain::load_provider_config(ProviderAccount::AzureOpenAI)
                        .map_err(|_| TranscriptionError::ApiKeyMissing)?
                        .ok_or(TranscriptionError::ApiKeyMissing)?;
                Ok(Box::new(AzureClient::new(
                    SecretString::from(config.api_key),
                    config.endpoint,
                )))
            }
            Provider::CustomEndpoint => {
                let config: CustomEndpointConfig =
                    keychain::load_provider_config(ProviderAccount::CustomEndpoint)
                        .map_err(|_| TranscriptionError::ApiKeyMissing)?
                        .ok_or(TranscriptionError::ApiKeyMissing)?;
                let api_key = config.api_key.map(SecretString::from);
                Ok(Box::new(CustomEndpointClient::new(
                    api_key,
                    config.base_url,
                    config.model,
                )))
            }
            Provider::Local | Provider::OpenRouter => Err(TranscriptionError::ApiError(
                "Local/OpenRouter provider doesn't use standard API client".to_string(),
            )),
        }
    }

    /// Create local transcription service with validation.
    fn create_local_service(
        app: &AppHandle,
    ) -> Result<Box<dyn TranscriptionService>, TranscriptionError> {
        // Load local model config
        let config_store = app.state::<config::Config>();

        let local_config: Option<LocalModelConfig> = config_store.get(&ConfigKey::LOCAL_MODEL);
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
            Provider::OpenAI => Box::new(OpenAIClient::new(SecretString::from(
                config.api_key.expose_secret().to_owned(),
            ))),
            Provider::AzureOpenAI => Box::new(AzureClient::new(
                SecretString::from(config.api_key.expose_secret().to_owned()),
                config.endpoint.clone(),
            )),
            Provider::Local | Provider::OpenRouter | Provider::CustomEndpoint => {
                // Local, OpenRouter, and Custom Endpoint do not use this path for testing.
                // We provide a dummy client to satisfy the compiler's exhaustiveness check.
                Box::new(OpenAIClient::new(SecretString::from(String::new())))
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

/// Helper function to convert 16kHz mono WAV file to MP3 using pure-Rust shine-rs encoder
fn convert_wav_to_mp3(wav_path: &Path) -> Result<PathBuf, TranscriptionError> {
    use hound::WavReader;
    use shine_rs::{Mp3Encoder, Mp3EncoderConfig, StereoMode};
    use std::fs::File;
    use std::io::Write;

    let mut reader = WavReader::open(wav_path).map_err(|e| {
        TranscriptionError::IoError(std::io::Error::other(format!("Failed to open WAV: {}", e)))
    })?;
    let spec = reader.spec();

    if spec.channels != 1 {
        return Err(TranscriptionError::ApiError(
            "Only mono audio is supported for MP3 conversion".to_string(),
        ));
    }
    if spec.sample_rate != 16000 {
        return Err(TranscriptionError::ApiError(
            "Only 16kHz audio is supported for MP3 conversion".to_string(),
        ));
    }

    let mp3_path = wav_path.with_extension("mp3");
    let mut mp3_file = File::create(&mp3_path).map_err(|e| {
        TranscriptionError::IoError(std::io::Error::other(format!(
            "Failed to create MP3 file: {}",
            e
        )))
    })?;

    let config = Mp3EncoderConfig::new()
        .sample_rate(spec.sample_rate)
        .bitrate(64) // 64 kbps mono is excellent quality and very compact
        .channels(1)
        .stereo_mode(StereoMode::Mono);

    let mut encoder = Mp3Encoder::new(config).map_err(|e| {
        TranscriptionError::ApiError(format!("Failed to create MP3 encoder: {:?}", e))
    })?;

    let mut pcm_buffer = Vec::new();
    for sample in reader.samples::<i16>() {
        let sample = sample.map_err(|e| {
            TranscriptionError::IoError(std::io::Error::other(format!(
                "Failed to read WAV sample: {}",
                e
            )))
        })?;
        pcm_buffer.push(sample);

        if pcm_buffer.len() == encoder.samples_per_frame() {
            let mp3_frames = encoder.encode_interleaved(&pcm_buffer).map_err(|e| {
                TranscriptionError::ApiError(format!("MP3 encoding failed: {:?}", e))
            })?;
            for frame in mp3_frames {
                mp3_file.write_all(&frame).map_err(|e| {
                    TranscriptionError::IoError(std::io::Error::other(format!(
                        "Failed to write MP3 frame: {}",
                        e
                    )))
                })?;
            }
            pcm_buffer.clear();
        }
    }

    // Flush remaining samples by padding with zeros if necessary
    if !pcm_buffer.is_empty() {
        pcm_buffer.resize(encoder.samples_per_frame(), 0);
        let mp3_frames = encoder
            .encode_interleaved(&pcm_buffer)
            .map_err(|e| TranscriptionError::ApiError(format!("MP3 encoding failed: {:?}", e)))?;
        for frame in mp3_frames {
            mp3_file.write_all(&frame).map_err(|e| {
                TranscriptionError::IoError(std::io::Error::other(format!(
                    "Failed to write final MP3 frame: {}",
                    e
                )))
            })?;
        }
    }

    let final_data = encoder.finish().map_err(|e| {
        TranscriptionError::ApiError(format!("Failed to finalize MP3 encoder: {:?}", e))
    })?;
    mp3_file.write_all(&final_data).map_err(|e| {
        TranscriptionError::IoError(std::io::Error::other(format!(
            "Failed to write final MP3 bytes: {}",
            e
        )))
    })?;

    Ok(mp3_path)
}
