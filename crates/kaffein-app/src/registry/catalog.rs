//! The command table. One entry per executable action.
//!
//! Menus, the toolbar, shortcuts, tests, and future automation read this table;
//! adding a command means adding a row here, not editing four lists. Titles and
//! labels are localization keys, not display text.

use kaffein_i18n::keys;

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
            title_key: keys::COMMAND_PROJECT_OPEN_TITLE,
            category_key: keys::CATEGORY_PROJECT,
            menu: Some(MenuPlacement {
                menu: Menu::File,
                order: 10,
                separator_before: false,
            }),
            toolbar: Some(ToolbarPlacement {
                order: 10,
                label_key: keys::COMMAND_PROJECT_OPEN_TOOLBAR,
            }),
            shortcuts: &[],
            args: ArgKind::Directory,
            can_execute: always,
            checked: None,
            run: commands::project_open,
        },
        Command {
            id: command_id::FILE_NEW,
            title_key: keys::COMMAND_FILE_NEW_TITLE,
            category_key: keys::CATEGORY_FILE,
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
            title_key: keys::COMMAND_FILE_OPEN_TITLE,
            category_key: keys::CATEGORY_FILE,
            menu: Some(MenuPlacement {
                menu: Menu::File,
                order: 20,
                separator_before: false,
            }),
            toolbar: Some(ToolbarPlacement {
                order: 20,
                label_key: keys::COMMAND_FILE_OPEN_TOOLBAR,
            }),
            shortcuts: OPEN_SHORTCUTS,
            args: ArgKind::File,
            can_execute: always,
            checked: None,
            run: commands::file_open,
        },
        Command {
            id: command_id::FILE_SAVE,
            title_key: keys::COMMAND_FILE_SAVE_TITLE,
            category_key: keys::CATEGORY_FILE,
            menu: Some(MenuPlacement {
                menu: Menu::File,
                order: 30,
                separator_before: false,
            }),
            toolbar: Some(ToolbarPlacement {
                order: 30,
                label_key: keys::COMMAND_FILE_SAVE_TOOLBAR,
            }),
            shortcuts: SAVE_SHORTCUTS,
            args: ArgKind::None,
            can_execute: has_document,
            checked: None,
            run: commands::file_save,
        },
        Command {
            id: command_id::FILE_CLOSE,
            title_key: keys::COMMAND_FILE_CLOSE_TITLE,
            category_key: keys::CATEGORY_FILE,
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
            title_key: keys::COMMAND_EDIT_UNDO_TITLE,
            category_key: keys::CATEGORY_EDIT,
            menu: Some(MenuPlacement {
                menu: Menu::Edit,
                order: 10,
                separator_before: false,
            }),
            toolbar: Some(ToolbarPlacement {
                order: 40,
                label_key: keys::COMMAND_EDIT_UNDO_TOOLBAR,
            }),
            shortcuts: UNDO_SHORTCUTS,
            args: ArgKind::None,
            can_execute: can_undo,
            checked: None,
            run: commands::edit_undo,
        },
        Command {
            id: command_id::EDIT_REDO,
            title_key: keys::COMMAND_EDIT_REDO_TITLE,
            category_key: keys::CATEGORY_EDIT,
            menu: Some(MenuPlacement {
                menu: Menu::Edit,
                order: 20,
                separator_before: false,
            }),
            toolbar: Some(ToolbarPlacement {
                order: 50,
                label_key: keys::COMMAND_EDIT_REDO_TOOLBAR,
            }),
            shortcuts: REDO_SHORTCUTS,
            args: ArgKind::None,
            can_execute: can_redo,
            checked: None,
            run: commands::edit_redo,
        },
        Command {
            id: command_id::VIEW_TOGGLE_PROJECT,
            title_key: keys::COMMAND_VIEW_TOGGLE_PROJECT_TITLE,
            category_key: keys::CATEGORY_VIEW,
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
            title_key: keys::COMMAND_VIEW_TOGGLE_BOTTOM_TITLE,
            category_key: keys::CATEGORY_VIEW,
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
            title_key: keys::COMMAND_VIEW_TOGGLE_THEME_TITLE,
            category_key: keys::CATEGORY_VIEW,
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
            title_key: keys::COMMAND_APP_QUIT_TITLE,
            category_key: keys::CATEGORY_APPLICATION,
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
        Command {
            id: command_id::APP_SET_LOCALE,
            title_key: keys::COMMAND_APP_SET_LOCALE_TITLE,
            category_key: keys::CATEGORY_APPLICATION,
            menu: None,
            toolbar: None,
            shortcuts: &[],
            args: ArgKind::Locale,
            can_execute: always,
            checked: None,
            run: commands::app_set_locale,
        },
        // Internal commands: registered so tests and future automation use the
        // same implementation, but not shown in any menu.
        Command {
            id: command_id::EDITOR_REPLACE_TEXT,
            title_key: keys::COMMAND_EDITOR_REPLACE_TEXT_TITLE,
            category_key: keys::CATEGORY_EDITOR,
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
            title_key: keys::COMMAND_EDITOR_ACTIVATE_TITLE,
            category_key: keys::CATEGORY_EDITOR,
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
