//! File lifecycle commands: new, open, save, close.

use kaffein_domain::Document;

use crate::command::{CommandArgs, CommandOutcome, ErrorKind};
use crate::services::Services;
use crate::state::AppState;

use super::helpers::{path_from_args, target_document, user_error};

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
        return user_error("Enter a file path.");
    }
    let Some(file_name) = path.file_name() else {
        return user_error("Enter a file name, not a folder.");
    };
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let parent = match services.fs.canonicalize(parent) {
        Ok(parent) => parent,
        Err(error) => {
            return user_error(format!(
                "Cannot create a file in {}: {error}",
                crate::display_path(parent)
            ));
        }
    };

    let target = parent.join(file_name);
    if services.fs.canonicalize(&target).is_ok() {
        return user_error(format!(
            "{} already exists. Open it instead.",
            crate::display_path(&target)
        ));
    }

    let id = state.allocate_document_id();
    let name = file_name.to_string_lossy().into_owned();
    state.add_document(Document::new(id, target));
    CommandOutcome::success(format!("Created {name} (unsaved)"))
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
        return user_error("Enter a file path.");
    }

    let canonical = match services.fs.canonicalize(&path) {
        Ok(path) => path,
        Err(error) => {
            return user_error(format!(
                "Cannot open {}: {error}",
                crate::display_path(&path)
            ));
        }
    };

    if let Some(existing) = state.find_document_by_path(&canonical) {
        let id = existing.id();
        let name = existing.display_name();
        state.activate(id);
        return CommandOutcome::success(format!("Switched to {name}"));
    }

    let raw = match services.fs.read_text(&canonical) {
        Ok(text) => text,
        Err(error) => {
            return CommandOutcome::failed(
                ErrorKind::UnsupportedState,
                format!("Cannot read {}: {error}", crate::display_path(&canonical)),
            );
        }
    };

    let id = state.allocate_document_id();
    let name = canonical
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| crate::display_path(&canonical));
    state.add_document(Document::from_loaded(id, canonical, &raw));
    CommandOutcome::success(format!("Opened {name}"))
}

pub(crate) fn file_save(
    state: &mut AppState,
    services: &mut Services,
    _args: CommandArgs,
) -> CommandOutcome {
    let Some(id) = state.active_id() else {
        return CommandOutcome::no_op("No document is open");
    };
    let Some(document) = state.document(id) else {
        return CommandOutcome::no_op("No document is open");
    };
    if !document.is_dirty() {
        return CommandOutcome::no_op("No changes to save");
    }

    let path = document.path().to_path_buf();
    let name = document.display_name();
    let text = document.text_for_save();

    if let Err(error) = services.fs.write_text(&path, &text) {
        return CommandOutcome::failed(
            ErrorKind::ToolFailure,
            format!("Cannot save {}: {error}", crate::display_path(&path)),
        );
    }

    if let Some(document) = state.document_mut(id) {
        document.mark_saved();
    }
    CommandOutcome::success(format!("Saved {name}"))
}

pub(crate) fn file_close(
    state: &mut AppState,
    _services: &mut Services,
    args: CommandArgs,
) -> CommandOutcome {
    let Some(id) = target_document(state, &args) else {
        return CommandOutcome::no_op("No document is open");
    };
    let Some(document) = state.document(id) else {
        return CommandOutcome::no_op("Document is no longer open");
    };

    if document.is_dirty() {
        return CommandOutcome::no_op(format!(
            "{} has unsaved changes. Save it before closing.",
            document.display_name()
        ));
    }

    let name = document.display_name();
    state.remove_document(id);
    CommandOutcome::success(format!("Closed {name}"))
}
