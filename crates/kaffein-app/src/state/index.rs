//! AppState accessors for the symbol index and completion session.

use kaffein_domain::SymbolIndex;

use super::{AppState, CompletionSession};

impl AppState {
    pub fn index(&self) -> &SymbolIndex {
        &self.index
    }

    pub(crate) fn index_mut(&mut self) -> &mut SymbolIndex {
        &mut self.index
    }

    pub fn completion(&self) -> Option<&CompletionSession> {
        self.completion.as_ref()
    }

    pub(crate) fn set_completion(&mut self, session: CompletionSession) {
        self.completion = Some(session);
    }

    pub(crate) fn completion_mut(&mut self) -> Option<&mut CompletionSession> {
        self.completion.as_mut()
    }

    pub(crate) fn clear_completion(&mut self) {
        self.completion = None;
    }
}
