//! Symbol indexing and completion dispatch tests.

use crate::command::{command_id, CommandArgs};

use super::support::{add_dir, add_file, edit, open, workspace};

const SOURCE: &str = "class Counter { int count; void run() { cou } }";

fn caret_after(source: &str, marker: &str) -> usize {
    source.rfind(marker).expect("marker") + marker.len()
}

#[test]
fn opening_a_file_indexes_its_declarations() {
    let (mut workspace, disk) = workspace();
    add_file(
        &disk,
        "C:/project/Person.java",
        "package demo; public class Person { String name; }",
    );

    open(&mut workspace, "C:/project/Person.java");

    let (_, ty) = workspace
        .state()
        .index()
        .find_type("demo.Person")
        .expect("indexed Person");
    assert_eq!(ty.members[0].name, "name");
}

#[test]
fn opening_a_project_indexes_every_java_file() {
    let (mut workspace, disk) = workspace();
    add_dir(&disk, "C:/project");
    add_file(&disk, "C:/project/A.java", "package demo; class A {}");
    add_file(&disk, "C:/project/B.java", "package demo; class B {}");
    add_file(&disk, "C:/project/notes.txt", "not java");

    let outcome = workspace.dispatch(command_id::PROJECT_OPEN, CommandArgs::path("C:/project"));

    assert!(outcome.is_success(), "{outcome:?}");
    assert_eq!(workspace.state().index().len(), 2);
    assert!(workspace.state().index().find_type("demo.A").is_some());
    assert!(workspace.state().index().find_type("demo.B").is_some());
}

#[test]
fn saving_replaces_the_indexed_file() {
    let (mut workspace, disk) = workspace();
    add_file(&disk, "C:/project/Counter.java", "class Counter {}");
    let id = open(&mut workspace, "C:/project/Counter.java");
    edit(
        &mut workspace,
        id,
        "class Counter { int count; void increment() {} }",
    );

    let outcome = workspace.dispatch(command_id::FILE_SAVE, CommandArgs::None);

    assert!(outcome.is_success(), "{outcome:?}");
    let ty = &workspace
        .state()
        .index()
        .file("C:/project/Counter.java".as_ref())
        .expect("indexed file")
        .types[0];
    assert!(ty.members.iter().any(|member| member.name == "count"));
    assert!(ty.members.iter().any(|member| member.name == "increment"));
}

#[test]
fn completion_offers_a_field_and_accepting_inserts_it() {
    let (mut workspace, disk) = workspace();
    add_file(&disk, "C:/project/Counter.java", SOURCE);
    let id = open(&mut workspace, "C:/project/Counter.java");
    edit(&mut workspace, id, SOURCE);
    let caret = caret_after(SOURCE, "cou");

    let outcome = workspace.dispatch(
        command_id::EDITOR_COMPLETE,
        CommandArgs::Complete {
            document: id,
            caret,
        },
    );

    assert!(outcome.is_success(), "{outcome:?}");
    let session = workspace.state().completion().expect("completion session");
    assert!(session.items.iter().any(|item| item.label == "count"));

    let outcome = workspace.dispatch(
        command_id::EDITOR_COMPLETE_ACCEPT,
        CommandArgs::CompleteAccept {
            document: id,
            at_ms: 2_000,
            item: None,
        },
    );

    assert!(outcome.is_success(), "{outcome:?}");
    assert!(workspace.state().completion().is_none());
    let buffer = &workspace
        .state()
        .active_document()
        .expect("document")
        .buffer();
    assert!(buffer.contains("count"), "{buffer}");
    assert!(!buffer.contains("cou }"), "{buffer}");
}

#[test]
fn completion_with_an_unknown_prefix_clears_the_session() {
    let (mut workspace, disk) = workspace();
    add_file(
        &disk,
        "C:/project/App.java",
        "class App { void run() { zzz } }",
    );
    let id = open(&mut workspace, "C:/project/App.java");
    let source = "class App { void run() { zzz } }";
    edit(&mut workspace, id, source);
    let caret = caret_after(source, "zzz");

    let outcome = workspace.dispatch(
        command_id::EDITOR_COMPLETE,
        CommandArgs::Complete {
            document: id,
            caret,
        },
    );

    assert!(outcome.is_success(), "{outcome:?}");
    assert!(workspace.state().completion().is_none());
}

#[test]
fn completion_selection_wraps_and_dismiss_clears() {
    let (mut workspace, disk) = workspace();
    add_file(&disk, "C:/project/Counter.java", SOURCE);
    let id = open(&mut workspace, "C:/project/Counter.java");
    edit(&mut workspace, id, SOURCE);
    let caret = caret_after(SOURCE, "cou");
    workspace.dispatch(
        command_id::EDITOR_COMPLETE,
        CommandArgs::Complete {
            document: id,
            caret,
        },
    );
    assert!(workspace.state().completion().is_some());

    workspace.dispatch(
        command_id::EDITOR_COMPLETE_SELECT,
        CommandArgs::CompleteSelect { delta: -1 },
    );
    assert_eq!(
        workspace.state().completion().expect("session").selected,
        workspace.state().completion().expect("session").items.len() - 1
    );

    let outcome = workspace.dispatch(command_id::EDITOR_COMPLETE_DISMISS, CommandArgs::None);
    assert!(outcome.is_success(), "{outcome:?}");
    assert!(workspace.state().completion().is_none());
}
