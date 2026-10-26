//! View commands: panel visibility and theme preference.

use crate::command::{CommandArgs, CommandOutcome};
use crate::services::Services;
use crate::state::AppState;

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
