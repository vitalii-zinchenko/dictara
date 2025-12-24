#[cfg(not(debug_assertions))]
use std::sync::Mutex;
use std::sync::{
    atomic::{AtomicBool, AtomicU8, Ordering},
    Arc,
};
#[cfg(not(debug_assertions))]
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;

/// Check interval: 30 minutes (for testing - change to 4 hours for production)
#[cfg(not(debug_assertions))]
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(30 * 60);

/// Idle check interval: how often to check if user is idle
#[cfg(not(debug_assertions))]
const IDLE_CHECK_INTERVAL: Duration = Duration::from_secs(10);

/// Required idle time before installing update (1 minute)
#[cfg(not(debug_assertions))]
const REQUIRED_IDLE_SECONDS: f64 = 60.0;

/// Recording states (matches controller.rs)
const STATE_READY: u8 = 0;

/// Stores a downloaded update ready for installation
#[cfg(not(debug_assertions))]
struct PendingInstall {
    bytes: Vec<u8>,
    version: String,
}

/// Shared state for the updater
pub struct UpdaterState {
    /// Whether an update check is currently in progress
    checking: AtomicBool,
    /// Whether there's a pending update that was deferred due to recording
    pending_update: AtomicBool,
    /// Reference to the recording state (shared with Controller)
    recording_state: Arc<AtomicU8>,
    /// Downloaded update bytes waiting for installation
    #[cfg(not(debug_assertions))]
    pending_install: Mutex<Option<PendingInstall>>,
}

impl UpdaterState {
    #[cfg(not(debug_assertions))]
    pub fn new(recording_state: Arc<AtomicU8>) -> Self {
        Self {
            checking: AtomicBool::new(false),
            pending_update: AtomicBool::new(false),
            recording_state,
            pending_install: Mutex::new(None),
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

    /// Check if there's a downloaded update ready to install
    #[cfg(not(debug_assertions))]
    fn has_pending_install(&self) -> bool {
        self.pending_install.lock().unwrap().is_some()
    }

    /// Store downloaded update for later installation
    #[cfg(not(debug_assertions))]
    fn set_pending_install(&self, bytes: Vec<u8>, version: String) {
        *self.pending_install.lock().unwrap() = Some(PendingInstall { bytes, version });
    }

    /// Take the pending install (removes it from storage)
    #[cfg(not(debug_assertions))]
    fn take_pending_install(&self) -> Option<PendingInstall> {
        self.pending_install.lock().unwrap().take()
    }
}

/// Get the number of seconds since the last user input event (keyboard/mouse)
#[cfg(all(target_os = "macos", not(debug_assertions)))]
fn get_idle_seconds() -> f64 {
    // Direct FFI call to CoreGraphics
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state_id: i32, event_type: u32) -> f64;
    }

    // kCGEventSourceStateHIDSystemState = 1
    // kCGAnyInputEventType = 0xFFFFFFFF (u32::MAX)
    unsafe { CGEventSourceSecondsSinceLastEventType(1, u32::MAX) }
}

/// Start periodic update checking and idle-based installation
/// Should be called from setup after the app is initialized
#[cfg(not(debug_assertions))]
pub fn start_periodic_update_check(app_handle: tauri::AppHandle, updater_state: Arc<UpdaterState>) {
    println!("[Updater] Starting periodic update check (every 30 minutes for testing)");

    // Initial check after a short delay
    let handle = app_handle.clone();
    let state = updater_state.clone();
    tauri::async_runtime::spawn(async move {
        // Wait 5 seconds for app to fully initialize
        tokio::time::sleep(Duration::from_secs(5)).await;
        check_and_download_update(handle, state).await;
    });

    // Periodic checks for new updates
    let handle = app_handle.clone();
    let state = updater_state.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(UPDATE_CHECK_INTERVAL).await;
            println!("[Updater] Periodic update check triggered");
            check_and_download_update(handle.clone(), state.clone()).await;
        }
    });

    // Idle monitor - checks if user is idle and installs pending update
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(IDLE_CHECK_INTERVAL).await;

            // Only proceed if there's a pending install
            if !updater_state.has_pending_install() {
                continue;
            }

            // Don't install if app is busy
            if updater_state.is_busy() {
                println!("[Updater] App busy, deferring install");
                continue;
            }

            // Check idle time
            #[cfg(target_os = "macos")]
            {
                let idle_seconds = get_idle_seconds();
                if idle_seconds >= REQUIRED_IDLE_SECONDS {
                    println!(
                        "[Updater] User idle for {:.0}s (>= {:.0}s), installing update...",
                        idle_seconds, REQUIRED_IDLE_SECONDS
                    );
                    install_pending_update(&app_handle, &updater_state);
                }
            }
        }
    });
}

