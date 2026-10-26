//! Implementations of the registered commands, grouped by area.
//!
//! Each submodule owns the behaviour for one command namespace; the re-exports
//! below keep the registry table readable (`commands::file_open`) without
//! making the module layout flat again.

mod app;
mod edit;
mod file;
mod helpers;
mod project;
mod view;

pub(crate) use app::{app_quit, app_set_locale};
pub(crate) use edit::{edit_redo, edit_undo, editor_activate, editor_replace_text};
pub(crate) use file::{file_close, file_new, file_open, file_save};
pub(crate) use project::project_open;
pub(crate) use view::{view_toggle_bottom, view_toggle_project, view_toggle_theme};
