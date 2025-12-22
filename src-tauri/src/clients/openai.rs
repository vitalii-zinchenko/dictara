use crate::config::{Provider, ProviderConfig};
use crate::keychain::{self, KeychainAccount};
use async_openai::{
    config::OpenAIConfig,
    types::{AudioResponseFormat, CreateTranscriptionRequestArgs},
    Client,
};
use std::path::PathBuf;

const MIN_AUDIO_DURATION_MS: u64 = 500; // Minimum 0.5 seconds
const MAX_FILE_SIZE_BYTES: u64 = 25 * 1024 * 1024; // 25MB limit

// Azure API version
const AZURE_API_VERSION: &str = "2024-06-01";

// OpenAI endpoints
const OPENAI_MODELS_URL: &str = "https://api.openai.com/v1/models";
const OPENAI_TRANSCRIPTION_URL: &str = "https://api.openai.com/v1/audio/transcriptions";

#[derive(Debug)]
pub enum TranscriptionError {
    AudioTooShort { duration_ms: u64 },
    FileTooLarge { size_bytes: u64 },
    FileNotFound(String),
    ApiError(String),
    IoError(std::io::Error),
    ApiKeyMissing,
}

impl From<std::io::Error> for TranscriptionError {
    fn from(err: std::io::Error) -> Self {
        TranscriptionError::IoError(err)
    }
}

impl std::fmt::Display for TranscriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscriptionError::AudioTooShort { duration_ms } => {
                write!(f, "Audio too short: {}ms (minimum 500ms)", duration_ms)
            }
            TranscriptionError::FileTooLarge { size_bytes } => {
                write!(f, "File too large: {} bytes (maximum 25MB)", size_bytes)
            }
            TranscriptionError::FileNotFound(path) => {
                write!(f, "Audio file not found: {}", path)
            }
            TranscriptionError::ApiError(msg) => {
                write!(f, "API error: {}", msg)
            }
            TranscriptionError::IoError(err) => {
                write!(f, "IO error: {}", err)
            }
            TranscriptionError::ApiKeyMissing => {
                write!(f, "API key not configured")
            }
        }
    }
}

impl TranscriptionError {
    /// Returns a user-friendly error message suitable for display in the UI
    pub fn user_message(&self) -> String {
        match self {
            TranscriptionError::AudioTooShort { duration_ms } => {
                format!(
                    "Recording too short ({}ms). Please speak for at least 0.5 seconds.",
                    duration_ms
                )
            }
            TranscriptionError::FileTooLarge { size_bytes } => {
                let mb = size_bytes / (1024 * 1024);
                format!("Audio file too large ({}MB). Maximum is 25MB.", mb)
            }
            TranscriptionError::FileNotFound(_) => {
                "Audio file not found. Please try recording again.".to_string()
            }
            TranscriptionError::ApiError(msg) => {
                // Parse for specific errors
                if msg.contains("429") || msg.to_lowercase().contains("rate limit") {
                    "Rate limit reached. Please wait and retry.".to_string()
                } else if msg.contains("401") {
                    "Invalid API key. Check your settings.".to_string()
                } else {
                    format!("Transcription failed: {}", msg)
                }
            }
            TranscriptionError::IoError(_) => {
                "Failed to read audio file. Please try again.".to_string()
            }
            TranscriptionError::ApiKeyMissing => {
                "API key not configured. Please add it in Preferences.".to_string()
            }
        }
    }

    /// Returns true if this error can be retried
    pub fn can_retry(&self) -> bool {
        matches!(
            self,
            TranscriptionError::ApiError(_) | TranscriptionError::FileNotFound(_)
        )
    }
}

/// Configuration for making API calls
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub provider: Provider,
    pub api_key: String,
    pub endpoint: String, // Full transcription endpoint for Azure (without api-version), unused for OpenAI
}

impl ApiConfig {
    /// Construct the full transcription URL based on provider
    fn transcription_url(&self) -> String {
        match self.provider {
            Provider::OpenAI => OPENAI_TRANSCRIPTION_URL.to_string(),
            Provider::Azure => {
                // Azure URL format: user provides full endpoint path, we just add api-version
                // Example: https://xxx.cognitiveservices.azure.com/openai/deployments/whisper/audio/transcriptions
                format!(
                    "{}?api-version={}",
                    self.endpoint.trim_end_matches('/'),
                    AZURE_API_VERSION
                )
            }
        }
    }

