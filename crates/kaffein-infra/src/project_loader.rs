use std::path::Path;

use kaffein_domain::{FsError, ProjectEntry, ProjectLoader, ProjectTree};

/// Directory names that are never worth showing in the foundation shell.
/// Build output and tool metadata return once project settings can configure
/// the exclusions.
const IGNORED_DIRECTORIES: &[&str] = &[
    ".git",
    ".gradle",
    ".idea",
    ".venv",
    "bin",
    "build",
    "node_modules",
    "obj",
    "out",
    "target",
];

const MAX_DEPTH: usize = 8;
const MAX_ENTRIES_PER_DIRECTORY: usize = 500;

#[derive(Debug, Clone)]
pub struct FsProjectLoader;

impl FsProjectLoader {
    pub fn new() -> Self {
        Self
    }

    fn scan_directory(&self, directory: &Path, depth: usize) -> Result<Vec<ProjectEntry>, FsError> {
        if depth > MAX_DEPTH {
            return Ok(Vec::new());
        }

        let entries = match std::fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(_) if depth > 0 => return Ok(Vec::new()),
            Err(error) => return Err(FsError::io(directory, &error)),
        };

        let mut result = Vec::new();
        for entry in entries.flatten().take(MAX_ENTRIES_PER_DIRECTORY) {
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => continue,
            };
            // Symlinks are skipped deliberately: resolving them could leave the
            // project root, which is a security boundary.
            if file_type.is_symlink() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().into_owned();
            if name.is_empty() || IGNORED_DIRECTORIES.contains(&name.as_str()) {
                continue;
            }
            let path = entry.path();

            if file_type.is_dir() {
                let children = self.scan_directory(&path, depth + 1)?;
                result.push(ProjectEntry::directory(name, path, children));
            } else if file_type.is_file() {
                result.push(ProjectEntry::file(name, path));
            }
        }

        result.sort_by(|left, right| {
            right
                .is_directory()
                .cmp(&left.is_directory())
                .then_with(|| left.name().to_lowercase().cmp(&right.name().to_lowercase()))
        });
        Ok(result)
    }
}

impl Default for FsProjectLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectLoader for FsProjectLoader {
    fn load(&self, root: &Path) -> Result<ProjectTree, FsError> {
        let metadata = std::fs::metadata(root).map_err(|error| FsError::io(root, &error))?;
        if !metadata.is_dir() {
            return Err(FsError::not_a_directory(root));
        }

        let name = root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.display().to_string());
        let children = self.scan_directory(root, 0)?;
        Ok(ProjectTree::new(root.to_path_buf(), name, children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    #[test]
    fn scans_directories_first_and_skips_ignored_folders() {
        let dir = TempDir::new("project-scan");
        std::fs::create_dir_all(dir.path().join("src")).expect("src");
        std::fs::create_dir_all(dir.path().join(".git")).expect("git");
        std::fs::write(dir.path().join("README.md"), "readme").expect("readme");
        std::fs::write(dir.path().join("src/App.java"), "class App {}").expect("app");

        let tree = FsProjectLoader.load(dir.path()).expect("project");
        let names: Vec<&str> = tree.children().iter().map(ProjectEntry::name).collect();

        assert_eq!(names, ["src", "README.md"]);
        assert_eq!(tree.children()[0].children()[0].name(), "App.java");
    }

    #[test]
    fn rejects_a_file_as_project_root() {
        let dir = TempDir::new("project-file-root");
        let file = dir.path().join("App.java");
        std::fs::write(&file, "class App {}").expect("file");

        assert!(FsProjectLoader.load(&file).is_err());
    }
}
