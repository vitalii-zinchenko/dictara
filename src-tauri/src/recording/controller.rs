use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tauri_specta::Event;
use tokio::sync::mpsc::Receiver;

use crate::clients::{Transcriber, TranscriptionError};
use crate::recording::{
    audio_recorder::{cleanup_recording_file, AudioRecorder},
    commands::RecordingCommand,
    events::RecordingStateChanged,
    LastRecordingState, Recording, RecordingAction, RecordingStateManager, TransitionResult,
};
use crate::ui::menu::Menu;
use crate::ui::window::{close_recording_popup, open_recording_popup};
use crate::updater;

/// Bytes per second for 16kHz mono 16-bit audio (~32KB/s)
const AUDIO_BYTES_PER_SECOND: u64 = 32000;

/// Minimum speech duration in milliseconds to proceed with transcription
/// Audio shorter than this is considered "no speech detected"
const MIN_SPEECH_DURATION_MS: u64 = 500;

/// Whether to delete audio files after transcription completes
/// Set to false to keep recordings for debugging
const CLEANUP_AUDIO_AFTER_TRANSCRIPTION: bool = true;

/// Error type for controller action failures
///
/// Captures all context needed for centralized error handling:
/// - What type of error occurred (for categorization)
/// - User-friendly message (for display)
/// - Optional audio file path (enables retry for transcription errors)
#[derive(Debug)]
struct ActionError {
    /// Error category: "recording", "transcription", "cancel"
    error_type: String,
    /// Technical error message for logging
    error_message: String,
    /// User-friendly message for display
    user_message: String,
    /// Audio file path if available (enables retry)
    audio_file_path: Option<String>,
}

impl ActionError {
    /// Create a recording error (no audio file, no retry)
    fn recording(message: String, user_message: String) -> Self {
        Self {
            error_type: "recording".to_string(),
            error_message: message,
            user_message,
            audio_file_path: None,
        }
    }

    /// Create a transcription error (has audio file, can retry)
    fn transcription(error: &TranscriptionError, audio_file_path: String) -> Self {
        Self {
            error_type: "transcription".to_string(),
            error_message: format!("{}", error),
            user_message: error.user_message(),
            audio_file_path: Some(audio_file_path),
        }
    }

    /// Create a cancel error (cleanup failed)
    fn cancel(message: String) -> Self {
        Self {
            error_type: "cancel".to_string(),
            error_message: message.clone(),
            user_message: format!("Failed to cancel recording: {}", message),
            audio_file_path: None,
        }
    }

    /// Create a stop error (recording.stop() failed, before transcription)
    fn stop(message: String, audio_file_path: Option<String>) -> Self {
        Self {
            error_type: "recording".to_string(),
            error_message: message.clone(),
            user_message: format!("Failed to stop recording: {}", message),
            audio_file_path,
        }
    }