    /// Construct the models URL for API key validation
    fn models_url(&self) -> String {
        match self.provider {
            Provider::OpenAI => OPENAI_MODELS_URL.to_string(),
            Provider::Azure => {
                format!(
                    "{}/openai/deployments?api-version={}",
                    self.endpoint.trim_end_matches('/'),
                    AZURE_API_VERSION
                )
            }
        }
    }

    /// Add authentication header to request builder
    fn add_auth_header(
        &self,
        request: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        match self.provider {
            Provider::OpenAI => request.bearer_auth(&self.api_key),
            Provider::Azure => request.header("api-key", &self.api_key),
        }
    }
}

pub struct OpenAIClient {
    client: Client<OpenAIConfig>,
}

impl Clone for OpenAIClient {
    fn clone(&self) -> Self {
        OpenAIClient {
            client: Client::new(),
        }
    }
}

impl OpenAIClient {
    /// Create a new OpenAI client
    pub fn new() -> Self {
        println!("[OpenAI Client] Initializing client");
        OpenAIClient {
            client: Client::new(),
        }
    }

    /// Load API configuration from keychain and config store
    pub fn load_config(config: &ProviderConfig) -> Result<ApiConfig, TranscriptionError> {
        let provider = config
            .enabled_provider
            .as_ref()
            .ok_or(TranscriptionError::ApiKeyMissing)?;

        let (api_key, endpoint) = match provider {
            Provider::OpenAI => {
                let key = keychain::load_api_key(KeychainAccount::OpenAI)
                    .map_err(|_| TranscriptionError::ApiKeyMissing)?
                    .ok_or(TranscriptionError::ApiKeyMissing)?;
                (key, String::new())
            }
            Provider::Azure => {
                let key = keychain::load_api_key(KeychainAccount::Azure)
                    .map_err(|_| TranscriptionError::ApiKeyMissing)?
                    .ok_or(TranscriptionError::ApiKeyMissing)?;
                let endpoint =
                    config
                        .azure_endpoint
                        .clone()
                        .ok_or(TranscriptionError::ApiError(
                            "Azure endpoint not configured".to_string(),
                        ))?;
                (key, endpoint)
            }
        };

        Ok(ApiConfig {
            provider: provider.clone(),
            api_key,
            endpoint,
        })
    }

