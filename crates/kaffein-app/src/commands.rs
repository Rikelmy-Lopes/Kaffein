use kaffein_domain::Document;

use crate::command::{CommandArgs, CommandOutcome, ErrorKind};
use crate::services::Services;
use crate::state::AppState;
use crate::workspace::{internal_error, path_from_args, target_document, user_error};

pub(crate) fn project_open(
    state: &mut AppState,
    services: &mut Services,
    args: CommandArgs,
) -> CommandOutcome {
    let path = match path_from_args(args) {
        Ok(path) => path,
        Err(outcome) => return outcome,
    };
    if path.as_os_str().is_empty() {
        return user_error("Enter a project folder path.");
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

    let project = match services.projects.load(&canonical) {
        Ok(project) => project,
        Err(error) => {
            return CommandOutcome::failed(
                ErrorKind::UnsupportedState,
                format!(
                    "Cannot read project {}: {error}",
                    crate::display_path(&canonical)
                ),
            );
        }
    };

    let name = project.name().to_owned();
    state.set_project(project);
    CommandOutcome::success(format!("Opened project {name}"))
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

pub(crate) fn view_toggle_project(
    state: &mut AppState,
    _services: &mut Services,
    _args: CommandArgs,
) -> CommandOutcome {
    state.toggle_project_panel();
    CommandOutcome::success_silent()
}

pub(crate) fn view_toggle_bottom(
    state: &mut AppState,
    _services: &mut Services,
    _args: CommandArgs,
) -> CommandOutcome {
    state.toggle_bottom_panel();
    CommandOutcome::success_silent()
}

pub(crate) fn view_toggle_theme(
    state: &mut AppState,
    _services: &mut Services,
    _args: CommandArgs,
) -> CommandOutcome {
    let theme = state.toggle_theme();
    CommandOutcome::success(format!("Theme: {}", theme.label()))
}

pub(crate) fn app_quit(
    state: &mut AppState,
    _services: &mut Services,
    _args: CommandArgs,
) -> CommandOutcome {
    let dirty = state.dirty_document_count();
    if dirty > 0 {
        return CommandOutcome::no_op(format!(
            "{dirty} document(s) have unsaved changes. Save them before quitting."
        ));
    }
    state.request_quit();
    CommandOutcome::success_silent()
}
