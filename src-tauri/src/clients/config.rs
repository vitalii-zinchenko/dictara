use crate::config::Provider;

/// Configuration for making transcription API calls
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub provider: Provider,
    pub api_key: String,
    /// Full transcription endpoint for Azure (without api-version), unused for OpenAI
    pub endpoint: String,
}
