mod api_transcriber;
mod azure_client;
mod client;
mod config;
mod error;
mod local_transcriber;
mod openai_client;
mod service;
mod transcriber;

// Re-export public types
pub use config::ApiConfig;
pub use error::TranscriptionError;
pub use transcriber::Transcriber;
