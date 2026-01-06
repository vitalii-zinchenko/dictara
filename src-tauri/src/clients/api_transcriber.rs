//! API-based transcription service implementation.
//!
//! Handles transcription via HTTP APIs (OpenAI, Azure OpenAI).

use std::path::Path;

use log::{error, info};

use super::client::TranscriptionClient;
use super::error::TranscriptionError;
use super::service::TranscriptionService;

/// API-based transcription service.
///
/// Uses HTTP APIs (OpenAI Whisper API or Azure OpenAI) for transcription.
pub struct ApiTranscriber {
    client: Box<dyn TranscriptionClient>,
}

impl ApiTranscriber {
    /// Create a new API transcriber with the given client.
    pub fn new(client: Box<dyn TranscriptionClient>) -> Self {
        Self { client }
    }
}

impl TranscriptionService for ApiTranscriber {
    fn transcribe(&self, audio_path: &Path) -> Result<String, TranscriptionError> {
        // Build multipart form from file
        let form = self.client.build_form_from_path(audio_path)?;

        // Send request
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

        info!("API transcription successful: {} characters", text.len());

        Ok(text)
    }
}
