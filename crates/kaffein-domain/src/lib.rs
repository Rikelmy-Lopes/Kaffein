//! Domain model for Kaffein.
//!
//! This crate holds the editor document, the project tree, and the ports that
//! infrastructure implements. It deliberately has no dependency on egui, the
//! filesystem, or process execution.

pub mod document;
pub mod ports;
pub mod project;

pub use document::{Document, DocumentId, LineEnding};
pub use ports::{FileSystem, FsError, FsErrorKind, ProjectLoader};
pub use project::{ProjectEntry, ProjectTree};
