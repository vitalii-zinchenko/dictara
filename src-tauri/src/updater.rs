use std::sync::{
    atomic::{AtomicBool, AtomicU8, Ordering},
    Arc,
};
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;

/// Check interval: 4 hours
#[cfg(not(debug_assertions))]
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(4 * 60 * 60);

/// Recording states (matches controller.rs)
const STATE_READY: u8 = 0;

/// Shared state for the updater
pub struct UpdaterState {
    /// Whether an update check is currently in progress
    checking: AtomicBool,
    /// Whether there's a pending update that was deferred due to recording
    pending_update: AtomicBool,
    /// Reference to the recording state (shared with Controller)
    recording_state: Arc<AtomicU8>,
}

impl UpdaterState {
    #[cfg(not(debug_assertions))]
    pub fn new(recording_state: Arc<AtomicU8>) -> Self {
        Self {
            checking: AtomicBool::new(false),
            pending_update: AtomicBool::new(false),
            recording_state,
        }
    }

    /// Check if the app is currently recording/transcribing
    pub fn is_busy(&self) -> bool {
        self.recording_state.load(Ordering::Relaxed) != STATE_READY
    }

    /// Check if an update check is in progress
    pub fn is_checking(&self) -> bool {
        self.checking.load(Ordering::Relaxed)
    }

    /// Set checking state
    fn set_checking(&self, value: bool) {
        self.checking.store(value, Ordering::Relaxed);
    }

    /// Check if there's a pending update
    pub fn has_pending_update(&self) -> bool {
        self.pending_update.load(Ordering::Relaxed)
    }

    /// Set pending update state
    fn set_pending_update(&self, value: bool) {
        self.pending_update.store(value, Ordering::Relaxed);
    }
}

/// Start periodic update checking
/// Should be called from setup after the app is initialized
#[cfg(not(debug_assertions))]
pub fn start_periodic_update_check(app_handle: tauri::AppHandle, updater_state: Arc<UpdaterState>) {
    println!("[Updater] Starting periodic update check (every 4 hours)");

    // Initial check after a short delay
    let handle = app_handle.clone();
    let state = updater_state.clone();
    tauri::async_runtime::spawn(async move {
        // Wait 5 seconds for app to fully initialize
        tokio::time::sleep(Duration::from_secs(5)).await;
        check_for_updates_internal(handle, state, false).await;
    });

    // Periodic checks
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(UPDATE_CHECK_INTERVAL).await;
            println!("[Updater] Periodic update check triggered");
            check_for_updates_internal(app_handle.clone(), updater_state.clone(), false).await;
        }
    });
}

/// Check for updates (internal implementation)
///
/// # Arguments
/// * `app_handle` - Tauri app handle
/// * `updater_state` - Shared updater state
/// * `force` - If true, check even when app is busy (for manual checks)
async fn check_for_updates_internal(
    app_handle: tauri::AppHandle,
    updater_state: Arc<UpdaterState>,
    force: bool,
) {
    // Skip if app is busy (unless forced)
    if updater_state.is_busy() && !force {
        println!("[Updater] App is busy (recording), deferring update check");
        updater_state.set_pending_update(true);
        return;
    }

    // Skip if already checking
    if updater_state.is_checking() {
        println!("[Updater] Update check already in progress, skipping");
        return;
    }

    updater_state.set_checking(true);

    let result = check_and_prompt_update(&app_handle, &updater_state, force).await;

    if let Err(e) = result {
        eprintln!("[Updater] Update check failed: {:?}", e);
    }

    updater_state.set_checking(false);
}

