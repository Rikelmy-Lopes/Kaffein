//! Token-stream Java parser for completion.
//!
//! The parser is deliberately small and error-tolerant. It recognises type
//! declarations, members, and method-local declarations; it never aborts on
//! malformed input and never panics. Comments are removed before parsing, so
//! every token index is a significant token.

mod body;
mod declarations;

use super::ast::{JavaFile, Member, MemberKind};
use super::token::{Token, TokenKind};
use super::tokenize;

#[cfg(test)]
mod tests;

struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
}

pub fn parse(source: &str) -> JavaFile {
    let tokens = tokenize(source)
        .into_iter()
        .filter(|token| {
            !matches!(
                token.kind,
                TokenKind::LineComment | TokenKind::BlockComment | TokenKind::JavadocComment
            )
        })
        .collect();
    Parser { source, tokens }.parse_file()
}

impl<'a> Parser<'a> {
    fn text(&self, index: usize) -> &'a str {
        self.tokens
            .get(index)
            .map_or("", |token| &self.source[token.start..token.end])
    }

    fn same(&self, index: usize, text: &str) -> bool {
        self.tokens
            .get(index)
            .is_some_and(|token| &self.source[token.start..token.end] == text)
    }

    fn is_word(&self, index: usize) -> bool {
        self.tokens.get(index).is_some_and(|token| {
            matches!(
                token.kind,
                TokenKind::Identifier
                    | TokenKind::Type
                    | TokenKind::Method
                    | TokenKind::Field
                    | TokenKind::Constant
            )
        })
    }

    fn is_type_start(&self, index: usize) -> bool {
        matches!(
            self.text(index),
            "class" | "interface" | "enum" | "record" | "@interface"
        )
    }

    fn is_type_token(&self, index: usize) -> bool {
        self.is_word(index)
            || matches!(
                self.text(index),
                "void"
                    | "boolean"
                    | "byte"
                    | "short"
                    | "int"
                    | "long"
                    | "char"
                    | "float"
                    | "double"
                    | "var"
            )
    }

    fn find(&self, mut index: usize, end: usize, text: &str) -> Option<usize> {
        while index < end {
            if self.same(index, text) {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    fn matching(&self, open: usize, open_text: &str, close_text: &str) -> Option<usize> {
        let mut depth = 0usize;
        for index in open..self.tokens.len() {
            if self.same(index, open_text) {
                depth += 1;
            } else if self.same(index, close_text) {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
        }
        None
    }

    fn skip_modifiers(&self, mut index: usize) -> (usize, bool, bool) {
        let mut is_static = false;
        let mut is_final = false;
        loop {
            if self.tokens.get(index).map(|token| token.kind) == Some(TokenKind::Annotation)
                && self.text(index) != "@interface"
            {
                index += 1;
                if self.same(index, "(") {
                    index = self
                        .matching(index, "(", ")")
                        .map_or(index + 1, |end| end + 1);
                }
                continue;
            }
            match self.text(index) {
                "public" | "protected" | "private" | "abstract" | "native" | "synchronized"
                | "transient" | "volatile" | "strictfp" | "default" | "sealed" => index += 1,
                "static" => {
                    is_static = true;
                    index += 1;
                }
                "final" => {
                    is_final = true;
                    index += 1;
                }
                _ => break,
            }
        }
        (index, is_static, is_final)
    }

    fn parse_declaration(
        &self,
        start: usize,
        end: usize,
        is_static: bool,
        is_final: bool,
    ) -> Option<(Vec<Member>, usize)> {
        if self.is_word(start) && self.same(start + 1, "(") {
            let member = self.parse_callable(start, None, start + 1, is_static, is_final)?;
            return Some((vec![member.0], member.1));
        }
        if !self.is_type_token(start) {
            return None;
        }
        let (name_index, mut type_name) = self.find_name(start, end)?;
        if self.same(name_index + 1, "(") {
            let member = self.parse_callable(
                name_index,
                Some(type_name),
                name_index + 1,
                is_static,
                is_final,
            )?;
            return Some((vec![member.0], member.1));
        }
        let extras = self.parse_array_suffix(name_index + 1);
        type_name.push_str(&extras.0);
        let mut names = vec![name_index];
        let mut cursor = extras.1;
        let mut depth = 0usize;
        while cursor < end {
            if depth == 0 && self.same(cursor, ";") {
                cursor += 1;
                break;
            }
            if depth == 0 && self.same(cursor, ",") && self.is_word(cursor + 1) {
                names.push(cursor + 1);
                let suffix = self.parse_array_suffix(cursor + 2);
                cursor = suffix.1;
                continue;
            }
            if matches!(self.text(cursor), "(" | "{" | "[") {
                depth += 1;
            } else if matches!(self.text(cursor), ")" | "}" | "]") {
                depth = depth.saturating_sub(1);
            }
            cursor += 1;
        }
        let members = names
            .into_iter()
            .map(|index| Member {
                name: self.text(index).to_owned(),
                kind: MemberKind::Field,
                type_name: Some(type_name.clone()),
                params: Vec::new(),
                is_static,
                is_final,
                body: None,
                locals: Vec::new(),
            })
            .collect();
        Some((members, cursor))
    }

    fn parse_callable(
        &self,
        name_index: usize,
        type_name: Option<String>,
        params_open: usize,
        is_static: bool,
        is_final: bool,
    ) -> Option<(Member, usize)> {
        let params = self.parse_params(params_open)?;
        let params_close = self.matching(params_open, "(", ")")?;
        let mut cursor = params_close + 1;
        while cursor < self.tokens.len() && !self.same(cursor, "{") && !self.same(cursor, ";") {
            cursor += 1;
        }
        let (body, locals, next) = if self.same(cursor, "{") {
            let close = self.matching(cursor, "{", "}")?;
            let locals = self.parse_locals(cursor, close, &params);
            (
                Some(self.tokens[cursor].start..self.tokens[close].end),
                locals,
                close + 1,
            )
        } else {
            (None, Vec::new(), cursor + 1)
        };
        Some((
            Member {
                name: self.text(name_index).to_owned(),
                kind: if type_name.is_some() {
                    MemberKind::Method
                } else {
                    MemberKind::Constructor
                },
                type_name,
                params,
                is_static,
                is_final,
                body,
                locals,
            },
            next,
        ))
    }

    fn skip_statement(&self, mut index: usize, end: usize) -> usize {
        let mut depth = 0usize;
        while index < end {
            if matches!(self.text(index), "(" | "{" | "[") {
                depth += 1;
            } else if matches!(self.text(index), ")" | "}" | "]") {
                if depth == 0 {
                    return index;
                }
                depth -= 1;
            } else if self.same(index, ";") && depth == 0 {
                return index + 1;
            }
            index += 1;
        }
        end
    }
}
