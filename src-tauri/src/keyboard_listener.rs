use crate::recording::{RecordingCommand, RecordingStateManager};
use dictara_keyboard::{grab, EventType, Key};
use log::error;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc;

/// Lock modifier key (Space) for hands-free mode
const LOCK_MODIFIER: Key = Key::Space;

/// Keyboard listener that detects key events and emits recording commands
pub struct KeyListener {
    _thread_handle: Option<JoinHandle<()>>,
}

impl KeyListener {
    pub fn start(
        command_tx: mpsc::Sender<RecordingCommand>,
        state_manager: Arc<RecordingStateManager>,
        recording_trigger: Key,
    ) -> Self {
        // For triggers with both left and right keys, match both variants
        // Note: Mac keyboards have left+right for Option and Command, but only left Control
        let trigger_alt = match recording_trigger {
            Key::Alt => Some(Key::AltGr),          // Right Option key
            Key::MetaLeft => Some(Key::MetaRight), // Right Command key
            _ => None,
        };

        let thread_handle = thread::spawn(move || {
            if let Err(err) = grab(move |event| {
                let is_trigger = |key: Key| key == recording_trigger || trigger_alt == Some(key);

                match event.event_type {
                    EventType::KeyPress(key) if is_trigger(key) => {
                        let _ = command_tx.blocking_send(RecordingCommand::StartRecording);
                        // Pass through - emoji picker is blocked by globe_key::fix_globe_key_if_needed()
                        // which sets macOS system preference, not by swallowing events
                        Some(event)
                    }
                    EventType::KeyRelease(key) if is_trigger(key) => {
                        let _ = command_tx.blocking_send(RecordingCommand::StopRecording);
                        Some(event)
                    }
                    EventType::KeyPress(key) if key == LOCK_MODIFIER => {
                        if state_manager.is_busy() {
                            let _ = command_tx.blocking_send(RecordingCommand::LockRecording);
                            None // Avoid inserting a space while recording
                        } else {
                            Some(event) // Pass through
                        }
                    }
                    _ => Some(event), // Pass through all other events
                }
            }) {
                error!(
                    "Keyboard grab failed: {}. Keyboard shortcuts will not work.",
                    err
                );
            }
        });

        Self {
            _thread_handle: Some(thread_handle),
        }
    }
}
