use tauri::{Manager, Listener};
use cpal::traits::StreamTrait;

mod keyboard_listener;
mod audio_recorder;
use std::sync::OnceLock;

use crate::keyboard_listener::{start_fn_key_listener, FnKeyEvent};
use crate::audio_recorder::AudioRecorder;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn check_accessibility_permission() -> bool {
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
fn request_accessibility_permission() {
    #[cfg(target_os = "macos")]
    {
        // This will show macOS system dialog and open System Settings
        macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
    }
}

static FN_LISTENER_STARTED: OnceLock<()> = OnceLock::new();

#[tauri::command]
fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Check accessibility permission on macOS
            #[cfg(target_os = "macos")]
            {
                let has_permission =
                    macos_accessibility_client::accessibility::application_is_trusted();
                if !has_permission {
                    println!("⚠️  Accessibility permission not granted. Listener will fail.");
                    // Frontend will handle permission request flow
                } else {
                    println!("Accessibility is granted!")
                }
            }

            start_fn_key_listener(app.app_handle().clone());

            // Initialize audio recorder
            let recorder = AudioRecorder::new(app.app_handle().clone());

            // Since CPAL Stream is not Send, we leak it and manage via raw pointer
            // This is safe because we control when it's created/destroyed
            use std::sync::atomic::{AtomicPtr, Ordering};
            use std::ptr;

            static STREAM_PTR: AtomicPtr<cpal::Stream> = AtomicPtr::new(ptr::null_mut());

            let recorder_clone = recorder.clone();

            // Subscribe to FN key events to control recording
            app.listen("fn-key-event", move |event| {
                let payload = event.payload();
                match serde_json::from_str::<FnKeyEvent>(payload) {
                    Ok(fn_event) => {
                        if fn_event.pressed {
                            println!("[Audio] FN key pressed - starting recording");
                            match recorder_clone.start_recording() {
                                Ok(stream) => {
                                    // Store stream via raw pointer (unsafe but controlled)
                                    let stream_box = Box::new(stream);
                                    let stream_ptr = Box::into_raw(stream_box);
                                    STREAM_PTR.store(stream_ptr, Ordering::SeqCst);
                                }
                                Err(e) => {
                                    eprintln!("[Audio] Start error: {:?}", e);
                                    recorder_clone.emit_error(&format!("{:?}", e), "start_error");
                                }
                            }
                        } else {
                            println!("[Audio] FN key released - stopping recording");

                            // Retrieve and drop the stream
                            let stream_ptr = STREAM_PTR.swap(ptr::null_mut(), Ordering::SeqCst);
                            if !stream_ptr.is_null() {
                                unsafe {
                                    let stream = Box::from_raw(stream_ptr);
                                    stream.pause().ok();
                                    drop(stream);
                                }
                            }

                            if let Err(e) = recorder_clone.stop_recording() {
                                eprintln!("[Audio] Stop error: {:?}", e);
                                recorder_clone.emit_error(&format!("{:?}", e), "stop_error");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[Audio] Failed to parse FN key event: {}", e);
                    }
                }
            });

            // Store recorder in app state for potential future commands
            app.manage(recorder);

            // Build menu items
            let about_item = tauri::menu::MenuItemBuilder::with_id("about", "About").build(app)?;
            let preferences_item =
                tauri::menu::MenuItemBuilder::with_id("preferences", "Preferences").build(app)?;
            let quit_item = tauri::menu::MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            // Build menu
            let menu = tauri::menu::MenuBuilder::new(app)
                .item(&about_item)
                .item(&preferences_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Build tray icon
            let _tray = tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "about" => {
                            println!("About clicked - placeholder");
                            // TODO: Implement About dialog
                        }
                        "preferences" => {
                            println!("Preferences clicked - placeholder");
                            // TODO: Implement Preferences window
                        }
                        "quit" => {
                            println!("Quit clicked");
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            check_accessibility_permission,
            request_accessibility_permission,
            // start_fn_listener,
            restart_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
