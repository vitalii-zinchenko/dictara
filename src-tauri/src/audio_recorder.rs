use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use hound::{WavWriter, WavSpec};
use serde::Serialize;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tauri::Emitter;

#[derive(Clone, Serialize)]
pub struct RecordingStartedEvent {
    pub timestamp: u128,
    pub filename: String,
}

#[derive(Clone, Serialize)]
pub struct RecordingStoppedEvent {
    pub timestamp: u128,
    pub filename: String,
    pub duration_ms: u64,
    pub file_size_bytes: u64,
}

#[derive(Clone, Serialize)]
pub struct RecordingErrorEvent {
    pub error: String,
    pub error_type: String,
    pub timestamp: u128,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RecordingStatus {
    Idle,
    Recording,
}

struct RecorderState {
    status: RecordingStatus,
    writer: Option<Arc<Mutex<WavWriter<BufWriter<File>>>>>,
    start_timestamp: Option<SystemTime>,
    filename: Option<String>,
}

pub struct AudioRecorder {
    state: Arc<Mutex<RecorderState>>,
    app_handle: tauri::AppHandle,
}

impl Clone for AudioRecorder {
    fn clone(&self) -> Self {
        AudioRecorder {
            state: Arc::clone(&self.state),
            app_handle: self.app_handle.clone(),
        }
    }
}

#[derive(Debug)]
pub enum RecorderError {
    PermissionDenied,
    NoInputDevice,
    DeviceError(String),
    IoError(std::io::Error),
    AlreadyRecording,
    NotRecording,
}

impl From<std::io::Error> for RecorderError {
    fn from(err: std::io::Error) -> Self {
        RecorderError::IoError(err)
    }
}

impl From<cpal::BuildStreamError> for RecorderError {
    fn from(err: cpal::BuildStreamError) -> Self {
        RecorderError::DeviceError(format!("Build stream error: {}", err))
    }
}

impl From<cpal::PlayStreamError> for RecorderError {
    fn from(err: cpal::PlayStreamError) -> Self {
        RecorderError::DeviceError(format!("Play stream error: {}", err))
    }
}

impl From<cpal::PauseStreamError> for RecorderError {
    fn from(err: cpal::PauseStreamError) -> Self {
        RecorderError::DeviceError(format!("Pause stream error: {}", err))
    }
}

impl AudioRecorder {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        AudioRecorder {
            state: Arc::new(Mutex::new(RecorderState {
                status: RecordingStatus::Idle,
                writer: None,
                start_timestamp: None,
                filename: None,
            })),
            app_handle,
        }
    }

    pub fn start_recording(&self) -> Result<cpal::Stream, RecorderError> {
        let mut state = self.state.lock().unwrap();

        // Check if already recording
        if state.status == RecordingStatus::Recording {
            eprintln!("[Audio Recorder] Already recording, ignoring start request");
            return Err(RecorderError::AlreadyRecording);
        }

        println!("[Audio Recorder] Starting recording...");

        // Ensure audio directory exists
        let audio_dir = ensure_audio_dir_exists()?;

        // Get audio host and device first
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(RecorderError::NoInputDevice)?;

        println!("[Audio Recorder] Using input device: {}", device.name().unwrap_or_else(|_| "Unknown".to_string()));

        // Get device config to determine actual sample rate
        let config = device
            .default_input_config()
            .map_err(|e| RecorderError::DeviceError(format!("Failed to get input config: {}", e)))?;

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

        println!("[Audio Recorder] WAV spec: {} channels, {} Hz, 16-bit", spec.channels, spec.sample_rate);

        let writer = WavWriter::create(file_path, spec)
            .map_err(|e| RecorderError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        let writer = Arc::new(Mutex::new(writer));

        // Build input stream
        let writer_clone = Arc::clone(&writer);
        let err_writer_clone = Arc::clone(&writer);

        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => build_input_stream::<i8>(&device, &config.into(), writer_clone)?,
            cpal::SampleFormat::I16 => build_input_stream::<i16>(&device, &config.into(), writer_clone)?,
            cpal::SampleFormat::I32 => build_input_stream::<i32>(&device, &config.into(), writer_clone)?,
            cpal::SampleFormat::F32 => build_input_stream::<f32>(&device, &config.into(), writer_clone)?,
            _ => {
                return Err(RecorderError::DeviceError(
                    "Unsupported sample format".to_string(),
                ))
            }
        };

        // Start the stream
        stream.play()?;
        println!("[Audio Recorder] Stream started successfully");

        // Update state
        let start_timestamp = SystemTime::now();
        state.status = RecordingStatus::Recording;
        state.writer = Some(err_writer_clone);
        state.start_timestamp = Some(start_timestamp);
        state.filename = Some(filename.clone());

        // Emit event
        let timestamp = start_timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        self.app_handle
            .emit(
                "recording-started",
                RecordingStartedEvent {
                    timestamp,
                    filename,
                },
            )
            .ok();

        Ok(stream)
    }

