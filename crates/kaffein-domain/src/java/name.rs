//! Source identity helpers used when a buffer is handed to `javac`.
//!
//! The compiler needs a file name matching the public type and a fully
//! qualified class name for the launcher. Both are read from the token stream
//! rather than guessed from the first line.

#[cfg(test)]
mod tests;

use super::{tokenize, Token, TokenKind};

/// The type named by the first public type declaration, or the first type
/// declaration when none is public.
pub fn primary_type_name(source: &str) -> Option<&str> {
    let tokens = tokenize(source);
    let mut fallback = None;

    for (index, token) in tokens.iter().enumerate() {
        if !is_type_declaration(token, source) {
            continue;
        }
        let Some(next) = tokens.get(index + 1) else {
            continue;
        };
        if next.kind != TokenKind::Type {
            continue;
        }

        let name = token_text(source, next);
        if is_public_declaration(&tokens, index, source) {
            return Some(name);
        }
        fallback.get_or_insert(name);
    }

    fallback
}

/// The package declared by the source, if any.
pub fn package_name(source: &str) -> Option<String> {
    let tokens = tokenize(source);
    let start = tokens.iter().position(|token| {
        token.kind == TokenKind::Keyword && token_text(source, token) == "package"
    })?;

    let mut name = String::new();
    for token in &tokens[start + 1..] {
        match token.kind {
            TokenKind::Identifier
            | TokenKind::Type
            | TokenKind::Method
            | TokenKind::Field
            | TokenKind::Constant => name.push_str(token_text(source, token)),
            TokenKind::Operator | TokenKind::Punctuation if token_text(source, token) == "." => {
                name.push('.')
            }
            TokenKind::Punctuation if token_text(source, token) == ";" => {
                return (!name.is_empty()).then_some(name);
            }
            _ => {}
        }
    }

    None
}

fn is_type_declaration(token: &Token, source: &str) -> bool {
    match token.kind {
        TokenKind::Keyword => matches!(
            token_text(source, token),
            "class" | "interface" | "enum" | "record"
        ),
        TokenKind::Annotation => token_text(source, token) == "@interface",
        _ => false,
    }
}

fn is_public_declaration(tokens: &[Token], index: usize, source: &str) -> bool {
    for token in tokens[..index].iter().rev() {
        match token.kind {
            TokenKind::Punctuation if matches!(token_text(source, token), ";" | "{" | "}") => {
                return false;
            }
            TokenKind::Keyword if token_text(source, token) == "public" => return true,
            _ => {}
        }
    }

    false
}

fn token_text<'source>(source: &'source str, token: &Token) -> &'source str {
    &source[token.start..token.end]
}
