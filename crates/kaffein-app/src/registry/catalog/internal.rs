//! Internal commands dispatched by the UI or by tests, never shown in a menu.

use kaffein_i18n::keys;

use crate::command::{command_id, ArgKind, Command};
use crate::commands;

use super::super::enablers::always;

pub(super) fn commands() -> Vec<Command> {
    vec![
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
        Command {
            id: command_id::EDITOR_COMPLETE,
            title_key: keys::COMMAND_EDITOR_COMPLETE_TITLE,
            category_key: keys::CATEGORY_EDITOR,
            menu: None,
            toolbar: None,
            shortcuts: &[],
            args: ArgKind::None,
            can_execute: always,
            checked: None,
            run: commands::editor_complete,
        },
        Command {
            id: command_id::EDITOR_COMPLETE_SELECT,
            title_key: keys::COMMAND_EDITOR_COMPLETE_SELECT_TITLE,
            category_key: keys::CATEGORY_EDITOR,
            menu: None,
            toolbar: None,
            shortcuts: &[],
            args: ArgKind::None,
            can_execute: always,
            checked: None,
            run: commands::editor_complete_select,
        },
        Command {
            id: command_id::EDITOR_COMPLETE_ACCEPT,
            title_key: keys::COMMAND_EDITOR_COMPLETE_ACCEPT_TITLE,
            category_key: keys::CATEGORY_EDITOR,
            menu: None,
            toolbar: None,
            shortcuts: &[],
            args: ArgKind::None,
            can_execute: always,
            checked: None,
            run: commands::editor_complete_accept,
        },
        Command {
            id: command_id::EDITOR_COMPLETE_DISMISS,
            title_key: keys::COMMAND_EDITOR_COMPLETE_DISMISS_TITLE,
            category_key: keys::CATEGORY_EDITOR,
            menu: None,
            toolbar: None,
            shortcuts: &[],
            args: ArgKind::None,
            can_execute: always,
            checked: None,
            run: commands::editor_complete_dismiss,
        },
    ]
}
