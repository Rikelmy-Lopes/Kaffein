//! The command table. One entry per executable action.
//!
//! Menus, the toolbar, shortcuts, tests, and future automation read this table;
//! adding a command means adding a row here, not editing four lists.

use crate::command::{
    command_id, ArgKind, Command, Key, KeyBinding, Menu, MenuPlacement, ToolbarPlacement,
};
use crate::commands;

use super::enablers::{
    always, bottom_panel_visible, can_redo, can_undo, dark_theme, has_document,
    project_panel_visible,
};

const NEW_SHORTCUTS: &[KeyBinding] = &[KeyBinding::ctrl(Key::N)];
const OPEN_SHORTCUTS: &[KeyBinding] = &[KeyBinding::ctrl(Key::O)];
const SAVE_SHORTCUTS: &[KeyBinding] = &[KeyBinding::ctrl(Key::S)];
const UNDO_SHORTCUTS: &[KeyBinding] = &[KeyBinding::ctrl(Key::Z)];
const REDO_SHORTCUTS: &[KeyBinding] = &[KeyBinding::ctrl_shift(Key::Z), KeyBinding::ctrl(Key::Y)];
const QUIT_SHORTCUTS: &[KeyBinding] = &[KeyBinding::ctrl(Key::Q)];

pub(super) fn commands() -> Vec<Command> {
    vec![
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
        // Internal commands: registered so tests and future automation use the
        // same implementation, but not shown in any menu.
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
    ]
}
