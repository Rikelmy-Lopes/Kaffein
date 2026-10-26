//! The single registry of commands.
//!
//! The registry owns one [`Command`] descriptor per action and answers the
//! questions every entry point asks: which commands belong to a menu, which
//! appear in the toolbar, which have shortcuts, and whether a command is
//! currently enabled. The command table lives in `catalog`; enablement
//! predicates live in `enablers`.

mod catalog;
mod enablers;

use crate::command::{Command, Menu};

pub struct CommandRegistry {
    commands: Vec<Command>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: catalog::commands(),
        }
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn get(&self, id: &str) -> Option<Command> {
        self.commands
            .iter()
            .copied()
            .find(|command| command.id == id)
    }

    pub fn menu_commands(&self, menu: Menu) -> Vec<Command> {
        let mut commands: Vec<Command> = self
            .commands
            .iter()
            .copied()
            .filter(|command| command.menu.is_some_and(|placement| placement.menu == menu))
            .collect();
        commands.sort_by_key(|command| command.menu.map(|placement| placement.order));
        commands
    }

    pub fn toolbar_commands(&self) -> Vec<Command> {
        let mut commands: Vec<Command> = self
            .commands
            .iter()
            .copied()
            .filter(|command| command.toolbar.is_some())
            .collect();
        commands.sort_by_key(|command| command.toolbar.map(|placement| placement.order));
        commands
    }

    pub fn with_shortcuts(&self) -> impl Iterator<Item = Command> + '_ {
        self.commands
            .iter()
            .copied()
            .filter(|command| !command.shortcuts.is_empty())
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
