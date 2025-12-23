use std::sync::mpsc;
use tauri::{Manager, Monitor};

type AnyError = Box<dyn std::error::Error + Send + Sync>;

const POPUP_WIDTH_NORMAL: u32 = 80;
const POPUP_WIDTH_ERROR: u32 = 400; // 5x wider for error display
const POPUP_HEIGHT: u32 = 74;
const BOTTOM_MARGIN: i32 = 100;

/// Show a window without stealing focus (macOS only).
/// Uses `orderFront:` instead of `makeKeyAndOrderFront:` to avoid activating the app.
#[cfg(target_os = "macos")]
fn show_window_without_focus(window: &tauri::WebviewWindow) -> Result<(), AnyError> {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    use objc2_app_kit::{NSApplicationActivationOptions, NSWorkspace};
    use std::ptr;

    println!("[Window] show_window_without_focus: getting ns_window...");

    // Capture the currently frontmost app so we can restore focus after showing our popup.
    let frontmost_app = NSWorkspace::sharedWorkspace().frontmostApplication();

    // Get the raw NSWindow pointer from Tauri
    let ns_window_ptr = match window.ns_window() {
        Ok(ptr) => {
            println!("[Window] Got ns_window pointer");
            ptr as *mut AnyObject
        }
        Err(e) => {
            eprintln!(
                "[Window] Failed to get ns_window: {:?}, falling back to show()",
                e
            );
            window.show()?;
            return Ok(());
        }
    };

    println!("[Window] Calling native macOS methods...");

    // Safety: ns_window_ptr is a valid NSWindow pointer from Tauri
    unsafe {
        // First, make the window visible (setIsVisible:YES doesn't activate)
        let _: () = msg_send![ns_window_ptr, setIsVisible: true];
        println!("[Window] setIsVisible done");
        // Then bring to front without making key (orderFront: vs makeKeyAndOrderFront:)
        let _: () = msg_send![ns_window_ptr, orderFront: ptr::null::<AnyObject>()];
        println!("[Window] orderFront done");
    }

    // Give focus back to whoever had it before we showed the popup.
    if let Some(app) = frontmost_app {
        let app_handle = window.app_handle();
        let _ = app_handle.run_on_main_thread(move || {
            let _ = app.activateWithOptions(NSApplicationActivationOptions(0));
        });
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn show_window_without_focus(window: &tauri::WebviewWindow) -> Result<(), AnyError> {
    window.show()?;
    Ok(())
}

/// Find the monitor containing the cursor position.
///
/// All monitors share a virtual desktop coordinate space. Each monitor has:
/// - **Position**: where it sits in the virtual space (e.g., secondary monitor at x=1920)
/// - **Size**: its own dimensions
///
/// ```text
///   (0,0)                    (1920,0)
///     +----------------------+----------------------+
///     |                      |                      |
///     |     Primary          |     Secondary        |
///     |     Monitor          |     Monitor          |
///     |     1920x1080        |     1920x1080        |
///     |                      |                      |
///     +----------------------+----------------------+
///                        (1920,1080)            (3840,1080)
/// ```
///
/// This function checks which monitor's rectangle contains the cursor coordinates.
fn get_monitor_at_cursor(app_handle: &tauri::AppHandle) -> Option<Monitor> {
    let cursor_pos = app_handle.cursor_position().ok()?;

    let monitors = app_handle.available_monitors().ok()?;

    for monitor in monitors {
        let pos = monitor.position();
        let size = monitor.size();

        // Check if cursor is within this monitor's bounds (physical coordinates)
        if cursor_pos.x >= pos.x as f64
            && cursor_pos.x < (pos.x + size.width as i32) as f64
            && cursor_pos.y >= pos.y as f64
            && cursor_pos.y < (pos.y + size.height as i32) as f64
        {
            return Some(monitor);
        }
    }

    None
}

fn run_on_main_thread_sync<T, F>(app_handle: &tauri::AppHandle, f: F) -> Result<T, AnyError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, AnyError> + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let app_handle_clone = app_handle.clone();

    // macOS AppKit APIs (NSWindow) must run on the main thread; wrap the work and
    // shuttle the result back through a channel so callers can stay synchronous.
    app_handle_clone.run_on_main_thread(move || {
        let _ = tx.send(f());
    })?;

    rx.recv()
        .unwrap_or_else(|_| Err("Failed to receive result from main thread task".into()))
}

pub fn open_recording_popup(app_handle: &tauri::AppHandle) -> Result<(), AnyError> {
    let app_handle_for_closure = app_handle.clone();
    run_on_main_thread_sync(app_handle, move || {
        open_recording_popup_inner(&app_handle_for_closure)
    })
}

pub fn close_recording_popup(app_handle: &tauri::AppHandle) -> Result<(), AnyError> {
    let app_handle_for_closure = app_handle.clone();
    run_on_main_thread_sync(app_handle, move || {
        close_recording_popup_inner(&app_handle_for_closure)
    })
}

fn open_recording_popup_inner(app_handle: &tauri::AppHandle) -> Result<(), AnyError> {
    if let Some(window) = app_handle.get_webview_window("recording-popup") {
        // Set size
        if let Err(e) = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: POPUP_WIDTH_NORMAL as f64,
            height: POPUP_HEIGHT as f64,
        })) {
            eprintln!("[Window] Failed to set window size: {}", e);
        }

        // Get monitor at cursor, fallback to primary monitor
        let monitor = get_monitor_at_cursor(app_handle)
            .or_else(|| app_handle.primary_monitor().ok().flatten());

        if let Some(monitor) = monitor {
            let scale_factor = monitor.scale_factor();
            let monitor_size = monitor.size();
            let monitor_position = monitor.position();

            // Convert physical to logical coordinates
            let logical_width = monitor_size.width as f64 / scale_factor;
            let logical_height = monitor_size.height as f64 / scale_factor;
            let logical_x = monitor_position.x as f64 / scale_factor;
            let logical_y = monitor_position.y as f64 / scale_factor;

            // Calculate centered horizontal position
            let x = logical_x + (logical_width - POPUP_WIDTH_NORMAL as f64) / 2.0;

            // Calculate position from bottom
            let y = logical_y + logical_height - POPUP_HEIGHT as f64 - BOTTOM_MARGIN as f64;

            if let Err(e) =
                window.set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }))
            {
                eprintln!("[Window] Failed to set window position: {}", e);
            }
        } else {
            eprintln!("[Window] Failed to get monitor at cursor or primary monitor");
        }

        if let Err(e) = show_window_without_focus(&window) {
            eprintln!("[Window] Failed to show recording popup: {}", e);
            return Err(e);
        }
    } else {
        return Err("Recording popup window not found".into());
    }

    Ok(())
}

