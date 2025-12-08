use rdev::{listen, Event, EventType, Key, ListenError};
use std::thread;
use std::time::SystemTime;
use serde::Serialize;
use tauri::Emitter;

#[derive(Clone, Serialize)]
struct FnKeyEvent {
    pressed: bool,
    timestamp: u128,
}

#[derive(Clone, Serialize)]
struct ListenerError {
    error: String,
    is_permission_error: bool,
}

pub fn start_fn_key_listener(app_handle: tauri::AppHandle) {
    thread::spawn(move || {
        println!("[FN Key Listener] Starting global keyboard listener...");

        // Clone app_handle for error handling before moving into listen closure
        let app_handle_for_error = app_handle.clone();

        if let Err(error) = listen(move |event: Event| {
            // Only handle Function key events
            if let EventType::KeyPress(Key::Function) | EventType::KeyRelease(Key::Function) = event.event_type {
                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();

                let pressed = matches!(event.event_type, EventType::KeyPress(_));
                println!("[{}] FN Key {}", timestamp, if pressed { "PRESSED" } else { "RELEASED" });

                let payload = FnKeyEvent {
                    pressed,
                    timestamp,
                };

                app_handle.emit("fn-key-event", payload).ok();
            }
        }) {
            // Handle errors
            let is_permission_error = matches!(error, ListenError::EventTapError);
            eprintln!("[FN Key Listener] Error: {:?}", error);

            let error_msg = match error {
                ListenError::EventTapError => {
                    "macOS Accessibility permission denied. Please grant permission and restart.".to_string()
                }
                _ => format!("Keyboard listener failed: {:?}", error)
            };

            let error_payload = ListenerError {
                error: error_msg,
                is_permission_error,
            };

            app_handle_for_error.emit("fn-listener-error", error_payload).ok();
        }
    });
}
