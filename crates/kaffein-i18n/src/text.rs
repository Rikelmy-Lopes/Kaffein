//! User-facing text that is resolved only when it is rendered.
//!
//! Commands store [`Text`] instead of a formatted `String`, so a status message
//! written before a locale change is re-rendered in the new language on the
//! next frame.

use crate::Localizer;

/// A piece of interface text with its localization key and arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Text {
    /// A catalog entry with optional named arguments.
    Message {
        key: &'static str,
        arguments: Vec<TextArgument>,
    },
    /// A plural family entry (`<key>.one`, `<key>.other`).
    Plural {
        key: &'static str,
        count: usize,
        arguments: Vec<TextArgument>,
    },
    /// Runtime text that is not translated: file names, OS details, and test
    /// fixtures.
    Literal(String),
}

/// A named placeholder inside a [`Text`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextArgument {
    pub name: &'static str,
    pub value: TextValue,
}

/// The value substituted for a named placeholder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextValue {
    Literal(String),
    Key(&'static str),
    Number(i64),
    Text(Box<Text>),
}

impl Text {
    pub fn key(key: &'static str) -> Self {
        Self::Message {
            key,
            arguments: Vec::new(),
        }
    }

    pub fn message(key: &'static str) -> Self {
        Self::key(key)
    }

    pub fn plural(key: &'static str, count: usize) -> Self {
        Self::Plural {
            key,
            count,
            arguments: Vec::new(),
        }
    }

    pub fn literal(text: impl Into<String>) -> Self {
        Self::Literal(text.into())
    }

    /// Adds a named placeholder. Calling this on [`Text::Literal`] is a
    /// programming error because literal text has no placeholders.
    pub fn arg(mut self, name: &'static str, value: impl Into<TextValue>) -> Self {
        let arguments = match &mut self {
            Self::Message { arguments, .. } | Self::Plural { arguments, .. } => arguments,
            Self::Literal(_) => {
                debug_assert!(false, "cannot add an argument to literal text");
                return self;
            }
        };
        arguments.push(TextArgument {
            name,
            value: value.into(),
        });
        self
    }

    /// Resolves the text against the active locale.
    pub fn resolve(&self, localizer: &Localizer) -> String {
        match self {
            Self::Literal(text) => text.clone(),
            Self::Message { key, arguments } => {
                localizer.render(key, None, &resolve_arguments(arguments, localizer))
            }
            Self::Plural {
                key,
                count,
                arguments,
            } => localizer.render(key, Some(*count), &resolve_arguments(arguments, localizer)),
        }
    }
}

fn resolve_arguments(
    arguments: &[TextArgument],
    localizer: &Localizer,
) -> Vec<(&'static str, String)> {
    arguments
        .iter()
        .map(|argument| (argument.name, argument.value.resolve(localizer)))
        .collect()
}

impl TextValue {
    fn resolve(&self, localizer: &Localizer) -> String {
        match self {
            Self::Literal(text) => text.clone(),
            Self::Key(key) => localizer.text(key).to_owned(),
            Self::Number(value) => localizer.number(*value),
            Self::Text(text) => text.resolve(localizer),
        }
    }
}

impl From<&str> for TextValue {
    fn from(value: &str) -> Self {
        Self::Literal(value.to_owned())
    }
}

impl From<String> for TextValue {
    fn from(value: String) -> Self {
        Self::Literal(value)
    }
}

impl From<&String> for TextValue {
    fn from(value: &String) -> Self {
        Self::Literal(value.clone())
    }
}

impl From<i64> for TextValue {
    fn from(value: i64) -> Self {
        Self::Number(value)
    }
}

impl From<usize> for TextValue {
    fn from(value: usize) -> Self {
        Self::Number(i64::try_from(value).unwrap_or(i64::MAX))
    }
}

impl From<u64> for TextValue {
    fn from(value: u64) -> Self {
        Self::Number(i64::try_from(value).unwrap_or(i64::MAX))
    }
}

impl From<Text> for TextValue {
    fn from(value: Text) -> Self {
        Self::Text(Box::new(value))
    }
}
