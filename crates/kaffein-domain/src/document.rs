//! In-memory document and its undo history.
//!
//! The buffer is normalised to `\n`; [`LineEnding`] records the convention the
//! file was loaded with and is restored when the document is written back.
//! Only UTF-8 is supported. Encoding conversion is therefore never silent: a
//! file that is not valid UTF-8 fails to open.

mod line_ending;

#[cfg(test)]
mod tests;

pub use line_ending::LineEnding;

use std::path::{Path, PathBuf};

/// How long consecutive single-character edits are grouped into one undo step.
const COALESCE_WINDOW_MS: u64 = 600;

/// File name shown for a document that has no path on disk.
const SCRATCH_DISPLAY_NAME: &str = "Scratch.java";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DocumentId(pub u64);

#[derive(Clone, Debug)]
struct Snapshot {
    text: String,
    at_ms: u64,
}

#[derive(Debug)]
pub struct Document {
    id: DocumentId,
    /// `None` for an in-memory scratch document that has never been saved.
    path: Option<PathBuf>,
    buffer: String,
    /// `None` until the document has been loaded from or written to disk.
    saved_text: Option<String>,
    line_ending: LineEnding,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    revision: u64,
    last_edit_ms: Option<u64>,
}

impl Document {
    /// Creates a document from text that was just read from disk.
    pub fn from_loaded(id: DocumentId, path: impl Into<PathBuf>, raw: &str) -> Self {
        let line_ending = LineEnding::detect(raw);
        let buffer = normalize_newlines(raw);
        Self {
            id,
            path: Some(path.into()),
            saved_text: Some(buffer.clone()),
            buffer,
            line_ending,
            undo: Vec::new(),
            redo: Vec::new(),
            revision: 0,
            last_edit_ms: None,
        }
    }

    /// Creates an in-memory scratch document with no path on disk.
    ///
    /// The default text is treated as the scratch document's saved content so
    /// the tab starts clean. Edits mark it dirty, but it is disposable: closing
    /// it discards the buffer instead of asking for a path.
    pub fn scratch(id: DocumentId, text: impl Into<String>) -> Self {
        let raw = text.into();
        let buffer = normalize_newlines(&raw);
        Self {
            id,
            path: None,
            saved_text: Some(buffer.clone()),
            buffer,
            line_ending: LineEnding::Lf,
            undo: Vec::new(),
            redo: Vec::new(),
            revision: 0,
            last_edit_ms: None,
        }
    }

    /// Creates an empty document that does not exist on disk yet.
    ///
    /// It is dirty from the start so the first save writes it, even when the
    /// user has not typed anything.
    pub fn new(id: DocumentId, path: impl Into<PathBuf>) -> Self {
        Self {
            id,
            path: Some(path.into()),
            buffer: String::new(),
            saved_text: None,
            line_ending: LineEnding::Lf,
            undo: Vec::new(),
            redo: Vec::new(),
            revision: 0,
            last_edit_ms: None,
        }
    }

    pub fn id(&self) -> DocumentId {
        self.id
    }

    /// The backing file path, or `None` for a scratch document.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn is_scratch(&self) -> bool {
        self.path.is_none()
    }

    pub fn display_name(&self) -> String {
        self.path
            .as_deref()
            .map(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.to_string_lossy().into_owned())
            })
            .unwrap_or_else(|| SCRATCH_DISPLAY_NAME.to_owned())
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    pub fn line_count(&self) -> usize {
        self.buffer.bytes().filter(|byte| *byte == b'\n').count() + 1
    }

    pub fn is_dirty(&self) -> bool {
        self.saved_text.as_deref() != Some(self.buffer.as_str())
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Replaces the whole buffer, recording an undo step.
    ///
    /// Consecutive single-character edits within [`COALESCE_WINDOW_MS`] share
    /// one undo step so that typing produces one undo per pause, not one per
    /// keystroke. Any other replacement, or an edit after a pause, starts a new
    /// step.
    pub fn replace_text(&mut self, text: impl Into<String>, at_ms: u64) -> bool {
        let text = text.into();
        if text == self.buffer {
            return false;
        }

        let coalesce = self.last_edit_ms.is_some_and(|previous_ms| {
            at_ms.saturating_sub(previous_ms) <= COALESCE_WINDOW_MS
                && is_single_char_edit(&self.buffer, &text)
        });

        if !coalesce {
            self.undo.push(Snapshot {
                text: std::mem::replace(&mut self.buffer, text),
                at_ms,
            });
        } else {
            self.buffer = text;
        }

        self.redo.clear();
        self.last_edit_ms = Some(at_ms);
        self.revision += 1;
        true
    }

    /// Restores the previous snapshot. Returns `false` when there is nothing
    /// to undo.
    pub fn undo(&mut self) -> bool {
        let Some(snapshot) = self.undo.pop() else {
            return false;
        };
        let current = std::mem::replace(&mut self.buffer, snapshot.text);
        self.redo.push(Snapshot {
            text: current,
            at_ms: snapshot.at_ms,
        });
        self.last_edit_ms = None;
        self.revision += 1;
        true
    }

    /// Reapplies the last undone snapshot.
    pub fn redo(&mut self) -> bool {
        let Some(snapshot) = self.redo.pop() else {
            return false;
        };
        let current = std::mem::replace(&mut self.buffer, snapshot.text);
        self.undo.push(Snapshot {
            text: current,
            at_ms: snapshot.at_ms,
        });
        self.last_edit_ms = None;
        self.revision += 1;
        true
    }

    /// Marks the current buffer as the saved content. The file itself is
    /// written by infrastructure; this method only updates domain state.
    pub fn mark_saved(&mut self) {
        self.saved_text = Some(self.buffer.clone());
    }

    /// Text to hand to the file system, with the original line ending restored.
    pub fn text_for_save(&self) -> String {
        match self.line_ending {
            LineEnding::Lf => self.buffer.clone(),
            LineEnding::Crlf => self.buffer.replace('\n', "\r\n"),
        }
    }
}

fn normalize_newlines(raw: &str) -> String {
    raw.replace("\r\n", "\n").replace('\r', "\n")
}

fn is_single_char_edit(previous: &str, next: &str) -> bool {
    if previous.len().abs_diff(next.len()) != 1 {
        return false;
    }
    let (shorter, longer) = if previous.len() < next.len() {
        (previous, next)
    } else {
        (next, previous)
    };
    let mut prefix = 0;
    while prefix < shorter.len() && shorter.as_bytes()[prefix] == longer.as_bytes()[prefix] {
        prefix += 1;
    }
    shorter[prefix..] == longer[prefix + 1..]
}
