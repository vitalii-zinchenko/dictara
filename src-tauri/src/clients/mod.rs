mod azure_client;
mod client;
mod config;
mod error;
mod openai_client;
mod transcriber;

// Re-export public types
pub use config::ApiConfig;
pub use error::TranscriptionError;
pub use transcriber::Transcriber;
