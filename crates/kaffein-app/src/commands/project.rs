//! Project commands.

use kaffein_i18n::{keys, Text};

use crate::command::{CommandArgs, CommandOutcome, ErrorKind};
use crate::services::Services;
use crate::state::AppState;

use super::helpers::{fs_error_text, path_from_args, user_error};

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
        return user_error(Text::key(keys::MESSAGE_PROJECT_ENTER_PATH));
    }

    let canonical = match services.fs.canonicalize(&path) {
        Ok(path) => path,
        Err(error) => {
            return user_error(
                Text::message(keys::MESSAGE_PROJECT_CANNOT_OPEN)
                    .arg("error", fs_error_text(&error)),
            );
        }
    };

    let project = match services.projects.load(&canonical) {
        Ok(project) => project,
        Err(error) => {
            return CommandOutcome::failed(
                ErrorKind::UnsupportedState,
                Text::message(keys::MESSAGE_PROJECT_CANNOT_READ)
                    .arg("error", fs_error_text(&error)),
            );
        }
    };

    let name = project.name().to_owned();
    state.set_project(project);
    CommandOutcome::success(Text::message(keys::MESSAGE_PROJECT_OPENED).arg("name", name))
}
