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

/// Which step of a Java run produced the captured output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaRunStage {
    Compile,
    Run,
}

/// Captured output from a compiler or launcher invocation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaRunOutput {
    pub stage: JavaRunStage,
    pub stdout: String,
    pub stderr: String,
    /// `None` when no exit code is available, including a timed-out process
    /// that was killed.
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaRunErrorKind {
    /// `javac` or `java` could not be started, usually because the JDK is missing.
    Unavailable,
    /// A temporary file or directory could not be created or written.
    Io,
    Other,
}

/// The reason a run could not be started or completed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaRunError {
    kind: JavaRunErrorKind,
    detail: String,
}

impl JavaRunError {
    pub fn unavailable(detail: impl Into<String>) -> Self {
        Self {
            kind: JavaRunErrorKind::Unavailable,
            detail: detail.into(),
        }
    }

    pub fn io(detail: impl Into<String>) -> Self {
        Self {
            kind: JavaRunErrorKind::Io,
            detail: detail.into(),
        }
    }

    pub fn other(detail: impl Into<String>) -> Self {
        Self {
            kind: JavaRunErrorKind::Other,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> JavaRunErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl std::fmt::Display for JavaRunError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "java run error {:?}: {}", self.kind, self.detail)
    }
}

impl std::error::Error for JavaRunError {}

/// Compiles and runs a Java source buffer without saving it to a project file.
///
/// Implementations materialise the source in a private temporary directory,
/// invoke the JDK with argument lists (never a shell), and clean up before
/// returning. The method blocks, so the application calls it from a worker
/// thread rather than from the UI frame.
pub trait JavaRunner: Send + Sync {
    fn run_source(&self, source: &str) -> Result<JavaRunOutput, JavaRunError>;
}
