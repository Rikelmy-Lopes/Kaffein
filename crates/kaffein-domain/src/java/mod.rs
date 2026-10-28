//! Java language support.
//!
//! This module owns lexical analysis and line-local semantic classification for
//! syntax highlighting. It has no knowledge of colours, egui, or files; it turns
//! source text into token ranges.

pub mod ast;
pub mod completion;
pub mod index;
mod keyword;
mod lexer;
mod name;
pub mod parser;
mod token;

pub use ast::JavaFile;
pub use completion::{complete, Completion, CompletionItem, CompletionKind};
pub use index::{IndexedFile, IndexedMember, IndexedType, SymbolIndex};
pub use lexer::{tokenize, tokenize_line, LexState, LineTokens};
pub use name::{package_name, primary_type_name};
pub use parser::parse;
pub use token::{Token, TokenKind};
