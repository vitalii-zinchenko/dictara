//! Cross-platform keyboard event grabbing library.
//!
//! This crate provides a simple API for capturing and optionally blocking
//! keyboard events at the system level. It's designed as a lightweight
//! alternative to rdev with a focus on stability.
//!
//! # Example
//!
//! ```no_run
//! use dictara_keyboard::{grab, Event, EventType, Key};
//!
//! fn main() -> Result<(), dictara_keyboard::GrabError> {
//!     grab(|event| {
//!         match event.event_type {
//!             EventType::KeyPress(Key::Function) => {
//!                 println!("Fn key pressed!");
//!                 None // Swallow the event
//!             }
//!             _ => Some(event) // Pass through
//!         }
//!     })
//! }
//! ```

mod event;
mod key;

#[cfg(target_os = "macos")]
mod macos;

pub use event::{Event, EventType};
pub use key::Key;

use thiserror::Error;

/// Errors that can occur when grabbing keyboard events.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GrabError {
    /// Accessibility permission is not granted (macOS).
    #[error("Accessibility permission not granted")]
    AccessibilityNotGranted,

    /// Failed to create event tap (macOS). Usually means accessibility permission is missing.
    #[error("Failed to create event tap (check accessibility permissions)")]
    EventTapError,

    /// Failed to create run loop source (macOS).
    #[error("Failed to create run loop source")]
    LoopSourceError,

    /// Failed to get current run loop (macOS).
    #[error("Failed to get current run loop")]
    RunLoopError,

    /// Platform not supported.
    #[error("Platform not supported")]
    UnsupportedPlatform,
}

/// Callback type for the grab function.
///
/// Return `Some(event)` to pass the event through to the system.
/// Return `None` to swallow/block the event.
pub type GrabCallback = dyn FnMut(Event) -> Option<Event>;

/// Start grabbing keyboard events.
///
/// This function blocks the current thread and runs an event loop.
/// The callback is invoked for each keyboard event.
///
/// # Arguments
///
/// * `callback` - A closure that receives each event and returns `Some(event)`
///   to pass it through or `None` to swallow it.
///
/// # Platform Support
///
/// - **macOS**: Uses CGEvent tap. Requires Accessibility permission.
/// - **Windows**: Not yet implemented.
/// - **Linux**: Not yet implemented.
///
/// # Example
///
/// ```no_run
/// use dictara_keyboard::{grab, Event, EventType, Key};
///
/// grab(|event| {
///     if let EventType::KeyPress(Key::Function) = event.event_type {
///         println!("Fn pressed!");
///         return None; // Block the event
///     }
///     Some(event) // Pass through
/// }).expect("Failed to grab");
/// ```
#[cfg(target_os = "macos")]
pub fn grab<F>(callback: F) -> Result<(), GrabError>
where
    F: FnMut(Event) -> Option<Event> + 'static,
{
    macos::grab(callback)
}

#[cfg(not(target_os = "macos"))]
pub fn grab<F>(_callback: F) -> Result<(), GrabError>
where
    F: FnMut(Event) -> Option<Event> + 'static,
{
    Err(GrabError::UnsupportedPlatform)
}
