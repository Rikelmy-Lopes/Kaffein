//! Small adapters shared by command implementations.
//!
//! These do not belong in the workspace: they translate command arguments into
//! domain values, turn infrastructure errors into localized text, and build
//! structured outcomes. A command-specific helper belongs in the module that
//! owns the command.

use kaffein_domain::{Document, FsError, FsErrorKind};
use kaffein_i18n::{keys, Text};

use crate::command::{CommandArgs, CommandOutcome, ErrorKind};
use crate::state::AppState;

pub(crate) fn is_java_document(document: &Document) -> bool {
    document.path().is_none()
        || document
            .path()
            .and_then(std::path::Path::extension)
            .is_some_and(|extension| extension.to_string_lossy().eq_ignore_ascii_case("java"))
}

pub(crate) fn target_document(
    state: &AppState,
    args: &CommandArgs,
) -> Option<kaffein_domain::DocumentId> {
    match args {
        CommandArgs::Document(id) => Some(*id),
        _ => state.active_id(),
    }
}

pub(crate) fn internal_error(message: impl Into<Text>) -> CommandOutcome {
    CommandOutcome::failed(ErrorKind::Internal, message)
}

pub(crate) fn user_error(message: impl Into<Text>) -> CommandOutcome {
    CommandOutcome::failed(ErrorKind::UserInput, message)
}

pub(crate) fn path_from_args(args: CommandArgs) -> Result<std::path::PathBuf, CommandOutcome> {
    match args {
        CommandArgs::Path(path) => Ok(path),
        _ => Err(internal_error(Text::key(keys::MESSAGE_PATH_REQUIRED))),
    }
}

/// Turns a structured filesystem error into deferred, localized text. The
/// error already names the path, so command messages add only the action.
pub(crate) fn fs_error_text(error: &FsError) -> Text {
    let path = crate::display_path(error.path());
    match error.kind() {
        FsErrorKind::Io | FsErrorKind::Other => Text::message(keys::INFRA_IO)
            .arg("path", path)
            .arg("error", error.detail().unwrap_or_default()),
        FsErrorKind::NotAFile => Text::message(keys::INFRA_NOT_A_FILE).arg("path", path),
        FsErrorKind::FileTooLarge => Text::message(keys::INFRA_FILE_TOO_LARGE)
            .arg("path", path)
            .arg("size", error.size().unwrap_or_default())
            .arg("limit", error.limit().unwrap_or_default()),
        FsErrorKind::InvalidUtf8 => Text::message(keys::INFRA_INVALID_UTF8).arg("path", path),
        FsErrorKind::WriteTooLarge => Text::message(keys::INFRA_WRITE_TOO_LARGE)
            .arg("path", path)
            .arg("limit", error.limit().unwrap_or_default()),
        FsErrorKind::NotADirectory => Text::message(keys::INFRA_NOT_A_DIRECTORY).arg("path", path),
    }
}
