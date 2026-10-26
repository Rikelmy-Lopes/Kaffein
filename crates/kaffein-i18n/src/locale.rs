//! Locale identity and validation.
//!
//! A locale is never an enum variant: external catalogs may add a language
//! without recompiling the IDE. The code follows the familiar `language` or
//! `language-REGION` shape (`en`, `pt-BR`, `zh-Hans`, `es-419`).

use std::fmt;

/// A language the interface can be rendered in.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Locale {
    code: String,
    name: String,
}

impl Locale {
    pub fn new(code: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
        }
    }

    /// The stable language tag used by settings, menus, and automation.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The name shown to the user, written in the language itself.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Rejects codes that could escape the locale directory or never be a
    /// language tag.
    pub fn is_valid_code(code: &str) -> bool {
        let mut parts = code.split('-');
        let Some(language) = parts.next() else {
            return false;
        };
        if !(2..=3).contains(&language.len()) || !language.chars().all(|c| c.is_ascii_alphabetic())
        {
            return false;
        }
        parts.all(|part| {
            (1..=8).contains(&part.len()) && part.chars().all(|c| c.is_ascii_alphanumeric())
        })
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.code)
    }
}
