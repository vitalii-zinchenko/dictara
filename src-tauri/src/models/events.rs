//! Typesafe events for the models module.
//!
//! These events are emitted from Rust and can be listened to in TypeScript
//! with full type safety via tauri-specta.
//!
//! Uses discriminated unions (tagged enums) for cleaner event handling,
//! similar to RecordingStateChanged pattern.

use serde::{Deserialize, Serialize};

/// Model download state change event - single event stream for all download state transitions
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum ModelDownloadStateChanged {
    /// Download is in progress
    #[serde(rename = "progress")]
    Progress {
        #[serde(rename = "modelName")]
        model_name: String,
        #[serde(rename = "downloadedBytes")]
        downloaded_bytes: u64,
        #[serde(rename = "totalBytes")]
        total_bytes: u64,
        percentage: f64,
    },
    /// Download complete, verifying checksum
    #[serde(rename = "verifying")]
    Verifying {
        #[serde(rename = "modelName")]
        model_name: String,
    },
    /// Download completed successfully
    #[serde(rename = "complete")]
    Complete {
        #[serde(rename = "modelName")]
        model_name: String,
    },
    /// Download failed with an error
    #[serde(rename = "error")]
    Error {
        #[serde(rename = "modelName")]
        model_name: String,
        error: String,
    },
}

/// Model loading state change event - single event stream for all loading state transitions
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum ModelLoadingStateChanged {
    /// Model loading has started
    #[serde(rename = "started")]
    Started {
        #[serde(rename = "modelName")]
        model_name: String,
    },
    /// Model loaded successfully
    #[serde(rename = "complete")]
    Complete {
        #[serde(rename = "modelName")]
        model_name: String,
    },
    /// Model loading failed with an error
    #[serde(rename = "error")]
    Error {
        #[serde(rename = "modelName")]
        model_name: String,
        error: String,
    },
}
