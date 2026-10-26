//! Application lifecycle dispatch tests.

use crate::command::{command_id, CommandArgs, CommandOutcome};

use super::support::{add_file, edit, open, workspace};

#[test]
fn quitting_with_a_dirty_document_is_refused() {
    let (mut workspace, disk) = workspace();
    add_file(&disk, "C:/project/App.java", "a");
    let document_id = open(&mut workspace, "C:/project/App.java");
    edit(&mut workspace, document_id, "ab");

    let outcome = workspace.dispatch(command_id::APP_QUIT, CommandArgs::None);

    assert!(matches!(outcome, CommandOutcome::NoOp { .. }));
    assert!(!workspace.state().quit_requested());
}

#[test]
fn quitting_with_clean_documents_requests_shutdown() {
    let (mut workspace, _disk) = workspace();

    let outcome = workspace.dispatch(command_id::APP_QUIT, CommandArgs::None);

    assert!(outcome.is_success(), "{outcome:?}");
    assert!(workspace.state().quit_requested());
}
