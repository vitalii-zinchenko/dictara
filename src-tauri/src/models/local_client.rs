use std::path::Path;

use log::{debug, error, info};
use parakeet_rs::{ParakeetTDT, Transcriber};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::clients::TranscriptionError;

use super::catalog::ModelType;

/// Unified transcription engine supporting multiple backends
enum TranscriptionEngine {
    Whisper(WhisperContext),
    Parakeet(Box<ParakeetTDT>),
}

/// Local transcription client for offline transcription.
/// Supports both Whisper (via whisper.cpp with Metal) and Parakeet (via ONNX Runtime).
pub struct LocalClient {
    engine: TranscriptionEngine,
    model_type: ModelType,
}

impl LocalClient {
    /// Load a transcription model into memory.
    /// This is a blocking operation that can take several seconds for large models.
    ///
    /// # Arguments
    /// * `model_path` - Path to the model file (Whisper: .bin file, Parakeet: directory)
    /// * `model_type` - Type of model (Whisper or Parakeet)
    ///
    /// # Returns
    /// * `Ok(LocalClient)` - Model loaded successfully
    /// * `Err(TranscriptionError)` - Failed to load model
    pub fn new(model_path: &Path, model_type: ModelType) -> Result<Self, TranscriptionError> {
        info!("Loading {:?} model from: {:?}", model_type, model_path);

        if !model_path.exists() {
            return Err(TranscriptionError::ModelNotDownloaded(
                model_path.to_string_lossy().to_string(),
            ));
        }

        let engine = match model_type {
            ModelType::Whisper => {
                let params = WhisperContextParameters::default();
                let ctx = WhisperContext::new_with_params(&model_path.to_string_lossy(), params)
                    .map_err(|e| TranscriptionError::ModelLoadFailed(e.to_string()))?;
                TranscriptionEngine::Whisper(ctx)
            }
            ModelType::Parakeet => {
                // Parakeet expects a directory containing model files
                let parakeet = ParakeetTDT::from_pretrained(model_path, None)
                    .map_err(|e| TranscriptionError::ModelLoadFailed(e.to_string()))?;
                TranscriptionEngine::Parakeet(Box::new(parakeet))
            }
        };

        info!("{:?} model loaded successfully", model_type);
        Ok(Self { engine, model_type })
    }

    /// Transcribe an audio file to text.
    ///
    /// # Arguments
    /// * `audio_path` - Path to the audio file (WAV format, 16kHz mono preferred)
    ///
    /// # Returns
    /// * `Ok(String)` - Transcribed text
    /// * `Err(TranscriptionError)` - Transcription failed
    pub fn transcribe_file(&mut self, audio_path: &Path) -> Result<String, TranscriptionError> {
        debug!(
            "Transcribing file with {:?}: {:?}",
            self.model_type, audio_path
        );

        // Load audio samples (both engines use the same format)
        let samples = self.load_audio(audio_path)?;

        let text = match &mut self.engine {
            TranscriptionEngine::Whisper(ctx) => {
                // Create transcription state
                let mut state = ctx
                    .create_state()
                    .map_err(|e| TranscriptionError::LocalTranscriptionFailed(e.to_string()))?;

                // Configure transcription parameters
                let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

                // Set language to auto-detect
                params.set_language(Some("auto"));

                // Disable printing to stdout
                params.set_print_special(false);
                params.set_print_progress(false);
                params.set_print_realtime(false);
                params.set_print_timestamps(false);

                // Run transcription
                state
                    .full(params, &samples)
                    .map_err(|e| TranscriptionError::LocalTranscriptionFailed(e.to_string()))?;

                // Extract text from segments
                self.extract_whisper_text(&state)?
            }
            TranscriptionEngine::Parakeet(parakeet) => {
                // Use the file path directly (parakeet handles audio loading internally)
                let result = parakeet
                    .transcribe_file(audio_path, None)
                    .map_err(|e| TranscriptionError::LocalTranscriptionFailed(e.to_string()))?;
                result.text
            }
        };

        info!("Transcription complete: {} characters", text.len());
        Ok(text)
    }

    /// Load audio file as f32 samples.
    /// Expects 16kHz mono WAV (as produced by the recording layer).
    fn load_audio(&self, audio_path: &Path) -> Result<Vec<f32>, TranscriptionError> {
        let reader = hound::WavReader::open(audio_path).map_err(|e| {
            error!("Failed to open audio file: {}", e);
            TranscriptionError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to open audio file: {}", e),
            ))
        })?;

        let spec = reader.spec();
        debug!(
            "Audio spec: {} Hz, {} channels, {} bits",
            spec.sample_rate, spec.channels, spec.bits_per_sample
        );

        // Verify expected format (recording layer provides 16kHz mono)
        if spec.sample_rate != 16000 {
            error!(
                "Audio format error: sample rate {} Hz (expected 16000 Hz)",
                spec.sample_rate
            );
            return Err(TranscriptionError::LocalTranscriptionFailed(format!(
                "Audio file has unsupported sample rate ({} Hz). \
                The recording system should produce 16kHz audio. \
                This may indicate a recording configuration issue.",
                spec.sample_rate
            )));
        }
        if spec.channels != 1 {
            error!(
                "Audio format error: {} channels (expected 1 mono)",
                spec.channels
            );
            return Err(TranscriptionError::LocalTranscriptionFailed(format!(
                "Audio file has {} channels but mono (1 channel) is required. \
                The recording system should produce mono audio. \
                This may indicate a recording configuration issue.",
                spec.channels
            )));
        }

        // Read samples based on format
        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => {
                let max_val = (1 << (spec.bits_per_sample - 1)) as f32;
                reader
                    .into_samples::<i32>()
                    .filter_map(Result::ok)
                    .map(|s| s as f32 / max_val)
                    .collect()
            }
            hound::SampleFormat::Float => reader
                .into_samples::<f32>()
                .filter_map(Result::ok)
                .collect(),
        };

        debug!("Loaded {} audio samples", samples.len());
        Ok(samples)
    }

    /// Extract transcribed text from all Whisper segments.
    fn extract_whisper_text(
        &self,
        state: &whisper_rs::WhisperState,
    ) -> Result<String, TranscriptionError> {
        let num_segments = state.full_n_segments().map_err(|e| {
            TranscriptionError::LocalTranscriptionFailed(format!("Failed to get segments: {}", e))
        })?;

        let mut text = String::new();

        for i in 0..num_segments {
            let segment_text = state.full_get_segment_text(i).map_err(|e| {
                TranscriptionError::LocalTranscriptionFailed(format!(
                    "Failed to get segment text: {}",
                    e
                ))
            })?;
            text.push_str(&segment_text);
        }

        Ok(text.trim().to_string())
    }
}
