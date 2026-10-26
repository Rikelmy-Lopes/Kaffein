//! Edit commands: undo, redo, and the editor's internal text commands.

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
        return CommandOutcome::no_op("No document is open");
    };
    let Some(document) = state.document_mut(id) else {
        return CommandOutcome::no_op("No document is open");
    };
    if document.undo() {
        CommandOutcome::success_silent()
    } else {
        CommandOutcome::no_op("Nothing to undo")
    }
}

pub(crate) fn edit_redo(
    state: &mut AppState,
    _services: &mut Services,
    _args: CommandArgs,
) -> CommandOutcome {
    let Some(id) = state.active_id() else {
        return CommandOutcome::no_op("No document is open");
    };
    let Some(document) = state.document_mut(id) else {
        return CommandOutcome::no_op("No document is open");
    };
    if document.redo() {
        CommandOutcome::success_silent()
    } else {
        CommandOutcome::no_op("Nothing to redo")
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
        return internal_error("editor.replace_text requires text arguments");
    };

    let Some(target) = state.document_mut(document) else {
        return CommandOutcome::failed(
            ErrorKind::UnsupportedState,
            "The document is no longer open",
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
        return internal_error("editor.activate requires a document argument");
    };
    if state.activate(id) {
        CommandOutcome::success_silent()
    } else {
        CommandOutcome::failed(
            ErrorKind::UnsupportedState,
            "The document is no longer open",
        )
    }
}
