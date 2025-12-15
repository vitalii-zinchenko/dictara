use tauri::{Manager, Monitor};

const POPUP_WIDTH: u32 = 112;
const POPUP_HEIGHT: u32 = 100;
const BOTTOM_MARGIN: i32 = 100;

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

pub fn open_recording_popup(
    app_handle: &tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app_handle.get_webview_window("recording-popup") {
        // Set size
        if let Err(e) = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: POPUP_WIDTH as f64,
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
            let x = logical_x + (logical_width - POPUP_WIDTH as f64) / 2.0;

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

        if let Err(e) = window.show() {
            eprintln!("[Window] Failed to show recording popup: {}", e);
            return Err(Box::new(e));
        }
    } else {
        return Err("Recording popup window not found".into());
    }

    Ok(())
}

pub fn close_recording_popup(
    app_handle: &tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
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
