use std::{thread, time::SystemTime};

use rdev::{listen, EventType, Key};
use tauri::Manager;

mod keyboard_listener;
use std::sync::OnceLock;

use crate::keyboard_listener::start_fn_key_listener;

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
