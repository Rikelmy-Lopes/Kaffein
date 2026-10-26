//! Formatting for paths shown to the user.

/// Formats a path for user-facing text.
///
/// `std::fs::canonicalize` returns extended-length paths on Windows
/// (`\\?\C:\...`). The prefix is an implementation detail, not something a
/// user should see in a status message or prompt.
pub fn display_path(path: &std::path::Path) -> String {
    let text = path.display().to_string();
    #[cfg(windows)]
    {
        if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{rest}");
        }
        if let Some(rest) = text.strip_prefix(r"\\?\") {
            return rest.to_owned();
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::display_path;
    use std::path::Path;

    #[test]
    fn leaves_ordinary_paths_untouched() {
        assert_eq!(
            display_path(Path::new("C:/work/App.java")),
            "C:/work/App.java"
        );
    }

    #[cfg(windows)]
    #[test]
    fn strips_the_windows_verbatim_prefix() {
        assert_eq!(
            display_path(Path::new(r"\\?\C:\work\App.java")),
            r"C:\work\App.java"
        );
        assert_eq!(
            display_path(Path::new(r"\\?\UNC\server\share\App.java")),
            r"\\server\share\App.java"
        );
    }
}
