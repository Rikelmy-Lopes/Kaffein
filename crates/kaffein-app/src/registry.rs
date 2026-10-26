use crate::command::{
    command_id, ArgKind, Command, Enablement, Key, KeyBinding, Menu, MenuPlacement,
    ToolbarPlacement,
};
use crate::commands;
use crate::state::AppState;

const NO_DOCUMENT: &str = "No document is open";
const NEW_SHORTCUTS: &[KeyBinding] = &[KeyBinding::ctrl(Key::N)];
const OPEN_SHORTCUTS: &[KeyBinding] = &[KeyBinding::ctrl(Key::O)];
const SAVE_SHORTCUTS: &[KeyBinding] = &[KeyBinding::ctrl(Key::S)];
const UNDO_SHORTCUTS: &[KeyBinding] = &[KeyBinding::ctrl(Key::Z)];
const REDO_SHORTCUTS: &[KeyBinding] = &[KeyBinding::ctrl_shift(Key::Z), KeyBinding::ctrl(Key::Y)];
const QUIT_SHORTCUTS: &[KeyBinding] = &[KeyBinding::ctrl(Key::Q)];

fn always(_: &AppState) -> Enablement {
    Enablement::Enabled
}

fn has_document(state: &AppState) -> Enablement {
    if state.active_document().is_some() {
        Enablement::Enabled
    } else {
        Enablement::Disabled(NO_DOCUMENT)
    }
}

fn can_undo(state: &AppState) -> Enablement {
    match state.active_document() {
        Some(document) if document.can_undo() => Enablement::Enabled,
        Some(_) => Enablement::Disabled("Nothing to undo"),
        None => Enablement::Disabled(NO_DOCUMENT),
    }
}

fn can_redo(state: &AppState) -> Enablement {
    match state.active_document() {
        Some(document) if document.can_redo() => Enablement::Enabled,
        Some(_) => Enablement::Disabled("Nothing to redo"),
        None => Enablement::Disabled(NO_DOCUMENT),
    }
}

fn project_panel_visible(state: &AppState) -> bool {
    state.show_project_panel()
}

fn bottom_panel_visible(state: &AppState) -> bool {
    state.show_bottom_panel()
}

fn dark_theme(state: &AppState) -> bool {
    state.theme() == crate::state::ThemePreference::Dark
}

