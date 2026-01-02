//! macOS Globe/Fn Key Configuration
//!
//! This module handles reading and writing the macOS Globe key behavior setting.
//! The setting is stored in `com.apple.HIToolbox` user defaults as `AppleFnUsageType`.
//!
//! Values (verified on macOS Sequoia):
//! - 0: Do Nothing
//! - 1: Change Input Source
//! - 2: Show Emoji & Symbols (default on most Macs)
//! - 3: Start Dictation

use log::{info, warn};
use std::process::Command;

/// Globe key behavior options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum GlobeKeyBehavior {
    /// Do Nothing (value 0) - perfect for Dictara
    DoNothing = 0,
    /// Change Input Source (value 1)
    ChangeInputSource = 1,
    /// Show Emoji & Symbols picker (value 2) - macOS default
    ShowEmoji = 2,
    /// Start Dictation (value 3)
    StartDictation = 3,
}

impl GlobeKeyBehavior {
    /// Convert from integer value
    fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::DoNothing),
            1 => Some(Self::ChangeInputSource),
            2 => Some(Self::ShowEmoji),
            3 => Some(Self::StartDictation),
            _ => None,
        }
    }
}

/// Read the current Globe key behavior setting from macOS preferences.
///
/// Returns `None` if the setting doesn't exist (user never changed it)
/// or if the command fails.
#[cfg(target_os = "macos")]
pub fn get_globe_key_behavior() -> Option<GlobeKeyBehavior> {
    let output = Command::new("defaults")
        .args(["read", "com.apple.HIToolbox", "AppleFnUsageType"])
        .output()
        .ok()?;

    if output.status.success() {
        let value_str = String::from_utf8_lossy(&output.stdout);
        let value: i32 = value_str.trim().parse().ok()?;
        GlobeKeyBehavior::from_i32(value)
    } else {
        // Key doesn't exist - user never changed it from default
        None
    }
}

/// Set the Globe key behavior in macOS preferences.
///
/// Returns `Ok(())` if successful, `Err` with error message otherwise.
#[cfg(target_os = "macos")]
pub fn set_globe_key_behavior(behavior: GlobeKeyBehavior) -> Result<(), String> {
    let value = behavior as i32;

    let status = Command::new("defaults")
        .args([
            "write",
            "com.apple.HIToolbox",
            "AppleFnUsageType",
            "-int",
            &value.to_string(),
        ])
        .status()
        .map_err(|e| format!("Failed to run defaults command: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("defaults write command failed".into())
    }
}

/// Fix the Globe key setting if it would interfere with Dictara's Fn-based recording.
///
/// This is called during onboarding to silently configure the Globe key.
/// We change to "Do Nothing" unless it's already set to "Do Nothing" or "Change Input Source"
/// (both of which are compatible with Dictara).
///
/// "Show Emoji" and "Start Dictation" both interfere because they trigger system actions
/// when the Fn key is pressed.
///
/// Returns `true` if a change was made, `false` if no change was needed.
#[cfg(target_os = "macos")]
pub fn fix_globe_key_if_needed() -> bool {
    let current = get_globe_key_behavior();

    // Only these values are compatible with Dictara:
    // - DoNothing (0): Fn key does nothing, perfect for us
    // - ChangeInputSource (1): Fn key switches input source, doesn't show UI
    //
    // These values interfere and need to be fixed:
    // - None (not set, defaults to ShowEmoji on most Macs)
    // - ShowEmoji (2): Shows emoji picker popup
    // - StartDictation (3): Starts macOS dictation
    let is_compatible = matches!(
        current,
        Some(GlobeKeyBehavior::DoNothing) | Some(GlobeKeyBehavior::ChangeInputSource)
    );

    if !is_compatible {
        match set_globe_key_behavior(GlobeKeyBehavior::DoNothing) {
            Ok(()) => {
                info!("Globe key behavior changed from {:?} to DoNothing", current);
                true
            }
            Err(e) => {
                warn!("Failed to change Globe key behavior: {}", e);
                false
            }
        }
    } else {
        info!(
            "Globe key behavior is already {:?}, no change needed",
            current
        );
        false
    }
}

// Stub implementations for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub fn get_globe_key_behavior() -> Option<GlobeKeyBehavior> {
    None
}

#[cfg(not(target_os = "macos"))]
pub fn set_globe_key_behavior(_behavior: GlobeKeyBehavior) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn fix_globe_key_if_needed() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_globe_key_behavior_from_i32() {
        assert_eq!(
            GlobeKeyBehavior::from_i32(0),
            Some(GlobeKeyBehavior::DoNothing)
        );
        assert_eq!(
            GlobeKeyBehavior::from_i32(1),
            Some(GlobeKeyBehavior::ChangeInputSource)
        );
        assert_eq!(
            GlobeKeyBehavior::from_i32(2),
            Some(GlobeKeyBehavior::ShowEmoji)
        );
        assert_eq!(
            GlobeKeyBehavior::from_i32(3),
            Some(GlobeKeyBehavior::StartDictation)
        );
        assert_eq!(GlobeKeyBehavior::from_i32(4), None);
        assert_eq!(GlobeKeyBehavior::from_i32(-1), None);
    }

    #[test]
    fn test_globe_key_behavior_to_i32() {
        assert_eq!(GlobeKeyBehavior::DoNothing as i32, 0);
        assert_eq!(GlobeKeyBehavior::ChangeInputSource as i32, 1);
        assert_eq!(GlobeKeyBehavior::ShowEmoji as i32, 2);
        assert_eq!(GlobeKeyBehavior::StartDictation as i32, 3);
    }
}
