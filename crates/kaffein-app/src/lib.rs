//! Application layer: command registry, dispatch, and application state.
//!
//! No egui type may appear here. The UI dispatches commands and reads state; it
//! never mutates documents directly.
//!
//! Module layout:
//!
//! - `command` — command descriptors and value types,
//! - `commands` — one module per command area,
//! - `registry` — the command table and enablement predicates,
//! - `state` — everything the UI reads,
//! - `services` — infrastructure injected by the binary,
//! - `workspace` — the single dispatch path,
//! - `paths` — user-facing path formatting.

pub mod command;
pub mod commands;
pub mod registry;
pub mod services;
pub mod state;
pub mod workspace;

mod defaults;
mod paths;

pub use command::{
    command_id, ArgKind, Command, CommandArgs, CommandError, CommandOutcome, Enablement, ErrorKind,
    Key, KeyBinding, Menu, MenuPlacement, ToolbarPlacement,
};
pub use defaults::DEFAULT_JAVA_SOURCE;
pub use paths::display_path;
pub use registry::CommandRegistry;
pub use services::Services;
pub use state::{
    AppState, CompletionSession, MessageSeverity, RunState, StatusMessage, ThemePreference,
};
pub use workspace::Workspace;

/// Stable identifiers for every command. The strings are the contract used by
/// menus, keybindings, tests, and future automation.
pub mod ids {
    pub use crate::command::command_id::*;
}
