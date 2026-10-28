//! Domain model for Kaffein.
//!
//! This crate holds the editor document, the project tree, and the ports that
//! infrastructure implements. It deliberately has no dependency on egui, the
//! filesystem, or process execution.

pub mod document;
pub mod java;
pub mod ports;
pub mod project;

pub use document::{Document, DocumentId, LineEnding};
pub use java::{
    package_name, primary_type_name, tokenize, tokenize_line, LexState, LineTokens, Token,
    TokenKind,
};
pub use ports::{
    FileSystem, FsError, FsErrorKind, JavaRunError, JavaRunErrorKind, JavaRunOutput, JavaRunStage,
    JavaRunner, ProjectLoader,
};
pub use project::{ProjectEntry, ProjectTree};
