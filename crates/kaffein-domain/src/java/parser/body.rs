//! Declaration, parameter, and local-variable scanning for the Java parser.

use super::Parser;
use crate::java::ast::{Local, LocalKind, Param};

impl Parser<'_> {
    /// The declared name and its type text.
    ///
    /// Candidate names are word tokens at angle-bracket depth zero that are
    /// followed by a call parenthesis or by a declarator terminator.
    pub(super) fn find_name(&self, start: usize, end: usize) -> Option<(usize, String)> {
        let mut angle = 0i32;
        for index in start..end {
            match self.text(index) {
                "<" => {
                    angle += 1;
                    continue;
                }
                ">" => {
                    angle = (angle - 1).max(0);
                    continue;
                }
                ">>" => {
                    angle = (angle - 2).max(0);
                    continue;
                }
                ">>>" => {
                    angle = (angle - 3).max(0);
                    continue;
                }
                _ => {}
            }
            if angle > 0 || !self.is_word(index) {
                continue;
            }
            let next = index + 1;
            let terminator = matches!(self.text(next), "=" | ";" | "," | ":" | ")");
            let array_name = self.same(next, "[")
                && self.same(next + 1, "]")
                && matches!(self.text(next + 2), "=" | ";" | "," | ":" | ")");
            if self.same(next, "(") || terminator || array_name || self.same(next, "...") {
                let type_name = self.source[self.tokens[start].start..self.tokens[index].start]
                    .trim()
                    .to_owned();
                if !type_name.is_empty() && index > start {
                    return Some((index, type_name));
                }
            }
        }
        None
    }

    pub(super) fn parse_array_suffix(&self, start: usize) -> (String, usize) {
        let mut suffix = String::new();
        let mut index = start;
        while self.same(index, "[") && self.same(index + 1, "]") {
            suffix.push_str("[]");
            index += 2;
        }
        (suffix, index)
    }

    pub(super) fn parse_params(&self, open: usize) -> Option<Vec<Param>> {
        let close = self.matching(open, "(", ")")?;
        let mut params = Vec::new();
        let mut start = open + 1;
        let mut depth = 0i32;
        for index in open + 1..=close {
            if index < close && matches!(self.text(index), "(" | "[" | "<") {
                depth += 1;
            } else if index < close && matches!(self.text(index), ")" | "]" | ">") {
                depth = (depth - 1).max(0);
            } else if index < close && matches!(self.text(index), ">>" | ">>>") {
                depth = (depth - 2).max(0);
            }
            let separator = index == close || (depth == 0 && self.same(index, ","));
            if separator {
                if let Some(param) = self.parse_param(start, index) {
                    params.push(param);
                }
                start = index + 1;
            }
        }
        Some(params)
    }

    fn parse_param(&self, start: usize, end: usize) -> Option<Param> {
        let (start, _, _) = self.skip_modifiers(start);
        if start >= end {
            return None;
        }
        let name_index = (start..end).rev().find(|&index| self.is_word(index))?;
        if name_index <= start {
            return None;
        }
        let mut type_name = self.source[self.tokens[start].start..self.tokens[name_index].start]
            .trim()
            .to_owned();
        if type_name.ends_with("...") {
            type_name.truncate(type_name.len() - 3);
            type_name.push_str("[]");
        }
        let suffix = self.parse_array_suffix(name_index + 1);
        type_name.push_str(&suffix.0);
        Some(Param {
            name: self.text(name_index).to_owned(),
            type_name,
        })
    }

    pub(super) fn parse_locals(&self, open: usize, close: usize, params: &[Param]) -> Vec<Local> {
        let body_start = self.tokens[open].end;
        let body_end = self.tokens[close].start;
        let mut locals: Vec<Local> = params
            .iter()
            .map(|param| Local {
                name: param.name.clone(),
                type_name: param.type_name.clone(),
                kind: LocalKind::Parameter,
                visible_from: body_start,
                scope_end: body_end,
            })
            .collect();
        let mut blocks: Vec<usize> = Vec::new();
        let mut index = open + 1;
        while index < close {
            if self.same(index, "{") {
                blocks.push(index);
                index += 1;
                continue;
            }
            if self.same(index, "}") {
                blocks.pop();
                index += 1;
                continue;
            }
            if !self.is_statement_start(open, index) {
                index += 1;
                continue;
            }
            let (start, _, _) = self.skip_modifiers(index);
            if start >= close || !self.is_type_token(start) {
                index += 1;
                continue;
            }
            let Some((name_index, mut type_name)) = self.find_name(start, close) else {
                index += 1;
                continue;
            };
            if name_index == start || self.same(name_index + 1, "(") {
                index += 1;
                continue;
            }
            let suffix = self.parse_array_suffix(name_index + 1);
            type_name.push_str(&suffix.0);
            if type_name == "var" {
                type_name = self.infer_var_type(name_index, close);
            }
            let scope_end = blocks
                .last()
                .and_then(|&block| self.matching(block, "{", "}"))
                .map_or(body_end, |block| self.tokens[block].start);
            locals.push(Local {
                name: self.text(name_index).to_owned(),
                type_name,
                kind: LocalKind::Variable,
                visible_from: self.tokens[name_index].end,
                scope_end,
            });
            index += 1;
        }
        locals
    }

    fn is_statement_start(&self, open: usize, index: usize) -> bool {
        if index == open + 1 {
            return true;
        }
        matches!(
            self.text(index.saturating_sub(1)),
            "{" | "}" | ";" | "(" | "," | ":"
        )
    }

    fn infer_var_type(&self, name_index: usize, end: usize) -> String {
        let mut index = name_index + 1;
        while index < end && !self.same(index, "=") && !self.same(index, ";") {
            index += 1;
        }
        if !self.same(index, "=") {
            return "var".to_owned();
        }
        index += 1;
        if !self.same(index, "new") {
            return "var".to_owned();
        }
        let (name, _) = self.parse_qualified(index + 1, end);
        if name.is_empty() {
            "var".to_owned()
        } else {
            name
        }
    }
}
