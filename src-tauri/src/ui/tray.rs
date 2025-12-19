use derive_more::Display;
use std::sync::Mutex;
use tauri::Manager;

// Tray icon state for managing icon changes
pub struct TrayIconState {
    pub tray: Mutex<Option<tauri::tray::TrayIcon>>,
}

// State for the paste last recording menu item
pub struct PasteMenuItemState {
    pub item: tauri::menu::MenuItem<tauri::Wry>,
}

// Custom error type for tray operations
#[derive(Debug, Display)]
pub enum TrayError {
    #[display("Tray state not found in app")]
    StateNotFound,
    #[display("Failed to acquire lock on tray")]
    LockFailed,
    #[display("Tray icon not initialized")]
    TrayNotInitialized,
    #[display("Failed to decode icon image")]
    IconDecodeFailed,
    #[display("Failed to set icon: {}", _0)]
    IconSetFailed(String),
    #[display("Default icon not found")]
    DefaultIconNotFound,
}

impl std::error::Error for TrayError {}

/// Sets the tray icon to recording state (red circle)
pub fn set_recording_icon(app_handle: &tauri::AppHandle) -> Result<(), TrayError> {
    println!("[Tray] Recording started - changing icon to red circle");

    let state = app_handle
        .try_state::<TrayIconState>()
        .ok_or(TrayError::StateNotFound)?;

    let tray_lock = state.tray.lock().map_err(|_| TrayError::LockFailed)?;

    let tray = tray_lock.as_ref().ok_or(TrayError::TrayNotInitialized)?;

    // Load recording icon from embedded bytes
    const RECORDING_ICON_BYTES: &[u8] = include_bytes!("../../icons/recording.png");

    // Decode PNG to RGBA
    let img =
        image::load_from_memory(RECORDING_ICON_BYTES).map_err(|_| TrayError::IconDecodeFailed)?;

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let icon = tauri::image::Image::new_owned(rgba.into_raw(), width, height);

    tray.set_icon(Some(icon))
        .map_err(|e| TrayError::IconSetFailed(e.to_string()))?;

    println!("[Tray]  Icon changed to recording state");
    Ok(())
}

/// Sets the tray icon back to default state
pub fn set_default_icon(app_handle: &tauri::AppHandle) -> Result<(), TrayError> {
    println!("[Tray] Recording stopped - changing icon back to default");

    let state = app_handle
        .try_state::<TrayIconState>()
        .ok_or(TrayError::StateNotFound)?;

    let tray_lock = state.tray.lock().map_err(|_| TrayError::LockFailed)?;

    let tray = tray_lock.as_ref().ok_or(TrayError::TrayNotInitialized)?;

    // Restore default icon
    let default_icon = app_handle
        .default_window_icon()
        .ok_or(TrayError::DefaultIconNotFound)?;

    tray.set_icon(Some(default_icon.clone()))
        .map_err(|e| TrayError::IconSetFailed(e.to_string()))?;

    println!("[Tray]  Icon restored to default state");
    Ok(())
}

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
