use std::path::Path;

use super::error::TranscriptionError;

/// Trait for transcription API clients (OpenAI, Azure, etc.)
///
/// Each implementation knows how to:
/// - Construct the correct API URL
/// - Add proper authentication headers
/// - Build the multipart form with provider-specific fields
pub trait TranscriptionClient: Send + Sync {
    /// Get the transcription API endpoint URL
    fn transcription_url(&self) -> String;

    /// Add authentication to the request builder
    fn add_auth(
        &self,
        request: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder;

    /// Build multipart form from file path
    fn build_form_from_path(
        &self,
        file_path: &Path,
    ) -> Result<reqwest::blocking::multipart::Form, TranscriptionError>;

    /// Build multipart form from raw bytes (for testing with static audio)
    fn build_form_from_bytes(
        &self,
        audio_bytes: &[u8],
        filename: &str,
    ) -> Result<reqwest::blocking::multipart::Form, TranscriptionError>;
}
