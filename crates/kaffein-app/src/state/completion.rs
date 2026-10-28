//! The active completion list stored in application state.

use kaffein_domain::{Completion, CompletionItem, DocumentId};

/// Suggestions currently offered for one document and one buffer range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionSession {
    pub document: DocumentId,
    pub start: usize,
    pub end: usize,
    pub items: Vec<CompletionItem>,
    pub selected: usize,
}

impl CompletionSession {
    pub fn new(document: DocumentId, completion: Completion) -> Self {
        Self {
            document,
            start: completion.start,
            end: completion.end,
            items: completion.items,
            selected: 0,
        }
    }

    pub fn selected_item(&self) -> Option<&CompletionItem> {
        self.items.get(self.selected)
    }

    pub fn select_delta(&mut self, delta: isize) {
        if self.items.is_empty() {
            return;
        }
        let len = self.items.len() as isize;
        self.selected = ((self.selected as isize + delta).rem_euclid(len)) as usize;
    }

    pub fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = index;
        }
    }
}
