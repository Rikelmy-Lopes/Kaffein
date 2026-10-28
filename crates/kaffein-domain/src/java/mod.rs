//! Java language support.
//!
//! This module owns lexical analysis and line-local semantic classification for
//! syntax highlighting. It has no knowledge of colours, egui, or files; it turns
//! source text into token ranges.

mod lexer;
mod name;
mod token;

pub use lexer::{tokenize, tokenize_line, LexState, LineTokens};
pub use name::{package_name, primary_type_name};
pub use token::{Token, TokenKind};
