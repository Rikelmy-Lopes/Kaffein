//! Locale selection and deferred-text dispatch tests.

use crate::command::{command_id, CommandArgs, CommandOutcome};

use super::support::workspace;

#[test]
fn switching_locale_rerenders_text_captured_before_the_switch() {
    let (mut workspace, _disk) = workspace();
    let failed = workspace.dispatch(
        command_id::FILE_OPEN,
        CommandArgs::path("C:/project/Missing.java"),
    );
    assert!(matches!(failed, CommandOutcome::Failed(_)));
    let status_text = workspace.state().status().expect("status").text.clone();

    let outcome = workspace.dispatch(
        command_id::APP_SET_LOCALE,
        CommandArgs::Locale("pt-BR".to_owned()),
    );

    assert!(outcome.is_success(), "{outcome:?}");
    assert_eq!(workspace.localizer().locale().code(), "pt-BR");
    assert!(status_text
        .resolve(workspace.localizer())
        .starts_with("entrada inválida:"));
    assert_eq!(
        workspace
            .state()
            .status()
            .expect("status")
            .text
            .resolve(workspace.localizer()),
        "Idioma: Português (Brasil)"
    );
}

#[test]
fn an_unknown_locale_fails_and_keeps_the_previous_language() {
    let (mut workspace, _disk) = workspace();

    let outcome = workspace.dispatch(
        command_id::APP_SET_LOCALE,
        CommandArgs::Locale("zz".to_owned()),
    );

    assert!(matches!(outcome, CommandOutcome::Failed(_)));
    assert_eq!(workspace.localizer().locale().code(), "en");
    assert_eq!(
        workspace
            .state()
            .status()
            .expect("status")
            .text
            .resolve(workspace.localizer()),
        "invalid input: The language 'zz' is not available"
    );
}
