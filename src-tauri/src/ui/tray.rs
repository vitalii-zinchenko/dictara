use crate::recording::LastRecordingState;
use crate::ui::{
    menu::{Menu, MenuId},
    window,
};
use log::{error, warn};
use std::str::FromStr;
use tauri::{self, menu::MenuEvent, tray, Manager, Wry};

const TRAY_ICON_BYTES: &[u8] = include_bytes!("../../icons/tray-icon.png");

pub struct Tray {
    #[allow(dead_code)]
    tray_icon: tray::TrayIcon<Wry>,
}

impl Tray {
    pub fn new(app: &tauri::App<Wry>, menu: &Menu) -> Result<Self, tauri::Error> {
        let icon = Self::load_icon();
        let tray_icon = Self::build_tray(app, icon, menu)?;

        Ok(Tray { tray_icon })
    }

    fn load_icon() -> tauri::image::Image<'static> {
        let tray_icon_image = image::load_from_memory(TRAY_ICON_BYTES)
            .expect("Failed to load tray icon")
            .to_rgba8();
        let (width, height) = tray_icon_image.dimensions();
        tauri::image::Image::new_owned(tray_icon_image.into_raw(), width, height)
    }

    fn build_tray(
        app: &tauri::App<Wry>,
        icon: tauri::image::Image<'static>,
        menu: &Menu,
    ) -> Result<tray::TrayIcon<Wry>, tauri::Error> {
        tray::TrayIconBuilder::new()
            .icon(icon)
            .icon_as_template(true) // macOS template image - auto-adapts to light/dark mode
            .menu(&menu.menu)
            .show_menu_on_left_click(true)
            .on_menu_event(Self::handle_menu_event)
            .build(app)
    }

    fn handle_menu_event(app: &tauri::AppHandle<Wry>, event: MenuEvent) {
        let Ok(menu_id) = MenuId::from_str(event.id().as_ref()) else {
            warn!("Unknown menu event id: {}", event.id().as_ref());
            return;
        };

        match menu_id {
            MenuId::Preferences => {
                if let Err(e) = window::open_preferences_window(app) {
                    error!("Failed to open preferences window: {}", e);
                }
            }
            MenuId::PasteLastRecording => {
                Self::handle_paste_last_recording(app);
            }
            MenuId::Quit => {
                app.exit(0);
            }
        }
    }

    // TODO: Refactor: I do not like this nesting. Will leave it like this for now and will refactor later.
    // Also might need to pull the pasting into a separate module
    fn handle_paste_last_recording(app: &tauri::AppHandle<Wry>) {
        if let Some(state) = app.try_state::<LastRecordingState>() {
            if let Ok(last_recording) = state.lock() {
                if let Some(text) = &last_recording.text {
                    if let Err(e) = crate::text_paster::paste_text(text) {
                        error!("Failed to paste last recording: {:?}", e);
                    }
                }
            } else {
                error!("Failed to lock last recording state");
            }
        }
    }
}
