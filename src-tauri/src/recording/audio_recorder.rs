use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use hound::{WavSpec, WavWriter};
use log::{error, info, warn};
use rubato::{FftFixedInOut, Resampler};
use std::fs::{self, File};
use std::io::{self, BufWriter};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tauri::ipc::Channel;
use tauri::Manager;

use super::vad::{SileroVad, SmoothedVad, VoiceActivityDetector, FRAME_SAMPLES};

/// Sample rate for transcription (16kHz mono)
const SAMPLE_RATE: u32 = 16000;

/// VAD threshold - probability above which a frame is considered speech
/// Silero V6 is well-calibrated, 0.5 is the standard threshold
/// Lower = more sensitive to quiet speech, but may pick up noise
/// Higher = stricter, may miss whispers
const VAD_THRESHOLD: f32 = 0.5;

/// Number of frames to buffer before speech onset (lookback)
/// 14 frames × 32ms = 448ms of audio captured before speech onset
const VAD_PREFILL_FRAMES: usize = 14;

/// Number of silent frames allowed during speech before ending segment (~448ms at 32ms frames)
const VAD_HANGOVER_FRAMES: usize = 14;

/// Number of consecutive speech frames required to trigger onset
/// 2 frames × 32ms = 64ms of consecutive speech required to trigger
const VAD_ONSET_FRAMES: usize = 2;

/// Debug: save raw audio file before VAD filtering
/// When true, saves both raw and VAD-filtered files for comparison
/// TODO: Set back to false after debugging VAD
const SAVE_RAW_AUDIO_DEBUG: bool = false;

#[derive(Debug, Clone)]
pub struct RecordingResult {
    pub file_path: String,
    /// Total wall-clock duration of the recording in milliseconds (for logging/debug)
    #[allow(dead_code)]
    pub duration_ms: u64,
    /// Duration of detected speech in milliseconds (after VAD filtering)
    pub speech_duration_ms: u64,
}

/// Active recording session - owns all recording state and lifecycle
pub struct Recording {
    stream: cpal::Stream,
    writer: Arc<Mutex<WavWriter<BufWriter<File>>>>,
    /// Optional raw audio writer (before VAD) for debugging
    raw_writer: Option<Arc<Mutex<WavWriter<BufWriter<File>>>>>,
    start_timestamp: SystemTime,
    filename: String,
    app_handle: tauri::AppHandle,
    /// Count of speech samples written (for calculating speech duration)
    speech_sample_count: Arc<AtomicUsize>,
}

impl Recording {
    /// Stop the recording and return the result
    pub fn stop(self) -> Result<RecordingResult, RecorderError> {
        use cpal::traits::StreamTrait;

        // Pause and drop the stream
        self.stream.pause().ok();
        drop(self.stream);

        // Construct file path
        let audio_dir = ensure_audio_dir_exists(&self.app_handle)?;
        let file_path = audio_dir.join(&self.filename);

        // Finalize VAD-filtered WAV file
        if let Ok(writer_mutex) = Arc::try_unwrap(self.writer) {
            if let Ok(writer) = writer_mutex.into_inner() {
                if let Err(e) = writer.finalize() {
                    error!("Error finalizing WAV: {}", e);
                }
            }
        }

        // Finalize raw WAV file (if debug mode enabled)
        if let Some(raw_writer) = self.raw_writer {
            if let Ok(writer_mutex) = Arc::try_unwrap(raw_writer) {
                if let Ok(writer) = writer_mutex.into_inner() {
                    if let Err(e) = writer.finalize() {
                        error!("Error finalizing raw WAV: {}", e);
                    }
                }
            }
        }

        // Calculate wall-clock duration
        let duration_ms = SystemTime::now()
            .duration_since(self.start_timestamp)
            .unwrap()
            .as_millis() as u64;

        // Calculate speech duration from VAD-filtered samples
        let speech_samples = self.speech_sample_count.load(Ordering::Relaxed);
        let speech_duration_ms = (speech_samples as u64 * 1000) / SAMPLE_RATE as u64;

        info!(
            "Recording stopped: wall-clock={}ms, speech={}ms ({} samples)",
            duration_ms, speech_duration_ms, speech_samples
        );

        Ok(RecordingResult {
            file_path: file_path.to_string_lossy().to_string(),
            duration_ms,
            speech_duration_ms,
        })
    }
}

