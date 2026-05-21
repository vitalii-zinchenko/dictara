mod api_transcriber;
mod azure_client;
mod client;
mod config;
mod custom_endpoint_client;
mod error;
mod local_transcriber;
mod openai_client;
mod openrouter_transcriber;
mod service;
mod transcriber;

// Re-export public types
pub use api_transcriber::ApiTranscriber;
pub use config::ApiConfig;
pub use custom_endpoint_client::CustomEndpointClient;
pub use error::TranscriptionError;
pub use openrouter_transcriber::OpenRouterTranscriber;
pub use service::TranscriptionService;
pub use transcriber::Transcriber;