/// Check for updates and download if available (but don't install yet)
#[cfg(not(debug_assertions))]
async fn check_and_download_update(app_handle: tauri::AppHandle, updater_state: Arc<UpdaterState>) {
    // Skip if already has a pending install
    if updater_state.has_pending_install() {
        println!("[Updater] Already have a downloaded update, skipping check");
        return;
    }

    // Skip if app is busy
    if updater_state.is_busy() {
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

    let result = download_update_only(&app_handle, &updater_state).await;

    if let Err(e) = result {
        eprintln!("[Updater] Update check/download failed: {:?}", e);
    }

    updater_state.set_checking(false);
}

/// Check for updates and download (without installing)
#[cfg(not(debug_assertions))]
async fn download_update_only(
    app_handle: &tauri::AppHandle,
    updater_state: &UpdaterState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[Updater] Checking for updates...");

    let updater = app_handle.updater()?;
    let update = updater.check().await?;

    let Some(update) = update else {
        println!("[Updater] No update available");
        return Ok(());
    };

    let version = update.version.clone();
    println!("[Updater] Update available: {}", version);

    // Check if app is busy - defer if so
    if updater_state.is_busy() {
        println!("[Updater] App is busy, deferring download");
        updater_state.set_pending_update(true);
        return Ok(());
    }

    println!("[Updater] Downloading update (will install when user is idle)...");

    // Download only (don't install yet)
    let bytes = update
        .download(
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

    println!(
        "[Updater] Update downloaded ({} bytes), waiting for user to be idle...",
        bytes.len()
    );

    // Store the downloaded bytes for later installation
    updater_state.set_pending_install(bytes, version);

    Ok(())
}

/// Install the pending update (called when user is idle)
#[cfg(not(debug_assertions))]
fn install_pending_update(app_handle: &tauri::AppHandle, updater_state: &UpdaterState) {
    let Some(pending) = updater_state.take_pending_install() else {
        return;
    };

    println!(
        "[Updater] Installing update v{} ({} bytes)...",
        pending.version,
        pending.bytes.len()
    );

    // We need to get the update object again to call install
    // Since install() is a method on Update, we need to re-fetch it
    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let result: Result<(), Box<dyn std::error::Error + Send + Sync>> = async {
            let updater = handle.updater()?;
            let update = updater.check().await?;

            let Some(update) = update else {
                return Err("Update no longer available".into());
            };

            // Install the downloaded bytes
            update.install(pending.bytes)?;

            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                println!("[Updater] Update installed, restarting app...");
                handle.restart();
            }
            Err(e) => {
                eprintln!("[Updater] Failed to install update: {:?}", e);
            }
        }
    });
}

/// Manual update check triggered from frontend
/// Returns: true if update is available, false otherwise
#[tauri::command]
#[specta::specta]
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

/// Manual check implementation - this one installs immediately (user requested)
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

    // Download and install immediately (user explicitly requested)
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

            #[cfg(not(debug_assertions))]
            {
                let handle = app_handle.clone();
                let state_clone = state.inner().clone();
                tauri::async_runtime::spawn(async move {
                    // Small delay to let the UI settle
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    check_and_download_update(handle, state_clone).await;
                });
            }
        }
    }
}
