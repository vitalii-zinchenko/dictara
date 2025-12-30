use tauri::{self, menu, Wry};

#[derive(strum::AsRefStr, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum MenuId {
    Preferences,
    PasteLastRecording,
    Quit,
}

pub struct Menu {
    pub menu: menu::Menu<Wry>,
    paste_last_item: menu::MenuItem<Wry>,
}

impl Menu {
    pub fn new(app: &tauri::App<tauri::Wry>) -> Result<Menu, tauri::Error> {
        let preferences_item = Self::create_preferences_item(app)?;
        let paste_last_item = Self::create_paste_last_item(app)?;
        let quit_item = Self::create_quit_item(app)?;

        let menu = Self::create_menu(app, &[&preferences_item, &paste_last_item, &quit_item])?;

        Ok(Menu {
            menu,
            paste_last_item,
        })
    }

    pub fn set_paste_last_active(&self) -> Result<(), tauri::Error> {
        self.paste_last_item.set_enabled(true)
    }

    pub fn set_paste_last_inactive(&self) -> Result<(), tauri::Error> {
        self.paste_last_item.set_enabled(false)
    }

    fn create_preferences_item(
        app: &tauri::App<tauri::Wry>,
    ) -> Result<menu::MenuItem<Wry>, tauri::Error> {
        menu::MenuItemBuilder::with_id(MenuId::Preferences.as_ref(), "Preferences").build(app)
    }

    fn create_paste_last_item(
        app: &tauri::App<tauri::Wry>,
    ) -> Result<menu::MenuItem<Wry>, tauri::Error> {
        menu::MenuItemBuilder::with_id(MenuId::PasteLastRecording.as_ref(), "Paste Last Recording")
            .enabled(false) // Initially disabled until first recording
            .build(app)
    }

    fn create_quit_item(app: &tauri::App<tauri::Wry>) -> Result<menu::MenuItem<Wry>, tauri::Error> {
        menu::MenuItemBuilder::with_id(MenuId::Quit.as_ref(), "Quit").build(app)
    }

    /// Creates the menu with a separator before the last item
    fn create_menu(
        app: &tauri::App<tauri::Wry>,
        items: &[&menu::MenuItem<Wry>],
    ) -> Result<menu::Menu<Wry>, tauri::Error> {
        let mut builder = menu::MenuBuilder::new(app);

        let len = items.len();
        for (i, item) in items.iter().enumerate() {
            if i == len - 1 && len > 1 {
                builder = builder.separator();
            }
            builder = builder.item(*item);
        }

        builder.build()
    }
}
