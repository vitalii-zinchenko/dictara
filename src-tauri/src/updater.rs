#[cfg(not(debug_assertions))]
use log::warn;
use log::{error, info};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;

use crate::recording::RecordingStateManager;

/// Check interval: 30 minutes in release, 1 minute in debug for testing
#[cfg(not(debug_assertions))]
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(30 * 60);
#[cfg(debug_assertions)]
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(60);

/// Idle check interval: how often to check if user is idle
const IDLE_CHECK_INTERVAL: Duration = Duration::from_secs(10);

/// Required idle time before installing update (1 minute in release, 10 seconds in debug)
#[cfg(not(debug_assertions))]
const REQUIRED_IDLE_SECONDS: f64 = 60.0;
#[cfg(debug_assertions)]
const REQUIRED_IDLE_SECONDS: f64 = 10.0;

/// Fallback: max time to wait for idle before installing anyway (5 minutes)
const MAX_WAIT_FOR_IDLE: Duration = Duration::from_secs(5 * 60);

/// Stores a downloaded update ready for installation
struct PendingInstall {
    bytes: Vec<u8>,
    version: String,
    downloaded_at: std::time::Instant,
}

/// Manages automatic update checking, downloading, and installation
pub struct Updater {
    /// Whether an update check is currently in progress
    checking: AtomicBool,
    /// Whether there's a pending update that was deferred due to recording
    pending_update: AtomicBool,
    /// Reference to the recording state manager (shared with Controller and KeyListener)
    state_manager: Arc<RecordingStateManager>,
    /// Downloaded update bytes waiting for installation
    pending_install: Mutex<Option<PendingInstall>>,
}

impl Updater {
    pub fn new(state_manager: Arc<RecordingStateManager>) -> Self {
        Self {
            checking: AtomicBool::new(false),
            pending_update: AtomicBool::new(false),
            state_manager,
            pending_install: Mutex::new(None),
        }
    }

    /// Check if the app is currently recording/transcribing
    pub fn is_busy(&self) -> bool {
        self.state_manager.is_busy()
    }

    /// Check if an update check is in progress
    pub fn is_checking(&self) -> bool {
        self.checking.load(Ordering::Acquire)
    }

    /// Set checking state
    fn set_checking(&self, value: bool) {
        self.checking.store(value, Ordering::Release);
    }

    /// Check if there's a pending update
    pub fn has_pending_update(&self) -> bool {
        self.pending_update.load(Ordering::Acquire)
    }

    /// Set pending update state
    fn set_pending_update(&self, value: bool) {
        self.pending_update.store(value, Ordering::Release);
    }

    /// Check if there's a downloaded update ready to install
    fn has_pending_install(&self) -> bool {
        match self.pending_install.lock() {
            Ok(guard) => guard.is_some(),
            Err(e) => {
                error!("Mutex poisoned in has_pending_install: {}", e);
                false
            }
        }
    }

    /// Get pending install info without taking it (for idle timeout check)
    fn get_pending_install_age(&self) -> Option<Duration> {
        match self.pending_install.lock() {
            Ok(guard) => guard.as_ref().map(|p| p.downloaded_at.elapsed()),
            Err(e) => {
                error!("Mutex poisoned in get_pending_install_age: {}", e);
                None
            }
        }
    }

    /// Store downloaded update for later installation
    fn set_pending_install(&self, bytes: Vec<u8>, version: String) {
        if let Ok(mut guard) = self.pending_install.lock() {
            *guard = Some(PendingInstall {
                bytes,
                version,
                downloaded_at: std::time::Instant::now(),
            });
        } else {
            error!("Failed to acquire lock for pending install - mutex poisoned");
        }
    }

    /// Take the pending install (removes it from storage)
    fn take_pending_install(&self) -> Option<PendingInstall> {
        match self.pending_install.lock() {
            Ok(mut guard) => guard.take(),
            Err(e) => {
                error!("Mutex poisoned in take_pending_install: {}", e);
                None
            }
        }
    }
}

/// Get the number of seconds since the last user input event (keyboard/mouse)
#[cfg(target_os = "macos")]
fn get_idle_seconds() -> f64 {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state_id: i32, event_type: u32) -> f64;
    }

    // SAFETY: CGEventSourceSecondsSinceLastEventType is a stable CoreGraphics API that:
    // - Takes simple integer parameters with no preconditions
    // - Returns a simple f64 value
    // - Has no side effects or memory management requirements
    // We use kCGEventSourceStateHIDSystemState (1) and kCGAnyInputEventType (u32::MAX)
    unsafe { CGEventSourceSecondsSinceLastEventType(1, u32::MAX) }
}

/// Fallback idle detection for non-macOS platforms
/// Returns a large value to indicate "idle" (will rely on timeout fallback)
#[cfg(not(target_os = "macos"))]
fn get_idle_seconds() -> f64 {
    // On non-macOS platforms, we can't easily detect idle time.
    // Return 0 to indicate "not idle" - we'll rely on the timeout fallback.
    0.0
}

/// Check if it's safe to install based on idle time or timeout
fn should_install_now(updater_state: &Updater) -> bool {
    // Never install if app is busy (recording/transcribing)
    if updater_state.is_busy() {
        return false;
    }

    let idle_seconds = get_idle_seconds();

    // Install if user has been idle long enough
    if idle_seconds >= REQUIRED_IDLE_SECONDS {
        return true;
    }

    // Fallback: install if we've been waiting too long (prevents updates from never installing)
    if let Some(age) = updater_state.get_pending_install_age() {
        if age >= MAX_WAIT_FOR_IDLE {
            info!(
                "Update has been pending for {:?}, installing despite only {:.0}s idle time",
                age, idle_seconds
            );
            return true;
        }
    }

    false
}

