//! Where a command appears in the menu bar and toolbar.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Menu {
    File,
    Edit,
    View,
}

impl Menu {
    pub fn title(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::View => "View",
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
    pub label: &'static str,
}
