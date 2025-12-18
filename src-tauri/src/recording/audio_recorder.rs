use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use hound::{WavSpec, WavWriter};
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

        // Get file size
        if let Ok(metadata) = fs::metadata(&file_path) {
            file_size = metadata.len();
            println!("[Recording] File size: {} bytes", file_size);
        }

        // Calculate duration
        let duration_ms = SystemTime::now()
            .duration_since(self.start_timestamp)
            .unwrap()
            .as_millis() as u64;

        println!(
            "[Recording] Recording stopped successfully. Duration: {}ms",
            duration_ms
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

        // Get device config to determine actual sample rate
        let config = device
            .default_input_config()
            .map_err(|_| RecorderError::DeviceError)?;

        println!("[Audio Recorder] Input config: {:?}", config);

        // Generate filename
        let filename = generate_filename();
        let file_path = audio_dir.join(&filename);
        println!("[Audio Recorder] Recording to: {:?}", file_path);

        // Create WAV writer using the device's actual sample rate
        let spec = WavSpec {
            channels: config.channels() as u16,
            sample_rate: config.sample_rate().0, // Use device's actual sample rate
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        println!(
            "[Audio Recorder] WAV spec: {} channels, {} Hz, 16-bit",
            spec.channels, spec.sample_rate
        );

        let writer = WavWriter::create(file_path, spec).map_err(|_| RecorderError::IoError)?;
        let writer = Arc::new(Mutex::new(writer));

        // Build input stream
        let writer_clone = Arc::clone(&writer);
        let err_writer_clone = Arc::clone(&writer);

        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => {
                build_input_stream::<i8>(&device, &config.into(), writer_clone, level_channel)?
            }
            cpal::SampleFormat::I16 => {
                build_input_stream::<i16>(&device, &config.into(), writer_clone, level_channel)?
            }
            cpal::SampleFormat::I32 => {
                build_input_stream::<i32>(&device, &config.into(), writer_clone, level_channel)?
            }
            cpal::SampleFormat::F32 => {
                build_input_stream::<f32>(&device, &config.into(), writer_clone, level_channel)?
            }
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
            write_input_data::<T>(data, &writer, &level_channel);
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
) where
    T: Sample,
    i16: FromSample<T>,
    f32: FromSample<T>,
{
    // Calculate RMS (Root Mean Square) for audio level visualization
    if let Some(channel) = level_channel {
        if !input.is_empty() {
            // Convert samples to f32 and calculate RMS
            let sum_of_squares: f32 = input
                .iter()
                .map(|&sample| {
                    let sample_f32: f32 = sample.to_sample();
                    sample_f32 * sample_f32
                })
                .sum();

            let rms = (sum_of_squares / input.len() as f32).sqrt();

            // Normalize to 0.0-1.0 range (audio samples are typically -1.0 to 1.0)
            // Multiply by a high gain factor to make visualization more visible
            // Boost from 3x to 100x for better visibility with quiet microphones
            let level = (rms * 100.0).min(1.0);

            // Send level to frontend (ignore errors if channel is closed)
            let _ = channel.send(level);
        }
    }

    // Write audio data to WAV file
    if let Ok(mut guard) = writer.lock() {
        for &sample in input.iter() {
            let sample_i16: i16 = sample.to_sample();
            guard.write_sample(sample_i16).ok();
        }
    }
}
