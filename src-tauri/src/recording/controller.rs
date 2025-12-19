use serde::Serialize;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc, Mutex,
};
use tauri::ipc::Channel;
use tauri::Emitter;
use tauri_plugin_store::StoreExt;
use tokio::sync::mpsc::Receiver;

use crate::clients::openai::OpenAIClient;
use crate::config;
use crate::error::Error;
use crate::recording::{
    audio_recorder::{cleanup_recording_file, AudioRecorder},
    commands::RecordingCommand,
    LastRecordingState, Recording,
};
use crate::sound_player;
use crate::ui::window::{close_recording_popup, open_recording_popup};

// Event payload for recording-stopped
#[derive(Clone, Serialize)]
pub struct RecordingStoppedPayload {
    pub text: String,
}

// Event payload for recording-error
#[derive(Clone, Serialize)]
pub struct RecordingErrorPayload {
    pub error_type: String,              // "recording" | "transcription"
    pub error_message: String,           // Technical error for debugging
    pub user_message: String,            // User-friendly message
    pub can_retry: bool,                 // Show retry button?
    pub audio_file_path: Option<String>, // For retry
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum ControllerState {
    /// Controller is ready to start recording
    Ready,
    /// Controller is currently recording
    Recording,
    /// Recording is locked - Fn release will be ignored
    RecordingLocked,
}

pub struct Controller {
    command_rx: Receiver<RecordingCommand>,
    audio_recorder: AudioRecorder,
    openai_client: OpenAIClient,
    app_handle: tauri::AppHandle,
    state: ControllerState,
    shared_state: Arc<AtomicU8>,
    audio_level_channel: Arc<Mutex<Option<Channel<f32>>>>,
    last_recording_state: LastRecordingState,
}

impl Controller {
    pub fn new(
        command_rx: Receiver<RecordingCommand>,
        app_handle: tauri::AppHandle,
        openai_client: OpenAIClient,
        shared_state: Arc<AtomicU8>,
        audio_level_channel: Arc<Mutex<Option<Channel<f32>>>>,
        last_recording_state: LastRecordingState,
    ) -> Self {
        let audio_recorder = AudioRecorder::new(app_handle.clone());

        shared_state.store(0, Ordering::Relaxed);

        Controller {
            command_rx,
            audio_recorder,
            openai_client,
            app_handle,
            state: ControllerState::Ready,
            shared_state,
            audio_level_channel,
            last_recording_state,
        }
    }

    /// Main control loop - consumes self, runs in blocking thread
    pub fn run(mut self) {
        // Recording session lives here (not Send, so stays in this thread)
        let mut current_recording: Option<Recording> = None;

        println!("[Controller] Starting command processing loop");

        while let Some(command) = self.command_rx.blocking_recv() {
            match command {
                RecordingCommand::FnDown => {
                    match self.state {
                        ControllerState::Ready => {
                            // Start recording
                            self.set_state(ControllerState::Recording);
                            match self.handle_start() {
                                Ok(recording) => current_recording = Some(recording),
                                Err(e) => {
                                    eprintln!("[Controller] Error starting recording: {:?}", e);
                                    self.set_state(ControllerState::Ready);
                                }
                            }
                        }
                        ControllerState::RecordingLocked => {
                            // Stop locked recording
                            if let Some(rec) = current_recording.take() {
                                if let Err(e) = self.handle_stop(rec) {
                                    eprintln!("[Controller] Error stopping recording: {:?}", e);
                                }
                            }
                            self.set_state(ControllerState::Ready);
                        }
                        _ => {
                            println!("[Controller] FnDown ignored in Recording state");
                        }
                    }
                }
                RecordingCommand::FnUp => {
                    match self.state {
                        ControllerState::Recording => {
                            // Stop recording normally
                            if let Some(rec) = current_recording.take() {
                                if let Err(e) = self.handle_stop(rec) {
                                    eprintln!("[Controller] Error stopping recording: {:?}", e);
                                }
                            }
                            self.set_state(ControllerState::Ready);
                        }
                        _ => {
                            println!("[Controller] FnUp ignored (Ready or RecordingLocked state)");
                        }
                    }
                }
                RecordingCommand::Lock => {
                    match self.state {
                        ControllerState::Recording => {
                            // Lock the recording
                            self.set_state(ControllerState::RecordingLocked);
                            // Play start sound to confirm lock
                            sound_player::play_start();
                            println!("[Controller] Recording locked - FnUp will be ignored");
                        }
                        _ => {
                            println!("[Controller] Lock ignored (not in Recording state)");
                        }
                    }
                }
                RecordingCommand::Cancel => {
                    // Cancel works in both Recording and RecordingLocked states
                    if self.state != ControllerState::Ready {
                        if let Some(rec) = current_recording.take() {
                            if let Err(e) = self.handle_cancel(rec) {
                                eprintln!("[Controller] Error cancelling recording: {:?}", e);
                            }
                        }
                        self.set_state(ControllerState::Ready);
                    }
                }
                RecordingCommand::RetryTranscription => {
                    println!("[Controller] Received RetryTranscription command");
                    if let Err(e) = self.handle_retry_transcription() {
                        eprintln!("[Controller] Error retrying transcription: {:?}", e);
                    }
                }
            }
        }

        println!("[Controller] Channel closed, shutting down");
    }

