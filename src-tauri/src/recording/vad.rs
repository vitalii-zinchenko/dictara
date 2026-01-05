//! Voice Activity Detection using Silero VAD
//!
//! This module implements VAD using the official Silero VAD model directly,
//! based on: https://github.com/snakers4/silero-vad/tree/master/examples/rust-example

use ndarray::{Array, Array1, Array2, ArrayBase, ArrayD, Dim, IxDynImpl, OwnedRepr};
use ort::session::Session;
use ort::value::Value;
use std::collections::VecDeque;
use std::mem::take;
use std::path::Path;
use thiserror::Error;

/// Sample rate expected by Whisper/transcription (16kHz)
const SAMPLE_RATE: u32 = 16000;

/// Silero VAD V5/V6 requires exactly 512 samples per frame at 16kHz
/// This is different from V4 which used 480 samples (30ms)
/// 512 samples at 16kHz = 32ms per frame
pub const FRAME_SAMPLES: usize = 512;

/// Context size for 16kHz (64 samples, 32 for 8kHz)
const CONTEXT_SIZE: usize = 64;

/// VAD-specific errors
#[derive(Debug, Error)]
pub enum VadError {
    #[error("Invalid threshold: must be between 0.0 and 1.0")]
    InvalidThreshold,

    #[error("Failed to create VAD: {0}")]
    InitError(String),

    #[error("Invalid frame size: expected {expected} samples, got {actual}")]
    InvalidFrameSize { expected: usize, actual: usize },

    #[error("VAD compute error: {0}")]
    ComputeError(String),
}

/// Result of processing an audio frame through VAD
#[derive(Debug)]
pub enum VadFrame<'a> {
    /// Frame contains speech - include in output
    #[allow(dead_code)]
    Speech(&'a [f32]),
    /// Frame is silence/noise - exclude from output
    Noise,
}

impl VadFrame<'_> {
    /// Returns true if this frame contains speech
    pub fn is_speech(&self) -> bool {
        matches!(self, VadFrame::Speech(_))
    }
}

/// Trait for voice activity detection implementations
pub trait VoiceActivityDetector: Send {
    /// Process a single audio frame and determine if it contains speech
    ///
    /// # Arguments
    /// * `frame` - Audio samples (must be exactly FRAME_SAMPLES length at 16kHz)
    ///
    /// # Returns
    /// * `VadFrame::Speech` - Frame contains voice, include in output
    /// * `VadFrame::Noise` - Frame is silence/noise, exclude from output
    fn push_frame<'a>(&'a mut self, frame: &'a [f32]) -> Result<VadFrame<'a>, VadError>;

    /// Check if a frame contains speech (simpler boolean API)
    fn is_voice(&mut self, frame: &[f32]) -> Result<bool, VadError> {
        Ok(self.push_frame(frame)?.is_speech())
    }

    /// Reset the VAD state (clear any buffered frames)
    fn reset(&mut self);
}

/// Silero VAD implementation using official ONNX model
///
/// Based on: https://github.com/snakers4/silero-vad/tree/master/examples/rust-example
/// Supports Silero VAD V5 and V6 models.
pub struct SileroVad {
    session: Session,
    sample_rate: ArrayBase<OwnedRepr<i64>, Dim<[usize; 1]>>,
    state: ArrayBase<OwnedRepr<f32>, Dim<IxDynImpl>>,
    context: Array1<f32>,
    threshold: f32,
}

impl SileroVad {
    /// Create a new Silero VAD instance
    ///
    /// # Arguments
    /// * `model_path` - Path to the silero_vad.onnx model file (V5 or V6)
    /// * `threshold` - Speech probability threshold (0.0-1.0), typically 0.5
    pub fn new<P: AsRef<Path>>(model_path: P, threshold: f32) -> Result<Self, VadError> {
        if !(0.0..=1.0).contains(&threshold) {
            return Err(VadError::InvalidThreshold);
        }

        let session = Session::builder()
            .map_err(|e| VadError::InitError(e.to_string()))?
            .commit_from_file(model_path)
            .map_err(|e| VadError::InitError(e.to_string()))?;

        // Initialize state tensor [2, 1, 128] for RNN hidden/cell states
        let state = ArrayD::<f32>::zeros([2, 1, 128].as_slice());

        // Context buffer for continuity between frames
        let context = Array1::<f32>::zeros(CONTEXT_SIZE);

        // Sample rate tensor
        let sample_rate = Array::from_shape_vec([1], vec![SAMPLE_RATE as i64])
            .map_err(|e| VadError::InitError(e.to_string()))?;

        log::info!(
            "SileroVad initialized: threshold={}, frame_samples={}, context_size={}",
            threshold,
            FRAME_SAMPLES,
            CONTEXT_SIZE
        );

        Ok(Self {
            session,
            sample_rate,
            state,
            context,
            threshold,
        })
    }

