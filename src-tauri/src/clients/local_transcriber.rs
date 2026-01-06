//! Local transcription service implementation.
//!
//! Handles transcription via local Whisper model.

use std::path::Path;
use std::sync::Arc;

use log::info;

use crate::models::ModelLoader;

use super::error::TranscriptionError;
use super::service::TranscriptionService;

/// Local transcription service using Whisper model.
///
/// Uses the ModelLoader to access the loaded Whisper model for transcription.
/// Ensures the correct model is loaded before transcribing, handling race
/// conditions where the model could be unloaded or swapped.
pub struct LocalTranscriber {
    loader: Arc<ModelLoader>,
    selected_model: String,
}

impl LocalTranscriber {
    /// Create a new local transcriber with the given model loader and selected model.
    pub fn new(loader: Arc<ModelLoader>, selected_model: String) -> Self {
        Self {
            loader,
            selected_model,
        }
    }
}

impl TranscriptionService for LocalTranscriber {
    fn transcribe(&self, audio_path: &Path) -> Result<String, TranscriptionError> {
        // Use transcribe_with_model which handles:
        // 1. Loading the model if not already loaded
        // 2. Verifying the correct model is loaded (handles race conditions)
        // 3. Transcribing the audio
        let text = self
            .loader
            .transcribe_with_model(&self.selected_model, audio_path)
            .map_err(TranscriptionError::LocalTranscriptionFailed)?;

        info!("Local transcription successful: {} characters", text.len());

        Ok(text)
    }
}
