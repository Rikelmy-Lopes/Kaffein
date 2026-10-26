//! Close dispatch tests.

use crate::command::{command_id, CommandArgs, CommandOutcome};

use super::support::{add_file, edit, open, workspace};

#[test]
fn closing_a_dirty_document_is_refused() {
    let (mut workspace, disk) = workspace();
    add_file(&disk, "C:/project/App.java", "a");
    let document_id = open(&mut workspace, "C:/project/App.java");
    edit(&mut workspace, document_id, "ab");

    let outcome = workspace.dispatch(command_id::FILE_CLOSE, CommandArgs::None);

    assert!(matches!(outcome, CommandOutcome::NoOp { .. }));
    assert_eq!(workspace.state().documents().len(), 1);
}

#[test]
fn closing_a_clean_document_removes_it() {
    let (mut workspace, disk) = workspace();
    add_file(&disk, "C:/project/App.java", "a");
    open(&mut workspace, "C:/project/App.java");

    let outcome = workspace.dispatch(command_id::FILE_CLOSE, CommandArgs::None);

    assert!(outcome.is_success(), "{outcome:?}");
    assert!(workspace.state().documents().is_empty());
    assert!(workspace.state().active_document().is_none());
}
