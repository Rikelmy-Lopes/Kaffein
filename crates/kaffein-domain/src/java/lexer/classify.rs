//! Line-local semantic classification for Java identifiers.
//!
//! The scanners know characters; this pass assigns the categories the editor
//! colours differently (types, methods, fields, constants). It is deliberately
//! heuristic and never crosses a line boundary, so the incremental line cache
//! can re-lex one line without reading the rest of the file.

use super::super::token::{Token, TokenKind};

#[cfg(test)]
mod tests;

/// Retags identifier tokens in one line.
pub(super) fn classify(line: &str, tokens: &mut [Token]) {
    let package_or_import = is_package_or_import_line(line, tokens);

    for index in 0..tokens.len() {
        if tokens[index].kind != TokenKind::Identifier {
            continue;
        }
        let kind = classify_identifier(line, tokens, index, package_or_import);
        tokens[index].kind = kind;
    }
}

fn classify_identifier(
    line: &str,
    tokens: &[Token],
    index: usize,
    package_or_import: bool,
) -> TokenKind {
    let token = tokens[index];
    let word = &line[token.start..token.end];
    let previous = previous_significant(tokens, index);
    let next_index = next_significant_index(tokens, index);
    let next = next_index.map(|next_index| &tokens[next_index]);

    if previous.is_some_and(|previous| is_keyword(line, previous, "new")) {
        return TokenKind::Type;
    }

    if previous.is_some_and(|previous| is_declaration_keyword(line, previous)) {
        return TokenKind::Type;
    }

    if next.is_some_and(|next| is_punctuation(line, next, "(")) {
        if previous.is_some_and(|previous| is_punctuation(line, previous, "."))
            || previous.is_some_and(|previous| is_operator(line, previous, "::"))
        {
            return TokenKind::Method;
        }
        if starts_uppercase(word) {
            return TokenKind::Type;
        }
        return TokenKind::Method;
    }

    if next.is_some_and(|next| is_punctuation(line, next, "..."))
        || next_index.is_some_and(|next_index| is_empty_brackets(line, tokens, next_index))
    {
        return TokenKind::Type;
    }

    if starts_uppercase(word)
        && next_index.is_some_and(|next_index| is_class_literal(line, tokens, next_index))
    {
        return TokenKind::Type;
    }

    if !package_or_import && is_constant_name(word) && !next_is_identifier(next) {
        return TokenKind::Constant;
    }

    if starts_uppercase(word) {
        return TokenKind::Type;
    }

    if !package_or_import && previous.is_some_and(|previous| is_punctuation(line, previous, ".")) {
        return TokenKind::Field;
    }

    if previous.is_some_and(|previous| is_operator(line, previous, "::")) {
        return TokenKind::Method;
    }

    TokenKind::Identifier
}

fn is_package_or_import_line(line: &str, tokens: &[Token]) -> bool {
    tokens
        .iter()
        .find(|token| !is_comment(token.kind))
        .is_some_and(|token| {
            is_keyword(line, token, "package") || is_keyword(line, token, "import")
        })
}

fn previous_significant(tokens: &[Token], index: usize) -> Option<&Token> {
    tokens[..index]
        .iter()
        .rev()
        .find(|token| !is_comment(token.kind))
}

fn next_significant_index(tokens: &[Token], index: usize) -> Option<usize> {
    (index + 1..tokens.len()).find(|&next| !is_comment(tokens[next].kind))
}

fn next_significant(tokens: &[Token], index: usize) -> Option<&Token> {
    next_significant_index(tokens, index).map(|next| &tokens[next])
}

fn is_comment(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::LineComment | TokenKind::BlockComment | TokenKind::JavadocComment
    )
}

fn is_declaration_keyword(line: &str, token: &Token) -> bool {
    token.kind == TokenKind::Keyword
        && matches!(
            &line[token.start..token.end],
            "class" | "interface" | "enum" | "record"
        )
}

fn is_keyword(line: &str, token: &Token, keyword: &str) -> bool {
    token.kind == TokenKind::Keyword && &line[token.start..token.end] == keyword
}

fn is_punctuation(line: &str, token: &Token, punctuation: &str) -> bool {
    token.kind == TokenKind::Punctuation && &line[token.start..token.end] == punctuation
}

fn is_operator(line: &str, token: &Token, operator: &str) -> bool {
    token.kind == TokenKind::Operator && &line[token.start..token.end] == operator
}

fn is_empty_brackets(line: &str, tokens: &[Token], index: usize) -> bool {
    is_punctuation(line, &tokens[index], "[")
        && next_significant(tokens, index).is_some_and(|token| is_punctuation(line, token, "]"))
}

fn is_class_literal(line: &str, tokens: &[Token], index: usize) -> bool {
    is_punctuation(line, &tokens[index], ".")
        && next_significant(tokens, index).is_some_and(|token| is_keyword(line, token, "class"))
}

fn next_is_identifier(next: Option<&Token>) -> bool {
    next.is_some_and(|token| token.kind == TokenKind::Identifier)
}

fn starts_uppercase(word: &str) -> bool {
    word.chars().next().is_some_and(char::is_uppercase)
}

fn is_constant_name(word: &str) -> bool {
    word.chars().count() > 1
        && word.chars().any(char::is_uppercase)
        && word.chars().all(|ch| !ch.is_lowercase())
}
