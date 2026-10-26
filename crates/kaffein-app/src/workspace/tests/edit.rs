//! Edit and tab activation dispatch tests.

use crate::command::{command_id, CommandArgs};

use super::support::{add_file, edit, open, workspace};

#[test]
fn undo_and_redo_dispatch_through_the_document() {
    let (mut workspace, disk) = workspace();
    add_file(&disk, "C:/project/App.java", "a");
    let document_id = open(&mut workspace, "C:/project/App.java");
    edit(&mut workspace, document_id, "ab");

    assert!(workspace
        .dispatch(command_id::EDIT_UNDO, CommandArgs::None)
        .is_success());
    assert_eq!(
        workspace
            .state()
            .active_document()
            .expect("document")
            .buffer(),
        "a"
    );

    assert!(workspace
        .dispatch(command_id::EDIT_REDO, CommandArgs::None)
        .is_success());
    assert_eq!(
        workspace
            .state()
            .active_document()
            .expect("document")
            .buffer(),
        "ab"
    );
}

#[test]
fn activating_an_open_document_switches_the_active_tab() {
    let (mut workspace, disk) = workspace();
    add_file(&disk, "C:/project/A.java", "class A {}");
    add_file(&disk, "C:/project/B.java", "class B {}");
    open(&mut workspace, "C:/project/A.java");
    let first = workspace.state().active_id().expect("active");
    open(&mut workspace, "C:/project/B.java");
    let second = workspace.state().active_id().expect("active");

    let outcome = workspace.dispatch(command_id::EDITOR_ACTIVATE, CommandArgs::Document(first));

    assert!(outcome.is_success(), "{outcome:?}");
    assert_eq!(workspace.state().active_id(), Some(first));
    assert_ne!(first, second);
}
