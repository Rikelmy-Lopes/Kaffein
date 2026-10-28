//! Presentation-only state. It is not part of the application model.

use kaffein_app::{AppState, Command, CommandOutcome, ThemePreference};
use kaffein_i18n::Localizer;

use crate::bottom_panel::BottomTab;
use crate::editor::EditorBuffer;
use crate::path_prompt::PathPrompt;
use crate::syntax::HighlightCache;

#[derive(Debug, Default)]
pub struct UiState {
    pub(crate) prompt: Option<PathPrompt>,
    pub(crate) bottom_tab: BottomTab,
    pub(crate) editor: EditorBuffer,
    pub(crate) syntax: HighlightCache,
    pub(crate) applied_theme: Option<ThemePreference>,
    pub(crate) seen_run_id: u64,
}

impl UiState {
    /// Called by the binary after dispatching an action, so a failed path
    /// prompt can stay open and show the error.
    pub fn after_dispatch(&mut self, command: &str, outcome: &CommandOutcome) {
        let Some(prompt) = self.prompt.as_mut() else {
            return;
        };
        if prompt.command != command {
            return;
        }
        match outcome {
            CommandOutcome::Success { .. } | CommandOutcome::Cancelled => {
                self.prompt = None;
            }
            CommandOutcome::Failed(_) | CommandOutcome::NoOp { .. } => {
                prompt.error = outcome.message().cloned();
            }
        }
    }

    pub fn prompt_is_open(&self) -> bool {
        self.prompt.is_some()
    }

    pub(crate) fn open_prompt(
        &mut self,
        command: Command,
        state: &AppState,
        localizer: &Localizer,
    ) {
        self.prompt = Some(PathPrompt::for_command(command, state, localizer));
    }
}
