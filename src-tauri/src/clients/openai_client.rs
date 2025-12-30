use std::path::Path;

use super::client::TranscriptionClient;
use super::error::TranscriptionError;

const OPENAI_TRANSCRIPTION_URL: &str = "https://api.openai.com/v1/audio/transcriptions";
const OPENAI_MODEL: &str = "whisper-1";

/// OpenAI Whisper API client
pub struct OpenAIClient {
    api_key: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl TranscriptionClient for OpenAIClient {
    fn transcription_url(&self) -> String {
        OPENAI_TRANSCRIPTION_URL.to_string()
    }

    fn add_auth(
        &self,
        request: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        request.bearer_auth(&self.api_key)
    }

    fn build_form_from_path(
        &self,
        file_path: &Path,
    ) -> Result<reqwest::blocking::multipart::Form, TranscriptionError> {
        let form = reqwest::blocking::multipart::Form::new()
            .file("file", file_path)
            .map_err(|e| {
                TranscriptionError::IoError(std::io::Error::other(format!(
                    "Failed to read file: {}",
                    e
                )))
            })?
            .text("model", OPENAI_MODEL)
            .text("temperature", "0.0")
            .text("response_format", "json");

        Ok(form)
    }

    fn build_form_from_bytes(
        &self,
        audio_bytes: &[u8],
        filename: &str,
    ) -> Result<reqwest::blocking::multipart::Form, TranscriptionError> {
        let audio_part = reqwest::blocking::multipart::Part::bytes(audio_bytes.to_vec())
            .file_name(filename.to_string())
            .mime_str("audio/wav")
            .map_err(|e| {
                TranscriptionError::ApiError(format!("Failed to create audio part: {}", e))
            })?;

        let form = reqwest::blocking::multipart::Form::new()
            .part("file", audio_part)
            .text("model", OPENAI_MODEL)
            .text("temperature", "0.0")
            .text("response_format", "json");

        Ok(form)
    }
}