pub struct AudioRecorder {
    app_handle: tauri::AppHandle,
    /// VAD instance - created once, reused across recordings
    vad: Option<Arc<Mutex<Box<dyn VoiceActivityDetector>>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum RecorderError {
    #[error("No input device")]
    NoInputDevice,

    #[error("Device error")]
    DeviceError,

    #[error("Failed to build stream: {0}")]
    BuildStreamError(#[from] cpal::BuildStreamError),

    #[error("Failed to play stream: {0}")]
    PlayStreamError(#[from] cpal::PlayStreamError),

    #[error("Failed to pause stream: {0}")]
    PauseStreamError(#[from] cpal::PauseStreamError),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Audio encoding error: {0}")]
    EncodingError(#[from] hound::Error),
}

// TODO: this should be moved to the controller layer
impl RecorderError {
    /// Returns a user-friendly error message suitable for display in the UI
    pub fn user_message(&self) -> String {
        match self {
            RecorderError::NoInputDevice | RecorderError::DeviceError => {
                "No microphone found. Please connect one and try again.".to_string()
            }
            RecorderError::BuildStreamError(_)
            | RecorderError::PlayStreamError(_)
            | RecorderError::PauseStreamError(_) => {
                "Microphone error. Check your audio settings.".to_string()
            }
            RecorderError::IoError(_) | RecorderError::EncodingError(_) => {
                "Failed to save recording. Check disk space.".to_string()
            }
        }
    }
}

impl AudioRecorder {
    /// Create a new AudioRecorder with VAD initialized
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        let vad = Self::create_vad(&app_handle);
        AudioRecorder { app_handle, vad }
    }

    /// Start a new recording session
    pub fn start(&self, level_channel: Option<Channel<f32>>) -> Result<Recording, RecorderError> {
        // Ensure audio directory exists
        let audio_dir = ensure_audio_dir_exists(&self.app_handle)?;

        // Get audio host and device first
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(RecorderError::NoInputDevice)?;

        // Get default device config - we'll always resample to 16kHz
        let config = device
            .default_input_config()
            .map_err(|_| RecorderError::DeviceError)?;

        // Generate filename
        let filename = generate_filename();
        let file_path = audio_dir.join(&filename);

        // Always write 16kHz mono to file (optimal for speech transcription)
        let spec = WavSpec {
            channels: 1,              // Always mono
            sample_rate: SAMPLE_RATE, // Always 16kHz
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let needs_channel_conversion = config.channels() != 1;

        let writer = AudioRecorder::create_wav_writer(file_path.clone(), spec)?;
        let writer = Arc::new(Mutex::new(writer));

        // Create raw audio writer for debugging (before VAD filtering)
        let raw_writer = if SAVE_RAW_AUDIO_DEBUG {
            let raw_filename = filename.replace(".wav", "_raw.wav");
            let raw_path = audio_dir.join(&raw_filename);
            match AudioRecorder::create_wav_writer(raw_path, spec) {
                Ok(w) => {
                    info!("Debug: saving raw audio to {}", raw_filename);
                    Some(Arc::new(Mutex::new(w)))
                }
                Err(e) => {
                    warn!("Failed to create raw audio writer: {:?}", e);
                    None
                }
            }
        } else {
            None
        };

        // Always create resampler (device sample rate → 16kHz)
        let input_rate = config.sample_rate().0 as usize;
        let output_rate = SAMPLE_RATE as usize;
        let channels = config.channels() as usize;

        let (resampler, required_chunk_size) =
            match FftFixedInOut::<f32>::new(input_rate, output_rate, 1024, channels) {
                Ok(r) => {
                    let input_frames = r.input_frames_next();
                    (Arc::new(Mutex::new(r)), input_frames)
                }
                Err(e) => {
                    error!("Failed to create resampler: {:?}", e);
                    return Err(RecorderError::DeviceError);
                }
            };

        // Create sample buffer for accumulating samples before resampling
        let sample_buffer: Arc<Mutex<Vec<Vec<f32>>>> =
            Arc::new(Mutex::new(vec![Vec::new(); channels]));

        // Reset and clone VAD for this recording session
        let vad = self.vad.clone();
        if let Some(ref vad_arc) = vad {
            if let Ok(mut vad_guard) = vad_arc.lock() {
                vad_guard.reset();
            }
        }

        // Speech sample counter for tracking VAD-filtered duration
        let speech_sample_count = Arc::new(AtomicUsize::new(0));

        // Build input stream
        let writer_clone = Arc::clone(&writer);
        let err_writer_clone = Arc::clone(&writer);
        let speech_count_clone = Arc::clone(&speech_sample_count);

        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => build_input_stream::<i8>(
                &device,
                &config.into(),
                writer_clone,
                raw_writer.clone(),
                level_channel,
                resampler.clone(),
                sample_buffer.clone(),
                required_chunk_size,
                needs_channel_conversion,
                vad,
                speech_count_clone,
            )?,
            cpal::SampleFormat::I16 => build_input_stream::<i16>(
                &device,
                &config.into(),
                writer_clone,
                raw_writer.clone(),
                level_channel,
                resampler.clone(),
                sample_buffer.clone(),
                required_chunk_size,
                needs_channel_conversion,
                vad,
                speech_count_clone,
            )?,
            cpal::SampleFormat::I32 => build_input_stream::<i32>(
                &device,
                &config.into(),
                writer_clone,
                raw_writer.clone(),
                level_channel,
                resampler.clone(),
                sample_buffer.clone(),
                required_chunk_size,
                needs_channel_conversion,
                vad,
                speech_count_clone,
            )?,
            cpal::SampleFormat::F32 => build_input_stream::<f32>(
                &device,
                &config.into(),
                writer_clone,
                raw_writer.clone(),
                level_channel,
                resampler.clone(),
                sample_buffer.clone(),
                required_chunk_size,
                needs_channel_conversion,
                vad,
                speech_count_clone,
            )?,
            _ => return Err(RecorderError::DeviceError),
        };

        // Start the stream
        stream.play()?;

        // Record start timestamp
        let start_timestamp = SystemTime::now();

        // Return Recording session
        Ok(Recording {
            stream,
            writer: err_writer_clone,
            raw_writer,
            start_timestamp,
            filename,
            app_handle: self.app_handle.clone(),
            speech_sample_count,
        })
    }

    /// Create VAD instance for filtering silence
    ///
    /// The VAD chain is: SmoothedVad → SileroVad
    /// - SileroVad: Silero V6 neural network-based voice detection (official implementation)
    /// - SmoothedVad: Adds hysteresis (prefill, hangover, onset) for smooth detection
    fn create_vad(
        app_handle: &tauri::AppHandle,
    ) -> Option<Arc<Mutex<Box<dyn VoiceActivityDetector>>>> {
        // Resolve VAD model path from resources (Silero V6)
        let vad_path = app_handle
            .path()
            .resolve(
                "resources/models/silero_vad_v6.onnx",
                tauri::path::BaseDirectory::Resource,
            )
            .ok()?;

        if !vad_path.exists() {
            warn!(
                "VAD model not found at {:?}, recording without VAD",
                vad_path
            );
            return None;
        }

        // Create Silero VAD V6 (official implementation)
        let silero = match SileroVad::new(&vad_path, VAD_THRESHOLD) {
            Ok(vad) => vad,
            Err(e) => {
                warn!("Failed to create Silero VAD: {}, recording without VAD", e);
                return None;
            }
        };

        // Wrap in SmoothedVad for hysteresis (prefill, hangover, onset filtering)
        let smoothed = SmoothedVad::new(
            Box::new(silero),
            VAD_PREFILL_FRAMES,
            VAD_HANGOVER_FRAMES,
            VAD_ONSET_FRAMES,
        );

        info!("VAD V6 initialized from {:?}", vad_path);
        Some(Arc::new(Mutex::new(
            Box::new(smoothed) as Box<dyn VoiceActivityDetector>
        )))
    }

    fn create_wav_writer(
        file_path: PathBuf,
        spec: WavSpec,
    ) -> Result<WavWriter<io::BufWriter<fs::File>>, RecorderError> {
        let file = fs::File::create(file_path)?;
        let buf_writer = io::BufWriter::new(file);
        Ok(WavWriter::new(buf_writer, spec)?)
    }
}

fn ensure_audio_dir_exists(app_handle: &tauri::AppHandle) -> Result<PathBuf, RecorderError> {
    let cache_dir = app_handle.path().app_cache_dir().map_err(|_| {
        RecorderError::IoError(io::Error::new(
            io::ErrorKind::NotFound,
            "Cache directory not found",
        ))
    })?;

    let audio_dir = cache_dir.join("recordings");

    if !audio_dir.exists() {
        fs::create_dir_all(&audio_dir)?;
    }
    Ok(audio_dir)
}

/// Clean up a recording file
/// Logs errors but doesn't fail - cleanup is best-effort
pub fn cleanup_recording_file(file_path: &str) {
    if let Err(e) = fs::remove_file(file_path) {
        error!("Failed to cleanup recording file {}: {}", file_path, e);
    }
}

/// Clean up old recording files on app startup
/// Only deletes files matching pattern: recording_*.wav
pub fn cleanup_old_recordings(app_handle: &tauri::AppHandle) {
    let recordings_dir = match app_handle.path().app_cache_dir() {
        Ok(cache_dir) => cache_dir.join("recordings"),
        Err(_) => return,
    };

    let entries = match fs::read_dir(&recordings_dir) {
        Ok(entries) => entries,
        Err(_) => return, // Directory doesn't exist yet, nothing to clean
    };

    let mut cleaned = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let is_old_recording = filename.starts_with("recording_") && filename.ends_with(".wav");
        if !is_old_recording {
            continue;
        }

        if fs::remove_file(&path).is_ok() {
            cleaned += 1;
        }
    }

    if cleaned > 0 {
        info!("Cleaned up {} old recording(s)", cleaned);
    }
}

fn generate_filename() -> String {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("recording_{}.wav", timestamp)
}

#[allow(clippy::too_many_arguments)]
fn build_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    writer: Arc<Mutex<WavWriter<BufWriter<File>>>>,
    raw_writer: Option<Arc<Mutex<WavWriter<BufWriter<File>>>>>,
    level_channel: Option<Channel<f32>>,
    resampler: Arc<Mutex<FftFixedInOut<f32>>>,
    sample_buffer: Arc<Mutex<Vec<Vec<f32>>>>,
    required_chunk_size: usize,
    needs_channel_conversion: bool,
    vad: Option<Arc<Mutex<Box<dyn VoiceActivityDetector>>>>,
    speech_sample_count: Arc<AtomicUsize>,
) -> Result<cpal::Stream, RecorderError>
where
    T: Sample + FromSample<i16> + FromSample<f32> + std::fmt::Debug + cpal::SizedSample,
    i16: FromSample<T>,
    f32: FromSample<T>,
{
    let err_fn = |err| {
        error!("Stream error: {}", err);
    };

    // VAD frame buffer for accumulating samples into FRAME_SAMPLES chunks
    let vad_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));

    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            write_input_data::<T>(
                data,
                &writer,
                &raw_writer,
                &level_channel,
                &resampler,
                &sample_buffer,
                required_chunk_size,
                needs_channel_conversion,
                &vad,
                &vad_buffer,
                &speech_sample_count,
            );
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

#[allow(clippy::too_many_arguments)]
fn write_input_data<T>(
    input: &[T],
    writer: &Arc<Mutex<WavWriter<BufWriter<File>>>>,
    raw_writer: &Option<Arc<Mutex<WavWriter<BufWriter<File>>>>>,
    level_channel: &Option<Channel<f32>>,
    resampler: &Arc<Mutex<FftFixedInOut<f32>>>,
    sample_buffer: &Arc<Mutex<Vec<Vec<f32>>>>,
    required_chunk_size: usize,
    needs_channel_conversion: bool,
    vad: &Option<Arc<Mutex<Box<dyn VoiceActivityDetector>>>>,
    vad_buffer: &Arc<Mutex<Vec<f32>>>,
    speech_sample_count: &Arc<AtomicUsize>,
) where
    T: Sample,
    i16: FromSample<T>,
    f32: FromSample<T>,
{
    // Calculate RMS (Root Mean Square) for audio level visualization (use original samples)
    if !input.is_empty() {
        let sum_of_squares: f32 = input
            .iter()
            .map(|&sample| {
                let sample_f32: f32 = sample.to_sample();
                sample_f32 * sample_f32
            })
            .sum();
        let rms = (sum_of_squares / input.len() as f32).sqrt();

        if let Some(channel) = level_channel {
            let level = (rms * 100.0).min(1.0);
            let _ = channel.send(level);
        }
    }

    // Convert samples to f32 and organize by channel, then append to buffer
    let num_channels = if needs_channel_conversion { 2 } else { 1 };

    let mut buffer_guard = match sample_buffer.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    // Append incoming samples to buffer
    for (i, &sample) in input.iter().enumerate() {
        let channel_idx = i % num_channels;
        let sample_f32: f32 = sample.to_sample();
        buffer_guard[channel_idx].push(sample_f32);
    }

    // Process complete chunks of required_chunk_size samples
    while buffer_guard[0].len() >= required_chunk_size {
        // Extract required_chunk_size samples from each channel
        let channel_chunks: Vec<Vec<f32>> = buffer_guard
            .iter_mut()
            .map(|ch| ch.drain(..required_chunk_size).collect())
            .collect();

        // Release buffer lock before resampling (to avoid holding multiple locks)
        drop(buffer_guard);

        // Resample the chunk
        let resampled = {
            let mut resampler_guard = match resampler.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };

            let channel_refs: Vec<&[f32]> = channel_chunks.iter().map(|v| v.as_slice()).collect();

            match resampler_guard.process(&channel_refs, None) {
                Ok(resampled) => resampled,
                Err(_) => return,
            }
        };

        // Convert to mono if needed (average stereo channels)
        let mono_samples = if needs_channel_conversion && resampled.len() >= 2 {
            let mut mono = Vec::with_capacity(resampled[0].len());
            for (left, right) in resampled[0].iter().zip(resampled[1].iter()) {
                let mixed = (left + right) / 2.0;
                mono.push(mixed);
            }
            mono
        } else {
            // Already mono, just use first channel
            resampled[0].clone()
        };

        // Write raw audio before VAD (for debugging)
        if let Some(raw_w) = raw_writer {
            write_samples_to_raw_wav(&mono_samples, raw_w);
        }

        // Process through VAD and write only speech frames
        process_through_vad_and_write(&mono_samples, writer, vad, vad_buffer, speech_sample_count);

        // Re-acquire buffer lock for next iteration
        buffer_guard = match sample_buffer.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
    }
    // Remaining samples (< required_chunk_size) stay in buffer for next call
}

