#[derive(Debug, thiserror::Error)]
pub enum TranscriptionError {
    #[error("File too large: {size_bytes} bytes")]
    FileTooLarge { size_bytes: u64 },
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("API key not configured")]
    ApiKeyMissing,
}

// TODO: this should be moved to the controller layer
impl TranscriptionError {
    /// Returns a user-friendly error message suitable for display in the UI
    pub fn user_message(&self) -> String {
        match self {
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
}
