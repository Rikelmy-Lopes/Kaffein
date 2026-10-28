//! Java syntax highlighting: token colours, incremental cache, and layout.
//!
//! Tokenization itself lives in `kaffein-domain`; this module only maps token
//! kinds to colours and turns cached tokens into egui layouts.

mod buffer;
mod cache;
mod layout;
mod theme;

#[cfg(test)]
mod tests;

pub(crate) use buffer::TrackedBuffer;
pub(crate) use cache::HighlightCache;
pub use theme::SyntaxTheme;
