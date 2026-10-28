//! Scratch-document and run dispatch tests.

use std::sync::Arc;
use std::time::{Duration, Instant};

use kaffein_domain::JavaRunStage;
use kaffein_i18n::keys;

use crate::command::{command_id, CommandArgs, CommandOutcome, Enablement};
use crate::state::RunState;
use crate::workspace::Workspace;

use super::support::{add_file, edit, open, workspace, workspace_with_java_runner, FakeJavaRunner};

const SOURCE: &str = "public class Main {\n    public static void main(String[] args) {\n        System.out.println(\"hello\");\n    }\n}\n";

fn create_scratch(workspace: &mut Workspace) {
    let outcome = workspace.dispatch(command_id::FILE_NEW_SCRATCH, CommandArgs::None);
    assert!(outcome.is_success(), "{outcome:?}");
}

fn wait_for_run(workspace: &Workspace) -> Arc<RunState> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let state = workspace.state().run_state();
        if !state.is_running() && state.id() != 0 {
            return state;
        }
        assert!(
            Instant::now() < deadline,
            "run did not finish before the deadline: {state:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn a_scratch_document_is_editable_and_has_no_path() {
    let (mut workspace, _disk) = workspace();
    create_scratch(&mut workspace);

    let document = workspace.state().active_document().expect("scratch");
    assert!(document.is_scratch());
    assert!(document.buffer().contains("public class Main"));

    let id = document.id();
    edit(&mut workspace, id, SOURCE);

    let document = workspace.state().active_document().expect("scratch");
    assert_eq!(document.buffer(), SOURCE);
    assert!(document.is_dirty());
}

#[test]
fn running_a_scratch_document_uses_the_buffer_without_saving_it() {
    let runner = FakeJavaRunner::with_output("hello\n");
    let (mut workspace, disk, runner) = workspace_with_java_runner(runner);
    create_scratch(&mut workspace);
    let id = workspace.state().active_id().expect("scratch");
    edit(&mut workspace, id, SOURCE);

    let outcome = workspace.dispatch(command_id::RUN_EXECUTE, CommandArgs::None);

    assert!(outcome.is_success(), "{outcome:?}");
    let state = wait_for_run(&workspace);
    let RunState::Finished { output, .. } = &*state else {
        panic!("expected a finished run, got {state:?}");
    };
    assert_eq!(output.stage, JavaRunStage::Run);
    assert_eq!(output.exit_code, Some(0));
    assert_eq!(output.stdout, "hello\n");
    assert_eq!(runner.sources(), vec![SOURCE.to_owned()]);
    assert!(disk.lock().expect("fake disk").files.is_empty());
    assert!(workspace
        .state()
        .active_document()
        .expect("scratch")
        .is_dirty());
}

#[test]
fn saving_a_scratch_document_is_refused() {
    let (mut workspace, disk) = workspace();
    create_scratch(&mut workspace);
    let id = workspace.state().active_id().expect("scratch");
    edit(&mut workspace, id, SOURCE);

    let outcome = workspace.dispatch(command_id::FILE_SAVE, CommandArgs::None);

    assert!(matches!(outcome, CommandOutcome::NoOp { .. }));
    assert!(disk.lock().expect("fake disk").files.is_empty());
}

#[test]
fn closing_a_dirty_scratch_discards_it_without_a_prompt() {
    let (mut workspace, _disk) = workspace();
    create_scratch(&mut workspace);
    let id = workspace.state().active_id().expect("scratch");
    edit(&mut workspace, id, SOURCE);

    let outcome = workspace.dispatch(command_id::FILE_CLOSE, CommandArgs::None);

    assert!(outcome.is_success(), "{outcome:?}");
    assert!(workspace.state().documents().is_empty());
}

#[test]
fn run_is_disabled_for_a_non_java_document() {
    let (mut workspace, disk) = workspace();
    add_file(&disk, "C:/project/notes.txt", "notes");
    open(&mut workspace, "C:/project/notes.txt");

    assert_eq!(
        workspace.can_execute(command_id::RUN_EXECUTE),
        Enablement::Disabled(keys::ENABLEMENT_NOT_JAVA)
    );
}
