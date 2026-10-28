//! Type, import, and member declarations for the Java parser.

use super::super::ast::{Import, JavaFile, Member, TypeDecl, TypeKind};
use super::Parser;

impl Parser<'_> {
    pub(super) fn parse_file(&self) -> JavaFile {
        let mut file = JavaFile::default();
        let mut index = 0;
        while index < self.tokens.len() {
            if self.same(index, "package") {
                let (name, next) = self.parse_qualified(index + 1, self.tokens.len());
                file.package = (!name.is_empty()).then_some(name);
                index = next;
            } else if self.same(index, "import") {
                if let Some((import, next)) = self.parse_import(index) {
                    file.imports.push(import);
                    index = next;
                } else {
                    index += 1;
                }
            } else {
                let (start, _, _) = self.skip_modifiers(index);
                if self.is_type_start(start) {
                    if let Some((declaration, next)) = self.parse_type(start) {
                        file.types.push(declaration);
                        index = next;
                        continue;
                    }
                }
                index += 1;
            }
        }
        file
    }

    pub(super) fn parse_qualified(&self, mut index: usize, end: usize) -> (String, usize) {
        let mut name = String::new();
        loop {
            if index >= end || !self.is_word(index) {
                break;
            }
            if !name.is_empty() {
                name.push('.');
            }
            name.push_str(self.text(index));
            index += 1;
            if index < end && self.same(index, ".") {
                index += 1;
            } else {
                break;
            }
        }
        (name, index)
    }

    fn parse_import(&self, index: usize) -> Option<(Import, usize)> {
        let semicolon = self.find(index, self.tokens.len(), ";")?;
        let mut path = String::new();
        let mut is_static = false;
        let mut is_wildcard = false;
        let mut cursor = index + 1;
        while cursor < semicolon {
            match self.text(cursor) {
                "static" => is_static = true,
                "." => path.push('.'),
                "*" => is_wildcard = true,
                text if self.is_word(cursor) => path.push_str(text),
                _ => {}
            }
            cursor += 1;
        }
        Some((
            Import {
                path: path.trim_end_matches('.').to_owned(),
                is_static,
                is_wildcard,
            },
            semicolon + 1,
        ))
    }

    fn parse_type(&self, index: usize) -> Option<(TypeDecl, usize)> {
        let kind = match self.text(index) {
            "class" => TypeKind::Class,
            "interface" => TypeKind::Interface,
            "enum" => TypeKind::Enum,
            "record" => TypeKind::Record,
            _ => TypeKind::Annotation,
        };
        let name_index = index + 1;
        if !self.is_word(name_index) {
            return None;
        }
        let name = self.text(name_index).to_owned();
        let open = self.find(name_index + 1, self.tokens.len(), "{")?;
        let close = self
            .matching(open, "{", "}")
            .unwrap_or(self.tokens.len().saturating_sub(1));
        let supertypes = self.parse_supertypes(name_index + 1, open);
        let (enum_constants, members_start) = if kind == TypeKind::Enum {
            self.parse_enum_constants(open + 1, close)
        } else {
            (Vec::new(), open + 1)
        };
        let (members, nested) = self.parse_members(members_start, close);
        Some((
            TypeDecl {
                name,
                kind,
                is_static: false,
                supertypes,
                enum_constants,
                members,
                nested,
                body: self.tokens[open].start..self.tokens[close].end,
            },
            close + 1,
        ))
    }

    fn parse_supertypes(&self, mut index: usize, end: usize) -> Vec<String> {
        let mut supertypes = Vec::new();
        while index < end {
            if matches!(self.text(index), "extends" | "implements" | "permits") {
                index += 1;
                loop {
                    let (name, next) = self.parse_qualified(index, end);
                    if name.is_empty() {
                        break;
                    }
                    supertypes.push(name);
                    index = next;
                    if index < end && self.same(index, ",") {
                        index += 1;
                    } else {
                        break;
                    }
                }
            } else {
                index += 1;
            }
        }
        supertypes
    }

    fn parse_enum_constants(&self, start: usize, end: usize) -> (Vec<String>, usize) {
        let mut constants = Vec::new();
        let mut depth = 0usize;
        let mut expect_name = true;
        let mut index = start;
        while index < end {
            if depth == 0 && self.same(index, ";") {
                return (constants, index + 1);
            }
            if depth == 0 && self.same(index, ",") {
                expect_name = true;
                index += 1;
                continue;
            }
            if depth == 0
                && expect_name
                && self.tokens[index].kind != super::super::token::TokenKind::Annotation
                && self.is_word(index)
            {
                constants.push(self.text(index).to_owned());
                expect_name = false;
            }
            if self.same(index, "(") || self.same(index, "{") {
                depth += 1;
            } else if self.same(index, ")") || self.same(index, "}") {
                depth = depth.saturating_sub(1);
            }
            index += 1;
        }
        (constants, end)
    }

    fn parse_members(&self, start: usize, end: usize) -> (Vec<Member>, Vec<TypeDecl>) {
        let mut members = Vec::new();
        let mut nested = Vec::new();
        let mut index = start;
        while index < end {
            if self.same(index, ";") {
                index += 1;
                continue;
            }
            let (start, is_static, is_final) = self.skip_modifiers(index);
            if self.is_type_start(start) {
                if let Some((mut declaration, next)) = self.parse_type(start) {
                    declaration.is_static = is_static;
                    nested.push(declaration);
                    index = next;
                    continue;
                }
            }
            if start >= end {
                break;
            }
            if let Some((parsed, next)) = self.parse_declaration(start, end, is_static, is_final) {
                members.extend(parsed);
                index = next;
            } else {
                index = self.skip_statement(index, end);
            }
        }
        (members, nested)
    }
}
