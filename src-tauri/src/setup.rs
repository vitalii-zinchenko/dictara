use crate::{
    clients::openai::OpenAIClient,
    config::{self, Provider},
    keyboard_listener::KeyListener,
    keychain::{self, KeychainAccount},
    recording::{Controller, LastRecording, LastRecordingState, RecordingCommand},
    ui::{
        menu::build_menu,
        tray::{PasteMenuItemState, TrayIconState},
        window,
    },
};
use std::sync::{atomic::AtomicU8, Arc, Mutex};
use tauri::ipc::Channel;
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tokio::sync::mpsc;

pub struct RecordingCommandSender {
    pub sender: mpsc::Sender<RecordingCommand>,
}

pub struct AudioLevelChannel {
    pub channel: Arc<Mutex<Option<Channel<f32>>>>,
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

    #[cfg(target_os = "macos")]
    {
        // Keep the app running in the background
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }

    // Initialize OpenAI client (always succeeds, key checked at transcription time)
    let openai_client = OpenAIClient::new();

    // Load provider config and check if properly configured
    let store = app.store("config.json")?;
    let provider_config = config::load_config(&store);

    // Check if any provider is properly configured
    let needs_configuration = match &provider_config.enabled_provider {
        Some(Provider::OpenAI) => keychain::load_api_key(KeychainAccount::OpenAI)
            .ok()
            .flatten()
            .is_none(),
        Some(Provider::Azure) => {
            let has_key = keychain::load_api_key(KeychainAccount::Azure)
                .ok()
                .flatten()
                .is_some();
            let has_endpoint = provider_config.azure_endpoint.is_some();
            !has_key || !has_endpoint
        }
        None => true,
    };

    if needs_configuration {
        println!("⚠️  AI provider not configured. Opening Preferences...");
    } else {
        println!("✅ AI provider configured successfully");
    }

    // ========================================
    // CHANNEL-BASED ARCHITECTURE WITH CONTROLLER
    // Setup creates the channel and wires components together
    // ========================================

    // Create channel for recording commands (KeyListener → Controller)
    let (command_tx, command_rx) = mpsc::channel::<RecordingCommand>(100);
    let recording_state = Arc::new(AtomicU8::new(0));

    // Clone sender for Tauri state (mpsc::Sender is Clone + Send + Sync)
    let command_sender_state = RecordingCommandSender {
        sender: command_tx.clone(),
    };

    // Create audio level channel state
    let audio_level_channel = AudioLevelChannel {
        channel: Arc::new(Mutex::new(None)),
    };

    // Create last recording state for paste retry functionality
    let last_recording_state: LastRecordingState = Arc::new(Mutex::new(LastRecording::new()));

    // Initialize controller with OpenAI client
    let controller = Controller::new(
        command_rx,
        app.app_handle().clone(),
        openai_client,
        recording_state.clone(),
        audio_level_channel.channel.clone(),
        last_recording_state.clone(),
    );

    // Spawn controller in blocking thread (cpal::Stream is not Send)
    std::thread::spawn(move || {
        controller.run();
    });

    // Store sender and audio level channel in app state for Tauri commands
    app.manage(command_sender_state);
    app.manage(audio_level_channel);
    app.manage(last_recording_state.clone());

    // Start keyboard listener with command sender
    let _listener = KeyListener::start(command_tx, recording_state);

    let menu_with_items = build_menu(app)?;
    let paste_menu_item_state = PasteMenuItemState {
        item: menu_with_items.paste_last_item,
    };

    // Build tray icon
    let tray = tauri::tray::TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu_with_items.menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "about" => {
                    println!("About clicked - placeholder");
                    // TODO: Implement About dialog
                }
                "preferences" => {
                    println!("Preferences clicked");
                    if let Err(e) = window::open_preferences_window(app) {
                        eprintln!("Failed to open preferences window: {}", e);
                    }
                }
                "paste_last_recording" => {
                    println!("Paste Last Recording clicked");
                    // Get the last recording state
                    if let Some(state) = app.try_state::<LastRecordingState>() {
                        if let Ok(last_recording) = state.lock() {
                            if let Some(text) = &last_recording.text {
                                // Paste the last recording
                                if let Err(e) =
                                    crate::clipboard_paste::auto_paste_text_cgevent(text)
                                {
                                    eprintln!("Failed to paste last recording: {:?}", e);
                                }
                            } else {
                                println!("No text available to paste");
                            }
                        } else {
                            eprintln!("Failed to lock last recording state");
                        }
                    } else {
                        eprintln!("Last recording state not available");
                    }
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
    app.manage(paste_menu_item_state);

    // Open preferences window if configuration needed
    if needs_configuration {
        if let Err(e) = window::open_preferences_window(&app.app_handle()) {
            eprintln!("Failed to open preferences window: {}", e);
        }
    }

    Ok(())
}
