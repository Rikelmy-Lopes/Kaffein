//! Colours for Java token kinds.
//!
//! Syntax colours are deliberately separate from the chrome tokens in
//! `theme.rs`: an editor theme can change without changing panels, borders, or
//! status colours. Every value here is configurable through [`SyntaxTheme`].

use egui::Color32;
use kaffein_domain::TokenKind;

/// The colour assigned to each Java token kind.
///
/// The struct is the extension point for user or third-party themes: construct
/// it with different values and pass it to the editor layouter. Values are
/// `Implemented`, not visually verified against a running build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyntaxTheme {
    pub keyword: Color32,
    pub identifier: Color32,
    pub type_name: Color32,
    pub method: Color32,
    pub field: Color32,
    pub constant: Color32,
    pub string: Color32,
    pub character: Color32,
    pub number: Color32,
    pub line_comment: Color32,
    pub block_comment: Color32,
    pub javadoc_comment: Color32,
    pub annotation: Color32,
    pub operator: Color32,
    pub punctuation: Color32,
}

impl SyntaxTheme {
    pub fn dark() -> Self {
        Self {
            keyword: rgb(0xCF, 0x8E, 0x6D),
            identifier: rgb(0xDF, 0xE1, 0xE5),
            type_name: rgb(0x4E, 0xC9, 0xB0),
            method: rgb(0xDC, 0xDC, 0xAA),
            field: rgb(0x9C, 0xDC, 0xFE),
            constant: rgb(0xC5, 0x86, 0xC0),
            string: rgb(0x6A, 0xAB, 0x73),
            character: rgb(0x6A, 0xAB, 0x73),
            number: rgb(0x2A, 0xAC, 0xB8),
            line_comment: rgb(0x8C, 0x90, 0x99),
            block_comment: rgb(0x8C, 0x90, 0x99),
            javadoc_comment: rgb(0x9D, 0xA0, 0xA8),
            annotation: rgb(0xB3, 0xAE, 0x60),
            operator: rgb(0xDF, 0xE1, 0xE5),
            punctuation: rgb(0xDF, 0xE1, 0xE5),
        }
    }

    pub fn light() -> Self {
        Self {
            keyword: rgb(0x00, 0x33, 0xB3),
            identifier: rgb(0x1F, 0x23, 0x28),
            type_name: rgb(0x26, 0x7F, 0x99),
            method: rgb(0x79, 0x5E, 0x26),
            field: rgb(0x00, 0x10, 0x80),
            constant: rgb(0xAF, 0x00, 0xDB),
            string: rgb(0x06, 0x7D, 0x17),
            character: rgb(0x06, 0x7D, 0x17),
            number: rgb(0x17, 0x50, 0xEB),
            line_comment: rgb(0x6A, 0x6D, 0x73),
            block_comment: rgb(0x6A, 0x6D, 0x73),
            javadoc_comment: rgb(0x62, 0x6A, 0x78),
            annotation: rgb(0x7A, 0x68, 0x00),
            operator: rgb(0x1F, 0x23, 0x28),
            punctuation: rgb(0x1F, 0x23, 0x28),
        }
    }

    pub fn color(self, kind: TokenKind) -> Color32 {
        match kind {
            TokenKind::Keyword => self.keyword,
            TokenKind::Identifier => self.identifier,
            TokenKind::Type => self.type_name,
            TokenKind::Method => self.method,
            TokenKind::Field => self.field,
            TokenKind::Constant => self.constant,
            TokenKind::String => self.string,
            TokenKind::Character => self.character,
            TokenKind::Number => self.number,
            TokenKind::LineComment => self.line_comment,
            TokenKind::BlockComment => self.block_comment,
            TokenKind::JavadocComment => self.javadoc_comment,
            TokenKind::Annotation => self.annotation,
            TokenKind::Operator => self.operator,
            TokenKind::Punctuation => self.punctuation,
        }
    }
}

fn rgb(red: u8, green: u8, blue: u8) -> Color32 {
    Color32::from_rgb(red, green, blue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_token_kind_has_a_colour_in_both_themes() {
        for theme in [SyntaxTheme::dark(), SyntaxTheme::light()] {
            for kind in TokenKind::ALL {
                let colour = theme.color(kind);
                assert_ne!(colour, Color32::TRANSPARENT, "{kind:?}");
            }
        }
    }

    #[test]
    fn semantic_identifier_kinds_have_distinct_colours() {
        for theme in [SyntaxTheme::dark(), SyntaxTheme::light()] {
            let colours = [
                theme.identifier,
                theme.type_name,
                theme.method,
                theme.field,
                theme.constant,
            ];
            for (index, left) in colours.iter().enumerate() {
                for right in &colours[index + 1..] {
                    assert_ne!(left, right);
                }
            }
        }
    }

    #[test]
    fn dark_and_light_themes_are_distinct() {
        let dark = SyntaxTheme::dark();
        let light = SyntaxTheme::light();
        assert_ne!(dark.keyword, light.keyword);
        assert_ne!(dark.string, light.string);
        assert_ne!(dark.number, light.number);
        assert_ne!(dark.line_comment, light.line_comment);
        assert_ne!(dark.type_name, light.type_name);
        assert_ne!(dark.method, light.method);
        assert_ne!(dark.field, light.field);
        assert_ne!(dark.constant, light.constant);
    }
}