    pub fn stop_recording(&self) -> Result<(), RecorderError> {
        let mut state = self.state.lock().unwrap();

        // Check if actually recording
        if state.status != RecordingStatus::Recording {
            eprintln!("[Audio Recorder] Not recording, ignoring stop request");
            return Err(RecorderError::NotRecording);
        }

        println!("[Audio Recorder] Stopping recording...");

        // Finalize WAV file
        let filename = state.filename.take().unwrap_or_else(|| "unknown".to_string());
        let mut file_size = 0u64;

        if let Some(writer_arc) = state.writer.take() {
            // Try to finalize the writer
            if let Ok(writer_mutex) = Arc::try_unwrap(writer_arc) {
                if let Ok(writer) = writer_mutex.into_inner() {
                    let result = writer.finalize();
                    if let Err(e) = result {
                        eprintln!("[Audio Recorder] Error finalizing WAV: {}", e);
                    } else {
                        println!("[Audio Recorder] WAV file finalized successfully");
                    }
                }
            }

            // Get file size
            let audio_dir = ensure_audio_dir_exists()?;
            let file_path = audio_dir.join(&filename);
            if let Ok(metadata) = fs::metadata(&file_path) {
                file_size = metadata.len();
                println!("[Audio Recorder] File size: {} bytes", file_size);
            }
        }

        // Calculate duration
        let duration_ms = if let Some(start_time) = state.start_timestamp.take() {
            SystemTime::now()
                .duration_since(start_time)
                .unwrap()
                .as_millis() as u64
        } else {
            0
        };

        // Update state
        state.status = RecordingStatus::Idle;

        // Emit event
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        self.app_handle
            .emit(
                "recording-stopped",
                RecordingStoppedEvent {
                    timestamp,
                    filename,
                    duration_ms,
                    file_size_bytes: file_size,
                },
            )
            .ok();

        println!("[Audio Recorder] Recording stopped successfully. Duration: {}ms", duration_ms);

        Ok(())
    }

    pub fn emit_error(&self, error: &str, error_type: &str) {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        self.app_handle
            .emit(
                "recording-error",
                RecordingErrorEvent {
                    error: error.to_string(),
                    error_type: error_type.to_string(),
                    timestamp,
                },
            )
            .ok();
    }
}

fn ensure_audio_dir_exists() -> Result<PathBuf, RecorderError> {
    let audio_dir = PathBuf::from("/Users/vitaliizinchenko/Projects/typefree/audio");
    if !audio_dir.exists() {
        fs::create_dir_all(&audio_dir)?;
        println!("[Audio Recorder] Created audio directory: {:?}", audio_dir);
    }
    Ok(audio_dir)
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
) -> Result<cpal::Stream, RecorderError>
where
    T: Sample + FromSample<i16> + std::fmt::Debug + cpal::SizedSample,
    i16: FromSample<T>,
{
    let err_fn = |err| {
        eprintln!("[Audio Recorder] Stream error: {}", err);
    };

    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            write_input_data::<T>(data, &writer);
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn write_input_data<T>(input: &[T], writer: &Arc<Mutex<WavWriter<BufWriter<File>>>>)
where
    T: Sample,
    i16: FromSample<T>,
{
    if let Ok(mut guard) = writer.lock() {
        for &sample in input.iter() {
            let sample_i16: i16 = sample.to_sample();
            guard.write_sample(sample_i16).ok();
        }
    }
}
