//! File lifecycle commands: new, open, save, close.

use kaffein_domain::Document;
use kaffein_i18n::{keys, Text};

use crate::command::{CommandArgs, CommandOutcome, ErrorKind};
use crate::services::Services;
use crate::state::AppState;

use super::helpers::{fs_error_text, path_from_args, target_document, user_error};

pub(crate) fn file_new_scratch(
    state: &mut AppState,
    _services: &mut Services,
    _args: CommandArgs,
) -> CommandOutcome {
    let id = state.allocate_document_id();
    state.add_document(Document::scratch(id, crate::DEFAULT_JAVA_SOURCE));
    CommandOutcome::success_silent()
}

pub(crate) fn file_new(
    state: &mut AppState,
    services: &mut Services,
    args: CommandArgs,
) -> CommandOutcome {
    let path = match path_from_args(args) {
        Ok(path) => path,
        Err(outcome) => return outcome,
    };
    if path.as_os_str().is_empty() {
        return user_error(Text::key(keys::MESSAGE_FILE_ENTER_PATH));
    }
    let Some(file_name) = path.file_name() else {
        return user_error(Text::key(keys::MESSAGE_FILE_ENTER_NAME));
    };
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let parent = match services.fs.canonicalize(parent) {
        Ok(parent) => parent,
        Err(error) => {
            return user_error(
                Text::message(keys::MESSAGE_FILE_CANNOT_CREATE).arg("error", fs_error_text(&error)),
            );
        }
    };

    let target = parent.join(file_name);
    if services.fs.canonicalize(&target).is_ok() {
        return user_error(
            Text::message(keys::MESSAGE_FILE_EXISTS).arg("path", crate::display_path(&target)),
        );
    }

    let id = state.allocate_document_id();
    let name = file_name.to_string_lossy().into_owned();
    state.add_document(Document::new(id, target));
    CommandOutcome::success(Text::message(keys::MESSAGE_FILE_CREATED).arg("name", name))
}

pub(crate) fn file_open(
    state: &mut AppState,
    services: &mut Services,
    args: CommandArgs,
) -> CommandOutcome {
    let path = match path_from_args(args) {
        Ok(path) => path,
        Err(outcome) => return outcome,
    };
    if path.as_os_str().is_empty() {
        return user_error(Text::key(keys::MESSAGE_FILE_ENTER_PATH));
    }

    let canonical = match services.fs.canonicalize(&path) {
        Ok(path) => path,
        Err(error) => {
            return user_error(
                Text::message(keys::MESSAGE_FILE_CANNOT_OPEN).arg("error", fs_error_text(&error)),
            );
        }
    };

    if let Some(existing) = state.find_document_by_path(&canonical) {
        let id = existing.id();
        let name = existing.display_name();
        state.activate(id);
        return CommandOutcome::success(
            Text::message(keys::MESSAGE_FILE_SWITCHED).arg("name", name),
        );
    }

    let raw = match services.fs.read_text(&canonical) {
        Ok(text) => text,
        Err(error) => {
            return CommandOutcome::failed(
                ErrorKind::UnsupportedState,
                Text::message(keys::MESSAGE_FILE_CANNOT_READ).arg("error", fs_error_text(&error)),
            );
        }
    };

    let id = state.allocate_document_id();
    let index_path = canonical.clone();
    let name = canonical
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| crate::display_path(&canonical));
    state.add_document(Document::from_loaded(id, canonical, &raw));
    super::index_source(state, &index_path, &raw);
    CommandOutcome::success(Text::message(keys::MESSAGE_FILE_OPENED).arg("name", name))
}

pub(crate) fn file_save(
    state: &mut AppState,
    services: &mut Services,
    _args: CommandArgs,
) -> CommandOutcome {
    let Some(id) = state.active_id() else {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_FILE_NO_DOCUMENT));
    };
    let Some(document) = state.document(id) else {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_FILE_NO_DOCUMENT));
    };
    if document.is_scratch() {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_FILE_SCRATCH_SAVE));
    }
    if !document.is_dirty() {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_FILE_NO_CHANGES));
    }

    let Some(path) = document.path().map(std::path::Path::to_path_buf) else {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_FILE_SCRATCH_SAVE));
    };
    let name = document.display_name();
    let text = document.text_for_save();

    if let Err(error) = services.fs.write_text(&path, &text) {
        return CommandOutcome::failed(
            ErrorKind::ToolFailure,
            Text::message(keys::MESSAGE_FILE_CANNOT_SAVE).arg("error", fs_error_text(&error)),
        );
    }

    if let Some(document) = state.document_mut(id) {
        document.mark_saved();
    }
    super::index_source(state, &path, &text);
    CommandOutcome::success(Text::message(keys::MESSAGE_FILE_SAVED).arg("name", name))
}

pub(crate) fn file_close(
    state: &mut AppState,
    _services: &mut Services,
    args: CommandArgs,
) -> CommandOutcome {
    let Some(id) = target_document(state, &args) else {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_FILE_NO_DOCUMENT));
    };
    let Some(document) = state.document(id) else {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_FILE_DOCUMENT_GONE));
    };

    if !document.is_scratch() && document.is_dirty() {
        return CommandOutcome::no_op(
            Text::message(keys::MESSAGE_FILE_UNSAVED_CHANGES).arg("name", document.display_name()),
        );
    }

    let name = document.display_name();
    state.remove_document(id);
    CommandOutcome::success(Text::message(keys::MESSAGE_FILE_CLOSED).arg("name", name))
}
