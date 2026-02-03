use secrecy::SecretString;

use crate::config::Provider;

/// Configuration for making transcription API calls
#[derive(Debug)]
pub struct ApiConfig {
    pub provider: Provider,
    pub api_key: SecretString,
    /// Full transcription endpoint for Azure (without api-version), unused for OpenAI
    pub endpoint: String,
}
