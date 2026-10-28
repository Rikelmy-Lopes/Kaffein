//! Token categories produced by the Java lexer and semantic classifier.

/// A category the editor can colour independently.
///
/// The domain names the category; it does not choose a colour. The UI maps
/// each kind to a colour in its syntax theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Keyword,
    Identifier,
    /// A class, interface, enum, record, or other type name.
    Type,
    /// A method declaration or call.
    Method,
    /// A field or member accessed through `.` or `::`.
    Field,
    /// A constant named in `SCREAMING_SNAKE_CASE`.
    Constant,
    String,
    Character,
    Number,
    LineComment,
    BlockComment,
    JavadocComment,
    Annotation,
    Operator,
    Punctuation,
}

impl TokenKind {
    /// Every kind, in a stable order. Used by theme and completeness tests.
    pub const ALL: [TokenKind; 15] = [
        TokenKind::Keyword,
        TokenKind::Identifier,
        TokenKind::Type,
        TokenKind::Method,
        TokenKind::Field,
        TokenKind::Constant,
        TokenKind::String,
        TokenKind::Character,
        TokenKind::Number,
        TokenKind::LineComment,
        TokenKind::BlockComment,
        TokenKind::JavadocComment,
        TokenKind::Annotation,
        TokenKind::Operator,
        TokenKind::Punctuation,
    ];
}

/// A byte range of source text with a lexical category.
///
/// Offsets are byte offsets into the text that was tokenized. They always fall
/// on UTF-8 character boundaries, so the UI can slice the source safely.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

impl Token {
    pub fn new(kind: TokenKind, start: usize, end: usize) -> Self {
        Self { kind, start, end }
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}