    /// Test if an API key is valid
    ///
    /// # Arguments
    /// * `provider` - The provider type (OpenAI or Azure)
    /// * `key` - The API key to test
    /// * `endpoint` - Optional Azure endpoint (required for Azure, ignored for OpenAI)
    ///
    /// # Returns
    /// * `Ok(true)` - Key is valid
    /// * `Ok(false)` - Key is invalid (401 Unauthorized)
    /// * `Err(TranscriptionError)` - Network or other API error
    pub fn test_api_key(
        provider: Provider,
        key: &str,
        endpoint: Option<&str>,
    ) -> Result<bool, TranscriptionError> {
        println!(
            "[OpenAI Client] Testing API key validity for {:?}...",
            provider
        );

        match provider {
            Provider::OpenAI => {
                // OpenAI: Use models endpoint for quick validation
                let api_config = ApiConfig {
                    provider: provider.clone(),
                    api_key: key.to_string(),
                    endpoint: String::new(),
                };

                let client = reqwest::blocking::Client::new();
                let request = client.get(api_config.models_url());
                let request = api_config.add_auth_header(request);

                let response = request.send().map_err(|e| {
                    eprintln!("[OpenAI Client] Request failed: {}", e);
                    TranscriptionError::ApiError(format!("Request failed: {}", e))
                })?;

                let status = response.status();
                println!("[OpenAI Client] API test response status: {}", status);

                if status.is_success() {
                    println!("[OpenAI Client] ✅ API key is valid");
                    Ok(true)
                } else if status.as_u16() == 401 {
                    println!("[OpenAI Client] ❌ API key is invalid (401 Unauthorized)");
                    Ok(false)
                } else {
                    let error_text = response
                        .text()
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    eprintln!(
                        "[OpenAI Client] Unexpected API response ({}): {}",
                        status, error_text
                    );
                    Err(TranscriptionError::ApiError(format!(
                        "API returned status {}: {}",
                        status, error_text
                    )))
                }
            }
            Provider::Azure => {
                // Azure: Test with actual transcription since /deployments endpoint is deprecated
                println!("[OpenAI Client] Testing Azure with silent audio transcription...");

                // Generate a tiny silent audio file for testing
                let temp_dir = std::env::temp_dir();
                let test_audio_path = temp_dir.join("typefree_test_silent.wav");

                // Generate 1 second silent audio
                let ffmpeg_result = std::process::Command::new("ffmpeg")
                    .args([
                        "-f",
                        "lavfi",
                        "-i",
                        "anullsrc=r=16000:cl=mono",
                        "-t",
                        "1.0",
                        "-y",
                        test_audio_path.to_str().unwrap(),
                    ])
                    .output()
                    .map_err(|e| {
                        TranscriptionError::ApiError(format!(
                            "Failed to generate test audio: {}",
                            e
                        ))
                    })?;

                if !ffmpeg_result.status.success() {
                    return Err(TranscriptionError::ApiError(
                        "Failed to generate test audio with ffmpeg".to_string(),
                    ));
                }

                // Test transcription
                let api_config = ApiConfig {
                    provider: Provider::Azure,
                    api_key: key.to_string(),
                    endpoint: endpoint.unwrap_or("").to_string(),
                };

                let form = reqwest::blocking::multipart::Form::new()
                    .file("file", &test_audio_path)
                    .map_err(|e| {
                        TranscriptionError::IoError(std::io::Error::other(format!(
                            "Failed to read test file: {}",
                            e
                        )))
                    })?
                    .text("temperature", "0.0")
                    .text("response_format", "json");

                let client = reqwest::blocking::Client::new();
                let request = client.post(api_config.transcription_url());
                let request = api_config.add_auth_header(request);

                let response = request.multipart(form).send().map_err(|e| {
                    eprintln!("[OpenAI Client] Azure test request failed: {}", e);
                    TranscriptionError::ApiError(format!("Request failed: {}", e))
                })?;

                let status = response.status();
                println!("[OpenAI Client] Azure test response status: {}", status);

                // Clean up test file
                let _ = std::fs::remove_file(&test_audio_path);

                if status.is_success() {
                    println!("[OpenAI Client] ✅ Azure API key is valid");
                    Ok(true)
                } else if status.as_u16() == 401 {
                    println!("[OpenAI Client] ❌ Azure API key is invalid (401 Unauthorized)");
                    Ok(false)
                } else {
                    let error_text = response
                        .text()
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    eprintln!(
                        "[OpenAI Client] Azure test failed ({}): {}",
                        status, error_text
                    );
                    Err(TranscriptionError::ApiError(format!(
                        "API returned status {}: {}",
                        status, error_text
                    )))
                }
            }
        }
    }

    /// Transcribe audio file to text (blocking/synchronous version)
    ///
    /// # Arguments
    /// * `file_path` - Path to the audio file (WAV, MP3, etc.)
    /// * `duration_ms` - Duration of the recording in milliseconds (for validation)
    /// * `config` - Provider configuration (which provider to use and settings)
    ///
    /// # Returns
    /// * `Ok(String)` - Transcribed text
    /// * `Err(TranscriptionError)` - Error details
    pub fn transcribe_audio_sync(
        &self,
        file_path: PathBuf,
        duration_ms: u64,
        config: &ProviderConfig,
    ) -> Result<String, TranscriptionError> {
        println!(
            "[OpenAI Client] Transcribing (sync): {:?} (duration: {}ms)",
            file_path, duration_ms
        );

        // Validate minimum duration
        if duration_ms < MIN_AUDIO_DURATION_MS {
            eprintln!(
                "[OpenAI Client] Audio too short: {}ms < {}ms",
                duration_ms, MIN_AUDIO_DURATION_MS
            );
            return Ok("".to_string());
        }

        // Check if file exists
        if !file_path.exists() {
            eprintln!("[OpenAI Client] File not found: {:?}", file_path);
            return Err(TranscriptionError::FileNotFound(
                file_path.to_string_lossy().to_string(),
            ));
        }

        // Check file size
        let metadata = std::fs::metadata(&file_path)?;
        let file_size = metadata.len();

        if file_size > MAX_FILE_SIZE_BYTES {
            eprintln!(
                "[OpenAI Client] File too large: {} bytes > {} bytes",
                file_size, MAX_FILE_SIZE_BYTES
            );
            return Err(TranscriptionError::FileTooLarge {
                size_bytes: file_size,
            });
        }

        println!("[OpenAI Client] File size: {} bytes", file_size);

        // Load API configuration
        let api_config = Self::load_config(config)?;
        println!("[OpenAI Client] Using provider: {:?}", api_config.provider);

        // Build multipart form
        let mut form = reqwest::blocking::multipart::Form::new()
            .file("file", &file_path)
            .map_err(|e| {
                TranscriptionError::IoError(std::io::Error::other(format!(
                    "Failed to read file: {}",
                    e
                )))
            })?
            .text("temperature", "0.0")
            // .text("prompt", " ")
            .text("response_format", "json");

        // OpenAI requires model in form data, Azure embeds it in URL
        if api_config.provider == Provider::OpenAI {
            let model = "whisper-1";
            // let model = "gpt-4o-transcribe";
            form = form.text("model", model);
        }

        // Call API
        println!(
            "[OpenAI Client] Sending request to {} API...",
            if api_config.provider == Provider::OpenAI {
                "OpenAI"
            } else {
                "Azure"
            }
        );

        let client = reqwest::blocking::Client::new();
        let request = client.post(api_config.transcription_url());
        let request = api_config.add_auth_header(request);

        let response = request.multipart(form).send().map_err(|e| {
            eprintln!("[OpenAI Client] API request error: {}", e);
            TranscriptionError::ApiError(format!("Request failed: {}", e))
        })?;

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            eprintln!(
                "[OpenAI Client] API error response ({}): {}",
                status, error_text
            );
            return Err(TranscriptionError::ApiError(format!(
                "API returned status {}: {}",
                status, error_text
            )));
        }

