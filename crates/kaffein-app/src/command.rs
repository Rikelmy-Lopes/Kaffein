//! Command descriptors: identity, arguments, results, enablement, and placement.
//!
//! The pieces are split by concern so that a reader can find the shape of a
//! command without scrolling through its implementations:
//!
//! - `command_id` — stable identifiers,
//! - `args` — what a command receives,
//! - `outcome` — what a command returns,
//! - `keys` — keyboard bindings in application terms,
//! - `placement` — menu and toolbar metadata.

pub mod command_id;

mod args;
mod keys;
mod outcome;
mod placement;

pub use args::{ArgKind, CommandArgs};
pub use keys::{Key, KeyBinding};
pub use outcome::{CommandError, CommandOutcome, Enablement, ErrorKind};
pub use placement::{Menu, MenuPlacement, ToolbarPlacement};

use crate::services::Services as ServiceSet;
use crate::state::AppState;

pub type CommandFn = fn(&mut AppState, &mut ServiceSet, CommandArgs) -> CommandOutcome;
pub type EnablementFn = fn(&AppState) -> Enablement;
pub type CheckedFn = fn(&AppState) -> bool;

#[derive(Clone, Copy)]
pub struct Command {
    pub id: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub menu: Option<MenuPlacement>,
    pub toolbar: Option<ToolbarPlacement>,
    pub shortcuts: &'static [KeyBinding],
    pub args: ArgKind,
    pub can_execute: EnablementFn,
    pub checked: Option<CheckedFn>,
    pub run: CommandFn,
}

impl Command {
    pub fn primary_shortcut(&self) -> Option<KeyBinding> {
        self.shortcuts.first().copied()
    }

    pub fn enablement(&self, state: &AppState) -> Enablement {
        (self.can_execute)(state)
    }
}

impl std::fmt::Debug for Command {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Command")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("category", &self.category)
            .finish_non_exhaustive()
    }
}
