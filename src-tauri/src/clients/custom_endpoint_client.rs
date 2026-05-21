use std::path::Path;

use secrecy::{ExposeSecret, SecretString};

use super::client::TranscriptionClient;
use super::error::TranscriptionError;

/// OpenAI-Compatible Custom Endpoint API client
pub struct CustomEndpointClient {
    api_key: Option<SecretString>,
    base_url: String,
    model: String,
}

impl CustomEndpointClient {
    pub fn new(api_key: Option<SecretString>, base_url: String, model: String) -> Self {
        Self {
            api_key,
            base_url,
            model,
        }
    }
}

impl TranscriptionClient for CustomEndpointClient {
    fn transcription_url(&self) -> String {
        let mut url = self.base_url.trim().to_string();
        if !url.ends_with("/audio/transcriptions") {
            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str("audio/transcriptions");
        }
        url
    }

    fn add_auth(
        &self,
        request: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        if let Some(ref api_key) = self.api_key {
            let key = api_key.expose_secret();
            if !key.is_empty() {
                return request.bearer_auth(key);
            }
        }
        request
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
            .text("model", self.model.clone())
            .text("temperature", "0.0")
            .text("response_format", "json");

        Ok(form)
    }
}
