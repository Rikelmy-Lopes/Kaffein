//! Completion commands: compute suggestions, move the selection, accept, dismiss.

use kaffein_i18n::{keys, Text};

use crate::command::{CommandArgs, CommandOutcome, ErrorKind};
use crate::services::Services;
use crate::state::{AppState, CompletionSession};

use super::helpers::internal_error;

pub(crate) fn editor_complete(
    state: &mut AppState,
    _services: &mut Services,
    args: CommandArgs,
) -> CommandOutcome {
    let CommandArgs::Complete { document, caret } = args else {
        return internal_error(Text::key(keys::MESSAGE_COMPLETION_INVALID_ARGS));
    };
    if state.active_id() != Some(document) {
        state.clear_completion();
        return CommandOutcome::success_silent();
    }
    let Some(source) = state.document(document).map(|doc| doc.buffer().to_owned()) else {
        state.clear_completion();
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_COMPLETION_DOCUMENT_GONE));
    };
    let Some(completion) = kaffein_domain::complete(&source, caret, state.index()) else {
        state.clear_completion();
        return CommandOutcome::success_silent();
    };
    state.set_completion(CompletionSession::new(document, completion));
    CommandOutcome::success_silent()
}

pub(crate) fn editor_complete_select(
    state: &mut AppState,
    _services: &mut Services,
    args: CommandArgs,
) -> CommandOutcome {
    let CommandArgs::CompleteSelect { delta } = args else {
        return internal_error(Text::key(keys::MESSAGE_COMPLETION_INVALID_ARGS));
    };
    if let Some(session) = state.completion_mut() {
        session.select_delta(delta);
    }
    CommandOutcome::success_silent()
}

pub(crate) fn editor_complete_accept(
    state: &mut AppState,
    _services: &mut Services,
    args: CommandArgs,
) -> CommandOutcome {
    let CommandArgs::CompleteAccept {
        document,
        at_ms,
        item,
    } = args
    else {
        return internal_error(Text::key(keys::MESSAGE_COMPLETION_INVALID_ARGS));
    };
    let choice = state.completion().and_then(|session| {
        if session.document != document {
            return None;
        }
        let index = item.unwrap_or(session.selected);
        session
            .items
            .get(index)
            .map(|item| (session.start, session.end, item.insert_text.clone()))
    });
    let Some((start, end, insert)) = choice else {
        state.clear_completion();
        return CommandOutcome::success_silent();
    };
    let Some(target) = state.document_mut(document) else {
        state.clear_completion();
        return CommandOutcome::failed(
            ErrorKind::UnsupportedState,
            Text::key(keys::MESSAGE_COMPLETION_DOCUMENT_GONE),
        );
    };
    let mut text = target.buffer().to_owned();
    if start > end
        || end > text.len()
        || !text.is_char_boundary(start)
        || !text.is_char_boundary(end)
    {
        state.clear_completion();
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_COMPLETION_STALE));
    }
    text.replace_range(start..end, &insert);
    target.replace_text(text, at_ms);
    state.clear_completion();
    CommandOutcome::success_silent()
}

pub(crate) fn editor_complete_dismiss(
    state: &mut AppState,
    _services: &mut Services,
    _args: CommandArgs,
) -> CommandOutcome {
    state.clear_completion();
    CommandOutcome::success_silent()
}
