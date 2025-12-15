/// Commands for controlling audio recording
/// These are sent through channels (NOT Tauri events) for zero-overhead internal communication
#[derive(Debug, Clone)]
pub enum RecordingCommand {
    /// Start a new recording
    Start,
    /// Stop the current recording and transcribe
    Stop,
    /// Cancel the current recording without transcribing
    Cancel,
}
