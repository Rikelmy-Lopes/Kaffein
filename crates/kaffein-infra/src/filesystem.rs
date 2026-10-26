use std::path::{Path, PathBuf};

use kaffein_domain::{FileSystem, FsError};

/// Largest file the editor will load. This is a deliberate resource limit;
/// raising it requires streaming support and a measured memory budget.
pub const MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, Default)]
pub struct StdFileSystem;

impl StdFileSystem {
    pub fn new() -> Self {
        Self
    }
}

impl FileSystem for StdFileSystem {
    fn canonicalize(&self, path: &Path) -> Result<PathBuf, FsError> {
        std::fs::canonicalize(path).map_err(|error| FsError::io(path, &error))
    }

    fn read_text(&self, path: &Path) -> Result<String, FsError> {
        let metadata = std::fs::metadata(path).map_err(|error| FsError::io(path, &error))?;
        if !metadata.is_file() {
            return Err(FsError::not_a_file(path));
        }
        if metadata.len() > MAX_FILE_BYTES {
            return Err(FsError::file_too_large(
                path,
                metadata.len(),
                MAX_FILE_BYTES,
            ));
        }

        let bytes = std::fs::read(path).map_err(|error| FsError::io(path, &error))?;
        String::from_utf8(bytes).map_err(|_| FsError::invalid_utf8(path))
    }

    fn write_text(&self, path: &Path, text: &str) -> Result<(), FsError> {
        if text.len() as u64 > MAX_FILE_BYTES {
            return Err(FsError::write_too_large(path, MAX_FILE_BYTES));
        }
        std::fs::write(path, text).map_err(|error| FsError::io(path, &error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;
    use kaffein_domain::FsErrorKind;

    #[test]
    fn writes_and_reads_utf8_text() {
        let dir = TempDir::new("fs-roundtrip");
        let path = dir.path().join("App.java");
        let fs = StdFileSystem;

        fs.write_text(&path, "class App {}\n").expect("write");
        assert_eq!(fs.read_text(&path).expect("read"), "class App {}\n");
    }

    #[test]
    fn rejects_invalid_utf8() {
        let dir = TempDir::new("fs-utf8");
        let path = dir.path().join("Broken.java");
        std::fs::write(&path, [0x66, 0x6f, 0x80]).expect("write bytes");
        let fs = StdFileSystem;

        let error = fs.read_text(&path).expect_err("invalid UTF-8 must fail");

        assert_eq!(error.kind(), FsErrorKind::InvalidUtf8);
    }

    #[test]
    fn missing_paths_return_errors_instead_of_panicking() {
        let dir = TempDir::new("fs-missing");
        let path = dir.path().join("Missing.java");
        let fs = StdFileSystem;

        assert!(fs.read_text(&path).is_err());
        assert!(fs.canonicalize(&path).is_err());
    }
}
