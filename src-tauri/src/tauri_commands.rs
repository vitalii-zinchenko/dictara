use crate::recording::RecordingCommand;
use crate::setup::RecordingCommandSender;
use tauri::State;

#[tauri::command]
pub fn check_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos_accessibility_client::accessibility::application_is_trusted()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true // Other platforms don't need this permission
    }
}

#[tauri::command]
pub fn request_accessibility_permission() {
    #[cfg(target_os = "macos")]
    {
        // This will show macOS system dialog and open System Settings
        macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
    }
}

#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

#[tauri::command]
pub fn stop_recording(sender: State<RecordingCommandSender>) -> Result<(), String> {
    sender.sender.blocking_send(RecordingCommand::Stop)
        .map_err(|e| format!("Failed to send Stop command: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn cancel_recording(sender: State<RecordingCommandSender>) -> Result<(), String> {
    sender.sender.blocking_send(RecordingCommand::Cancel)
        .map_err(|e| format!("Failed to send Cancel command: {}", e))?;

    Ok(())
}