/// Actually perform the installation (or simulate in debug mode)
/// This is the only function that differs between debug and release builds
#[cfg(not(debug_assertions))]
fn perform_install(app_handle: &tauri::AppHandle, pending: PendingInstall) {
    info!("Installing update v{}...", pending.version);

    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let result: Result<(), Box<dyn std::error::Error + Send + Sync>> = async {
            let updater = handle.updater()?;
            let update = updater.check().await?;

            let Some(update) = update else {
                return Err("Update no longer available".into());
            };

            // Verify the version matches what we downloaded
            if update.version != pending.version {
                warn!(
                    "Version mismatch: downloaded v{} but server now has v{}. Installing anyway.",
                    pending.version, update.version
                );
            }

            update.install(pending.bytes)?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                info!("Update installed successfully, restarting app");
                handle.restart();
            }
            Err(e) => {
                error!("Failed to install update: {:?}", e);
            }
        }
    });
}

/// Debug mode: simulate installation by logging
#[cfg(debug_assertions)]
fn perform_install(_app_handle: &tauri::AppHandle, pending: PendingInstall) {
    info!("=================================================");
    info!("🔧 DEBUG MODE: Skipping actual update installation");
    info!("   Version: {}", pending.version);
    info!(
        "   Package size: {} bytes ({:.2} MB)",
        pending.bytes.len(),
        pending.bytes.len() as f64 / 1_048_576.0
    );
    info!("   In release mode, the app would restart now");
    info!("=================================================");
}

/// Start periodic update checking and idle-based installation
/// Should be called from setup after the app is initialized
pub fn start_periodic_update_check(app_handle: tauri::AppHandle, updater_state: Arc<Updater>) {
    #[cfg(debug_assertions)]
    info!("🔧 DEBUG MODE: Updater running in simulation mode - updates will be downloaded but not installed");

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

            // Check if it's safe to install now
            if should_install_now(&updater_state) {
                if let Some(pending) = updater_state.take_pending_install() {
                    perform_install(&app_handle, pending);
                }
            }
        }
    });
}

/// Check for updates and download if available (but don't install yet)
async fn check_and_download_update(app_handle: tauri::AppHandle, updater_state: Arc<Updater>) {
    // Skip if already has a pending install
    if updater_state.has_pending_install() {
        return;
    }

    // Skip if app is busy
    if updater_state.is_busy() {
        updater_state.set_pending_update(true);
        return;
    }

    // Skip if already checking
    if updater_state.is_checking() {
        return;
    }

    updater_state.set_checking(true);

    let result = download_update_only(&app_handle, &updater_state).await;

    if let Err(e) = result {
        error!("Update check/download failed: {:?}", e);
    }

    updater_state.set_checking(false);
}

/// Check for updates and download (without installing)
async fn download_update_only(
    app_handle: &tauri::AppHandle,
    updater_state: &Updater,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let updater = app_handle.updater()?;
    let update = updater.check().await?;

    let Some(update) = update else {
        info!("No updates available");
        return Ok(());
    };

    let version = update.version.clone();
    info!("Update available: v{}", version);

    // Check if app is busy - defer if so
    if updater_state.is_busy() {
        info!("App is busy, deferring update download");
        updater_state.set_pending_update(true);
        return Ok(());
    }

    info!("Downloading update v{}...", version);

    // Download only (don't install yet)
    let bytes = update.download(|_, _| {}, || {}).await?;

    info!(
        "Update v{} downloaded ({} bytes), waiting for idle to install",
        version,
        bytes.len()
    );

    // Store the downloaded bytes for later installation
    updater_state.set_pending_install(bytes, version);

    Ok(())
}

/// Manual update check triggered from frontend
/// Returns: true if update is available, false otherwise
#[tauri::command]
#[specta::specta]
pub async fn check_for_updates(
    app_handle: tauri::AppHandle,
    show_no_update_message: bool,
) -> Result<bool, String> {
    // Get updater state
    let updater_state = app_handle
        .try_state::<Arc<Updater>>()
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

/// Manual check implementation - downloads and optionally installs (user requested)
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

    info!("Manual check: Update available v{}", update.version);

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
        return Ok(true); // Update was available but declined
    }

    // Check if app is busy
    if let Some(state) = app_handle.try_state::<Arc<Updater>>() {
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

    info!("Downloading update v{}...", update.version);

    // Download the update
    let bytes = update
        .download(|_, _| {}, || {})
        .await
        .map_err(|e| format!("Failed to download update: {}", e))?;

    let version = update.version.clone();

    // In debug mode, just log; in release mode, actually install
    #[cfg(debug_assertions)]
    {
        info!("=================================================");
        info!("🔧 DEBUG MODE: Skipping manual update installation");
        info!("   Version: {}", version);
        info!(
            "   Package size: {} bytes ({:.2} MB)",
            bytes.len(),
            bytes.len() as f64 / 1_048_576.0
        );
        info!("   In release mode, the app would restart now");
        info!("=================================================");
    }

    #[cfg(not(debug_assertions))]
    {
        info!("Installing update v{}...", version);
        update
            .install(bytes)
            .map_err(|e| format!("Failed to install update: {}", e))?;

        info!("Update installed, restarting app");
        app_handle.restart();
    }

    #[allow(unreachable_code)]
    Ok(true)
}

/// Called when recording finishes to check for pending updates
pub fn on_recording_finished(app_handle: &tauri::AppHandle) {
    if let Some(state) = app_handle.try_state::<Arc<Updater>>() {
        if state.has_pending_update() {
            state.set_pending_update(false);

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
