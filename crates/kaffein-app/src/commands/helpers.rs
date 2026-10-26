//! Small adapters shared by command implementations.
//!
//! These do not belong in the workspace: they translate command arguments into
//! domain values and build structured outcomes. A command-specific helper
//! belongs in the module that owns the command.

use crate::command::{CommandArgs, CommandOutcome, ErrorKind};
use crate::state::AppState;

pub(crate) fn target_document(
    state: &AppState,
    args: &CommandArgs,
) -> Option<kaffein_domain::DocumentId> {
    match args {
        CommandArgs::Document(id) => Some(*id),
        _ => state.active_id(),
    }
}

pub(crate) fn internal_error(message: impl Into<String>) -> CommandOutcome {
    CommandOutcome::failed(ErrorKind::Internal, message)
}

pub(crate) fn user_error(message: impl Into<String>) -> CommandOutcome {
    CommandOutcome::failed(ErrorKind::UserInput, message)
}

pub(crate) fn path_from_args(args: CommandArgs) -> Result<std::path::PathBuf, CommandOutcome> {
    match args {
        CommandArgs::Path(path) => Ok(path),
        _ => Err(internal_error("This command requires a path argument")),
    }
}
