use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tauri_plugin_store::StoreExt;
use tauri_specta::Event;
use tokio::sync::mpsc::Receiver;

use crate::clients::{Transcriber, TranscriptionError};
use crate::config;
use crate::error::Error;
use crate::recording::{
    audio_recorder::{cleanup_recording_file, AudioRecorder},
    commands::RecordingCommand,
    events::RecordingStateChanged,
    LastRecordingState, Recording, RecordingAction, RecordingEvent, RecordingStateManager,
    TransitionResult,
};
use crate::ui::menu::Menu;
use crate::ui::window::{close_recording_popup, open_recording_popup};
use crate::updater;

/// Bytes per second for 16kHz mono 16-bit audio (~32KB/s)
const AUDIO_BYTES_PER_SECOND: u64 = 32000;

pub struct Controller {
    command_rx: Receiver<RecordingCommand>,
    audio_recorder: AudioRecorder,
    app_handle: tauri::AppHandle,
    state_manager: Arc<RecordingStateManager>,
    audio_level_channel: Arc<Mutex<Option<Channel<f32>>>>,
    last_recording_state: LastRecordingState,
    menu: Menu,
}

impl Controller {
    pub fn new(
        command_rx: Receiver<RecordingCommand>,
        app_handle: tauri::AppHandle,
        state_manager: Arc<RecordingStateManager>,
        audio_level_channel: Arc<Mutex<Option<Channel<f32>>>>,
        last_recording_state: LastRecordingState,
        menu: Menu,
    ) -> Self {
        let audio_recorder = AudioRecorder::new(app_handle.clone());

        // Ensure we start in Ready state
        state_manager.reset();

        Controller {
            command_rx,
            audio_recorder,
            app_handle,
            state_manager,
            audio_level_channel,
            last_recording_state,
            menu,
        }
    }

    /// Main control loop - consumes self, runs in blocking thread
    pub fn run(mut self) {
        // Recording session lives here (not Send, so stays in this thread)
        let mut current_recording: Option<Recording> = None;

        while let Some(command) = self.command_rx.blocking_recv() {
            // Attempt state transition
            match self.state_manager.transition(command.into()) {
                Ok(TransitionResult::Changed { action, .. }) => {
                    if let Some(action) = action {
                        self.execute_action(action, &mut current_recording);
                    }
                }
                Ok(TransitionResult::Unchanged) => {
                    // Valid event but no state change (edge case)
                }
                Err(rejection) => {
                    log::warn!("{}", rejection);
                }
            }
        }
    }

    /// Execute action returned by the state machine
    fn execute_action(&self, action: RecordingAction, recording: &mut Option<Recording>) {
        match action {
            RecordingAction::StartRecording => {
                match self.handle_start() {
                    Ok(rec) => *recording = Some(rec),
                    Err(e) => {
                        log::error!("Error starting recording: {:?}", e);
                        // Reset state on error
                        self.state_manager.reset();
                    }
                }
            }
            RecordingAction::StopAndTranscribe => {
                if let Some(rec) = recording.take() {
                    if let Err(e) = self.handle_stop(rec) {
                        log::error!("Error stopping recording: {:?}", e);
                    }
                }
                // Notify updater that recording/transcription finished
                updater::on_recording_finished(&self.app_handle);
            }
            RecordingAction::CancelRecording => {
                if let Some(rec) = recording.take() {
                    if let Err(e) = self.handle_cancel(rec) {
                        log::error!("Error cancelling recording: {:?}", e);
                    }
                }
                // Notify updater that recording was cancelled
                updater::on_recording_finished(&self.app_handle);
            }
            RecordingAction::RetryTranscription => {
                if let Err(e) = self.handle_retry_transcription() {
                    log::error!("Error retrying transcription: {:?}", e);
                }
                // Notify updater that transcription finished
                updater::on_recording_finished(&self.app_handle);
            }
        }
    }

