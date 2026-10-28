//! Behaviour tests for the Java line lexer.

use super::*;

fn tokens(source: &str) -> Vec<(TokenKind, &str)> {
    tokenize(source)
        .into_iter()
        .map(|token| (token.kind, &source[token.start..token.end]))
        .collect()
}

#[test]
fn keywords_literals_and_identifiers_are_distinguished() {
    let source = "public final class App { int value = 42; boolean ready = true; }";
    let found = tokens(source);

    assert!(found.contains(&(TokenKind::Keyword, "public")));
    assert!(found.contains(&(TokenKind::Keyword, "final")));
    assert!(found.contains(&(TokenKind::Keyword, "class")));
    assert!(found.contains(&(TokenKind::Type, "App")));
    assert!(found.contains(&(TokenKind::Identifier, "value")));
    assert!(found.contains(&(TokenKind::Number, "42")));
    assert!(found.contains(&(TokenKind::Keyword, "true")));
}

#[test]
fn strings_characters_and_all_comment_forms_are_recognised() {
    let source = "String s = \"hello\";\nchar c = 'x';\n// note\n/* block */\n/** doc */";
    let found = tokens(source);

    assert!(found.contains(&(TokenKind::String, "\"hello\"")));
    assert!(found.contains(&(TokenKind::Character, "'x'")));
    assert!(found.contains(&(TokenKind::LineComment, "// note")));
    assert!(found.contains(&(TokenKind::BlockComment, "/* block */")));
    assert!(found.contains(&(TokenKind::JavadocComment, "/** doc */")));
}

#[test]
fn comment_markers_inside_strings_are_not_comments() {
    let source = r#"String s = "http://example.com /* not a comment */";"#;
    let found = tokens(source);

    assert!(found
        .iter()
        .any(|(kind, text)| *kind == TokenKind::String && text.contains("http://")));
    assert!(!found
        .iter()
        .any(|(kind, _)| matches!(kind, TokenKind::LineComment | TokenKind::BlockComment)));
}

