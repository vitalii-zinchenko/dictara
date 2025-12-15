use serde::Serialize;
use std::path::PathBuf;
use tauri::{Emitter, Manager};
use tokio::sync::mpsc::Receiver;

use crate::clients::openai::OpenAIClient;
use crate::error::Error;
use crate::recording::{audio_recorder::AudioRecorder, commands::RecordingCommand, Recording};

// Event payload for recording-stopped
#[derive(Clone, Serialize)]
pub struct RecordingStoppedPayload {
    pub text: String,
}

#[derive(PartialEq)]
enum ControllerState {
    /// Controller is ready to start recording
    Ready,
    /// Controller is currently recording
    Recording,
}

pub struct Controller {
    command_rx: Receiver<RecordingCommand>,
    audio_recorder: AudioRecorder,
    openai_client: OpenAIClient,
    app_handle: tauri::AppHandle,
    state: ControllerState,
}

impl Controller {
    pub fn new(
        command_rx: Receiver<RecordingCommand>,
        app_handle: tauri::AppHandle,
        openai_client: OpenAIClient,
    ) -> Self {
        let audio_recorder = AudioRecorder::new(app_handle.clone());

        Controller {
            command_rx,
            audio_recorder,
            openai_client,
            app_handle,
            state: ControllerState::Ready,
        }
    }

    /// Main control loop - consumes self, runs in blocking thread
    pub fn run(mut self) {
        // Recording session lives here (not Send, so stays in this thread)
        let mut current_recording: Option<Recording> = None;

        println!("[Controller] Starting command processing loop");

        while let Some(command) = self.command_rx.blocking_recv() {
            match command {
                RecordingCommand::Start => {
                    if self.state != ControllerState::Ready {
                        println!("[Controller] Duplicate Start command received");
                        continue;
                    }
                    self.state = ControllerState::Recording;
                    match self.handle_start() {
                        Ok(recording) => {
                            current_recording = Some(recording);
                        }
                        Err(e) => {
                            eprintln!("[Controller] Error starting recording: {:?}", e);
                            self.state = ControllerState::Ready;
                        }
                    }
                }
                RecordingCommand::Stop => {
                    if self.state != ControllerState::Recording {
                        println!("[Controller] Duplicate Stop command received");
                        continue;
                    }
                    if current_recording.is_none() {
                        println!("[Controller] No recording to stop");
                        continue;
                    }

                    if let Err(e) = self.handle_stop(current_recording.take().unwrap()) {
                        eprintln!("[Controller] Error stopping recording: {:?}", e);
                    }
                    self.state = ControllerState::Ready;
                }
            }
        }

        println!("[Controller] Channel closed, shutting down");
    }

    fn handle_start(&self) -> Result<Recording, Error> {
        println!("[Controller] Received Start command");

        // Update tray icon to recording state
        if let Err(e) = crate::ui::tray::set_recording_icon(&self.app_handle) {
            eprintln!("[Controller] Failed to set recording icon: {}", e);
        }

        // Show recording popup window
        if let Some(window) = self.app_handle.get_webview_window("recording-popup") {
            if let Err(e) = window.show() {
                eprintln!("[Controller] Failed to show recording popup: {}", e);
            }
        }

        self.app_handle.emit("recording-started", ())?;

        let recording = self.audio_recorder.start()?;

        Ok(recording)
    }

    fn handle_stop(&self, recording: Recording) -> Result<(), Error> {
        println!("[Controller] Received Stop command");

        let recording_result = recording.stop()?;

        let text = self.openai_client.transcribe_audio_sync(
            PathBuf::from(&recording_result.file_path),
            recording_result.duration_ms,
        )?;

        crate::clipboard_paste::auto_paste_text_cgevent(&text)?;

        // Restore tray icon to default state
        if let Err(e) = crate::ui::tray::set_default_icon(&self.app_handle) {
            eprintln!("[Controller] Failed to set default icon: {}", e);
        }

        // Hide recording popup window
        if let Some(window) = self.app_handle.get_webview_window("recording-popup") {
            if let Err(e) = window.hide() {
                eprintln!("[Controller] Failed to hide recording popup: {}", e);
            }
        }

        self.app_handle.emit("recording-stopped", RecordingStoppedPayload {
            text: text.clone(),
        })?;

        Ok(())
    }
}