    fn handle_start(&self) -> Result<Recording, Error> {
        // Show recording popup window
        if let Err(e) = open_recording_popup(&self.app_handle) {
            log::error!("Failed to open recording popup: {}", e);
        }

        RecordingStateChanged::Started.emit(&self.app_handle)?;

        // Get the audio level channel if one is registered
        let level_channel = match self.audio_level_channel.lock() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                log::error!("Failed to lock audio_level_channel: {}", e);
                None
            }
        };

        let recording = match self.audio_recorder.start(level_channel) {
            Ok(rec) => rec,
            Err(e) => {
                log::error!("Error starting recording: {:?}", e);

                // Close popup since recording failed to start
                if let Err(close_err) = close_recording_popup(&self.app_handle) {
                    log::error!("Failed to close recording popup: {}", close_err);
                }

                // Emit error event to frontend
                let error_event = RecordingStateChanged::Error {
                    error_type: "recording".to_string(),
                    error_message: format!("{:?}", e),
                    user_message: e.user_message(),
                    audio_file_path: None,
                };

                if let Err(emit_err) = error_event.emit(&self.app_handle) {
                    log::error!("Failed to emit recording-error event: {}", emit_err);
                }

                return Err(Error::from(e));
            }
        };

        Ok(recording)
    }

    fn handle_stop(&self, recording: Recording) -> Result<(), Error> {
        let recording_result = recording.stop()?;

        if let Err(e) = RecordingStateChanged::Transcribing.emit(&self.app_handle) {
            log::error!("Failed to emit recording-transcribing event: {:?}", e);
        }

        self.perform_transcription(&recording_result.file_path, recording_result.duration_ms)
    }

    fn handle_cancel(&self, recording: Recording) -> Result<(), Error> {
        // Stop recording (creates file but we don't use it)
        let recording_result = recording.stop()?;

        // Clean up the cancelled recording file immediately
        cleanup_recording_file(&recording_result.file_path);

        // Hide recording popup window
        if let Err(e) = close_recording_popup(&self.app_handle) {
            log::error!("Failed to close recording popup: {}", e);
        }

        // Emit cancellation event for frontend awareness
        RecordingStateChanged::Cancelled.emit(&self.app_handle)?;

        Ok(())
    }

    fn handle_retry_transcription(&self) -> Result<(), Error> {
        // Get audio file path from last recording state
        let (audio_file_path, duration_ms) = {
            let last_recording = match self.last_recording_state.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    log::error!("Failed to lock last_recording_state: {}", e);
                    self.state_manager.reset(); // Return to Ready state
                    return Ok(());
                }
            };

            // No audio file available - nothing to retry
            let Some(path) = last_recording.audio_file_path.clone() else {
                log::debug!("No audio file available for retry, skipping");
                self.state_manager.reset(); // Return to Ready state
                return Ok(());
            };

            // Estimate duration from file size based on audio format
            let metadata = match std::fs::metadata(&path) {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Audio file not found for retry: {}", e);
                    self.state_manager.reset(); // Return to Ready state
                    return Ok(());
                }
            };
            let duration_ms = (metadata.len() * 1000) / AUDIO_BYTES_PER_SECOND;

            (path, duration_ms)
        };

        // Emit transcribing event
        if let Err(e) = RecordingStateChanged::Transcribing.emit(&self.app_handle) {
            log::error!("Failed to emit recording-transcribing event: {:?}", e);
        }

        self.perform_transcription(&audio_file_path, duration_ms)
    }

    /// Shared transcription logic used by both handle_stop and handle_retry_transcription
    fn perform_transcription(&self, audio_file_path: &str, duration_ms: u64) -> Result<(), Error> {
        // Load provider config
        let store = match self.app_handle.store("config.json") {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to load config store: {}", e);
                return Err(Error::from(TranscriptionError::ApiError(format!(
                    "Failed to load config: {}",
                    e
                ))));
            }
        };
        let app_config = config::load_app_config(&store);

        // Create transcriber from config and transcribe
        let transcriber = Transcriber::from_config(&app_config)?;
        let transcription_result =
            transcriber.transcribe(PathBuf::from(audio_file_path), duration_ms);

        match transcription_result {
            Ok(text) => {
                self.handle_transcription_success(&text, audio_file_path)?;
                Ok(())
            }
            Err(e) => {
                self.handle_transcription_error(&e, audio_file_path);
                Err(Error::from(e))
            }
        }
    }

    /// Handle successful transcription: cleanup, paste, update state, emit event
    fn handle_transcription_success(&self, text: &str, audio_file_path: &str) -> Result<(), Error> {
        // Transition state: Transcribing -> Ready
        if let Err(e) = self
            .state_manager
            .transition(RecordingEvent::TranscriptionComplete)
        {
            log::warn!("TranscriptionComplete transition failed: {}", e);
        }

        // Clean up recording file after successful transcription
        cleanup_recording_file(audio_file_path);

        if !text.is_empty() {
            crate::text_paster::paste_text(text)?;
        }

        // Update last recording state with successful transcription
        match self.last_recording_state.lock() {
            Ok(mut last_recording) => {
                last_recording.text = Some(text.to_string());
                last_recording.timestamp = Some(std::time::SystemTime::now());
                last_recording.audio_file_path = None;
            }
            Err(e) => {
                log::error!("Failed to lock last_recording_state: {}", e);
            }
        }

        self.menu.set_paste_last_active()?;

        // Hide recording popup window
        if let Err(e) = close_recording_popup(&self.app_handle) {
            log::error!("Failed to close recording popup: {}", e);
        }

        RecordingStateChanged::Stopped {
            text: text.to_string(),
        }
        .emit(&self.app_handle)?;

        Ok(())
    }

    /// Handle transcription error: update state, emit error event
    fn handle_transcription_error(&self, e: &TranscriptionError, audio_file_path: &str) {
        log::error!("Transcription error: {}", e);

        // Transition state: Transcribing -> Ready
        if let Err(transition_err) = self
            .state_manager
            .transition(RecordingEvent::TranscriptionFailed)
        {
            log::warn!("TranscriptionFailed transition failed: {}", transition_err);
        }

        // Update last recording state - keep audio file for retry
        match self.last_recording_state.lock() {
            Ok(mut last_recording) => {
                last_recording.text = None;
                last_recording.timestamp = None;
                last_recording.audio_file_path = Some(audio_file_path.to_string());
            }
            Err(lock_err) => {
                log::error!("Failed to lock last_recording_state: {}", lock_err);
            }
        }

        // Disable paste menu item since there's no valid text to paste
        if let Err(menu_err) = self.menu.set_paste_last_inactive() {
            log::error!("Failed to disable paste menu item: {}", menu_err);
        }

        // DON'T close popup - keep it open to show error
        // Emit error event to frontend
        let error_event = RecordingStateChanged::Error {
            error_type: "transcription".to_string(),
            error_message: format!("{}", e),
            user_message: e.user_message(),
            audio_file_path: Some(audio_file_path.to_string()),
        };

        if let Err(emit_err) = error_event.emit(&self.app_handle) {
            log::error!("Failed to emit recording-error event: {}", emit_err);
        }
    }
}
