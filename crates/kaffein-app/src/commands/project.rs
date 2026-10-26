//! Project commands.

use crate::command::{CommandArgs, CommandOutcome, ErrorKind};
use crate::services::Services;
use crate::state::AppState;

use super::helpers::{path_from_args, user_error};

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
