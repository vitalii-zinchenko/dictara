//! High-level transcription service abstraction.
//!
//! This trait abstracts away the transcription implementation details,
//! allowing uniform handling of both API-based (OpenAI, Azure) and
//! local (Whisper) transcription.

use std::path::Path;

use super::error::TranscriptionError;

/// High-level transcription service abstraction.
///
/// Implementations can be API-based (OpenAI, Azure) or local (Whisper).
/// The caller doesn't need to know which implementation is being used.
pub trait TranscriptionService: Send + Sync {
    /// Transcribe audio file to text.
    ///
    /// # Arguments
    /// * `audio_path` - Path to the audio file (WAV format, 16kHz mono)
    ///
    /// # Returns
    /// * `Ok(String)` - Transcribed text
    /// * `Err(TranscriptionError)` - Transcription failed
    fn transcribe(&self, audio_path: &Path) -> Result<String, TranscriptionError>;
}
