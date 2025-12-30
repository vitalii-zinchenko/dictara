use std::path::{Path, PathBuf};

use log::{error, info, warn};

use crate::config::{AppConfig, AzureOpenAIConfig, OpenAIConfig, Provider};
use crate::keychain::{self, ProviderAccount};

use super::azure_client::AzureClient;
use super::client::TranscriptionClient;
use super::config::ApiConfig;
use super::error::TranscriptionError;
use super::openai_client::OpenAIClient;

const MIN_AUDIO_DURATION_MS: u64 = 500; // Minimum 0.5 seconds
const MAX_FILE_SIZE_BYTES: u64 = 25 * 1024 * 1024; // 25MB limit

// Pre-generated 1-second silent WAV file (16kHz, mono) for API testing
static SILENT_WAV: &[u8] = include_bytes!("../../assets/silent_1s.wav");

/// Transcription service that orchestrates audio transcription
///
/// Uses a `TranscriptionClient` implementation based on the configured provider.
pub struct Transcriber {
    client: Box<dyn TranscriptionClient>,
}

impl Transcriber {
    /// Create a new Transcriber from application config
    ///
    /// Loads the appropriate client based on the active provider in config.
    pub fn from_config(config: &AppConfig) -> Result<Self, TranscriptionError> {
        let provider = config
            .active_provider
            .as_ref()
            .ok_or(TranscriptionError::ApiKeyMissing)?;

        let client = Self::create_client_from_keychain(provider)?;
        Ok(Self { client })
    }

    /// Test API credentials without creating a persistent instance
    ///
    /// Creates a temporary client and attempts to transcribe the embedded silent audio.
    ///
    /// # Returns
    /// * `Ok(true)` - Credentials are valid
    /// * `Ok(false)` - Credentials are invalid (401 Unauthorized)
    /// * `Err(TranscriptionError)` - Network or other API error
    pub fn test_api_key(config: &ApiConfig) -> Result<bool, TranscriptionError> {
        let client = Self::create_client_from_config(config);
        let transcriber = Self { client };

        match transcriber.transcribe_static_audio() {
            Ok(_) => Ok(true),
            Err(TranscriptionError::ApiError(msg)) if msg.contains("401") => {
                warn!("API key is invalid (401 Unauthorized)");
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }

    /// Transcribe audio file to text
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

        // Transcribe
        self.do_transcribe_file(&file_path)
    }

    // ========== Private methods ==========

    /// Create client from keychain-stored credentials
    fn create_client_from_keychain(
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
        }
    }

    /// Create client from explicit config (for testing credentials)
    fn create_client_from_config(config: &ApiConfig) -> Box<dyn TranscriptionClient> {
        match config.provider {
            Provider::OpenAI => Box::new(OpenAIClient::new(config.api_key.clone())),
            Provider::AzureOpenAI => Box::new(AzureClient::new(
                config.api_key.clone(),
                config.endpoint.clone(),
            )),
        }
    }

    /// Validate file exists and is within size limits
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

    /// Transcribe the embedded silent audio (for testing)
    fn transcribe_static_audio(&self) -> Result<String, TranscriptionError> {
        self.do_transcribe_bytes(SILENT_WAV, "silent.wav")
    }

    /// Core transcription from file path
    fn do_transcribe_file(&self, file_path: &Path) -> Result<String, TranscriptionError> {
        let form = self.client.build_form_from_path(file_path)?;
        self.send_and_parse(form)
    }

    /// Core transcription from bytes (for static test audio)
    fn do_transcribe_bytes(
        &self,
        audio_bytes: &[u8],
        filename: &str,
    ) -> Result<String, TranscriptionError> {
        let form = self.client.build_form_from_bytes(audio_bytes, filename)?;
        self.send_and_parse(form)
    }

    /// Send request and parse response
    fn send_and_parse(
        &self,
        form: reqwest::blocking::multipart::Form,
    ) -> Result<String, TranscriptionError> {
        let http_client = reqwest::blocking::Client::new();
        let request = http_client.post(self.client.transcription_url());
        let request = self.client.add_auth(request);

        let response = request.multipart(form).send().map_err(|e| {
            error!("API request error: {}", e);
            TranscriptionError::ApiError(format!("Request failed: {}", e))
        })?;

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("API error response ({}): {}", status, error_text);
            return Err(TranscriptionError::ApiError(format!(
                "API returned status {}: {}",
                status, error_text
            )));
        }

        // Parse JSON response
        let json: serde_json::Value = response.json().map_err(|e| {
            error!("Failed to parse response: {}", e);
            TranscriptionError::ApiError(format!("Failed to parse response: {}", e))
        })?;

        let text = json["text"].as_str().unwrap_or("").to_string();

        info!("Transcription successful: {} characters", text.len());

        Ok(text)
    }
}
