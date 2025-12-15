use crate::{
    clients::openai::OpenAIClient,
    keyboard_listener::KeyListener,
    recording::{Controller, RecordingCommand},
    ui::{menu::build_menu, tray::TrayIconState, window},
};
use std::sync::Mutex;
use tauri::Manager;
use tokio::sync::mpsc;

pub struct RecordingCommandSender {
    pub sender: mpsc::Sender<RecordingCommand>,
}

pub fn setup_app(app: &mut tauri::App<tauri::Wry>) -> Result<(), Box<dyn std::error::Error>> {
    // Check accessibility permission on macOS
    #[cfg(target_os = "macos")]
    {
        let has_permission = macos_accessibility_client::accessibility::application_is_trusted();
        if !has_permission {
            println!("⚠️  Accessibility permission not granted. Listener will fail.");
            // Frontend will handle permission request flow
        } else {
            println!("Accessibility is granted!")
        }
    }

    // Initialize OpenAI client
    let openai_client = match OpenAIClient::new() {
        Ok(client) => {
            println!("✅ OpenAI client initialized successfully");
            client
        }
        Err(e) => {
            eprintln!("⚠️  Failed to initialize OpenAI client: {}", e);
            eprintln!("    Transcription will be disabled.");
            eprintln!("    Set OPENAI_API_KEY in .env file to enable transcription.");
            eprintln!("    Application cannot start without OpenAI client.");
            return Err(format!("Failed to initialize OpenAI client: {}", e).into());
        }
    };

    // ========================================
    // CHANNEL-BASED ARCHITECTURE WITH CONTROLLER
    // Setup creates the channel and wires components together
    // ========================================

    // Create channel for recording commands (KeyListener → Controller)
    let (command_tx, command_rx) = mpsc::channel::<RecordingCommand>(100);

    // Clone sender for Tauri state (mpsc::Sender is Clone + Send + Sync)
    let command_sender_state = RecordingCommandSender {
        sender: command_tx.clone(),
    };

    // Initialize controller with OpenAI client
    let controller = Controller::new(
        command_rx,
        app.app_handle().clone(),
        openai_client,
    );

    // Spawn controller in blocking thread (cpal::Stream is not Send)
    std::thread::spawn(move || {
        controller.run();
    });

    // Store sender in app state for Tauri commands
    app.manage(command_sender_state);

    // Start keyboard listener with command sender
    let _listener = KeyListener::start(command_tx);


    let menu = build_menu(app)?;

    // Build tray icon
    let tray = tauri::tray::TrayIconBuilder::new()
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

    // Store tray icon in app state for dynamic icon updates
    let tray_state = TrayIconState {
        tray: Mutex::new(Some(tray)),
    };
    app.manage(tray_state);

    // Debug: Open recording popup on startup
    #[cfg(debug_assertions)]
    {
        if let Err(e) = window::open_recording_popup(&app.app_handle()) {
            eprintln!("[Debug] Failed to open recording popup: {}", e);
        }
    }

    Ok(())
}
