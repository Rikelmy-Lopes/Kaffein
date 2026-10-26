//! Application layer: command registry, dispatch, and application state.
//!
//! No egui type may appear here. The UI dispatches commands and reads state;
//! it never mutates documents directly.

pub mod command;
pub mod commands;
pub mod registry;
pub mod services;
pub mod state;
pub mod workspace;

pub use command::{
    command_id, ArgKind, Command, CommandArgs, CommandError, CommandOutcome, Enablement, ErrorKind,
    Key, KeyBinding, Menu, MenuPlacement, ToolbarPlacement,
};
pub use registry::CommandRegistry;
pub use services::Services;
pub use state::{AppState, MessageSeverity, StatusMessage, ThemePreference};
pub use workspace::Workspace;

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
mod display_path_tests {
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

/// Stable identifiers for every command. The strings are the contract used by
/// menus, keybindings, tests, and future automation.
pub mod ids {
    pub use crate::command::command_id::*;
}