    /// Process an audio frame and return speech probability
    ///
    /// # Arguments
    /// * `audio_frame` - Audio samples as f32 (normalized to -1.0 to 1.0)
    ///
    /// # Returns
    /// Speech probability (0.0 to 1.0)
    fn calc_level(&mut self, audio_frame: &[f32]) -> Result<f32, VadError> {
        // Concatenate context with current frame
        let mut input_with_context = Vec::with_capacity(CONTEXT_SIZE + audio_frame.len());
        input_with_context.extend_from_slice(self.context.as_slice().unwrap());
        input_with_context.extend_from_slice(audio_frame);

        let frame =
            Array2::<f32>::from_shape_vec([1, input_with_context.len()], input_with_context)
                .map_err(|e| VadError::ComputeError(e.to_string()))?;

        let frame_value =
            Value::from_array(frame).map_err(|e| VadError::ComputeError(e.to_string()))?;
        let state_value = Value::from_array(take(&mut self.state))
            .map_err(|e| VadError::ComputeError(e.to_string()))?;
        let sr_value = Value::from_array(self.sample_rate.clone())
            .map_err(|e| VadError::ComputeError(e.to_string()))?;

        let res = self
            .session
            .run([
                (&frame_value).into(),
                (&state_value).into(),
                (&sr_value).into(),
            ])
            .map_err(|e| VadError::ComputeError(e.to_string()))?;

        // Update internal state tensor
        let (shape, state_data) = res["stateN"]
            .try_extract_tensor::<f32>()
            .map_err(|e| VadError::ComputeError(e.to_string()))?;
        let shape_usize: Vec<usize> = shape.as_ref().iter().map(|&d| d as usize).collect();
        self.state = ArrayD::from_shape_vec(shape_usize.as_slice(), state_data.to_vec())
            .map_err(|e| VadError::ComputeError(e.to_string()))?;

        // Update context with last CONTEXT_SIZE samples from current frame
        if audio_frame.len() >= CONTEXT_SIZE {
            self.context =
                Array1::from_vec(audio_frame[audio_frame.len() - CONTEXT_SIZE..].to_vec());
        }

        // Extract speech probability
        let prob = *res["output"]
            .try_extract_tensor::<f32>()
            .map_err(|e| VadError::ComputeError(e.to_string()))?
            .1
            .first()
            .ok_or_else(|| VadError::ComputeError("Empty output tensor".to_string()))?;

        Ok(prob)
    }
}

impl VoiceActivityDetector for SileroVad {
    fn push_frame<'a>(&'a mut self, frame: &'a [f32]) -> Result<VadFrame<'a>, VadError> {
        if frame.len() != FRAME_SAMPLES {
            return Err(VadError::InvalidFrameSize {
                expected: FRAME_SAMPLES,
                actual: frame.len(),
            });
        }

        let prob = self.calc_level(frame)?;

        // Log probability for debugging (TODO: remove after VAD tuning)
        log::debug!("VAD prob: {:.3} (threshold: {:.2})", prob, self.threshold);

        if prob > self.threshold {
            Ok(VadFrame::Speech(frame))
        } else {
            Ok(VadFrame::Noise)
        }
    }

    fn reset(&mut self) {
        log::debug!("SileroVad::reset() - clearing RNN state and context");
        self.state = ArrayD::<f32>::zeros([2, 1, 128].as_slice());
        self.context = Array1::<f32>::zeros(CONTEXT_SIZE);
    }
}

