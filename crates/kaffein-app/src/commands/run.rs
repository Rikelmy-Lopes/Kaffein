//! Run command: compile and execute the active buffer without saving it.

use kaffein_i18n::{keys, Text};

use crate::command::{CommandArgs, CommandOutcome};
use crate::services::Services;
use crate::state::AppState;

use super::helpers::is_java_document;

pub(crate) fn run_execute(
    state: &mut AppState,
    services: &mut Services,
    _args: CommandArgs,
) -> CommandOutcome {
    let Some(document) = state.active_document() else {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_RUN_NO_DOCUMENT));
    };
    if !is_java_document(document) {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_RUN_NOT_JAVA));
    }
    if document.buffer().trim().is_empty() {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_RUN_EMPTY));
    }

    let source = document.buffer().to_owned();
    let source_name = document.display_name();
    let Some(job) = state.start_run(source_name) else {
        return CommandOutcome::no_op(Text::key(keys::MESSAGE_RUN_ALREADY_RUNNING));
    };

    let runner = services.java.clone();
    std::thread::spawn(move || {
        let result = runner.run_source(&source);
        job.finish(result);
    });

    CommandOutcome::success_silent()
}
