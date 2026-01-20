use serde::{Deserialize, Serialize};

/// Type of transcription model
#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Whisper,
    Parakeet,
}

/// Individual file within a multi-file model
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ModelFile {
    /// Filename to save as
    pub filename: String,
    /// Download URL
    pub url: String,
    /// SHA-256 checksum (hex string)
    pub sha256: String,
}

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
    /// Type of model (Whisper or Parakeet)
    pub model_type: ModelType,
    /// Filename on disk (e.g., "ggml-small.bin" for single-file, "parakeet-v3" for multi-file directory)
    pub filename: String,
    /// Actual size on disk in bytes
    pub size_bytes: u64,
    /// Approximate RAM usage when loaded in MB
    pub estimated_ram_mb: u64,
    /// Files to download (single file for Whisper, multiple files for Parakeet)
    pub files: Vec<ModelFile>,
}

/// Check if a model name exists in the catalog
pub fn is_model_in_catalog(name: &str) -> bool {
    get_model_catalog().iter().any(|e| e.name == name)
}

/// Hardcoded catalog of available transcription models
/// Whisper SHA-256 hashes from: https://huggingface.co/ggerganov/whisper.cpp
/// Parakeet models from NVIDIA via Hugging Face
pub fn get_model_catalog() -> Vec<ModelCatalogEntry> {
    vec![
        // Parakeet models - multi-file downloads (25 languages, faster CPU inference)
        ModelCatalogEntry {
            name: "parakeet-tdt-0.6b-v3-int8".into(),
            display_name: "Parakeet V3 INT8".into(),
            description: "Fast multilingual model optimized for CPU (25 languages). Recommended for 8GB RAM.".into(),
            model_type: ModelType::Parakeet,
            filename: "parakeet-tdt-0.6b-v3-int8".into(), // Directory name
            size_bytes: 670_479_942, // ~639 MB total
            estimated_ram_mb: 1200,
            files: vec![
                ModelFile {
                    filename: "encoder-model.int8.onnx".into(),
                    url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/encoder-model.int8.onnx".into(),
                    sha256: "6139d2fa7e1b086097b277c7149725edbab89cc7c7ae64b23c741be4055aff09".into(),
                },
                ModelFile {
                    filename: "decoder_joint-model.int8.onnx".into(),
                    url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/decoder_joint-model.int8.onnx".into(),
                    sha256: "eea7483ee3d1a30375daedc8ed83e3960c91b098812127a0d99d1c8977667a70".into(),
                },
                ModelFile {
                    filename: "vocab.txt".into(),
                    url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/vocab.txt".into(),
                    sha256: "d58544679ea4bc6ac563d1f545eb7d474bd6cfa467f0a6e2c1dc1c7d37e3c35d".into(),
                },
            ],
        },
        ModelCatalogEntry {
            name: "parakeet-tdt-0.6b-v3-fp32".into(),
            display_name: "Parakeet V3 FP32".into(),
            description: "High-accuracy multilingual model (25 languages). Best quality. Recommended for 16GB+ RAM.".into(),
            model_type: ModelType::Parakeet,
            filename: "parakeet-tdt-0.6b-v3-fp32".into(), // Directory name
            size_bytes: 2_549_805_858, // ~2.37 GB total
            estimated_ram_mb: 3200,
            files: vec![
                ModelFile {
                    filename: "encoder-model.onnx".into(),
                    url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/encoder-model.onnx".into(),
                    sha256: "98a74b21b4cc0017c1e7030319a4a96f4a9506e50f0708f3a516d02a77c96bb1".into(),
                },
                ModelFile {
                    filename: "encoder-model.onnx.data".into(),
                    url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/encoder-model.onnx.data".into(),
                    sha256: "9a22d372c51455c34f13405da2520baefb7125bd16981397561423ed32d24f36".into(),
                },
                ModelFile {
                    filename: "decoder_joint-model.onnx".into(),
                    url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/decoder_joint-model.onnx".into(),
                    sha256: "e978ddf6688527182c10fde2eb4b83068421648985ef23f7a86be732be8706c1".into(),
                },
                ModelFile {
                    filename: "vocab.txt".into(),
                    url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/vocab.txt".into(),
                    sha256: "d58544679ea4bc6ac563d1f545eb7d474bd6cfa467f0a6e2c1dc1c7d37e3c35d".into(),
                },
            ],
        },

        // Whisper models - single-file downloads
        ModelCatalogEntry {
            name: "whisper-small".into(),
            display_name: "Whisper Small".into(),
            description: "Fast, good for most use cases. Recommended for 8GB RAM.".into(),
            model_type: ModelType::Whisper,
            filename: "ggml-small.bin".into(),
            size_bytes: 487_601_967, // ~465 MB
            estimated_ram_mb: 800,
            files: vec![ModelFile {
                filename: "ggml-small.bin".into(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
                    .into(),
                sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b"
                    .into(),
            }],
        },
        ModelCatalogEntry {
            name: "whisper-medium".into(),
            display_name: "Whisper Medium".into(),
            description: "Better accuracy, requires more RAM. Recommended for 16GB RAM.".into(),
            model_type: ModelType::Whisper,
            filename: "ggml-medium.bin".into(),
            size_bytes: 1_533_763_059, // ~1.43 GB
            estimated_ram_mb: 2200,
            files: vec![ModelFile {
                filename: "ggml-medium.bin".into(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"
                    .into(),
                sha256: "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208"
                    .into(),
            }],
        },
        ModelCatalogEntry {
            name: "whisper-large-v3-turbo".into(),
            display_name: "Whisper Large v3 Turbo".into(),
            description: "Fast large model variant. Recommended for 16GB RAM.".into(),
            model_type: ModelType::Whisper,
            filename: "ggml-large-v3-turbo.bin".into(),
            size_bytes: 1_624_555_275, // ~1.51 GB
            estimated_ram_mb: 2500,
            files: vec![ModelFile {
                filename: "ggml-large-v3-turbo.bin".into(),
                url:
                    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin"
                        .into(),
                sha256: "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69"
                    .into(),
            }],
        },
        ModelCatalogEntry {
            name: "whisper-large-v3".into(),
            display_name: "Whisper Large v3".into(),
            description: "Best accuracy, requires significant RAM. Recommended for 16GB+ RAM."
                .into(),
            model_type: ModelType::Whisper,
            filename: "ggml-large-v3.bin".into(),
            size_bytes: 3_095_033_483, // ~2.88 GB
            estimated_ram_mb: 4000,
            files: vec![ModelFile {
                filename: "ggml-large-v3.bin".into(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
                    .into(),
                sha256: "64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2"
                    .into(),
            }],
        },
    ]
}
