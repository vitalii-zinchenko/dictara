use arboard::Clipboard;
use log::warn;
use std::{thread, time::Duration};

#[cfg(target_os = "macos")]
use objc2_core_graphics::{
    CGEvent, CGEventFlags, CGEventSource, CGEventSourceStateID, CGEventTapLocation, CGKeyCode,
};

#[derive(Debug, thiserror::Error)]
pub enum ClipboardPasteError {
    #[error("Faield to create event source")]
    EventSourceCreationFailed,
    #[error("Faield to create key event")]
    KeyEventCreationFailed,
    #[error("Empty text")]
    EmptyText,
    #[error("Clipboard error: {0}")]
    ClipboardError(#[from] arboard::Error),
    #[error("Unsupported platform")]
    #[cfg(not(target_os = "macos"))]
    UnsupportedPlatform,
}

/// Auto-paste text
///
/// This function:
/// 1. Saves the current clipboard content
/// 2. Sets the transcribed text to clipboard
/// 3. Simulates Cmd+V using Core Graphics events directly
/// 4. Restores the original clipboard after a delay
///
/// Returns Ok(()) on success, Err on clipboard or keyboard simulation failure
#[cfg(target_os = "macos")]
pub fn paste_text(text: &str) -> Result<(), ClipboardPasteError> {
    // Guard: Don't paste empty text
    if text.is_empty() {
        return Err(ClipboardPasteError::EmptyText);
    }

    // Save current clipboard content (if any)
    let previous_clipboard = match get_current_clipboard() {
        Ok(text) => Some(text),
        Err(_) => {
            warn!("Failed to get current clipboard content");
            None
        }
    };

    // Set transcribed text to clipboard
    set_current_clipboard(text)?;

    // Simulate paste
    simulate_paste()?;

    // Give the target application time to process the paste event
    // before restoring the original clipboard content
    thread::sleep(Duration::from_millis(100));

    // Restore previous clipboard content
    if let Some(previous_text) = previous_clipboard {
        if let Err(e) = set_current_clipboard(&previous_text) {
            warn!("Failed to set previous clipboard content: {}", e);
        }
    }

    Ok(())
}

fn get_current_clipboard() -> Result<String, arboard::Error> {
    let mut clipboard = Clipboard::new()?;
    clipboard.get_text()
}

fn set_current_clipboard(text: &str) -> Result<(), arboard::Error> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text.to_string())
}

/// Returns Ok(()) on success, Err on event creation/posting failure
#[cfg(target_os = "macos")]
pub fn simulate_paste() -> Result<(), ClipboardPasteError> {
    // Key code for 'V' key on macOS keyboard
    const V_KEYCODE: CGKeyCode = 9;

    // Create event source for HID system state
    let event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .ok_or(ClipboardPasteError::EventSourceCreationFailed)?;

    // Step 1: Create V key press event with Command modifier
    let key_down_event = CGEvent::new_keyboard_event(
        Some(&event_source),
        V_KEYCODE,
        true, // key down
    )
    .ok_or(ClipboardPasteError::KeyEventCreationFailed)?;

    // Set Command modifier flag (equivalent to Cmd key being held)
    CGEvent::set_flags(Some(&key_down_event), CGEventFlags::MaskCommand);

    // Post the key down event to HID event tap (system-level)
    CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&key_down_event));

    // Small delay to ensure key down is processed before key up
    // thread::sleep(Duration::from_millis(10));

    // Step 2: Create V key release event
    let key_up_event = CGEvent::new_keyboard_event(
        Some(&event_source),
        V_KEYCODE,
        false, // key up
    )
    .ok_or(ClipboardPasteError::KeyEventCreationFailed)?;

    // Keep Command modifier during key up (some apps need this consistency)
    CGEvent::set_flags(Some(&key_up_event), CGEventFlags::MaskCommand);

    // Post the key up event
    CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&key_up_event));

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn simulate_paste() -> Result<(), ClipboardPasteError> {
    warn!("Auto-paste not yet implemented for this platform");
    Err(ClipboardPasteError::UnsupportedPlatform)
}
