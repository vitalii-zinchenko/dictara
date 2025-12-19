use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use hound::{WavSpec, WavWriter};
use rubato::{FftFixedInOut, Resampler};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tauri::ipc::Channel;
use tauri::Manager;

#[derive(Debug, Clone)]
pub struct RecordingResult {
    pub file_path: String,
    pub duration_ms: u64,
}

/// Active recording session - owns all recording state and lifecycle
pub struct Recording {
    stream: cpal::Stream,
    writer: Arc<Mutex<WavWriter<BufWriter<File>>>>,
    start_timestamp: SystemTime,
    filename: String,
    app_handle: tauri::AppHandle,
}

impl Recording {
    /// Stop the recording and return the result
    pub fn stop(self) -> Result<RecordingResult, RecorderError> {
        use cpal::traits::StreamTrait;

        println!("[Recording] Stopping recording...");

        // Pause and drop the stream
        self.stream.pause().ok();
        drop(self.stream);

        // Construct file path
        let audio_dir = ensure_audio_dir_exists(&self.app_handle)?;
        let file_path = audio_dir.join(&self.filename);

        // Finalize WAV file
        let mut file_size = 0u64;
        if let Ok(writer_mutex) = Arc::try_unwrap(self.writer) {
            if let Ok(writer) = writer_mutex.into_inner() {
                let result = writer.finalize();
                if let Err(e) = result {
                    eprintln!("[Recording] Error finalizing WAV: {}", e);
                } else {
                    println!("[Recording] WAV file finalized successfully");
                }
            }
        }

        // Calculate duration
        let duration_ms = SystemTime::now()
            .duration_since(self.start_timestamp)
            .unwrap()
            .as_millis() as u64;
        let duration_sec = duration_ms as f64 / 1000.0;

        // Get file size
        if let Ok(metadata) = fs::metadata(&file_path) {
            file_size = metadata.len();
            let size_mb = file_size as f64 / (1024.0 * 1024.0);
            println!(
                "[Recording] File size: {} bytes ({:.2} MB)",
                file_size, size_mb
            );
        }

        println!(
            "[Recording] Recording stopped successfully. Duration: {}ms ({:.2}s)",
            duration_ms, duration_sec
        );

        Ok(RecordingResult {
            file_path: file_path.to_string_lossy().to_string(),
            duration_ms,
        })
    }
}

pub struct AudioRecorder {
    app_handle: tauri::AppHandle,
}

#[derive(Debug)]
pub enum RecorderError {
    NoInputDevice,
    DeviceError,
    IoError,
}

impl From<std::io::Error> for RecorderError {
    fn from(_err: std::io::Error) -> Self {
        RecorderError::IoError
    }
}

impl From<cpal::BuildStreamError> for RecorderError {
    fn from(_err: cpal::BuildStreamError) -> Self {
        RecorderError::DeviceError
    }
}

impl From<cpal::PlayStreamError> for RecorderError {
    fn from(_err: cpal::PlayStreamError) -> Self {
        RecorderError::DeviceError
    }
}

impl From<cpal::PauseStreamError> for RecorderError {
    fn from(_err: cpal::PauseStreamError) -> Self {
        RecorderError::DeviceError
    }
}

impl RecorderError {
    /// Returns a user-friendly error message suitable for display in the UI
    pub fn user_message(&self) -> String {
        match self {
            RecorderError::NoInputDevice => {
                "No microphone found. Please connect one and try again.".to_string()
            }
            RecorderError::DeviceError => {
                "Microphone error. Check your audio settings.".to_string()
            }
            RecorderError::IoError => "Failed to save recording. Check disk space.".to_string(),
        }
    }
}

