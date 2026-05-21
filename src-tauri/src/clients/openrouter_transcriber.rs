use std::path::Path;
use std::time::Duration;

use base64::{prelude::BASE64_STANDARD, Engine as _};
use log::error;
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;

use super::error::TranscriptionError;
use super::service::TranscriptionService;
use super::transcriber::TRANSCRIPTION_TIMEOUT_SECS;

/// OpenRouter Transcription Service (JSON + Base64)
pub struct OpenRouterTranscriber {
    api_key: SecretString,
    model: String,
}

impl OpenRouterTranscriber {
    pub fn new(api_key: SecretString, model: String) -> Self {
        Self { api_key, model }
    }
}

impl TranscriptionService for OpenRouterTranscriber {
    fn transcribe(&self, audio_path: &Path) -> Result<String, TranscriptionError> {
        // Read file to binary vector
        let audio_bytes = std::fs::read(audio_path).map_err(|e| {
            TranscriptionError::IoError(std::io::Error::other(format!(
                "Failed to read audio file for OpenRouter: {}",
                e
            )))
        })?;

        // Convert to Base64
        let base64_audio = BASE64_STANDARD.encode(&audio_bytes);

        let format = audio_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("wav")
            .to_lowercase();

        // Build JSON body
        let body = json!({
            "model": self.model,
            "input_audio": {
                "data": base64_audio,
                "format": format,
            }
        });

        // Create HTTP client with transcription timeout
        let http_client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(TRANSCRIPTION_TIMEOUT_SECS))
            .build()
            .map_err(|e| {
                TranscriptionError::ApiError(format!("Failed to create HTTP client: {}", e))
            })?;

        // Send POST request to OpenRouter
        let request = http_client
            .post("https://openrouter.ai/api/v1/audio/transcriptions")
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.expose_secret()),
            )
            .header("Content-Type", "application/json")
            .json(&body);

        let response = request.send().map_err(|e| {
            if e.is_timeout() {
                error!(
                    "OpenRouter request timed out after {}s",
                    TRANSCRIPTION_TIMEOUT_SECS
                );
                TranscriptionError::TranscriptionTimeout(TRANSCRIPTION_TIMEOUT_SECS)
            } else {
                error!("OpenRouter request error: {}", e);
                TranscriptionError::ApiError(format!("Request failed: {}", e))
            }
        })?;

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("OpenRouter error response ({}): {}", status, error_text);
            return Err(TranscriptionError::ApiError(format!(
                "OpenRouter returned status {}: {}",
                status, error_text
            )));
        }

        // Parse JSON response
        let json: serde_json::Value = response.json().map_err(|e| {
            error!("Failed to parse OpenRouter response: {}", e);
            TranscriptionError::ApiError(format!("Failed to parse response: {}", e))
        })?;

        let text = json["text"].as_str().unwrap_or("").to_string();

        Ok(text)
    }
}
