use derive_more::Display;
use tauri::Manager;

// State for the paste last recording menu item
pub struct PasteMenuItemState {
    pub item: tauri::menu::MenuItem<tauri::Wry>,
}

// Custom error type for tray operations
#[derive(Debug, Display)]
pub enum TrayError {
    #[display("Tray state not found in app")]
    StateNotFound,
    #[display("Failed to set icon: {}", _0)]
    IconSetFailed(String),
}

impl std::error::Error for TrayError {}

/// Updates the "Paste Last Recording" menu item enabled state
pub fn update_paste_menu_item(
    app_handle: &tauri::AppHandle,
    enabled: bool,
) -> Result<(), TrayError> {
    println!("[Tray] Updating paste menu item - enabled: {}", enabled);

    let state = app_handle
        .try_state::<PasteMenuItemState>()
        .ok_or(TrayError::StateNotFound)?;

    state.item.set_enabled(enabled).map_err(|e| {
        TrayError::IconSetFailed(format!("Failed to set menu item enabled state: {}", e))
    })?;

    println!("[Tray]  Paste menu item updated successfully");
    Ok(())
}