/// Smoothed VAD with hysteresis to prevent choppy detection
///
/// This wrapper adds:
/// - **Onset delay**: Requires N consecutive speech frames before triggering
/// - **Hangover**: Keeps N frames of silence before ending speech segment
/// - **Prefill buffer**: Includes N frames before speech onset
pub struct SmoothedVad {
    inner_vad: Box<dyn VoiceActivityDetector>,
    prefill_frames: usize,
    hangover_frames: usize,
    onset_frames: usize,

    frame_buffer: VecDeque<Vec<f32>>,
    hangover_counter: usize,
    onset_counter: usize,
    in_speech: bool,

    // Temporary buffer for returning prefill + current frame
    temp_out: Vec<f32>,
}

impl SmoothedVad {
    /// Create a new smoothed VAD
    ///
    /// # Arguments
    /// * `inner_vad` - The underlying VAD implementation
    /// * `prefill_frames` - Number of frames to include before speech onset (lookback)
    /// * `hangover_frames` - Number of silent frames to allow before ending speech
    /// * `onset_frames` - Number of consecutive speech frames required to start
    pub fn new(
        inner_vad: Box<dyn VoiceActivityDetector>,
        prefill_frames: usize,
        hangover_frames: usize,
        onset_frames: usize,
    ) -> Self {
        Self {
            inner_vad,
            prefill_frames,
            hangover_frames,
            onset_frames,
            frame_buffer: VecDeque::new(),
            hangover_counter: 0,
            onset_counter: 0,
            in_speech: false,
            temp_out: Vec::new(),
        }
    }
}

impl VoiceActivityDetector for SmoothedVad {
    fn push_frame<'a>(&'a mut self, frame: &'a [f32]) -> Result<VadFrame<'a>, VadError> {
        // 1. Buffer every incoming frame for possible pre-roll
        self.frame_buffer.push_back(frame.to_vec());
        while self.frame_buffer.len() > self.prefill_frames + 1 {
            self.frame_buffer.pop_front();
        }

        // 2. Delegate to the wrapped boolean VAD
        let is_voice = self.inner_vad.is_voice(frame)?;

        match (self.in_speech, is_voice) {
            // Potential start of speech - need to accumulate onset frames
            (false, true) => {
                self.onset_counter += 1;
                if self.onset_counter >= self.onset_frames {
                    // We have enough consecutive voice frames to trigger speech
                    self.in_speech = true;
                    self.hangover_counter = self.hangover_frames;
                    self.onset_counter = 0; // Reset for next time

                    // Collect prefill + current frame
                    self.temp_out.clear();
                    for buf in &self.frame_buffer {
                        self.temp_out.extend(buf);
                    }
                    log::debug!(
                        "SmoothedVad: SPEECH ONSET, returning {} samples from prefill",
                        self.temp_out.len()
                    );
                    Ok(VadFrame::Speech(&self.temp_out))
                } else {
                    // Not enough frames yet, still silence
                    Ok(VadFrame::Noise)
                }
            }

            // Ongoing Speech
            (true, true) => {
                self.hangover_counter = self.hangover_frames;
                Ok(VadFrame::Speech(frame))
            }

            // End of Speech or interruption during onset phase
            (true, false) => {
                if self.hangover_counter > 0 {
                    self.hangover_counter -= 1;
                    Ok(VadFrame::Speech(frame))
                } else {
                    self.in_speech = false;
                    log::debug!("SmoothedVad: SPEECH END");
                    Ok(VadFrame::Noise)
                }
            }

            // Silence or broken onset sequence
            (false, false) => {
                self.onset_counter = 0; // Reset onset counter on silence
                Ok(VadFrame::Noise)
            }
        }
    }

    fn reset(&mut self) {
        log::debug!("SmoothedVad::reset() - clearing buffers and state");
        self.frame_buffer.clear();
        self.hangover_counter = 0;
        self.onset_counter = 0;
        self.in_speech = false;
        self.temp_out.clear();
        self.inner_vad.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_samples_for_v6() {
        // Silero V5/V6 requires 512 samples at 16kHz
        assert_eq!(FRAME_SAMPLES, 512);
    }

    #[test]
    fn test_context_size() {
        // Context size is 64 for 16kHz
        assert_eq!(CONTEXT_SIZE, 64);
    }
}