/// Process mono samples through VAD and write only speech to WAV
fn process_through_vad_and_write(
    mono_samples: &[f32],
    writer: &Arc<Mutex<WavWriter<BufWriter<File>>>>,
    vad: &Option<Arc<Mutex<Box<dyn VoiceActivityDetector>>>>,
    vad_buffer: &Arc<Mutex<Vec<f32>>>,
    speech_sample_count: &Arc<AtomicUsize>,
) {
    // If no VAD, write everything (fallback behavior)
    let Some(vad_arc) = vad else {
        write_samples_to_wav(mono_samples, writer, speech_sample_count);
        return;
    };

    // Accumulate samples in VAD buffer
    let mut vad_buf = match vad_buffer.lock() {
        Ok(guard) => guard,
        Err(_) => {
            // On lock failure, write everything as fallback
            write_samples_to_wav(mono_samples, writer, speech_sample_count);
            return;
        }
    };

    vad_buf.extend_from_slice(mono_samples);

    // Process complete FRAME_SAMPLES chunks through VAD
    while vad_buf.len() >= FRAME_SAMPLES {
        let frame: Vec<f32> = vad_buf.drain(..FRAME_SAMPLES).collect();

        // Process frame through VAD - copy speech samples while holding lock
        let speech_samples: Option<Vec<f32>> = {
            let mut vad_guard = match vad_arc.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    // On lock failure, assume speech
                    write_samples_to_wav(&frame, writer, speech_sample_count);
                    continue;
                }
            };

            match vad_guard.push_frame(&frame) {
                Ok(super::vad::VadFrame::Speech(buf)) => {
                    // Copy the buffer while holding the lock
                    Some(buf.to_vec())
                }
                Ok(super::vad::VadFrame::Noise) => None,
                Err(e) => {
                    warn!("VAD error: {}, assuming speech", e);
                    Some(frame.clone())
                }
            }
        }; // vad_guard released here

        // Write speech samples after releasing VAD lock
        if let Some(samples) = speech_samples {
            write_samples_to_wav(&samples, writer, speech_sample_count);
        }
    }
    // Remaining samples (< FRAME_SAMPLES) stay in vad_buffer for next call
}

/// Write samples to WAV file and update speech sample count
fn write_samples_to_wav(
    samples: &[f32],
    writer: &Arc<Mutex<WavWriter<BufWriter<File>>>>,
    speech_sample_count: &Arc<AtomicUsize>,
) {
    if let Ok(mut guard) = writer.lock() {
        for sample_f32 in samples.iter() {
            let clamped = sample_f32.clamp(-1.0, 1.0);
            let sample_i16 = (clamped * 32767.0) as i16;
            guard.write_sample(sample_i16).ok();
        }
        // Track how many samples we've written (for speech duration calculation)
        speech_sample_count.fetch_add(samples.len(), Ordering::Relaxed);
    }
}

/// Write samples to raw WAV file (debug - before VAD filtering)
fn write_samples_to_raw_wav(samples: &[f32], writer: &Arc<Mutex<WavWriter<BufWriter<File>>>>) {
    if let Ok(mut guard) = writer.lock() {
        for sample_f32 in samples.iter() {
            let clamped = sample_f32.clamp(-1.0, 1.0);
            let sample_i16 = (clamped * 32767.0) as i16;
            guard.write_sample(sample_i16).ok();
        }
    }
}
