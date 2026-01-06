use std::path::Path;

use super::client::TranscriptionClient;
use super::error::TranscriptionError;

const AZURE_API_VERSION: &str = "2024-06-01";

/// Azure OpenAI Whisper API client
pub struct AzureClient {
    api_key: String,
    endpoint: String,
}

impl AzureClient {
    pub fn new(api_key: String, endpoint: String) -> Self {
        Self { api_key, endpoint }
    }
}

impl TranscriptionClient for AzureClient {
    fn transcription_url(&self) -> String {
        // Azure URL format: user provides full endpoint path, we just add api-version
        // Example: https://xxx.cognitiveservices.azure.com/openai/deployments/whisper/audio/transcriptions
        format!(
            "{}?api-version={}",
            self.endpoint.trim_end_matches('/'),
            AZURE_API_VERSION
        )
    }

    fn add_auth(
        &self,
        request: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        request.header("api-key", &self.api_key)
    }

    fn build_form_from_path(
        &self,
        file_path: &Path,
    ) -> Result<reqwest::blocking::multipart::Form, TranscriptionError> {
        // Azure doesn't need model in form - it's embedded in the endpoint URL
        let form = reqwest::blocking::multipart::Form::new()
            .file("file", file_path)
            .map_err(|e| {
                TranscriptionError::IoError(std::io::Error::other(format!(
                    "Failed to read file: {}",
                    e
                )))
            })?
            .text("temperature", "0.0")
            .text("response_format", "json");

        Ok(form)
    }
}
