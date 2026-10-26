//! Project tree model. Building the tree is infrastructure's job; this module
//! only defines the data the UI renders.

use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectTree {
    root: PathBuf,
    name: String,
    children: Vec<ProjectEntry>,
}

impl ProjectTree {
    pub fn new(root: PathBuf, name: String, children: Vec<ProjectEntry>) -> Self {
        Self {
            root,
            name,
            children,
        }
    }

    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn children(&self) -> &[ProjectEntry] {
        &self.children
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectEntry {
    name: String,
    path: PathBuf,
    kind: ProjectEntryKind,
    children: Vec<ProjectEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectEntryKind {
    Directory,
    File,
}

impl ProjectEntry {
    pub fn directory(name: String, path: PathBuf, children: Vec<ProjectEntry>) -> Self {
        Self {
            name,
            path,
            kind: ProjectEntryKind::Directory,
            children,
        }
    }

    pub fn file(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            kind: ProjectEntryKind::File,
            children: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn kind(&self) -> ProjectEntryKind {
        self.kind
    }

    pub fn is_directory(&self) -> bool {
        self.kind == ProjectEntryKind::Directory
    }

    pub fn children(&self) -> &[ProjectEntry] {
        &self.children
    }
}
