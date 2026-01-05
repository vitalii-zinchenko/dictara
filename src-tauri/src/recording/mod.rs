mod audio_recorder;
mod commands;
mod controller;
pub mod events;
mod state_manager;
pub mod vad;

use std::sync::{Arc, Mutex};
use std::time::SystemTime;

// Re-export state manager types
pub use state_manager::{RecordingAction, RecordingEvent, RecordingStateManager, TransitionResult};

// Public exports
pub use audio_recorder::{
    cleanup_old_recordings, cleanup_recording_file, RecorderError, Recording,
};
pub use commands::RecordingCommand;
pub use controller::Controller;

/// Stores the last recording attempt for paste retry functionality
#[derive(Debug, Clone)]
pub struct LastRecording {
    /// The transcribed text. Some = can paste, None = cannot paste (disable menu)
    pub text: Option<String>,
    /// Timestamp of when the recording was made
    pub timestamp: Option<SystemTime>,
    /// Audio file path. Some = transcription failed (keep for retry), None = succeeded (cleaned up)
    pub audio_file_path: Option<String>,
}

impl LastRecording {
    pub fn new() -> Self {
        Self {
            text: None,
            timestamp: None,
            audio_file_path: None,
        }
    }

    #[allow(dead_code)]
    pub fn can_paste(&self) -> bool {
        self.text.is_some()
    }
}

pub type LastRecordingState = Arc<Mutex<LastRecording>>;
