use crate::command::{CommandArgs, CommandOutcome, Enablement, ErrorKind};
use crate::registry::CommandRegistry;
use crate::services::Services;
use crate::state::{AppState, ThemePreference};

/// Owns application state, the command registry, and the injected services.
pub struct Workspace {
    state: AppState,
    registry: CommandRegistry,
    services: Services,
}

impl Workspace {
    pub fn new(services: Services, theme: ThemePreference) -> Self {
        Self {
            state: AppState::new(theme),
            registry: CommandRegistry::new(),
            services,
        }
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    /// The single dispatch path. Every entry point — menu, toolbar, shortcut,
    /// tab, or test — goes through here.
    pub fn dispatch(&mut self, id: &str, args: CommandArgs) -> CommandOutcome {
        let Some(command) = self.registry.get(id) else {
            let outcome = CommandOutcome::failed(
                ErrorKind::Internal,
                format!("Command '{id}' is not registered"),
            );
            self.record_outcome(&outcome);
            return outcome;
        };

        if command.args.needs_path_prompt() && !matches!(args, CommandArgs::Path(_)) {
            let outcome = CommandOutcome::failed(
                ErrorKind::Internal,
                format!("Command '{}' requires a path argument", command.id),
            );
            self.record_outcome(&outcome);
            return outcome;
        }

        let outcome = (command.run)(&mut self.state, &mut self.services, args);
        self.record_outcome(&outcome);
        outcome
    }

    pub fn can_execute(&self, id: &str) -> Enablement {
        self.registry
            .get(id)
            .map(|command| (command.can_execute)(&self.state))
            .unwrap_or_else(|| Enablement::Disabled("Unknown command"))
    }

    fn record_outcome(&mut self, outcome: &CommandOutcome) {
        match outcome {
            CommandOutcome::Success {
                message: Some(message),
            } => {
                self.state
                    .set_status(crate::state::StatusMessage::info(message.clone()));
            }
            CommandOutcome::Success { message: None } => {
                self.state.clear_status();
            }
            CommandOutcome::NoOp { reason } => {
                self.state
                    .set_status(crate::state::StatusMessage::warning(reason.clone()));
            }
            CommandOutcome::Cancelled => {
                self.state
                    .set_status(crate::state::StatusMessage::warning("Cancelled"));
            }
            CommandOutcome::Failed(error) => {
                self.state
                    .set_status(crate::state::StatusMessage::error(format!(
                        "{}: {}",
                        error.kind.label(),
                        error.message
                    )));
            }
        }
    }
}

pub(crate) fn target_document(
    state: &AppState,
    args: &CommandArgs,
) -> Option<kaffein_domain::DocumentId> {
    match args {
        CommandArgs::Document(id) => Some(*id),
        _ => state.active_id(),
    }
}

pub(crate) fn internal_error(message: impl Into<String>) -> CommandOutcome {
    CommandOutcome::failed(ErrorKind::Internal, message)
}

pub(crate) fn user_error(message: impl Into<String>) -> CommandOutcome {
    CommandOutcome::failed(ErrorKind::UserInput, message)
}

pub(crate) fn path_from_args(args: CommandArgs) -> Result<std::path::PathBuf, CommandOutcome> {
    match args {
        CommandArgs::Path(path) => Ok(path),
        _ => Err(internal_error("This command requires a path argument")),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use kaffein_domain::{FileSystem, FsError, ProjectEntry, ProjectLoader, ProjectTree};

    use super::*;
    use crate::command::command_id;
    use crate::state::MessageSeverity;

    #[derive(Default)]
    struct FakeDisk {
        files: std::collections::HashMap<PathBuf, String>,
        dirs: std::collections::HashSet<PathBuf>,
    }

    #[derive(Clone)]
    struct FakeFs {
        disk: Arc<Mutex<FakeDisk>>,
    }

    impl FileSystem for FakeFs {
        fn canonicalize(&self, path: &std::path::Path) -> Result<PathBuf, FsError> {
            let disk = self.disk.lock().expect("fake disk");
            if disk.files.contains_key(path) || disk.dirs.contains(path) {
                Ok(path.to_path_buf())
            } else {
                Err(FsError::new(format!("{} does not exist", path.display())))
            }
        }

        fn read_text(&self, path: &std::path::Path) -> Result<String, FsError> {
            self.disk
                .lock()
                .expect("fake disk")
                .files
                .get(path)
                .cloned()
                .ok_or_else(|| FsError::new(format!("{} is not a file", path.display())))
        }

        fn write_text(&self, path: &std::path::Path, text: &str) -> Result<(), FsError> {
            let mut disk = self.disk.lock().expect("fake disk");
            disk.files.insert(path.to_path_buf(), text.to_owned());
            Ok(())
        }
    }

    #[derive(Clone)]
    struct FakeProjects {
        disk: Arc<Mutex<FakeDisk>>,
    }

    impl ProjectLoader for FakeProjects {
        fn load(&self, root: &std::path::Path) -> Result<ProjectTree, FsError> {
            let disk = self.disk.lock().expect("fake disk");
            if !disk.dirs.contains(root) {
                return Err(FsError::new(format!(
                    "{} is not a directory",
                    root.display()
                )));
            }
            let children = disk
                .files
                .keys()
                .filter(|path| path.parent() == Some(root))
                .filter_map(|path| {
                    let name = path.file_name()?.to_string_lossy().into_owned();
                    Some(ProjectEntry::file(name, path.clone()))
                })
                .collect();
            let name = root
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| root.display().to_string());
            Ok(ProjectTree::new(root.to_path_buf(), name, children))
        }
    }

    fn workspace() -> (Workspace, Arc<Mutex<FakeDisk>>) {
        let disk = Arc::new(Mutex::new(FakeDisk::default()));
        let fs = FakeFs { disk: disk.clone() };
        let projects = FakeProjects { disk: disk.clone() };
        let services = Services::new(Box::new(fs), Box::new(projects));
        (Workspace::new(services, ThemePreference::Dark), disk)
    }

    fn add_file(disk: &Arc<Mutex<FakeDisk>>, path: &str, contents: &str) {
        disk.lock()
            .expect("fake disk")
            .files
            .insert(PathBuf::from(path), contents.to_owned());
    }

    fn open(workspace: &mut Workspace, path: &str) -> kaffein_domain::DocumentId {
        let outcome = workspace.dispatch(command_id::FILE_OPEN, CommandArgs::path(path));
        assert!(outcome.is_success(), "{outcome:?}");
        workspace.state().active_id().expect("active document")
    }

    fn edit(workspace: &mut Workspace, document: kaffein_domain::DocumentId, text: &str) {
        let outcome = workspace.dispatch(
            command_id::EDITOR_REPLACE_TEXT,
            CommandArgs::ReplaceText {
                document,
                text: text.to_owned(),
                at_ms: 1_000,
            },
        );
        assert!(outcome.is_success(), "{outcome:?}");
    }

    #[test]
    fn registry_ids_are_unique_and_namespaced() {
        let registry = CommandRegistry::new();
        let mut seen = std::collections::HashSet::new();
        for command in registry.commands() {
            assert!(
                command.id.contains('.'),
                "command id '{}' is not namespaced",
                command.id
            );
            assert!(
                seen.insert(command.id),
                "duplicate command id '{}'",
                command.id
            );
            assert!(!command.title.is_empty());
        }
    }

    #[test]
    fn every_menu_and_toolbar_entry_resolves_to_a_command() {
        let registry = CommandRegistry::new();
        for menu in crate::command::Menu::ALL {
            for command in registry.menu_commands(menu) {
                assert!(registry.get(command.id).is_some());
            }
        }
        for command in registry.toolbar_commands() {
            assert!(registry.get(command.id).is_some());
        }
    }

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
        disk.lock()
            .expect("fake disk")
            .dirs
            .insert(PathBuf::from("C:/project"));

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
        disk.lock()
            .expect("fake disk")
            .dirs
            .insert(PathBuf::from("C:/project"));
        add_file(&disk, "C:/project/App.java", "class App {}");

        let outcome = workspace.dispatch(
            command_id::FILE_NEW,
            CommandArgs::path("C:/project/App.java"),
        );

        assert!(matches!(outcome, CommandOutcome::Failed(_)));
        assert!(workspace.state().documents().is_empty());
    }

    #[test]
    fn saving_a_new_document_writes_it_and_clears_dirty_state() {
        let (mut workspace, disk) = workspace();
        disk.lock()
            .expect("fake disk")
            .dirs
            .insert(PathBuf::from("C:/project"));
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
        assert_eq!(status.severity, MessageSeverity::Error);
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

    #[test]
    fn opening_a_project_builds_the_tree() {
        let (mut workspace, disk) = workspace();
        disk.lock()
            .expect("fake disk")
            .dirs
            .insert(PathBuf::from("C:/project"));
        add_file(&disk, "C:/project/App.java", "class App {}");

        let outcome = workspace.dispatch(command_id::PROJECT_OPEN, CommandArgs::path("C:/project"));

        assert!(outcome.is_success(), "{outcome:?}");
        let project = workspace.state().project().expect("project");
        assert_eq!(project.name(), "project");
        assert_eq!(project.children().len(), 1);
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
}
