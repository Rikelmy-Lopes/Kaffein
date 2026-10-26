//! Shared fakes and helpers for workspace tests.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use kaffein_domain::{FileSystem, FsError, ProjectEntry, ProjectLoader, ProjectTree};
use kaffein_i18n::Localizer;

use crate::command::{command_id, CommandArgs};
use crate::services::Services;
use crate::state::ThemePreference;
use crate::workspace::Workspace;

#[derive(Default)]
pub(crate) struct FakeDisk {
    pub(crate) files: HashMap<PathBuf, String>,
    pub(crate) dirs: HashSet<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct FakeFs {
    pub(crate) disk: Arc<Mutex<FakeDisk>>,
}

impl FileSystem for FakeFs {
    fn canonicalize(&self, path: &std::path::Path) -> Result<PathBuf, FsError> {
        let disk = self.disk.lock().expect("fake disk");
        if disk.files.contains_key(path) || disk.dirs.contains(path) {
            Ok(path.to_path_buf())
        } else {
            Err(FsError::other(path, "does not exist"))
        }
    }

    fn read_text(&self, path: &std::path::Path) -> Result<String, FsError> {
        self.disk
            .lock()
            .expect("fake disk")
            .files
            .get(path)
            .cloned()
            .ok_or_else(|| FsError::other(path, "is not a file"))
    }

    fn write_text(&self, path: &std::path::Path, text: &str) -> Result<(), FsError> {
        let mut disk = self.disk.lock().expect("fake disk");
        disk.files.insert(path.to_path_buf(), text.to_owned());
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct FakeProjects {
    pub(crate) disk: Arc<Mutex<FakeDisk>>,
}

impl ProjectLoader for FakeProjects {
    fn load(&self, root: &std::path::Path) -> Result<ProjectTree, FsError> {
        let disk = self.disk.lock().expect("fake disk");
        if !disk.dirs.contains(root) {
            return Err(FsError::other(root, "is not a directory"));
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

pub(crate) fn workspace() -> (Workspace, Arc<Mutex<FakeDisk>>) {
    let disk = Arc::new(Mutex::new(FakeDisk::default()));
    let fs = FakeFs { disk: disk.clone() };
    let projects = FakeProjects { disk: disk.clone() };
    let services = Services::new(Box::new(fs), Box::new(projects), Localizer::english());
    (Workspace::new(services, ThemePreference::Dark), disk)
}

pub(crate) fn add_file(disk: &Arc<Mutex<FakeDisk>>, path: &str, contents: &str) {
    disk.lock()
        .expect("fake disk")
        .files
        .insert(PathBuf::from(path), contents.to_owned());
}

pub(crate) fn add_dir(disk: &Arc<Mutex<FakeDisk>>, path: &str) {
    disk.lock()
        .expect("fake disk")
        .dirs
        .insert(PathBuf::from(path));
}

pub(crate) fn open(workspace: &mut Workspace, path: &str) -> kaffein_domain::DocumentId {
    let outcome = workspace.dispatch(command_id::FILE_OPEN, CommandArgs::path(path));
    assert!(outcome.is_success(), "{outcome:?}");
    workspace.state().active_id().expect("active document")
}

pub(crate) fn edit(workspace: &mut Workspace, document: kaffein_domain::DocumentId, text: &str) {
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
