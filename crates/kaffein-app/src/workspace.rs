use crate::command::{CommandArgs, CommandOutcome, Enablement};
use crate::registry::CommandRegistry;
use crate::services::Services;
use crate::state::{AppState, StatusMessage, ThemePreference};

/// Owns application state, the command registry, and the injected services.
pub struct Workspace {
    state: AppState,
    registry: CommandRegistry,
    services: Services,
}

impl Workspace {
    pub fn new(services: Services, theme: ThemePreference) -> Self {
        Self {
            state: AppState::new(theme),
            registry: CommandRegistry::new(),
            services,
        }
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    /// The single dispatch path. Every entry point — menu, toolbar, shortcut,
    /// tab, or test — goes through here.
    pub fn dispatch(&mut self, id: &str, args: CommandArgs) -> CommandOutcome {
        let Some(command) = self.registry.get(id) else {
            let outcome = CommandOutcome::failed(
                crate::command::ErrorKind::Internal,
                format!("Command '{id}' is not registered"),
            );
            self.record_outcome(&outcome);
            return outcome;
        };

        if command.args.needs_path_prompt() && !matches!(args, CommandArgs::Path(_)) {
            let outcome = CommandOutcome::failed(
                crate::command::ErrorKind::Internal,
                format!("Command '{}' requires a path argument", command.id),
            );
            self.record_outcome(&outcome);
            return outcome;
        }

        let outcome = (command.run)(&mut self.state, &mut self.services, args);
        self.record_outcome(&outcome);
        outcome
    }

    pub fn can_execute(&self, id: &str) -> Enablement {
        self.registry
            .get(id)
            .map(|command| (command.can_execute)(&self.state))
            .unwrap_or_else(|| Enablement::Disabled("Unknown command"))
    }

    fn record_outcome(&mut self, outcome: &CommandOutcome) {
        match outcome {
            CommandOutcome::Success {
                message: Some(message),
            } => {
                self.state.set_status(StatusMessage::info(message.clone()));
            }
            CommandOutcome::Success { message: None } => {
                self.state.clear_status();
            }
            CommandOutcome::NoOp { reason } => {
                self.state
                    .set_status(StatusMessage::warning(reason.clone()));
            }
            CommandOutcome::Cancelled => {
                self.state.set_status(StatusMessage::warning("Cancelled"));
            }
            CommandOutcome::Failed(error) => {
                self.state.set_status(StatusMessage::error(format!(
                    "{}: {}",
                    error.kind.label(),
                    error.message
                )));
            }
        }
    }
}

#[cfg(test)]
mod tests;