#[test]
fn escaped_quotes_do_not_close_strings() {
    let found = tokens(r#"String s = "a\"b"; // after"#);

    assert!(found.contains(&(TokenKind::String, r#""a\"b""#)));
    assert!(found.contains(&(TokenKind::LineComment, "// after")));
}

#[test]
fn numbers_cover_java_literal_forms() {
    let source = "0 42 0777 0x1f 0b1010 1.5 1e-3 1.5e+10f 0x1.8p3 1_000L .5";
    let found = tokens(source);
    let numbers: Vec<&str> = found
        .iter()
        .filter(|(kind, _)| *kind == TokenKind::Number)
        .map(|(_, text)| *text)
        .collect();

    assert_eq!(
        numbers,
        [
            "0", "42", "0777", "0x1f", "0b1010", "1.5", "1e-3", "1.5e+10f", "0x1.8p3", "1_000L",
            ".5"
        ]
    );
}

#[test]
fn numbers_do_not_swallow_following_identifiers() {
    let found = tokens("1.xyz 123abc 0x 0b");

    assert!(found.contains(&(TokenKind::Number, "1.")));
    assert!(found.contains(&(TokenKind::Identifier, "xyz")));
    assert!(found.contains(&(TokenKind::Number, "123")));
    assert!(found.contains(&(TokenKind::Identifier, "abc")));
    assert!(found.contains(&(TokenKind::Number, "0")));
    assert!(found.contains(&(TokenKind::Identifier, "x")));
    assert!(found.contains(&(TokenKind::Identifier, "b")));
}

#[test]
fn operators_use_longest_match() {
    let found = tokens("a >>>= b; c <<= d; e -> f; g::h; i...j;");

    for operator in [">>>=", "<<=", "->", "::", "..."] {
        assert!(
            found.contains(&(TokenKind::Operator, operator))
                || found.contains(&(TokenKind::Punctuation, operator)),
            "missing {operator}"
        );
    }
}

#[test]
fn annotations_and_punctuation_are_separate() {
    let found = tokens("@Override public void run() {}");

    assert!(found.contains(&(TokenKind::Annotation, "@Override")));
    assert!(found.contains(&(TokenKind::Punctuation, "(")));
    assert!(found.contains(&(TokenKind::Punctuation, ")")));
    assert!(found.contains(&(TokenKind::Punctuation, "{")));
    assert!(found.contains(&(TokenKind::Punctuation, "}")));
}

#[test]
fn multi_line_block_comment_carries_state() {
    let first = tokenize_line("/* start", LexState::Normal);
    assert_eq!(first.end_state, LexState::BlockComment { javadoc: false });
    assert_eq!(
        first.tokens,
        vec![Token::new(TokenKind::BlockComment, 0, 8)]
    );

    let second = tokenize_line(" middle", first.end_state);
    assert_eq!(second.end_state, LexState::BlockComment { javadoc: false });
    assert_eq!(
        second.tokens,
        vec![Token::new(TokenKind::BlockComment, 0, 7)]
    );

    let third = tokenize_line(" end */ int x;", second.end_state);
    assert_eq!(third.end_state, LexState::Normal);
    assert_eq!(third.tokens[0], Token::new(TokenKind::BlockComment, 0, 7));
    assert!(third
        .tokens
        .iter()
        .any(|token| token.kind == TokenKind::Keyword));
}

#[test]
fn javadoc_state_is_preserved_across_lines() {
    let first = tokenize_line("/** docs", LexState::Normal);
    assert_eq!(first.end_state, LexState::BlockComment { javadoc: true });
    assert_eq!(first.tokens[0].kind, TokenKind::JavadocComment);

    let second = tokenize_line(" more */", first.end_state);
    assert_eq!(second.tokens[0].kind, TokenKind::JavadocComment);
    assert_eq!(second.end_state, LexState::Normal);
}

#[test]
fn an_empty_doc_comment_is_not_javadoc() {
    let lexed = tokenize_line("/**/", LexState::Normal);
    assert_eq!(lexed.tokens[0].kind, TokenKind::BlockComment);
}

#[test]
fn text_blocks_span_lines_and_close() {
    let first = tokenize_line("String s = \"\"\"", LexState::Normal);
    assert_eq!(first.end_state, LexState::TextBlock);
    assert_eq!(first.tokens.last().unwrap().kind, TokenKind::String);

    let second = tokenize_line("hello \"world\"", first.end_state);
    assert_eq!(second.end_state, LexState::TextBlock);
    assert_eq!(second.tokens[0].kind, TokenKind::String);
    assert_eq!(second.tokens[0].end, 13);

    let third = tokenize_line("bye\"\"\";", second.end_state);
    assert_eq!(third.end_state, LexState::Normal);
    assert_eq!(third.tokens[0], Token::new(TokenKind::String, 0, 6));
    assert!(third
        .tokens
        .iter()
        .any(|token| token.kind == TokenKind::Punctuation));
}

#[test]
fn text_blocks_skip_escaped_quotes() {
    let lexed = tokenize_line("String s = \"\"\"a\\\"b\"\"\";", LexState::Normal);
    assert_eq!(lexed.end_state, LexState::Normal);
    assert_eq!(lexed.tokens.last().unwrap().kind, TokenKind::Punctuation);
}

#[test]
fn unterminated_string_stops_at_the_line_end() {
    let line = "String s = \"oops";
    let lexed = tokenize_line(line, LexState::Normal);
    let last = lexed.tokens.last().expect("string token");

    assert_eq!(lexed.end_state, LexState::Normal);
    assert_eq!(last.kind, TokenKind::String);
    assert_eq!(last.end, line.len());
}

#[test]
fn block_comment_starting_mid_line_returns_to_normal() {
    let lexed = tokenize_line("int x; /* note */ int y;", LexState::Normal);

    assert!(lexed
        .tokens
        .iter()
        .any(|token| token.kind == TokenKind::BlockComment && token.start == 7 && token.end == 17));
    assert_eq!(lexed.end_state, LexState::Normal);
}

#[test]
fn unicode_identifiers_keep_utf8_boundaries() {
    let source = "class Café { String ação = \"olá\"; }";
    let found = tokens(source);

    assert!(found.contains(&(TokenKind::Type, "Café")));
    assert!(found.contains(&(TokenKind::Identifier, "ação")));
    for token in tokenize(source) {
        assert!(source.is_char_boundary(token.start));
        assert!(source.is_char_boundary(token.end));
        assert!(token.start < token.end);
    }
}

#[test]
fn line_tokens_are_relative_to_the_line() {
    let lexed = tokenize_line("  int x;", LexState::Normal);
    assert_eq!(lexed.tokens[0], Token::new(TokenKind::Keyword, 2, 5));
}

#[test]
fn contextual_keywords_are_coloured() {
    assert!(tokens("record Point(int x) {}").contains(&(TokenKind::Keyword, "record")));
    assert!(tokens("var value = 1;").contains(&(TokenKind::Keyword, "var")));
    assert!(tokens("sealed interface Shape permits Circle {}")
        .contains(&(TokenKind::Keyword, "permits")));
}

#[test]
fn empty_source_has_no_tokens() {
    assert!(tokenize("").is_empty());
}