impl AudioRecorder {
    /// Create a new AudioRecorder
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        AudioRecorder { app_handle }
    }

    /// Start a new recording session
    pub fn start(&self, level_channel: Option<Channel<f32>>) -> Result<Recording, RecorderError> {
        println!("[AudioRecorder] Starting recording...");

        // Ensure audio directory exists
        let audio_dir = ensure_audio_dir_exists(&self.app_handle)?;

        // Get audio host and device first
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(RecorderError::NoInputDevice)?;

        println!(
            "[Audio Recorder] Using input device: {}",
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );

        // Get default device config - we'll always resample to 16kHz
        let config = device
            .default_input_config()
            .map_err(|_| RecorderError::DeviceError)?;

        println!(
            "[Audio Recorder] Device config: {} channels, {} Hz, {:?}",
            config.channels(),
            config.sample_rate().0,
            config.sample_format()
        );

        // Generate filename
        let filename = generate_filename();
        let file_path = audio_dir.join(&filename);
        println!("[Audio Recorder] Recording to: {:?}", file_path);

        // Always write 16kHz mono to file (optimal for speech transcription)
        let spec = WavSpec {
            channels: 1,        // Always mono
            sample_rate: 16000, // Always 16kHz
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let needs_channel_conversion = config.channels() != 1;

        println!(
            "[Audio Recorder] Output: {} Hz mono → resampling from {} Hz {}",
            spec.sample_rate,
            config.sample_rate().0,
            if needs_channel_conversion {
                "stereo"
            } else {
                "mono"
            }
        );

        let writer = WavWriter::create(file_path, spec).map_err(|_| RecorderError::IoError)?;
        let writer = Arc::new(Mutex::new(writer));

        // Always create resampler (device sample rate → 16kHz)
        let input_rate = config.sample_rate().0 as usize;
        let output_rate = 16000;
        let channels = config.channels() as usize;

        let (resampler, required_chunk_size) = match FftFixedInOut::<f32>::new(
            input_rate,
            output_rate,
            1024,
            channels,
        ) {
            Ok(r) => {
                // Query the actual input chunk size the resampler needs
                let input_frames = r.input_frames_next();
                println!("[Audio Recorder] Created FFT resampler: {}Hz {}ch → 16kHz mono (needs {} input samples per chunk)", input_rate, channels, input_frames);
                (Arc::new(Mutex::new(r)), input_frames)
            }
            Err(e) => {
                eprintln!("[Audio Recorder] Failed to create resampler: {:?}", e);
                return Err(RecorderError::DeviceError);
            }
        };

        // Create sample buffer for accumulating samples before resampling
        // FftFixedInOut requires an exact number of samples (queried above)
        let sample_buffer: Arc<Mutex<Vec<Vec<f32>>>> =
            Arc::new(Mutex::new(vec![Vec::new(); channels]));

        // Build input stream
        let writer_clone = Arc::clone(&writer);
        let err_writer_clone = Arc::clone(&writer);

        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => build_input_stream::<i8>(
                &device,
                &config.into(),
                writer_clone,
                level_channel,
                resampler.clone(),
                sample_buffer.clone(),
                required_chunk_size,
                needs_channel_conversion,
            )?,
            cpal::SampleFormat::I16 => build_input_stream::<i16>(
                &device,
                &config.into(),
                writer_clone,
                level_channel,
                resampler.clone(),
                sample_buffer.clone(),
                required_chunk_size,
                needs_channel_conversion,
            )?,
            cpal::SampleFormat::I32 => build_input_stream::<i32>(
                &device,
                &config.into(),
                writer_clone,
                level_channel,
                resampler.clone(),
                sample_buffer.clone(),
                required_chunk_size,
                needs_channel_conversion,
            )?,
            cpal::SampleFormat::F32 => build_input_stream::<f32>(
                &device,
                &config.into(),
                writer_clone,
                level_channel,
                resampler.clone(),
                sample_buffer.clone(),
                required_chunk_size,
                needs_channel_conversion,
            )?,
            _ => return Err(RecorderError::DeviceError),
        };

        // Start the stream
        stream.play()?;
        println!("[AudioRecorder] Stream started successfully");

        // Record start timestamp
        let start_timestamp = SystemTime::now();

        // Return Recording session
        Ok(Recording {
            stream,
            writer: err_writer_clone,
            start_timestamp,
            filename,
            app_handle: self.app_handle.clone(),
        })
    }
}

fn ensure_audio_dir_exists(app_handle: &tauri::AppHandle) -> Result<PathBuf, RecorderError> {
    let cache_dir = app_handle
        .path()
        .app_cache_dir()
        .map_err(|_| RecorderError::IoError)?;

    let audio_dir = cache_dir.join("recordings");

    if !audio_dir.exists() {
        fs::create_dir_all(&audio_dir)?;
        println!("[Audio Recorder] Created audio directory: {:?}", audio_dir);
    }
    Ok(audio_dir)
}

