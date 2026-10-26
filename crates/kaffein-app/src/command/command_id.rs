//! Stable identifiers for every command.
//!
//! The strings are the contract used by menus, keybindings, tests, and future
//! automation. Renaming one is a breaking change, not a refactor.

pub const PROJECT_OPEN: &str = "project.open";
pub const FILE_NEW: &str = "file.new";
pub const FILE_OPEN: &str = "file.open";
pub const FILE_SAVE: &str = "file.save";
pub const FILE_CLOSE: &str = "file.close";
pub const EDIT_UNDO: &str = "edit.undo";
pub const EDIT_REDO: &str = "edit.redo";
pub const EDITOR_REPLACE_TEXT: &str = "editor.replace_text";
pub const EDITOR_ACTIVATE: &str = "editor.activate";
pub const VIEW_TOGGLE_PROJECT: &str = "view.toggle_project";
pub const VIEW_TOGGLE_BOTTOM: &str = "view.toggle_bottom";
pub const VIEW_TOGGLE_THEME: &str = "view.toggle_theme";
pub const APP_QUIT: &str = "app.quit";
