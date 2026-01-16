use crate::config::{self, ConfigKey, ConfigStore, ShortcutsConfig};
use crate::keyboard_listener::KeyListener;
use log::info;
use tauri::{AppHandle, State};

#[tauri::command]
#[specta::specta]
pub fn load_shortcuts_config(
    config_store: State<config::Config>,
) -> Result<ShortcutsConfig, String> {
    Ok(config_store.get(&ConfigKey::SHORTCUTS).unwrap_or_default())
}

#[tauri::command]
#[specta::specta]
pub fn save_shortcuts_config(
    config_store: State<config::Config>,
    key_listener: State<KeyListener>,
    config: ShortcutsConfig,
) -> Result<(), String> {
    // Validate all shortcuts
    config.push_to_record.validate()?;
    config.hands_free.validate()?;

    // Load old config for Fn key change detection
    let old_config = config_store.get(&ConfigKey::SHORTCUTS).unwrap_or_default();
    let old_uses_fn = KeyListener::uses_fn_key(&old_config);
    let new_uses_fn = KeyListener::uses_fn_key(&config);

    // Save to persistent storage
    config_store.set(&ConfigKey::SHORTCUTS, config.clone())?;
    info!(
        "Shortcuts config saved: push_to_record={:?}, hands_free={:?}",
        config.push_to_record.keys, config.hands_free.keys
    );

    // Hot-swap runtime config via channel (NO RESTART NEEDED!)
    key_listener.update_shortcuts(config)?;
    info!("Shortcuts config hot-swapped to KeyListener");

    // Update globe key fix if Fn usage changed
    if !old_uses_fn && new_uses_fn {
        crate::globe_key::fix_globe_key_if_needed();
        info!("Globe key fix applied (Fn key now in use)");
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn reset_shortcuts_config(
    config_store: State<config::Config>,
    key_listener: State<KeyListener>,
) -> Result<ShortcutsConfig, String> {
    let defaults = ShortcutsConfig::default();
    config_store.set(&ConfigKey::SHORTCUTS, defaults.clone())?;
    key_listener.update_shortcuts(defaults.clone())?;
    info!("Shortcuts config reset to defaults and hot-swapped to KeyListener");
    Ok(defaults)
}

#[tauri::command]
#[specta::specta]
pub fn start_key_capture(
    app_handle: AppHandle,
    key_listener: State<KeyListener>,
) -> Result<(), String> {
    info!("Entering key capture mode");
    // Switch KeyListener to capture mode
    key_listener.enter_capture_mode(app_handle)
}

#[tauri::command]
#[specta::specta]
pub fn stop_key_capture(
    key_listener: State<KeyListener>,
    config_store: State<config::Config>,
) -> Result<(), String> {
    // Load current shortcuts config
    let shortcuts = config_store
        .get(&ConfigKey::SHORTCUTS)
        .ok_or("Failed to load shortcuts config")?;

    info!("Exiting key capture mode, returning to normal mode");
    // Switch KeyListener back to normal mode
    key_listener.exit_capture_mode(shortcuts)
}
