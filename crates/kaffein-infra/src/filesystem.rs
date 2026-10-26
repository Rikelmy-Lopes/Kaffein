use std::path::{Path, PathBuf};

use kaffein_domain::{FileSystem, FsError};

/// Largest file the editor will load. This is a deliberate resource limit;
/// raising it requires streaming support and a measured memory budget.
pub const MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Default, Clone, Copy)]
pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn canonicalize(&self, path: &Path) -> Result<PathBuf, FsError> {
        std::fs::canonicalize(path)
            .map_err(|error| FsError::new(format!("{}: {error}", path.display())))
    }

    fn read_text(&self, path: &Path) -> Result<String, FsError> {
        let metadata = std::fs::metadata(path)
            .map_err(|error| FsError::new(format!("{}: {error}", path.display())))?;
        if !metadata.is_file() {
            return Err(FsError::new(format!("{} is not a file", path.display())));
        }
        if metadata.len() > MAX_FILE_BYTES {
            return Err(FsError::new(format!(
                "{} is {} bytes; the current limit is {} bytes",
                path.display(),
                metadata.len(),
                MAX_FILE_BYTES
            )));
        }

        let bytes = std::fs::read(path)
            .map_err(|error| FsError::new(format!("{}: {error}", path.display())))?;
        String::from_utf8(bytes).map_err(|_| {
            FsError::new(format!(
                "{} is not valid UTF-8; encoding conversion is not implemented",
                path.display()
            ))
        })
    }

    fn write_text(&self, path: &Path, text: &str) -> Result<(), FsError> {
        if text.len() as u64 > MAX_FILE_BYTES {
            return Err(FsError::new(format!(
                "{} would exceed the {} byte write limit",
                path.display(),
                MAX_FILE_BYTES
            )));
        }
        std::fs::write(path, text)
            .map_err(|error| FsError::new(format!("{}: {error}", path.display())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let unique = format!(
                "kaffein-{label}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|duration| duration.as_nanos())
                    .unwrap_or_default()
            );
            let path = std::env::temp_dir().join(unique);
            std::fs::create_dir_all(&path).expect("create temporary directory");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

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

        assert!(error.message().contains("UTF-8"));
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
