use serde::{Deserialize, Serialize};

/// Key capture event - streamed to frontend during shortcut configuration
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum KeyCaptureEvent {
    /// Key was pressed
    #[serde(rename = "keyDown")]
    KeyDown { keycode: u32, label: String },
    /// Key was released
    #[serde(rename = "keyUp")]
    KeyUp { keycode: u32, label: String },
}
