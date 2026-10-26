//! Application lifecycle commands.

use kaffein_i18n::{keys, LocaleError, Text};

use crate::command::{CommandArgs, CommandOutcome, ErrorKind};
use crate::services::Services;
use crate::state::AppState;

use super::helpers::internal_error;

pub(crate) fn app_quit(
    state: &mut AppState,
    _services: &mut Services,
    _args: CommandArgs,
) -> CommandOutcome {
    let dirty = state.dirty_document_count();
    if dirty > 0 {
        return CommandOutcome::no_op(Text::plural(keys::MESSAGE_APP_DIRTY_DOCUMENTS, dirty));
    }
    state.request_quit();
    CommandOutcome::success_silent()
}

pub(crate) fn app_set_locale(
    _state: &mut AppState,
    services: &mut Services,
    args: CommandArgs,
) -> CommandOutcome {
    let CommandArgs::Locale(code) = args else {
        return internal_error(Text::key(keys::MESSAGE_APP_LOCALE_REQUIRES_CODE));
    };

    match services.localizer.set_locale(&code) {
        Ok(()) => {
            let language = services.localizer.active_locale().name().to_owned();
            CommandOutcome::success(
                Text::message(keys::MESSAGE_APP_LOCALE_CHANGED).arg("language", language),
            )
        }
        Err(LocaleError::Unknown(code)) => CommandOutcome::failed(
            ErrorKind::UserInput,
            Text::message(keys::MESSAGE_APP_LOCALE_UNKNOWN).arg("code", code),
        ),
        Err(LocaleError::Persistence(detail)) => CommandOutcome::failed(
            ErrorKind::ToolFailure,
            Text::message(keys::MESSAGE_APP_LOCALE_SAVE_FAILED).arg("error", detail),
        ),
    }
}
