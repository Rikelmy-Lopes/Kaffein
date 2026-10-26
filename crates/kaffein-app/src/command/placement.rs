//! Where a command appears in the menu bar and toolbar.

use kaffein_i18n::keys;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Menu {
    File,
    Edit,
    View,
}

impl Menu {
    pub fn title_key(self) -> &'static str {
        match self {
            Self::File => keys::MENU_FILE,
            Self::Edit => keys::MENU_EDIT,
            Self::View => keys::MENU_VIEW,
        }
    }

    pub const ALL: [Menu; 3] = [Self::File, Self::Edit, Self::View];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuPlacement {
    pub menu: Menu,
    pub order: u16,
    pub separator_before: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolbarPlacement {
    pub order: u16,
    /// Localization key for the toolbar button label.
    pub label_key: &'static str,
}