    /// Create a no-speech error (user didn't say anything)
    fn no_speech() -> Self {
        Self {
            error_type: "no_speech".to_string(),
            error_message: "No speech detected".to_string(),
            user_message: "No speech detected. Please try again and speak clearly.".to_string(),
            audio_file_path: None, // Don't keep silent audio for retry
        }
    }
}

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
                        if let Err(error) = self.execute_action(action, &mut current_recording) {
                            self.handle_action_error(error);
                        }
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

    /// Centralized error handler for all action failures
    ///
    /// This ensures consistent error handling across all actions:
    /// 1. Logs the error
    /// 2. Resets state machine to Ready
    /// 3. Updates last recording state (preserves audio file for retry if applicable)
    /// 4. Updates menu state
    /// 5. Emits error event to frontend
    fn handle_action_error(&self, error: ActionError) {
        log::error!(
            "Action error [{}]: {}",
            error.error_type,
            error.error_message
        );

        // Reset state machine to Ready
        self.state_manager.reset();

        // Clear last recording state - user started a new recording so previous one is stale
        match self.last_recording_state.lock() {
            Ok(mut last_recording) => {
                last_recording.text = None;
                last_recording.timestamp = None;
                // Keep audio file for retry if available
                last_recording.audio_file_path = error.audio_file_path.clone();
            }
            Err(e) => {
                log::error!("Failed to lock last_recording_state: {}", e);
            }
        }

        // Disable paste menu item since there's no valid text
        if let Err(e) = self.menu.set_paste_last_inactive() {
            log::error!("Failed to disable paste menu item: {}", e);
        }

        // Emit error event to frontend
        let error_event = RecordingStateChanged::Error {
            error_type: error.error_type,
            error_message: error.error_message,
            user_message: error.user_message,
            audio_file_path: error.audio_file_path,
        };

        if let Err(e) = error_event.emit(&self.app_handle) {
            log::error!("Failed to emit error event: {}", e);
        }
    }

    /// Execute action returned by the state machine
    fn execute_action(
        &self,
        action: RecordingAction,
        recording: &mut Option<Recording>,
    ) -> Result<(), ActionError> {
        match action {
            RecordingAction::StartRecording => {
                let rec = self.handle_start()?;
                *recording = Some(rec);
            }
            RecordingAction::StopAndTranscribe => {
                if let Some(rec) = recording.take() {
                    self.handle_stop(rec)?;
                }
                // Notify updater that recording/transcription finished
                updater::on_recording_finished(&self.app_handle);
            }
            RecordingAction::CancelRecording => {
                if let Some(rec) = recording.take() {
                    self.handle_cancel(rec)?;
                }
                // Notify updater that recording was cancelled
                updater::on_recording_finished(&self.app_handle);
            }
            RecordingAction::RetryTranscription => {
                self.handle_retry_transcription()?;
                // Notify updater that transcription finished
                updater::on_recording_finished(&self.app_handle);
            }
        }
        Ok(())
    }

    fn handle_start(&self) -> Result<Recording, ActionError> {
        // Show recording popup window
        if let Err(e) = open_recording_popup(&self.app_handle) {
            log::error!("Failed to open recording popup: {}", e);
        }

        if let Err(e) = RecordingStateChanged::Started.emit(&self.app_handle) {
            return Err(ActionError::recording(
                format!("Failed to emit started event: {}", e),
                "Failed to start recording".to_string(),
            ));
        }

        // Get the audio level channel if one is registered
        let level_channel = match self.audio_level_channel.lock() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                log::error!("Failed to lock audio_level_channel: {}", e);
                None
            }
        };

        let recording = self.audio_recorder.start(level_channel).map_err(|e| {
            // Close popup since recording failed to start
            if let Err(close_err) = close_recording_popup(&self.app_handle) {
                log::error!("Failed to close recording popup: {}", close_err);
            }
            ActionError::recording(format!("{:?}", e), e.user_message())
        })?;

        Ok(recording)
    }

    fn handle_stop(&self, recording: Recording) -> Result<(), ActionError> {
        let recording_result = recording
            .stop()
            .map_err(|e| ActionError::stop(format!("{:?}", e), None))?;

        // Check if enough speech was detected (VAD filtering may have removed everything)
        if recording_result.speech_duration_ms < MIN_SPEECH_DURATION_MS {
            log::info!(
                "No speech detected: {}ms < {}ms minimum, skipping transcription",
                recording_result.speech_duration_ms,
                MIN_SPEECH_DURATION_MS
            );

            // Clean up the (nearly empty) audio file
            if CLEANUP_AUDIO_AFTER_TRANSCRIPTION {
                cleanup_recording_file(&recording_result.file_path);
            }

            // Hide recording popup
            if let Err(e) = close_recording_popup(&self.app_handle) {
                log::error!("Failed to close recording popup: {}", e);
            }

            return Err(ActionError::no_speech());
        }

        if let Err(e) = RecordingStateChanged::Transcribing.emit(&self.app_handle) {
            log::error!("Failed to emit recording-transcribing event: {:?}", e);
        }

        // Use speech_duration_ms for validation (actual content duration, not wall-clock time)
        self.perform_transcription(
            &recording_result.file_path,
            recording_result.speech_duration_ms,
        )
    }

    fn handle_cancel(&self, recording: Recording) -> Result<(), ActionError> {
        // Stop recording (creates file but we don't use it)
        let recording_result = recording
            .stop()
            .map_err(|e| ActionError::cancel(format!("{:?}", e)))?;

        // Clean up the cancelled recording file immediately
        if CLEANUP_AUDIO_AFTER_TRANSCRIPTION {
            cleanup_recording_file(&recording_result.file_path);
        }

        // Hide recording popup window
        if let Err(e) = close_recording_popup(&self.app_handle) {
            log::error!("Failed to close recording popup: {}", e);
        }

        // Emit cancellation event for frontend awareness
        RecordingStateChanged::Cancelled
            .emit(&self.app_handle)
            .map_err(|e| ActionError::cancel(format!("Failed to emit cancelled event: {}", e)))?;

        Ok(())
    }

    fn handle_retry_transcription(&self) -> Result<(), ActionError> {
        // Get audio file path from last recording state
        let (audio_file_path, duration_ms) = {
            let last_recording = self.last_recording_state.lock().map_err(|e| {
                ActionError::transcription(
                    &TranscriptionError::ApiError(format!("Failed to lock state: {}", e)),
                    String::new(),
                )
            })?;

            // No audio file available - nothing to retry
            let path = last_recording.audio_file_path.clone().ok_or_else(|| {
                ActionError::transcription(
                    &TranscriptionError::FileNotFound("No audio file available".to_string()),
                    String::new(),
                )
            })?;

            // Estimate duration from file size based on audio format
            let metadata = std::fs::metadata(&path).map_err(|e| {
                ActionError::transcription(
                    &TranscriptionError::FileNotFound(format!("Audio file not found: {}", e)),
                    path.clone(),
                )
            })?;
            let duration_ms = (metadata.len() * 1000) / AUDIO_BYTES_PER_SECOND;

            (path, duration_ms)
        };

        // Emit transcribing event
        if let Err(e) = RecordingStateChanged::Transcribing.emit(&self.app_handle) {
            log::error!("Failed to emit recording-transcribing event: {:?}", e);
        }

        self.perform_transcription(&audio_file_path, duration_ms)
    }

    /// Shared transcription logic used by both handle_stop and handle_retry_transcription.
    ///
    /// Uses the unified Transcriber abstraction which handles both API-based
    /// (OpenAI, Azure) and local (Whisper) transcription transparently.
    fn perform_transcription(
        &self,
        audio_file_path: &str,
        duration_ms: u64,
    ) -> Result<(), ActionError> {
        // Create transcriber from app handle - handles all providers uniformly
        let transcriber = Transcriber::from_app(&self.app_handle)
            .map_err(|e| ActionError::transcription(&e, audio_file_path.to_string()))?;

        // Transcribe - the transcriber handles API vs local internally
        let text = transcriber
            .transcribe(PathBuf::from(audio_file_path), duration_ms)
            .map_err(|e| ActionError::transcription(&e, audio_file_path.to_string()))?;

        self.handle_transcription_success(&text, audio_file_path)
    }

    /// Handle successful transcription: cleanup, paste, update state, emit event
    fn handle_transcription_success(
        &self,
        text: &str,
        audio_file_path: &str,
    ) -> Result<(), ActionError> {
        // Reset state: Transcribing -> Ready
        self.state_manager.reset();

        // Clean up recording file after successful transcription
        if CLEANUP_AUDIO_AFTER_TRANSCRIPTION {
            cleanup_recording_file(audio_file_path);
        }

        if !text.is_empty() {
            crate::text_paster::paste_text(text).map_err(|e| {
                ActionError::transcription(
                    &TranscriptionError::ApiError(format!("Failed to paste text: {}", e)),
                    audio_file_path.to_string(),
                )
            })?;
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

        if let Err(e) = self.menu.set_paste_last_active() {
            log::error!("Failed to enable paste menu item: {}", e);
        }

        // Hide recording popup window
        if let Err(e) = close_recording_popup(&self.app_handle) {
            log::error!("Failed to close recording popup: {}", e);
        }

        if let Err(e) = (RecordingStateChanged::Stopped {
            text: text.to_string(),
        })
        .emit(&self.app_handle)
        {
            log::error!("Failed to emit stopped event: {}", e);
        }

        Ok(())
    }
}
