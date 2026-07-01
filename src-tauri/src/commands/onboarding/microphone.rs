#[cfg(target_os = "macos")]
use tauri_plugin_opener::OpenerExt;

// ===== MICROPHONE PERMISSION COMMANDS =====

/// Check microphone permission status
/// Returns: "authorized", "denied", "restricted", or "not_determined"
#[tauri::command]
#[specta::specta]
pub fn check_microphone_permission() -> String {
    #[cfg(target_os = "macos")]
    {
        use objc2_av_foundation::{AVAuthorizationStatus, AVCaptureDevice, AVMediaTypeAudio};

        // Safety: AVMediaTypeAudio is an extern static that must be accessed in unsafe block
        let media_type = match unsafe { AVMediaTypeAudio } {
            Some(mt) => mt,
            None => return "unknown".to_string(),
        };

        // Safety: media_type is a valid NSString reference
        let status = unsafe { AVCaptureDevice::authorizationStatusForMediaType(media_type) };

        match status {
            AVAuthorizationStatus::Authorized => "authorized".to_string(),
            AVAuthorizationStatus::Denied => "denied".to_string(),
            AVAuthorizationStatus::Restricted => "restricted".to_string(),
            AVAuthorizationStatus::NotDetermined => "not_determined".to_string(),
            _ => "unknown".to_string(),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        "authorized".to_string() // Other platforms don't need this permission
    }
}

/// Request microphone permission (triggers native permission dialog)
#[tauri::command]
#[specta::specta]
pub async fn request_microphone_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        use objc2::runtime::Bool;
        use objc2_av_foundation::{AVCaptureDevice, AVMediaTypeAudio};
        use std::sync::mpsc;

        // Safety: AVMediaTypeAudio is an extern static that must be accessed in unsafe block
        let media_type = match unsafe { AVMediaTypeAudio } {
            Some(mt) => mt,
            None => return false,
        };

        // Create a channel to communicate between the callback and async context
        let (tx, rx) = mpsc::channel::<bool>();

        // Safety: media_type is a valid NSString reference
        // This will trigger the native permission dialog if not yet determined
        unsafe {
            let block = block2::RcBlock::new(move |granted: Bool| {
                let _ = tx.send(granted.as_bool());
            });
            AVCaptureDevice::requestAccessForMediaType_completionHandler(media_type, &block);
        }

        // Wait for the callback to complete
        rx.recv().unwrap_or(false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        true // Other platforms don't need this permission
    }
}

/// Open System Settings to the Microphone privacy pane
#[tauri::command]
#[specta::specta]
#[cfg_attr(not(target_os = "macos"), allow(unused_variables))]
pub fn open_microphone_settings(app: tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    {
        // macOS URL scheme for Privacy & Security > Microphone
        let _ = app.opener().open_url(
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone",
            None::<&str>,
        );
    }
}