fn close_recording_popup_inner(app_handle: &tauri::AppHandle) -> Result<(), AnyError> {
    if let Some(window) = app_handle.get_webview_window("recording-popup") {
        if let Err(e) = window.hide() {
            eprintln!("[Window] Failed to hide recording popup: {}", e);
            return Err(Box::new(e));
        }
    } else {
        return Err("Recording popup window not found".into());
    }

    Ok(())
}

pub fn resize_recording_popup_for_error(app_handle: &tauri::AppHandle) -> Result<(), AnyError> {
    let app_handle_for_closure = app_handle.clone();
    run_on_main_thread_sync(app_handle, move || {
        resize_recording_popup_inner(&app_handle_for_closure, POPUP_WIDTH_ERROR)
    })
}

fn resize_recording_popup_inner(app_handle: &tauri::AppHandle, width: u32) -> Result<(), AnyError> {
    if let Some(window) = app_handle.get_webview_window("recording-popup") {
        // Set new size
        window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: width as f64,
            height: POPUP_HEIGHT as f64,
        }))?;

        // Recalculate centered position
        let monitor = get_monitor_at_cursor(app_handle)
            .or_else(|| app_handle.primary_monitor().ok().flatten());

        if let Some(monitor) = monitor {
            let scale_factor = monitor.scale_factor();
            let monitor_size = monitor.size();
            let monitor_position = monitor.position();

            let logical_width = monitor_size.width as f64 / scale_factor;
            let logical_height = monitor_size.height as f64 / scale_factor;
            let logical_x = monitor_position.x as f64 / scale_factor;
            let logical_y = monitor_position.y as f64 / scale_factor;

            // Center horizontally with new width
            let x = logical_x + (logical_width - width as f64) / 2.0;
            let y = logical_y + logical_height - POPUP_HEIGHT as f64 - BOTTOM_MARGIN as f64;

            window.set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }))?;
        }

        Ok(())
    } else {
        Err("Recording popup window not found".into())
    }
}

pub fn open_preferences_window(app_handle: &tauri::AppHandle) -> Result<(), AnyError> {
    let (width, height) = (750.0, 650.0);

    let window = match app_handle.get_webview_window("preferences") {
        Some(w) => w,
        None => tauri::WebviewWindowBuilder::new(
            app_handle,
            "preferences",
            tauri::WebviewUrl::App("preferences".into()),
        )
        .title("Preferences")
        .inner_size(width, height)
        .min_inner_size(width, height)
        // .max_inner_size(width, height)
        .visible(false)
        .build()?,
    };

    window.show()?;
    window.set_focus()?;
    window.center()?;

    Ok(())
}