/// Clean up a recording file
/// Logs errors but doesn't fail - cleanup is best-effort
pub fn cleanup_recording_file(file_path: &str) {
    match fs::remove_file(file_path) {
        Ok(_) => println!("[Audio Recorder] Cleaned up recording file: {}", file_path),
        Err(e) => {
            eprintln!(
                "[Audio Recorder] Failed to cleanup recording file {}: {}",
                file_path, e
            );
        }
    }
}

fn generate_filename() -> String {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("recording_{}.wav", timestamp)
}

fn build_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    writer: Arc<Mutex<WavWriter<BufWriter<File>>>>,
    level_channel: Option<Channel<f32>>,
    resampler: Arc<Mutex<FftFixedInOut<f32>>>,
    sample_buffer: Arc<Mutex<Vec<Vec<f32>>>>,
    required_chunk_size: usize,
    needs_channel_conversion: bool,
) -> Result<cpal::Stream, RecorderError>
where
    T: Sample + FromSample<i16> + FromSample<f32> + std::fmt::Debug + cpal::SizedSample,
    i16: FromSample<T>,
    f32: FromSample<T>,
{
    let err_fn = |err| {
        eprintln!("[Audio Recorder] Stream error: {}", err);
    };

    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            write_input_data::<T>(
                data,
                &writer,
                &level_channel,
                &resampler,
                &sample_buffer,
                required_chunk_size,
                needs_channel_conversion,
            );
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn write_input_data<T>(
    input: &[T],
    writer: &Arc<Mutex<WavWriter<BufWriter<File>>>>,
    level_channel: &Option<Channel<f32>>,
    resampler: &Arc<Mutex<FftFixedInOut<f32>>>,
    sample_buffer: &Arc<Mutex<Vec<Vec<f32>>>>,
    required_chunk_size: usize,
    needs_channel_conversion: bool,
) where
    T: Sample,
    i16: FromSample<T>,
    f32: FromSample<T>,
{
    // Calculate RMS (Root Mean Square) for audio level visualization (use original samples)
    if let Some(channel) = level_channel {
        if !input.is_empty() {
            let sum_of_squares: f32 = input
                .iter()
                .map(|&sample| {
                    let sample_f32: f32 = sample.to_sample();
                    sample_f32 * sample_f32
                })
                .sum();
            let rms = (sum_of_squares / input.len() as f32).sqrt();
            let level = (rms * 100.0).min(1.0);
            let _ = channel.send(level);
        }
    }

    // Convert samples to f32 and organize by channel, then append to buffer
    let num_channels = if needs_channel_conversion { 2 } else { 1 };

    let mut buffer_guard = match sample_buffer.lock() {
        Ok(guard) => guard,
        Err(_) => {
            eprintln!("[Audio Recorder] Failed to lock sample buffer");
            return;
        }
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
                Err(_) => {
                    eprintln!("[Audio Recorder] Failed to lock resampler");
                    return;
                }
            };

            let channel_refs: Vec<&[f32]> = channel_chunks.iter().map(|v| v.as_slice()).collect();

            match resampler_guard.process(&channel_refs, None) {
                Ok(resampled) => resampled,
                Err(e) => {
                    eprintln!("[Audio Recorder] Resampling error: {:?}", e);
                    return;
                }
            }
        };

        // Convert to mono if needed (average stereo channels)
        let mono_samples = if needs_channel_conversion && resampled.len() >= 2 {
            let output_frames = resampled[0].len();
            let mut mono = Vec::with_capacity(output_frames);
            for i in 0..output_frames {
                let mixed = (resampled[0][i] + resampled[1][i]) / 2.0;
                mono.push(mixed);
            }
            mono
        } else {
            // Already mono, just use first channel
            resampled[0].clone()
        };

        // Write to WAV file as i16
        if let Ok(mut guard) = writer.lock() {
            for sample_f32 in mono_samples.iter() {
                let clamped = sample_f32.clamp(-1.0, 1.0);
                let sample_i16 = (clamped * 32767.0) as i16;
                guard.write_sample(sample_i16).ok();
            }
        }

        // Re-acquire buffer lock for next iteration
        buffer_guard = match sample_buffer.lock() {
            Ok(guard) => guard,
            Err(_) => {
                eprintln!("[Audio Recorder] Failed to re-lock sample buffer");
                return;
            }
        };
    }
    // Remaining samples (< required_chunk_size) stay in buffer for next call
}
