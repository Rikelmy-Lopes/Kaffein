//! Ports declared by the domain and implemented by infrastructure.
//!
//! Keeping these traits here means the application layer can be tested with
//! in-memory fakes and never names a concrete file system.

use std::path::{Path, PathBuf};

use crate::project::ProjectTree;

/// The reason a filesystem operation failed. User-facing wording belongs to
/// the localization catalogs, not to the domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FsErrorKind {
    Io,
    Other,
    NotAFile,
    FileTooLarge,
    InvalidUtf8,
    WriteTooLarge,
    NotADirectory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FsError {
    kind: FsErrorKind,
    path: PathBuf,
    detail: Option<String>,
    size: Option<u64>,
    limit: Option<u64>,
}

impl FsError {
    pub fn io(path: &Path, error: &std::io::Error) -> Self {
        Self {
            kind: FsErrorKind::Io,
            path: path.to_path_buf(),
            detail: Some(error.to_string()),
            size: None,
            limit: None,
        }
    }

    /// An adapter-specific failure with a technical detail.
    pub fn other(path: &Path, detail: impl Into<String>) -> Self {
        Self {
            kind: FsErrorKind::Other,
            path: path.to_path_buf(),
            detail: Some(detail.into()),
            size: None,
            limit: None,
        }
    }

    pub fn not_a_file(path: &Path) -> Self {
        Self::of_kind(FsErrorKind::NotAFile, path)
    }

    pub fn file_too_large(path: &Path, size: u64, limit: u64) -> Self {
        Self {
            kind: FsErrorKind::FileTooLarge,
            path: path.to_path_buf(),
            detail: None,
            size: Some(size),
            limit: Some(limit),
        }
    }

    pub fn invalid_utf8(path: &Path) -> Self {
        Self::of_kind(FsErrorKind::InvalidUtf8, path)
    }

    pub fn write_too_large(path: &Path, limit: u64) -> Self {
        Self {
            kind: FsErrorKind::WriteTooLarge,
            path: path.to_path_buf(),
            detail: None,
            size: None,
            limit: Some(limit),
        }
    }

    pub fn not_a_directory(path: &Path) -> Self {
        Self::of_kind(FsErrorKind::NotADirectory, path)
    }

    fn of_kind(kind: FsErrorKind, path: &Path) -> Self {
        Self {
            kind,
            path: path.to_path_buf(),
            detail: None,
            size: None,
            limit: None,
        }
    }

    pub fn kind(&self) -> FsErrorKind {
        self.kind
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    pub fn size(&self) -> Option<u64> {
        self.size
    }

    pub fn limit(&self) -> Option<u64> {
        self.limit
    }
}

impl std::fmt::Display for FsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "filesystem error {:?} at {}",
            self.kind,
            self.path.display()
        )?;
        if let Some(detail) = &self.detail {
            write!(formatter, ": {detail}")?;
        }
        Ok(())
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