        // Parse JSON response
        let json: serde_json::Value = response.json().map_err(|e| {
            eprintln!("[OpenAI Client] Failed to parse response: {}", e);
            TranscriptionError::ApiError(format!("Failed to parse response: {}", e))
        })?;

        let text = json["text"].as_str().unwrap_or("").to_string();

        println!(
            "[OpenAI Client] Transcription successful: {} characters",
            text.len()
        );
        println!("[OpenAI Client] Text: {}", text);

        Ok(text)
    }

    /// Transcribe audio file to text (async version)
    ///
    /// # Arguments
    /// * `file_path` - Path to the audio file (WAV, MP3, etc.)
    /// * `duration_ms` - Duration of the recording in milliseconds (for validation)
    ///
    /// # Returns
    /// * `Ok(String)` - Transcribed text
    /// * `Err(TranscriptionError)` - Error details
    #[allow(dead_code)]
    pub async fn transcribe_audio(
        &self,
        file_path: PathBuf,
        duration_ms: u64,
    ) -> Result<String, TranscriptionError> {
        println!(
            "[OpenAI Client] Transcribing: {:?} (duration: {}ms)",
            file_path, duration_ms
        );

        // Validate minimum duration
        if duration_ms < MIN_AUDIO_DURATION_MS {
            eprintln!(
                "[OpenAI Client] Audio too short: {}ms < {}ms",
                duration_ms, MIN_AUDIO_DURATION_MS
            );
            return Err(TranscriptionError::AudioTooShort { duration_ms });
        }

        // Check if file exists
        if !file_path.exists() {
            eprintln!("[OpenAI Client] File not found: {:?}", file_path);
            return Err(TranscriptionError::FileNotFound(
                file_path.to_string_lossy().to_string(),
            ));
        }

        // Check file size
        let metadata = std::fs::metadata(&file_path)?;
        let file_size = metadata.len();

        if file_size > MAX_FILE_SIZE_BYTES {
            eprintln!(
                "[OpenAI Client] File too large: {} bytes > {} bytes",
                file_size, MAX_FILE_SIZE_BYTES
            );
            return Err(TranscriptionError::FileTooLarge {
                size_bytes: file_size,
            });
        }

        println!("[OpenAI Client] File size: {} bytes", file_size);

        let model = "whisper-1";

        // Build transcription request
        let request = CreateTranscriptionRequestArgs::default()
            .file(file_path.to_string_lossy().to_string())
            .prompt("If input is empty do not return anything")
            .model(model)
            .temperature(0.0)
            .response_format(AudioResponseFormat::Json)
            .build()
            .map_err(|e| TranscriptionError::ApiError(format!("Failed to build request: {}", e)))?;

        // Call OpenAI API
        println!("[OpenAI Client] Sending request to OpenAI API...");
        let response = self.client.audio().transcribe(request).await.map_err(|e| {
            eprintln!("[OpenAI Client] API error: {}", e);
            TranscriptionError::ApiError(format!("{}", e))
        })?;

        println!(
            "[OpenAI Client] Transcription successful: {} characters",
            response.text.len()
        );
        println!("[OpenAI Client] Text: {}", response.text);

        Ok(response.text)
    }
}
