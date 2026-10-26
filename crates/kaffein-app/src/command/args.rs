//! Arguments passed to a command.

use std::path::PathBuf;

use kaffein_domain::DocumentId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandArgs {
    None,
    Path(PathBuf),
    Document(DocumentId),
    Locale(String),
    ReplaceText {
        document: DocumentId,
        text: String,
        at_ms: u64,
    },
}

impl CommandArgs {
    pub fn none() -> Self {
        Self::None
    }

    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self::Path(path.into())
    }
}

/// The argument a command needs from a user-facing dialog, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgKind {
    None,
    NewFile,
    File,
    Directory,
    Locale,
    Text,
}

impl ArgKind {
    pub fn needs_path_prompt(self) -> bool {
        matches!(self, Self::NewFile | Self::File | Self::Directory)
    }
}
