//! Caret context: the word being typed and the expression before a dot.

use crate::java::{tokenize, Token, TokenKind};

pub(super) struct Context {
    /// Byte offset where the typed prefix starts.
    pub(super) start: usize,
    pub(super) prefix: String,
    /// The expression before a `.` when completion follows a receiver.
    pub(super) receiver: Option<String>,
}

pub(super) fn analyze(source: &str, caret: usize) -> Option<Context> {
    let tokens = tokenize(source);
    if is_suppressed(&tokens, caret) {
        return None;
    }

    let mut start = caret;
    while start > 0 {
        let ch = source[..start].chars().next_back()?;
        if is_identifier_continue(ch) {
            start -= ch.len_utf8();
        } else {
            break;
        }
    }
    let prefix = source[start..caret].to_owned();
    if prefix.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        return None;
    }

    let receiver = receiver_before(source, start);
    if prefix.is_empty() && receiver.is_none() {
        return None;
    }
    Some(Context {
        start,
        prefix,
        receiver,
    })
}

fn is_suppressed(tokens: &[Token], caret: usize) -> bool {
    tokens.iter().any(|token| {
        let inside = token.start < caret && caret <= token.end;
        inside
            && matches!(
                token.kind,
                TokenKind::LineComment
                    | TokenKind::BlockComment
                    | TokenKind::JavadocComment
                    | TokenKind::String
                    | TokenKind::Character
            )
    })
}

fn receiver_before(source: &str, prefix_start: usize) -> Option<String> {
    let mut cursor = prefix_start;
    while cursor > 0 {
        let ch = source[..cursor].chars().next_back()?;
        if ch.is_whitespace() {
            cursor -= ch.len_utf8();
        } else {
            break;
        }
    }
    if cursor == 0 || source[..cursor].chars().next_back()? != '.' {
        return None;
    }
    let dot = cursor - 1;
    let mut start = dot;
    while start > 0 {
        let ch = source[..start].chars().next_back()?;
        if is_expression_char(ch) {
            start -= ch.len_utf8();
        } else {
            break;
        }
    }
    let receiver = source[start..dot].trim();
    (!receiver.is_empty()).then(|| receiver.to_owned())
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_alphanumeric()
}

fn is_expression_char(ch: char) -> bool {
    is_identifier_continue(ch)
        || matches!(
            ch,
            '.' | '(' | ')' | '[' | ']' | '<' | '>' | '?' | ',' | ' ' | '\t'
        )
}
