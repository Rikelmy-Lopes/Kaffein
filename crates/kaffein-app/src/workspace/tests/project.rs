//! Project dispatch tests.

use crate::command::{command_id, CommandArgs};

use super::support::{add_dir, add_file, workspace};

#[test]
fn opening_a_project_builds_the_tree() {
    let (mut workspace, disk) = workspace();
    add_dir(&disk, "C:/project");
    add_file(&disk, "C:/project/App.java", "class App {}");

    let outcome = workspace.dispatch(command_id::PROJECT_OPEN, CommandArgs::path("C:/project"));

    assert!(outcome.is_success(), "{outcome:?}");
    let project = workspace.state().project().expect("project");
    assert_eq!(project.name(), "project");
    assert_eq!(project.children().len(), 1);
}
