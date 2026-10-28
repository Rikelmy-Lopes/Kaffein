//! Context-aware Java completion.
//!
//! The engine is deliberately independent of the UI and of files: it receives
//! the current buffer, a caret byte offset, and a [`SymbolIndex`]. It parses the
//! buffer afresh so unsaved declarations and locals participate immediately.
//! Comments and string literals suppress completion.

mod context;
mod model;
mod resolve;

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use self::context::analyze;
use super::index::SymbolIndex;
use super::parser::parse;

/// The category of a suggestion. The UI renders it; no colour is chosen here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CompletionKind {
    Keyword,
    Type,
    Method,
    Field,
    Variable,
    EnumConstant,
}

/// One suggestion, ready to insert and display.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    pub detail: String,
    pub insert_text: String,
    pub kind: CompletionKind,
}

/// A completion result. `start..end` is the byte range the inserted text
/// replaces; `end` is the caret position that was queried.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    pub start: usize,
    pub end: usize,
    pub items: Vec<CompletionItem>,
}

/// Computes suggestions at `caret`, or `None` when there is nothing to offer.
pub fn complete(source: &str, caret: usize, index: &SymbolIndex) -> Option<Completion> {
    let caret = clamp_to_boundary(source, caret);
    let context = analyze(source, caret)?;
    let file = parse(source);
    let mut items = match context.receiver.as_deref() {
        Some(receiver) => resolve::receiver_symbols(&file, caret, receiver, index),
        None => resolve::visible_symbols(&file, caret, index),
    };
    let prefix = context.prefix.to_lowercase();
    items.retain(|item| item.label.to_lowercase().starts_with(&prefix));
    deduplicate(&mut items);
    items.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.label.to_lowercase().cmp(&right.label.to_lowercase()))
            .then_with(|| left.label.cmp(&right.label))
    });
    if items.is_empty() {
        return None;
    }
    Some(Completion {
        start: context.start,
        end: caret,
        items,
    })
}

fn deduplicate(items: &mut Vec<CompletionItem>) {
    let mut seen = HashSet::new();
    items.retain(|item| seen.insert((item.label.clone(), item.kind)));
}

fn clamp_to_boundary(source: &str, caret: usize) -> usize {
    let mut caret = caret.min(source.len());
    while caret > 0 && !source.is_char_boundary(caret) {
        caret -= 1;
    }
    caret
}
