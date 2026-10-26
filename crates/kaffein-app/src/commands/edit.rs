//! Edit commands: undo, redo, and the editor's internal text commands.

use kaffein_i18n::{keys, Text};

use crate::command::{CommandArgs, CommandOutcome, ErrorKind};
use crate::services::Services;
use crate::state::AppState;

use super::helpers::internal_error;

pub(crate) fn edit_undo(
    state: &mut AppState,
    _services: &mut Services,
    _args: CommandArgs,
) -> CommandOutcome {
    let Some(id) = state.active_id() else {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_EDIT_NO_DOCUMENT));
    };
    let Some(document) = state.document_mut(id) else {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_EDIT_NO_DOCUMENT));
    };
    if document.undo() {
        CommandOutcome::success_silent()
    } else {
        CommandOutcome::no_op(Text::key(keys::MESSAGE_EDIT_NOTHING_TO_UNDO))
    }
}

pub(crate) fn edit_redo(
    state: &mut AppState,
    _services: &mut Services,
    _args: CommandArgs,
) -> CommandOutcome {
    let Some(id) = state.active_id() else {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_EDIT_NO_DOCUMENT));
    };
    let Some(document) = state.document_mut(id) else {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_EDIT_NO_DOCUMENT));
    };
    if document.redo() {
        CommandOutcome::success_silent()
    } else {
        CommandOutcome::no_op(Text::key(keys::MESSAGE_EDIT_NOTHING_TO_REDO))
    }
}

pub(crate) fn editor_replace_text(
    state: &mut AppState,
    _services: &mut Services,
    args: CommandArgs,
) -> CommandOutcome {
    let CommandArgs::ReplaceText {
        document,
        text,
        at_ms,
    } = args
    else {
        return internal_error(Text::key(keys::MESSAGE_EDIT_REPLACE_REQUIRES_TEXT));
    };

    let Some(target) = state.document_mut(document) else {
        return CommandOutcome::failed(
            ErrorKind::UnsupportedState,
            Text::key(keys::MESSAGE_EDIT_DOCUMENT_GONE),
        );
    };

    target.replace_text(text, at_ms);
    CommandOutcome::success_silent()
}

pub(crate) fn editor_activate(
    state: &mut AppState,
    _services: &mut Services,
    args: CommandArgs,
) -> CommandOutcome {
    let CommandArgs::Document(id) = args else {
        return internal_error(Text::key(keys::MESSAGE_EDIT_ACTIVATE_REQUIRES_DOCUMENT));
    };
    if state.activate(id) {
        CommandOutcome::success_silent()
    } else {
        CommandOutcome::failed(
            ErrorKind::UnsupportedState,
            Text::key(keys::MESSAGE_EDIT_DOCUMENT_GONE),
        )
    }
}