/// Check for updates and prompt user if available
async fn check_and_prompt_update(
    app_handle: &tauri::AppHandle,
    updater_state: &UpdaterState,
    force: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[Updater] Checking for updates...");

    let updater = app_handle.updater()?;
    let update = updater.check().await?;

    let Some(update) = update else {
        println!("[Updater] No update available");
        return Ok(());
    };

    println!("[Updater] Update available: {}", update.version);

    // Check if app became busy while we were checking
    if updater_state.is_busy() && !force {
        println!("[Updater] App became busy, deferring update prompt");
        updater_state.set_pending_update(true);
        return Ok(());
    }

    // Build the message
    let message = if let Some(body) = &update.body {
        format!(
            "Version {} is available!\n\nRelease notes:\n{}",
            update.version, body
        )
    } else {
        format!("Version {} is available!", update.version)
    };

    // Show confirmation dialog
    let should_update = app_handle
        .dialog()
        .message(message)
        .title("Update Available")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Install & Restart".to_string(),
            "Later".to_string(),
        ))
        .blocking_show();

    if !should_update {
        println!("[Updater] User declined update");
        return Ok(());
    }

    // Double-check busy state before installing
    if updater_state.is_busy() {
        println!("[Updater] App is now busy, deferring installation");
        app_handle
            .dialog()
            .message("Cannot update while recording or transcribing. The update will be installed when the operation completes.")
            .title("Update Deferred")
            .kind(MessageDialogKind::Info)
            .blocking_show();
        updater_state.set_pending_update(true);
        return Ok(());
    }

    println!("[Updater] Downloading and installing update...");

    // Download and install
    update
        .download_and_install(
            |chunk_length, content_length| {
                println!(
                    "[Updater] Downloaded {} bytes of {:?}",
                    chunk_length, content_length
                );
            },
            || {
                println!("[Updater] Download finished");
            },
        )
        .await?;

    println!("[Updater] Update installed, restarting app...");
    app_handle.restart();
}

/// Manual update check triggered from frontend
/// Returns: true if update is available, false otherwise
#[tauri::command]
pub async fn check_for_updates(
    app_handle: tauri::AppHandle,
    show_no_update_message: bool,
) -> Result<bool, String> {
    println!("[Updater] Manual update check requested");

    // Get updater state
    let updater_state = app_handle
        .try_state::<Arc<UpdaterState>>()
        .ok_or_else(|| "Updater state not available".to_string())?;

    // Skip if already checking
    if updater_state.is_checking() {
        return Err("Update check already in progress".to_string());
    }

    updater_state.set_checking(true);

    let result = manual_check_and_prompt(&app_handle, show_no_update_message).await;

    updater_state.set_checking(false);

    result
}

/// Manual check implementation
async fn manual_check_and_prompt(
    app_handle: &tauri::AppHandle,
    show_no_update_message: bool,
) -> Result<bool, String> {
    let updater = app_handle
        .updater()
        .map_err(|e| format!("Failed to get updater: {}", e))?;

    let update = updater
        .check()
        .await
        .map_err(|e| format!("Failed to check for updates: {}", e))?;

    let Some(update) = update else {
        println!("[Updater] No update available");
        if show_no_update_message {
            app_handle
                .dialog()
                .message("You are on the latest version!")
                .title("No Update Available")
                .kind(MessageDialogKind::Info)
                .blocking_show();
        }
        return Ok(false);
    };

    println!("[Updater] Update available: {}", update.version);

    // Build the message
    let message = if let Some(body) = &update.body {
        format!(
            "Version {} is available!\n\nRelease notes:\n{}",
            update.version, body
        )
    } else {
        format!("Version {} is available!", update.version)
    };

    // Show confirmation dialog
    let should_update = app_handle
        .dialog()
        .message(message)
        .title("Update Available")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Install & Restart".to_string(),
            "Later".to_string(),
        ))
        .blocking_show();

    if !should_update {
        println!("[Updater] User declined update");
        return Ok(true); // Update was available but declined
    }

    // Check if app is busy
    if let Some(state) = app_handle.try_state::<Arc<UpdaterState>>() {
        if state.is_busy() {
            app_handle
                .dialog()
                .message("Cannot update while recording or transcribing. Please try again after the recording is complete.")
                .title("Update Deferred")
                .kind(MessageDialogKind::Warning)
                .blocking_show();
            return Ok(true);
        }
    }

    println!("[Updater] Downloading and installing update...");

    // Download and install
    update
        .download_and_install(
            |chunk_length, content_length| {
                println!(
                    "[Updater] Downloaded {} bytes of {:?}",
                    chunk_length, content_length
                );
            },
            || {
                println!("[Updater] Download finished");
            },
        )
        .await
        .map_err(|e| format!("Failed to install update: {}", e))?;

    println!("[Updater] Update installed, restarting app...");
    app_handle.restart();
}

/// Called when recording finishes to check for pending updates
pub fn on_recording_finished(app_handle: &tauri::AppHandle) {
    if let Some(state) = app_handle.try_state::<Arc<UpdaterState>>() {
        if state.has_pending_update() {
            println!("[Updater] Recording finished, checking deferred update");
            state.set_pending_update(false);

            let handle = app_handle.clone();
            let state_clone = state.inner().clone();
            tauri::async_runtime::spawn(async move {
                // Small delay to let the UI settle
                tokio::time::sleep(Duration::from_secs(2)).await;
                check_for_updates_internal(handle, state_clone, false).await;
            });
        }
    }
}
