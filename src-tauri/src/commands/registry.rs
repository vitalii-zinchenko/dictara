/// Single source of truth for all commands
/// This macro takes a wrapper macro path and applies it to the command list
#[macro_export]
macro_rules! with_commands {
    ($($wrapper:tt)*) => {
        $($wrapper)*![
            // App
            $crate::commands::get_app_version,
            // Accessibility
            $crate::commands::check_accessibility_permission,
            $crate::commands::request_accessibility_permission,
            // Microphone
            $crate::commands::check_microphone_permission,
            $crate::commands::request_microphone_permission,
            $crate::commands::open_microphone_settings,
            // App configuration
            $crate::commands::load_app_config,
            $crate::commands::save_app_config,
            // Provider selection
            $crate::commands::get_current_provider,
            $crate::commands::set_current_provider,
            $crate::commands::clear_current_provider,
            // OpenAI provider
            $crate::commands::load_openai_config,
            $crate::commands::save_openai_config,
            $crate::commands::delete_openai_config,
            $crate::commands::test_openai_config,
            // Azure OpenAI provider
            $crate::commands::load_azure_openai_config,
            $crate::commands::save_azure_openai_config,
            $crate::commands::delete_azure_openai_config,
            $crate::commands::test_azure_openai_config,
            // OpenRouter provider
            $crate::commands::load_openrouter_config,
            $crate::commands::save_openrouter_config,
            $crate::commands::delete_openrouter_config,
            $crate::commands::test_openrouter_config,
            // Custom Endpoint provider
            $crate::commands::load_custom_endpoint_config,
            $crate::commands::save_custom_endpoint_config,
            $crate::commands::delete_custom_endpoint_config,
            $crate::commands::test_custom_endpoint_config,
            // Local model provider
            $crate::commands::get_available_models,
            $crate::commands::download_model,
            $crate::commands::cancel_model_download,
            $crate::commands::delete_model,
            $crate::commands::load_model,
            $crate::commands::unload_model,
            $crate::commands::get_loaded_model,
            $crate::commands::load_local_model_config,
            $crate::commands::save_local_model_config,
            $crate::commands::delete_local_model_config,
            // Recording
            $crate::commands::stop_recording,
            $crate::commands::cancel_recording,
            $crate::commands::retry_transcription,
            $crate::commands::dismiss_error,
            $crate::commands::resize_popup_for_error,
            $crate::commands::register_audio_level_channel,
            // Onboarding
            $crate::commands::restart_app,
            $crate::commands::load_onboarding_config,
            $crate::commands::save_onboarding_step,
            $crate::commands::finish_onboarding,
            $crate::commands::skip_onboarding,
            $crate::commands::set_pending_restart,
            $crate::commands::restart_onboarding,
            // Shortcuts
            $crate::commands::load_shortcuts_config,
            $crate::commands::save_shortcuts_config,
            $crate::commands::reset_shortcuts_config,
            $crate::commands::start_key_capture,
            $crate::commands::stop_key_capture,
            // System
            $crate::commands::enable_autostart,
            $crate::commands::disable_autostart,
            $crate::commands::is_autostart_enabled,
            $crate::commands::mark_autostart_setup_done,
            $crate::commands::is_autostart_setup_done,
        ]
    };
}
