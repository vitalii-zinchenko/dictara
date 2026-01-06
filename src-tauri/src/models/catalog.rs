use serde::{Deserialize, Serialize};

/// Static information about a model available for download.
/// This is hardcoded and never changes at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogEntry {
    /// Unique identifier, no spaces (e.g., "whisper-small")
    pub name: String,
    /// Human-readable name (e.g., "Whisper Small")
    pub display_name: String,
    /// Description of the model
    pub description: String,
    /// Filename on disk (e.g., "ggml-small.bin")
    pub filename: String,
    /// Hugging Face download URL
    pub url: String,
    /// Approximate size on disk in bytes
    pub estimated_size_bytes: u64,
    /// Approximate RAM usage when loaded in MB
    pub estimated_ram_mb: u64,
    /// SHA-256 checksum for integrity verification (hex string)
    pub sha256: String,
}

/// Check if a model name exists in the catalog
pub fn is_model_in_catalog(name: &str) -> bool {
    get_model_catalog().iter().any(|e| e.name == name)
}

/// Hardcoded catalog of available Whisper models
/// SHA-256 hashes from: https://huggingface.co/ggerganov/whisper.cpp
pub fn get_model_catalog() -> Vec<ModelCatalogEntry> {
    vec![
        ModelCatalogEntry {
            name: "whisper-small".into(),
            display_name: "Whisper Small".into(),
            description: "Fast, good for most use cases. Recommended for 8GB RAM.".into(),
            filename: "ggml-small.bin".into(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".into(),
            estimated_size_bytes: 466_000_000, // ~466 MB
            estimated_ram_mb: 800,
            sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b".into(),
        },
        ModelCatalogEntry {
            name: "whisper-medium".into(),
            display_name: "Whisper Medium".into(),
            description: "Better accuracy, requires more RAM. Recommended for 16GB RAM.".into(),
            filename: "ggml-medium.bin".into(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".into(),
            estimated_size_bytes: 1_500_000_000, // ~1.5 GB
            estimated_ram_mb: 2200,
            sha256: "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208".into(),
        },
        ModelCatalogEntry {
            name: "whisper-large-v3-turbo".into(),
            display_name: "Whisper Large v3 Turbo".into(),
            description: "Fast large model variant. Recommended for 16GB RAM.".into(),
            filename: "ggml-large-v3-turbo.bin".into(),
            url:
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin"
                    .into(),
            estimated_size_bytes: 1_600_000_000, // ~1.6 GB
            estimated_ram_mb: 2500,
            sha256: "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69".into(),
        },
        ModelCatalogEntry {
            name: "whisper-large-v3".into(),
            display_name: "Whisper Large v3".into(),
            description: "Best accuracy, requires significant RAM. Recommended for 16GB+ RAM."
                .into(),
            filename: "ggml-large-v3.bin".into(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
                .into(),
            estimated_size_bytes: 3_000_000_000, // ~3 GB
            estimated_ram_mb: 4000,
            sha256: "64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2".into(),
        },
    ]
}
