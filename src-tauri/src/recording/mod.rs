mod audio_recorder;
mod commands;
mod controller;

// Public exports
pub use audio_recorder::{Recording, RecorderError, cleanup_recording_file};
pub use commands::RecordingCommand;
pub use controller::Controller;
