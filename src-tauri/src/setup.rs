#[cfg(not(debug_assertions))]
use crate::updater::{self, UpdaterState};
use crate::{
    clients::openai::OpenAIClient,
    config::{self, AzureOpenAIConfig, OnboardingStep, OpenAIConfig, Provider},
    keyboard_listener::KeyListener,
    keychain::{self, ProviderAccount},
    recording::{
        cleanup_old_recordings, Controller, LastRecording, LastRecordingState, RecordingCommand,
    },
    ui::{menu::build_menu, tray::PasteMenuItemState, window},
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
    println!("Dictara v{}", env!("CARGO_PKG_VERSION"));

    // Clean up old recordings from previous sessions
    cleanup_old_recordings(app.app_handle());

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

    // Load app config and check if properly configured
    let store = app.store("config.json")?;
    let app_config = config::load_app_config(&store);
    let mut onboarding_config = config::load_onboarding_config(&store);

    // Handle pending restart from accessibility step
    if onboarding_config.pending_restart {
        println!("🔄 Resuming onboarding after restart...");
        onboarding_config.pending_restart = false;

        // Check if accessibility is now granted
        #[cfg(target_os = "macos")]
        {
            let has_accessibility =
                macos_accessibility_client::accessibility::application_is_trusted();
            if has_accessibility {
                println!("✅ Accessibility granted after restart, moving to API Keys step");
                onboarding_config.current_step = OnboardingStep::ApiKeys;
            } else {
                println!("⚠️  Accessibility still not granted, staying on accessibility step");
            }
        }

        // Save the updated config
        config::save_onboarding_config(&store, &onboarding_config)?;
    }

    // Check if any provider is properly configured
    let needs_configuration = match &app_config.active_provider {
        Some(Provider::OpenAI) => {
            keychain::load_provider_config::<OpenAIConfig>(ProviderAccount::OpenAI)
                .ok()
                .flatten()
                .is_none()
        }
        Some(Provider::AzureOpenAI) => {
            keychain::load_provider_config::<AzureOpenAIConfig>(ProviderAccount::AzureOpenAI)
                .ok()
                .flatten()
                .is_none()
        }
        None => true,
    };

    if needs_configuration {
        println!("⚠️  AI provider not configured.");
    } else {
        println!("✅ AI provider configured successfully");
    }

    // Determine if we need to show onboarding
    let show_onboarding = !onboarding_config.finished;
    if show_onboarding {
        println!("🚀 Onboarding not finished, will show onboarding window");
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

    // Only start keyboard listener if accessibility permission is granted
    // This prevents the permission dialog from appearing during onboarding
    #[cfg(target_os = "macos")]
    let has_accessibility = macos_accessibility_client::accessibility::application_is_trusted();
    #[cfg(not(target_os = "macos"))]
    let has_accessibility = true;

    if has_accessibility {
        let _listener = KeyListener::start(command_tx, recording_state.clone());
    } else {
        println!("⚠️  Skipping keyboard listener - accessibility not granted yet");
    }

    let menu_with_items = build_menu(app)?;
    let paste_menu_item_state = PasteMenuItemState {
        item: menu_with_items.paste_last_item,
    };

    // Build tray icon with template image for menu bar
    const TRAY_ICON_BYTES: &[u8] = include_bytes!("../icons/tray-icon.png");
    let tray_icon_image = image::load_from_memory(TRAY_ICON_BYTES)
        .expect("Failed to load tray icon")
        .to_rgba8();
    let (width, height) = tray_icon_image.dimensions();
    let tray_icon = tauri::image::Image::new_owned(tray_icon_image.into_raw(), width, height);

    let _tray = tauri::tray::TrayIconBuilder::new()
        .icon(tray_icon)
        .icon_as_template(true) // macOS template image - auto-adapts to light/dark mode
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

    app.manage(paste_menu_item_state);

    // Initialize and start the updater (only in release builds)
    #[cfg(not(debug_assertions))]
    {
        let updater_state = Arc::new(UpdaterState::new(recording_state));
        app.manage(updater_state.clone());
        updater::start_periodic_update_check(app.app_handle().clone(), updater_state);
    }

    // Decide which window to open
    if show_onboarding {
        // Onboarding not finished - show onboarding window
        if let Err(e) = window::open_onboarding_window(app.app_handle()) {
            eprintln!("Failed to open onboarding window: {}", e);
        }
    } else if needs_configuration {
        // Onboarding finished but configuration missing - show preferences
        if let Err(e) = window::open_preferences_window(app.app_handle()) {
            eprintln!("Failed to open preferences window: {}", e);
        }
    }

    Ok(())
}
