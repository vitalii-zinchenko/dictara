/// Commands for controlling audio recording
/// These are sent through channels (NOT Tauri events) for zero-overhead internal communication
#[derive(Debug, Clone)]
pub enum RecordingCommand {
    /// Fn key pressed
    FnDown,
    /// Fn key released
    FnUp,
    /// Space key pressed - lock the recording
    Lock,
    /// Cancel the current recording without transcribing
    Cancel,
    /// Retry transcription of the last failed recording
    RetryTranscription,
}
