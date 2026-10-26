//! Save dispatch tests.

use std::path::PathBuf;

use crate::command::{command_id, CommandArgs, CommandOutcome};
use crate::state::MessageSeverity;

use super::support::{add_dir, add_file, edit, open, workspace};

#[test]
fn saving_a_new_document_writes_it_and_clears_dirty_state() {
    let (mut workspace, disk) = workspace();
    add_dir(&disk, "C:/project");
    let path = "C:/project/New.java";
    assert!(workspace
        .dispatch(command_id::FILE_NEW, CommandArgs::path(path))
        .is_success());

    let outcome = workspace.dispatch(command_id::FILE_SAVE, CommandArgs::None);

    assert!(outcome.is_success(), "{outcome:?}");
    assert!(!workspace
        .state()
        .active_document()
        .expect("active document")
        .is_dirty());
    assert_eq!(
        disk.lock()
            .expect("fake disk")
            .files
            .get(&PathBuf::from(path))
            .map(String::as_str),
        Some("")
    );
}

#[test]
fn editing_then_saving_writes_the_document_and_clears_dirty_state() {
    let (mut workspace, disk) = workspace();
    add_file(&disk, "C:/project/App.java", "class App {}\n");
    let document_id = open(&mut workspace, "C:/project/App.java");
    edit(&mut workspace, document_id, "class App { int x; }\n");
    assert!(workspace
        .state()
        .active_document()
        .expect("document")
        .is_dirty());

    let outcome = workspace.dispatch(command_id::FILE_SAVE, CommandArgs::None);

    assert!(outcome.is_success(), "{outcome:?}");
    assert!(!workspace
        .state()
        .active_document()
        .expect("document")
        .is_dirty());
    let disk = disk.lock().expect("fake disk");
    assert_eq!(
        disk.files
            .get(&PathBuf::from("C:/project/App.java"))
            .map(String::as_str),
        Some("class App { int x; }\n")
    );
}

#[test]
fn save_without_a_document_is_a_reported_no_op() {
    let (mut workspace, _disk) = workspace();

    let outcome = workspace.dispatch(command_id::FILE_SAVE, CommandArgs::None);

    assert!(matches!(outcome, CommandOutcome::NoOp { .. }));
    assert_eq!(
        workspace.state().status().expect("status").severity,
        MessageSeverity::Warning
    );
}
