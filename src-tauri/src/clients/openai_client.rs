use std::path::Path;

use secrecy::{ExposeSecret, SecretString};

use super::client::TranscriptionClient;
use super::error::TranscriptionError;
use crate::config::OpenAITranscriptionModel;

const OPENAI_TRANSCRIPTION_URL: &str = "https://api.openai.com/v1/audio/transcriptions";

/// OpenAI file-transcription API client
pub struct OpenAIClient {
    api_key: SecretString,
    model: OpenAITranscriptionModel,
}

impl OpenAIClient {
    pub fn new(api_key: SecretString, model: OpenAITranscriptionModel) -> Self {
        Self { api_key, model }
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
        request.bearer_auth(self.api_key.expose_secret())
    }

    fn build_form_from_path(
        &self,
        file_path: &Path,
    ) -> Result<reqwest::blocking::multipart::Form, TranscriptionError> {
        let mut form = reqwest::blocking::multipart::Form::new()
            .file("file", file_path)
            .map_err(|e| {
                TranscriptionError::IoError(std::io::Error::other(format!(
                    "Failed to read file: {}",
                    e
                )))
            })?
            .text("model", self.model.as_api_id())
            .text("response_format", self.model.response_format());

        // Only whisper-1 documents a sampling temperature; sending it to the
        // GPT-based transcription models is rejected as an unknown parameter.
        if self.model.supports_temperature() {
            form = form.text("temperature", "0.0");
        }

        // gpt-4o-transcribe-diarize requires a chunking strategy for audio
        // longer than 30 seconds.
        if self.model.requires_chunking_strategy() {
            form = form.text("chunking_strategy", "auto");
        }

        Ok(form)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client(model: OpenAITranscriptionModel) -> OpenAIClient {
        OpenAIClient::new(SecretString::from("sk-test".to_string()), model)
    }

    #[test]
    fn uses_the_selected_model_endpoint() {
        assert_eq!(
            client(OpenAITranscriptionModel::GptTranscribe).transcription_url(),
            OPENAI_TRANSCRIPTION_URL
        );
    }

    #[test]
    fn builds_a_form_for_every_supported_model() {
        let mut file = std::env::temp_dir();
        file.push("dictara_openai_client_form_test.wav");
        std::fs::write(&file, b"RIFF").expect("write temp audio");

        for model in [
            OpenAITranscriptionModel::GptTranscribe,
            OpenAITranscriptionModel::Gpt4oTranscribe,
            OpenAITranscriptionModel::Gpt4oMiniTranscribe,
            OpenAITranscriptionModel::Gpt4oTranscribeDiarize,
            OpenAITranscriptionModel::Whisper1,
        ] {
            assert!(
                client(model).build_form_from_path(&file).is_ok(),
                "form must build for {}",
                model.as_api_id()
            );
        }

        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn missing_file_is_an_io_error() {
        let missing = std::env::temp_dir().join("dictara_openai_client_missing.wav");
        let _ = std::fs::remove_file(&missing);

        let result = client(OpenAITranscriptionModel::Whisper1).build_form_from_path(&missing);
        assert!(matches!(result, Err(TranscriptionError::IoError(_))));
    }
}
