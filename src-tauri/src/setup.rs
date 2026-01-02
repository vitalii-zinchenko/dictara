use crate::updater::{self, Updater};
use crate::{
    config::{self, AzureOpenAIConfig, OnboardingStep, OpenAIConfig, Provider},
    globe_key,
    keyboard_listener::KeyListener,
    keychain::{self, ProviderAccount},
    recording::{
        cleanup_old_recordings, Controller, LastRecording, LastRecordingState, RecordingCommand,
        RecordingStateManager,
    },
    ui::{menu::Menu, tray::Tray, window},
};
use log::{error, info, warn};
use std::sync::{Arc, Mutex};
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
    info!("Dictara v{}", env!("CARGO_PKG_VERSION"));

    // Clean up old recordings from previous sessions
    cleanup_old_recordings(app.app_handle());

    // Check accessibility permission on macOS
    #[cfg(target_os = "macos")]
    {
        let has_permission = macos_accessibility_client::accessibility::application_is_trusted();
        if !has_permission {
            warn!("Accessibility permission not granted");
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Keep the app running in the background
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }

    // Load app config and check if properly configured
    let store = app.store("config.json")?;
    let app_config = config::load_app_config(&store);
    let mut onboarding_config = config::load_onboarding_config(&store);

    // Handle pending restart from accessibility step
    if onboarding_config.pending_restart {
        onboarding_config.pending_restart = false;

        // Check if accessibility is now granted
        #[cfg(target_os = "macos")]
        {
            let has_accessibility =
                macos_accessibility_client::accessibility::application_is_trusted();
            if has_accessibility {
                onboarding_config.current_step = OnboardingStep::ApiKeys;
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
        warn!("AI provider not configured");
    }

    // Determine if we need to show onboarding
    let show_onboarding = !onboarding_config.finished;

    // ========================================
    // CHANNEL-BASED ARCHITECTURE WITH CONTROLLER
    // Setup creates the channel and wires components together
    // ========================================

    // Create channel for recording commands (KeyListener → Controller)
    let (command_tx, command_rx) = mpsc::channel::<RecordingCommand>(100);
    let state_manager = Arc::new(RecordingStateManager::new());

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

    let menu = Menu::new(app)?;
    let _tray = Tray::new(app, &menu)?;

    // Initialize controller (transcriber created on-demand from config)
    let controller = Controller::new(
        command_rx,
        app.app_handle().clone(),
        state_manager.clone(),
        audio_level_channel.channel.clone(),
        last_recording_state.clone(),
        menu,
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
        let _listener = KeyListener::start(command_tx, state_manager.clone());
    }

    // Initialize and start the updater
    // In debug mode: checks and downloads updates but skips installation
    // In release mode: checks, downloads, and installs updates when user is idle
    let updater = Arc::new(Updater::new(state_manager));
    app.manage(updater.clone());
    updater::start_periodic_update_check(app.app_handle().clone(), updater);

    // Decide which window to open
    if show_onboarding {
        // During onboarding, silently fix the Globe key setting if it's set to "Show Emoji"
        // This prevents the emoji picker from appearing when using Fn for recording
        globe_key::fix_globe_key_if_needed();

        if let Err(e) = window::open_onboarding_window(app.app_handle()) {
            error!("Failed to open onboarding window: {}", e);
        }
    } else if needs_configuration {
        if let Err(e) = window::open_preferences_window(app.app_handle()) {
            error!("Failed to open preferences window: {}", e);
        }
    }

    Ok(())
}
