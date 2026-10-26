//! Open and new-file dispatch tests.

use std::path::PathBuf;

use crate::command::{command_id, CommandArgs, CommandOutcome, ErrorKind};

use super::support::{add_dir, add_file, open, workspace};

#[test]
fn file_open_loads_a_file_and_makes_it_active() {
    let (mut workspace, disk) = workspace();
    add_file(&disk, "C:/project/App.java", "class App {}\n");

    let outcome = workspace.dispatch(
        command_id::FILE_OPEN,
        CommandArgs::path("C:/project/App.java"),
    );

    assert!(outcome.is_success(), "{outcome:?}");
    let document = workspace
        .state()
        .active_document()
        .expect("active document");
    assert_eq!(document.buffer(), "class App {}\n");
    assert_eq!(workspace.state().documents().len(), 1);
    assert!(!document.is_dirty());
}

#[test]
fn creating_a_file_adds_a_dirty_document_without_writing() {
    let (mut workspace, disk) = workspace();
    add_dir(&disk, "C:/project");

    let outcome = workspace.dispatch(
        command_id::FILE_NEW,
        CommandArgs::path("C:/project/New.java"),
    );

    assert!(outcome.is_success(), "{outcome:?}");
    let document = workspace
        .state()
        .active_document()
        .expect("active document");
    assert_eq!(document.path(), PathBuf::from("C:/project/New.java"));
    assert_eq!(document.buffer(), "");
    assert!(document.is_dirty());
    assert!(disk.lock().expect("fake disk").files.is_empty());
}

#[test]
fn creating_a_file_that_already_exists_is_refused() {
    let (mut workspace, disk) = workspace();
    add_dir(&disk, "C:/project");
    add_file(&disk, "C:/project/App.java", "class App {}");

    let outcome = workspace.dispatch(
        command_id::FILE_NEW,
        CommandArgs::path("C:/project/App.java"),
    );

    assert!(matches!(outcome, CommandOutcome::Failed(_)));
    assert!(workspace.state().documents().is_empty());
}

#[test]
fn opening_the_same_path_twice_reuses_the_document() {
    let (mut workspace, disk) = workspace();
    add_file(&disk, "C:/project/App.java", "class App {}\n");
    open(&mut workspace, "C:/project/App.java");
    let first = workspace.state().active_id().expect("active");

    open(&mut workspace, "C:/project/App.java");

    assert_eq!(workspace.state().documents().len(), 1);
    assert_eq!(workspace.state().active_id(), Some(first));
}

#[test]
fn file_open_of_a_missing_file_fails_with_a_user_message() {
    let (mut workspace, _disk) = workspace();

    let outcome = workspace.dispatch(
        command_id::FILE_OPEN,
        CommandArgs::path("C:/project/Missing.java"),
    );

    assert!(matches!(outcome, CommandOutcome::Failed(_)));
    let status = workspace.state().status().expect("status");
    assert_eq!(status.severity, crate::state::MessageSeverity::Error);
    assert!(workspace.state().documents().is_empty());
}

#[test]
fn file_open_without_a_path_is_an_internal_error() {
    let (mut workspace, _disk) = workspace();

    let outcome = workspace.dispatch(command_id::FILE_OPEN, CommandArgs::None);

    assert!(matches!(
        outcome,
        CommandOutcome::Failed(crate::command::CommandError {
            kind: ErrorKind::Internal,
            ..
        })
    ));
}
