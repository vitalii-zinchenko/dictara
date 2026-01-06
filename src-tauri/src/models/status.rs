use serde::{Deserialize, Serialize};

/// Runtime status of a model - computed, not stored.
#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    /// Does the model file exist on disk?
    pub is_downloaded: bool,
    /// Is the model currently being downloaded?
    pub is_downloading: bool,
    /// Is the model currently loaded in memory?
    pub is_loaded: bool,
    /// Is the model currently being loaded into memory?
    pub is_loading: bool,
    /// Size of partial download file (for resume support)
    pub downloaded_bytes: u64,
}