    fn handle_start(&self) -> Result<Recording, Error> {
        println!("[Controller] Received Start command");

        // Play start sound
        sound_player::play_start();

        // Update tray icon to recording state
        if let Err(e) = crate::ui::tray::set_recording_icon(&self.app_handle) {
            eprintln!("[Controller] Failed to set recording icon: {}", e);
        }

        // Show recording popup window
        if let Err(e) = open_recording_popup(&self.app_handle) {
            eprintln!("[Controller] Failed to open recording popup: {}", e);
        }

        self.app_handle.emit("recording-started", ())?;

        // Get the audio level channel if one is registered
        let level_channel = self.audio_level_channel.lock().unwrap().clone();

        let recording = match self.audio_recorder.start(level_channel) {
            Ok(rec) => rec,
            Err(e) => {
                eprintln!("[Controller] Error starting recording: {:?}", e);

                // Emit error event to frontend
                let error_payload = RecordingErrorPayload {
                    error_type: "recording".to_string(),
                    error_message: format!("{:?}", e),
                    user_message: e.user_message(),
                    can_retry: false, // Recording errors cannot be retried
                    audio_file_path: None,
                };

                if let Err(emit_err) = self.app_handle.emit("recording-error", error_payload) {
                    eprintln!(
                        "[Controller] Failed to emit recording-error event: {}",
                        emit_err
                    );
                }

                return Err(Error::from(e));
            }
        };

        Ok(recording)
    }

    fn handle_stop(&self, recording: Recording) -> Result<(), Error> {
        println!("[Controller] Received Stop command");

        // Play stop sound
        sound_player::play_stop();

        let recording_result = recording.stop()?;

        println!("[Controller] Emitting recording-transcribing event");
        match self.app_handle.emit("recording-transcribing", ()) {
            Ok(_) => println!("[Controller] Successfully emitted recording-transcribing event"),
            Err(e) => eprintln!(
                "[Controller] Failed to emit recording-transcribing event: {:?}",
                e
            ),
        }

        // Load provider config
        let store = match self.app_handle.store("config.json") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[Controller] Failed to load config store: {}", e);
                return Err(Error::from(
                    crate::clients::openai::TranscriptionError::ApiError(format!(
                        "Failed to load config: {}",
                        e
                    )),
                ));
            }
        };
        let provider_config = config::load_config(&store);

        // Transcribe with loaded config
        let transcription_result = self.openai_client.transcribe_audio_sync(
            PathBuf::from(&recording_result.file_path),
            recording_result.duration_ms,
            &provider_config,
        );