pub struct CommandRegistry {
    commands: Vec<Command>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: vec![
                Command {
                    id: command_id::PROJECT_OPEN,
                    title: "Open Project…",
                    category: "Project",
                    menu: Some(MenuPlacement {
                        menu: Menu::File,
                        order: 10,
                        separator_before: false,
                    }),
                    toolbar: Some(ToolbarPlacement {
                        order: 10,
                        label: "Project",
                    }),
                    shortcuts: &[],
                    args: ArgKind::Directory,
                    can_execute: always,
                    checked: None,
                    run: commands::project_open,
                },
                Command {
                    id: command_id::FILE_NEW,
                    title: "New File…",
                    category: "File",
                    menu: Some(MenuPlacement {
                        menu: Menu::File,
                        order: 5,
                        separator_before: false,
                    }),
                    toolbar: None,
                    shortcuts: NEW_SHORTCUTS,
                    args: ArgKind::NewFile,
                    can_execute: always,
                    checked: None,
                    run: commands::file_new,
                },
                Command {
                    id: command_id::FILE_OPEN,
                    title: "Open File…",
                    category: "File",
                    menu: Some(MenuPlacement {
                        menu: Menu::File,
                        order: 20,
                        separator_before: false,
                    }),
                    toolbar: Some(ToolbarPlacement {
                        order: 20,
                        label: "Open",
                    }),
                    shortcuts: OPEN_SHORTCUTS,
                    args: ArgKind::File,
                    can_execute: always,
                    checked: None,
                    run: commands::file_open,
                },
                Command {
                    id: command_id::FILE_SAVE,
                    title: "Save",
                    category: "File",
                    menu: Some(MenuPlacement {
                        menu: Menu::File,
                        order: 30,
                        separator_before: false,
                    }),
                    toolbar: Some(ToolbarPlacement {
                        order: 30,
                        label: "Save",
                    }),
                    shortcuts: SAVE_SHORTCUTS,
                    args: ArgKind::None,
                    can_execute: has_document,
                    checked: None,
                    run: commands::file_save,
                },
                Command {
                    id: command_id::FILE_CLOSE,
                    title: "Close File",
                    category: "File",
                    menu: Some(MenuPlacement {
                        menu: Menu::File,
                        order: 40,
                        separator_before: true,
                    }),
                    toolbar: None,
                    shortcuts: &[],
                    args: ArgKind::None,
                    can_execute: has_document,
                    checked: None,
                    run: commands::file_close,
                },
                Command {
                    id: command_id::EDIT_UNDO,
                    title: "Undo",
                    category: "Edit",
                    menu: Some(MenuPlacement {
                        menu: Menu::Edit,
                        order: 10,
                        separator_before: false,
                    }),
                    toolbar: Some(ToolbarPlacement {
                        order: 40,
                        label: "Undo",
                    }),
                    shortcuts: UNDO_SHORTCUTS,
                    args: ArgKind::None,
                    can_execute: can_undo,
                    checked: None,
                    run: commands::edit_undo,
                },
                Command {
                    id: command_id::EDIT_REDO,
                    title: "Redo",
                    category: "Edit",
                    menu: Some(MenuPlacement {
                        menu: Menu::Edit,
                        order: 20,
                        separator_before: false,
                    }),
                    toolbar: Some(ToolbarPlacement {
                        order: 50,
                        label: "Redo",
                    }),
                    shortcuts: REDO_SHORTCUTS,
                    args: ArgKind::None,
                    can_execute: can_redo,
                    checked: None,
                    run: commands::edit_redo,
                },
                Command {
                    id: command_id::VIEW_TOGGLE_PROJECT,
                    title: "Project Panel",
                    category: "View",
                    menu: Some(MenuPlacement {
                        menu: Menu::View,
                        order: 10,
                        separator_before: false,
                    }),
                    toolbar: None,
                    shortcuts: &[],
                    args: ArgKind::None,
                    can_execute: always,
                    checked: Some(project_panel_visible),
                    run: commands::view_toggle_project,
                },
                Command {
                    id: command_id::VIEW_TOGGLE_BOTTOM,
                    title: "Bottom Panel",
                    category: "View",
                    menu: Some(MenuPlacement {
                        menu: Menu::View,
                        order: 20,
                        separator_before: false,
                    }),
                    toolbar: None,
                    shortcuts: &[],
                    args: ArgKind::None,
                    can_execute: always,
                    checked: Some(bottom_panel_visible),
                    run: commands::view_toggle_bottom,
                },
                Command {
                    id: command_id::VIEW_TOGGLE_THEME,
                    title: "Dark Theme",
                    category: "View",
                    menu: Some(MenuPlacement {
                        menu: Menu::View,
                        order: 30,
                        separator_before: true,
                    }),
                    toolbar: None,
                    shortcuts: &[],
                    args: ArgKind::None,
                    can_execute: always,
                    checked: Some(dark_theme),
                    run: commands::view_toggle_theme,
                },
                Command {
                    id: command_id::APP_QUIT,
                    title: "Quit",
                    category: "Application",
                    menu: Some(MenuPlacement {
                        menu: Menu::File,
                        order: 100,
                        separator_before: true,
                    }),
                    toolbar: None,
                    shortcuts: QUIT_SHORTCUTS,
                    args: ArgKind::None,
                    can_execute: always,
                    checked: None,
                    run: commands::app_quit,
                },
                // Internal commands: registered so tests and future automation
                // use the same implementation, but not shown in any menu.
                Command {
                    id: command_id::EDITOR_REPLACE_TEXT,
                    title: "Replace Document Text",
                    category: "Editor",
                    menu: None,
                    toolbar: None,
                    shortcuts: &[],
                    args: ArgKind::Text,
                    can_execute: always,
                    checked: None,
                    run: commands::editor_replace_text,
                },
                Command {
                    id: command_id::EDITOR_ACTIVATE,
                    title: "Activate Document",
                    category: "Editor",
                    menu: None,
                    toolbar: None,
                    shortcuts: &[],
                    args: ArgKind::None,
                    can_execute: always,
                    checked: None,
                    run: commands::editor_activate,
                },
            ],
        }
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn get(&self, id: &str) -> Option<Command> {
        self.commands
            .iter()
            .copied()
            .find(|command| command.id == id)
    }

    pub fn menu_commands(&self, menu: Menu) -> Vec<Command> {
        let mut commands: Vec<Command> = self
            .commands
            .iter()
            .copied()
            .filter(|command| command.menu.is_some_and(|placement| placement.menu == menu))
            .collect();
        commands.sort_by_key(|command| command.menu.map(|placement| placement.order));
        commands
    }

    pub fn toolbar_commands(&self) -> Vec<Command> {
        let mut commands: Vec<Command> = self
            .commands
            .iter()
            .copied()
            .filter(|command| command.toolbar.is_some())
            .collect();
        commands.sort_by_key(|command| command.toolbar.map(|placement| placement.order));
        commands
    }

    pub fn with_shortcuts(&self) -> impl Iterator<Item = Command> + '_ {
        self.commands
            .iter()
            .copied()
            .filter(|command| !command.shortcuts.is_empty())
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
