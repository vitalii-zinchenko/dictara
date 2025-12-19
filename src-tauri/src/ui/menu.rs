use tauri::{App, Wry};

pub struct MenuWithItems {
    pub menu: tauri::menu::Menu<Wry>,
    pub paste_last_item: tauri::menu::MenuItem<Wry>,
}

pub fn build_menu(app: &App<Wry>) -> Result<MenuWithItems, Box<dyn std::error::Error>> {
    // Build menu items
    let preferences_item =
        tauri::menu::MenuItemBuilder::with_id("preferences", "Preferences").build(app)?;
    let paste_last_item =
        tauri::menu::MenuItemBuilder::with_id("paste_last_recording", "Paste Last Recording")
            .enabled(false) // Initially disabled until first recording
            .build(app)?;
    let quit_item = tauri::menu::MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    // Build menu
    let menu = tauri::menu::MenuBuilder::new(app)
        .item(&preferences_item)
        .item(&paste_last_item)
        .separator()
        .item(&quit_item)
        .build()?;

    Ok(MenuWithItems {
        menu,
        paste_last_item,
    })
}