        match transcription_result {
            Ok(text) => {
                // Clean up recording file after successful transcription
                cleanup_recording_file(&recording_result.file_path);

                if !text.is_empty() {
                    crate::clipboard_paste::auto_paste_text_cgevent(&text)?;
                }

                // Update last recording state with successful transcription
                if let Ok(mut last_recording) = self.last_recording_state.lock() {
                    last_recording.text = Some(text.clone());
                    last_recording.timestamp = Some(std::time::SystemTime::now());
                    last_recording.audio_file_path = None;
                }

                // Enable the paste menu item
                if let Err(e) = crate::ui::tray::update_paste_menu_item(&self.app_handle, true) {
                    eprintln!("[Controller] Failed to enable paste menu item: {}", e);
                }

                // Restore tray icon to default state
                if let Err(e) = crate::ui::tray::set_default_icon(&self.app_handle) {
                    eprintln!("[Controller] Failed to set default icon: {}", e);
                }

                // Hide recording popup window
                if let Err(e) = close_recording_popup(&self.app_handle) {
                    eprintln!("[Controller] Failed to close recording popup: {}", e);
                }

                self.app_handle.emit(
                    "recording-stopped",
                    RecordingStoppedPayload { text: text.clone() },
                )?;

                Ok(())
            }
            Err(e) => {
                eprintln!("[Controller] Transcription error: {}", e);

                // Update last recording state with failed transcription
                // Keep the audio file for retry
                if let Ok(mut last_recording) = self.last_recording_state.lock() {
                    last_recording.text = None;
                    last_recording.timestamp = None;
                    last_recording.audio_file_path = Some(recording_result.file_path.clone());
                }

                // Disable the paste menu item since there's no text to paste
                if let Err(err) = crate::ui::tray::update_paste_menu_item(&self.app_handle, false) {
                    eprintln!("[Controller] Failed to disable paste menu item: {}", err);
                }

                // Restore tray icon to default state
                if let Err(err) = crate::ui::tray::set_default_icon(&self.app_handle) {
                    eprintln!("[Controller] Failed to set default icon: {}", err);
                }

                // DON'T close popup - keep it open to show error
                // Emit error event to frontend
                let error_payload = RecordingErrorPayload {
                    error_type: "transcription".to_string(),
                    error_message: format!("{}", e),
                    user_message: e.user_message(),
                    can_retry: e.can_retry(),
                    audio_file_path: Some(recording_result.file_path.clone()),
                };

                if let Err(emit_err) = self.app_handle.emit("recording-error", error_payload) {
                    eprintln!(
                        "[Controller] Failed to emit recording-error event: {}",
                        emit_err
                    );
                }

                Err(Error::from(e))
            }
        }
    }

    fn handle_cancel(&self, recording: Recording) -> Result<(), Error> {
        println!("[Controller] Received Cancel command");

        // Stop recording (creates file but we don't use it)
        let recording_result = recording.stop()?;

        // Clean up the cancelled recording file immediately
        cleanup_recording_file(&recording_result.file_path);

        // Restore tray icon to default state
        if let Err(e) = crate::ui::tray::set_default_icon(&self.app_handle) {
            eprintln!("[Controller] Failed to set default icon: {}", e);
        }

        // Hide recording popup window
        if let Err(e) = close_recording_popup(&self.app_handle) {
            eprintln!("[Controller] Failed to close recording popup: {}", e);
        }

        // Emit cancellation event for frontend awareness
        self.app_handle.emit("recording-cancelled", ())?;

        println!("[Controller] Recording cancelled successfully");
        Ok(())
    }

    fn handle_retry_transcription(&self) -> Result<(), Error> {
        println!("[Controller] Retrying transcription");

        // Get audio file path from last recording state
        let (audio_file_path, duration_ms) = {
            let last_recording = self.last_recording_state.lock().map_err(|e| {
                Error::from(crate::clients::openai::TranscriptionError::ApiError(
                    format!("Failed to lock state: {}", e),
                ))
            })?;

            let path = last_recording.audio_file_path.clone().ok_or_else(|| {
                Error::from(crate::clients::openai::TranscriptionError::ApiError(
                    "No audio file available for retry".to_string(),
                ))
            })?;

            // Estimate duration from file size: ~32KB per second for 16kHz mono 16-bit
            let metadata = std::fs::metadata(&path).map_err(|e| {
                Error::from(crate::clients::openai::TranscriptionError::FileNotFound(
                    format!("File not found: {}", e),
                ))
            })?;
            let duration_ms = (metadata.len() * 1000) / 32000;

            (path, duration_ms)
        };

        // Emit transcribing event
        println!("[Controller] Emitting recording-transcribing event for retry");
        self.app_handle.emit("recording-transcribing", ())?;

        // Load provider config
        let store = match self.app_handle.store("config.json") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[Controller] Failed to load config store: {}", e);
                return Err(Error::from(
                    crate::clients::openai::TranscriptionError::ApiError(format!(
                        "Failed to load config: {}",
                        e
                    )),
                ));
            }
        };
        let provider_config = config::load_config(&store);

        // Transcribe with loaded config
        let transcription_result = self.openai_client.transcribe_audio_sync(
            PathBuf::from(&audio_file_path),
            duration_ms,
            &provider_config,
        );

        match transcription_result {
            Ok(text) => {
                // Clean up recording file after successful transcription
                cleanup_recording_file(&audio_file_path);

                if !text.is_empty() {
                    crate::clipboard_paste::auto_paste_text_cgevent(&text)?;
                }

                // Update last recording state with successful transcription
                if let Ok(mut last_recording) = self.last_recording_state.lock() {
                    last_recording.text = Some(text.clone());
                    last_recording.timestamp = Some(std::time::SystemTime::now());
                    last_recording.audio_file_path = None;
                }

                // Enable the paste menu item
                if let Err(e) = crate::ui::tray::update_paste_menu_item(&self.app_handle, true) {
                    eprintln!("[Controller] Failed to enable paste menu item: {}", e);
                }

                // Restore tray icon to default state
                if let Err(e) = crate::ui::tray::set_default_icon(&self.app_handle) {
                    eprintln!("[Controller] Failed to set default icon: {}", e);
                }

                // Hide recording popup window
                if let Err(e) = close_recording_popup(&self.app_handle) {
                    eprintln!("[Controller] Failed to close recording popup: {}", e);
                }

                self.app_handle.emit(
                    "recording-stopped",
                    RecordingStoppedPayload { text: text.clone() },
                )?;

                Ok(())
            }
            Err(e) => {
                eprintln!("[Controller] Retry transcription error: {}", e);

                // Update last recording state - keep audio file for another retry
                if let Ok(mut last_recording) = self.last_recording_state.lock() {
                    last_recording.text = None;
                    last_recording.timestamp = None;
                    last_recording.audio_file_path = Some(audio_file_path.clone());
                }

                // Disable the paste menu item since there's no text to paste
                if let Err(err) = crate::ui::tray::update_paste_menu_item(&self.app_handle, false) {
                    eprintln!("[Controller] Failed to disable paste menu item: {}", err);
                }

                // Restore tray icon to default state
                if let Err(err) = crate::ui::tray::set_default_icon(&self.app_handle) {
                    eprintln!("[Controller] Failed to set default icon: {}", err);
                }

                // DON'T close popup - keep it open to show error
                // Emit error event to frontend
                let error_payload = RecordingErrorPayload {
                    error_type: "transcription".to_string(),
                    error_message: format!("{}", e),
                    user_message: e.user_message(),
                    can_retry: e.can_retry(),
                    audio_file_path: Some(audio_file_path),
                };

                if let Err(emit_err) = self.app_handle.emit("recording-error", error_payload) {
                    eprintln!(
                        "[Controller] Failed to emit recording-error event: {}",
                        emit_err
                    );
                }

                Err(Error::from(e))
            }
        }
    }

    fn set_state(&mut self, new_state: ControllerState) {
        self.state = new_state;
        let state_value = match new_state {
            ControllerState::Ready => 0,
            ControllerState::Recording => 1,
            ControllerState::RecordingLocked => 2,
        };
        self.shared_state.store(state_value, Ordering::Relaxed);
    }
}
