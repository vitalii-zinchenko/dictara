use derive_more::{Display, From};

#[derive(Debug, Display, From)]
#[allow(dead_code)]
pub enum Error {
    #[from]
    Recorder(crate::recording::RecorderError),

    #[from]
    Transcription(crate::clients::TranscriptionError),

    #[from]
    ClipboardPaste(crate::text_paster::ClipboardPasteError),

    #[from]
    Tauri(tauri::Error),

    #[from]
    Keyring(keyring::Error),

    #[from]
    SerdeJson(serde_json::Error),
}
