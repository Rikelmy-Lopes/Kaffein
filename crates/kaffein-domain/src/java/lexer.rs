//! Line-oriented Java lexical analysis and semantic classification for syntax highlighting.
//!
//! The lexer is a single pass over bytes and characters. It never allocates per
//! token beyond the token vector itself, and it is stateful across lines so a
//! block comment or text block can span many lines without re-scanning earlier
//! text.
//!
//! A construct that spans lines is emitted as one token segment per line. That
//! keeps the editor's line cache independent: editing one line only requires
//! re-lexing that line and the lines whose lexical state changed.

mod classify;
mod scan;

#[cfg(test)]
mod tests;

use super::keyword::is_keyword;
use scan::{
    find_block_comment_end, find_text_block_end, is_ident_start, match_operator, match_punctuation,
    next_char, scan_identifier, scan_number, scan_quoted,
};

use super::token::{Token, TokenKind};

/// The lexical state at the start or end of a line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LexState {
    #[default]
    Normal,
    BlockComment {
        javadoc: bool,
    },
    TextBlock,
}

/// Tokens for one line, with offsets relative to the line start.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LineTokens {
    pub tokens: Vec<Token>,
    pub end_state: LexState,
}

/// Tokenizes a complete source file.
///
/// Offsets in the returned tokens are absolute byte offsets into `source`.
/// Multi-line constructs produce one token per line.
pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut state = LexState::Normal;
    let mut line_start = 0;

    for line in source.split_inclusive('\n') {
        let content = line.strip_suffix('\n').unwrap_or(line);
        let lexed = tokenize_line(content, state);
        tokens.extend(
            lexed.tokens.into_iter().map(|token| {
                Token::new(token.kind, line_start + token.start, line_start + token.end)
            }),
        );
        state = lexed.end_state;
        line_start += line.len();
    }

    tokens
}

/// Tokenizes one line, carrying the state in from the previous line.
pub fn tokenize_line(line: &str, state: LexState) -> LineTokens {
    let mut tokens = Vec::new();
    let mut state = state;
    let mut pos = 0;

    if let LexState::BlockComment { javadoc } = state {
        let kind = if javadoc {
            TokenKind::JavadocComment
        } else {
            TokenKind::BlockComment
        };
        match find_block_comment_end(line, 0) {
            Some(end) => {
                push_token(&mut tokens, kind, 0, end);
                pos = end;
                state = LexState::Normal;
            }
            None => {
                push_token(&mut tokens, kind, 0, line.len());
                return finish(line, tokens, state);
            }
        }
    }

    if state == LexState::TextBlock {
        match find_text_block_end(line, 0) {
            Some(end) => {
                push_token(&mut tokens, TokenKind::String, 0, end);
                pos = end;
                state = LexState::Normal;
            }
            None => {
                push_token(&mut tokens, TokenKind::String, 0, line.len());
                return finish(line, tokens, state);
            }
        }
    }

    let bytes = line.as_bytes();
    while pos < bytes.len() {
        let Some((ch, width)) = next_char(line, pos) else {
            break;
        };

        if ch.is_whitespace() {
            pos += width;
            continue;
        }

        if ch == '/' {
            if bytes.get(pos + 1) == Some(&b'/') {
                push_token(&mut tokens, TokenKind::LineComment, pos, line.len());
                break;
            }
            if bytes.get(pos + 1) == Some(&b'*') {
                let javadoc = is_javadoc_start(line, pos);
                let kind = if javadoc {
                    TokenKind::JavadocComment
                } else {
                    TokenKind::BlockComment
                };
                match find_block_comment_end(line, pos + 2) {
                    Some(end) => {
                        push_token(&mut tokens, kind, pos, end);
                        pos = end;
                    }
                    None => {
                        push_token(&mut tokens, kind, pos, line.len());
                        state = LexState::BlockComment { javadoc };
                        break;
                    }
                }
                continue;
            }
        }

        if ch == '"' {
            if bytes.get(pos + 1) == Some(&b'"') && bytes.get(pos + 2) == Some(&b'"') {
                match find_text_block_end(line, pos + 3) {
                    Some(end) => {
                        push_token(&mut tokens, TokenKind::String, pos, end);
                        pos = end;
                    }
                    None => {
                        push_token(&mut tokens, TokenKind::String, pos, line.len());
                        state = LexState::TextBlock;
                        break;
                    }
                }
                continue;
            }
            let end = scan_quoted(line, pos, b'"');
            push_token(&mut tokens, TokenKind::String, pos, end);
            pos = end;
            continue;
        }

        if ch == '\'' {
            let end = scan_quoted(line, pos, b'\'');
            push_token(&mut tokens, TokenKind::Character, pos, end);
            pos = end;
            continue;
        }

        if ch.is_ascii_digit() || (ch == '.' && bytes.get(pos + 1).is_some_and(u8::is_ascii_digit))
        {
            let end = scan_number(line, pos);
            push_token(&mut tokens, TokenKind::Number, pos, end);
            pos = end;
            continue;
        }

        if is_ident_start(ch) {
            let end = scan_identifier(line, pos);
            let kind = if is_keyword(&line[pos..end]) {
                TokenKind::Keyword
            } else {
                TokenKind::Identifier
            };
            push_token(&mut tokens, kind, pos, end);
            pos = end;
            continue;
        }

        if ch == '@' {
            if let Some((next, _)) = next_char(line, pos + 1) {
                if is_ident_start(next) {
                    let end = scan_identifier(line, pos + 1);
                    push_token(&mut tokens, TokenKind::Annotation, pos, end);
                    pos = end;
                    continue;
                }
            }
        }

        if let Some(operator) = match_operator(line, pos) {
            push_token(&mut tokens, TokenKind::Operator, pos, pos + operator.len());
            pos += operator.len();
            continue;
        }
        if let Some(punctuation) = match_punctuation(line, pos) {
            push_token(
                &mut tokens,
                TokenKind::Punctuation,
                pos,
                pos + punctuation.len(),
            );
            pos += punctuation.len();
            continue;
        }

        push_token(&mut tokens, TokenKind::Punctuation, pos, pos + width);
        pos += width;
    }

    finish(line, tokens, state)
}

fn finish(line: &str, mut tokens: Vec<Token>, end_state: LexState) -> LineTokens {
    classify::classify(line, &mut tokens);
    LineTokens { tokens, end_state }
}

fn is_javadoc_start(line: &str, pos: usize) -> bool {
    line[pos..].starts_with("/**") && !line[pos + 3..].starts_with('/')
}

fn push_token(tokens: &mut Vec<Token>, kind: TokenKind, start: usize, end: usize) {
    if end > start {
        tokens.push(Token::new(kind, start, end));
    }
}
