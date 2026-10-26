//! Application lifecycle commands.

use crate::command::{CommandArgs, CommandOutcome};
use crate::services::Services;
use crate::state::AppState;

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
