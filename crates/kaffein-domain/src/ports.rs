//! Ports declared by the domain and implemented by infrastructure.
//!
//! Keeping these traits here means the application layer can be tested with
//! in-memory fakes and never names a concrete file system.

use std::path::{Path, PathBuf};

use crate::project::ProjectTree;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FsError {
    message: String,
}

impl FsError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for FsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for FsError {}

pub trait FileSystem {
    fn canonicalize(&self, path: &Path) -> Result<PathBuf, FsError>;
    fn read_text(&self, path: &Path) -> Result<String, FsError>;
    fn write_text(&self, path: &Path, text: &str) -> Result<(), FsError>;
}

pub trait ProjectLoader {
    fn load(&self, root: &Path) -> Result<ProjectTree, FsError>;
}
