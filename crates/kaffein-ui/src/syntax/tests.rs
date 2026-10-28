//! Tests for the incremental highlight cache, layout, and tracked buffer.

use std::cell::Cell;
use std::rc::Rc;

use egui::{Color32, FontId, TextBuffer};
use kaffein_domain::TokenKind;

use super::buffer::TrackedBuffer;
use super::cache::HighlightCache;
use super::layout::build_job;
use super::theme::SyntaxTheme;

#[test]
fn unchanged_lines_are_not_relexed() {
    let mut cache = HighlightCache::default();

    assert_eq!(cache.relex_for_test("class A {}\nclass B {}\n"), 2);
    assert_eq!(cache.relex_for_test("class A {}\nclass C {}\n"), 1);
    assert_eq!(
        cache.relex_for_test("class A {}\nclass C {}\nclass D {}\n"),
        1
    );
}

#[test]
fn a_changed_comment_state_relexes_the_suffix() {
    let mut cache = HighlightCache::default();

    assert_eq!(cache.relex_for_test("/* a */\nclass A {}\n"), 2);
    assert_eq!(cache.relex_for_test("/* a\nclass A {}\n"), 2);
}

#[test]
fn the_cache_exposes_keyword_and_string_tokens() {
    let mut cache = HighlightCache::default();
    cache.relex_for_test("class App { String s = \"hi\"; }\n");

    let lines = cache.lines_for_test();
    assert_eq!(lines.len(), 1);
    let kinds: Vec<TokenKind> = lines[0].tokens.iter().map(|token| token.kind).collect();

    assert!(kinds.contains(&TokenKind::Keyword));
    assert!(kinds.contains(&TokenKind::String));
}

#[test]
fn the_cache_exposes_semantic_identifier_kinds() {
    let mut cache = HighlightCache::default();
    cache.relex_for_test(
        "class App { static final int MAX = 1; void run() { System.out.println(MAX); } }\n",
    );

    let lines = cache.lines_for_test();
    let kinds: Vec<TokenKind> = lines[0].tokens.iter().map(|token| token.kind).collect();

    assert!(kinds.contains(&TokenKind::Type));
    assert!(kinds.contains(&TokenKind::Method));
    assert!(kinds.contains(&TokenKind::Field));
    assert!(kinds.contains(&TokenKind::Constant));
}

#[test]
fn the_layout_job_preserves_text_and_uses_token_colours() {
    let text =
        "class App { static final int MAX = 1; void run() { System.out.println(\"hi\"); } }\n";
    let mut cache = HighlightCache::default();
    cache.relex_for_test(text);
    let theme = SyntaxTheme::dark();

    let job = build_job(
        text,
        cache.lines_for_test(),
        FontId::monospace(13.0),
        400.0,
        &theme,
    );

    assert_eq!(job.text, text);
    for colour in [
        theme.keyword,
        theme.type_name,
        theme.method,
        theme.field,
        theme.constant,
        theme.string,
    ] {
        assert!(
            job.sections
                .iter()
                .any(|section| section.format.color == colour),
            "missing colour {colour:?}"
        );
    }
    assert!(job
        .sections
        .iter()
        .all(|section| !section.byte_range.is_empty()));
}

#[test]
fn a_custom_theme_can_colour_each_semantic_kind() {
    let text = "class App { static final int MAX = 1; void run() { System.out.println(MAX); } }\n";
    let mut cache = HighlightCache::default();
    cache.relex_for_test(text);
    let mut theme = SyntaxTheme::dark();
    theme.type_name = Color32::from_rgb(1, 2, 3);
    theme.method = Color32::from_rgb(4, 5, 6);
    theme.field = Color32::from_rgb(7, 8, 9);
    theme.constant = Color32::from_rgb(10, 11, 12);

    let job = build_job(
        text,
        cache.lines_for_test(),
        FontId::monospace(13.0),
        400.0,
        &theme,
    );

    for colour in [theme.type_name, theme.method, theme.field, theme.constant] {
        assert!(
            job.sections
                .iter()
                .any(|section| section.format.color == colour),
            "missing custom colour {colour:?}"
        );
    }
}

#[test]
fn an_empty_document_produces_an_empty_job() {
    let mut cache = HighlightCache::default();
    cache.relex_for_test("");

    let job = build_job(
        "",
        cache.lines_for_test(),
        FontId::monospace(13.0),
        100.0,
        &SyntaxTheme::dark(),
    );

    assert!(job.text.is_empty());
    assert!(job.sections.is_empty());
}

#[test]
fn tracked_buffer_reports_insertions_and_deletions() {
    let mut text = String::from("abc");
    let generation = Rc::new(Cell::new(0));

    {
        let mut buffer = TrackedBuffer::new(&mut text, Rc::clone(&generation));
        buffer.insert_text("X", 1);
        assert_eq!(buffer.as_str(), "aXbc");
        buffer.delete_char_range(0..1);
    }

    assert_eq!(text, "Xbc");
    assert_eq!(generation.get(), 2);
}
