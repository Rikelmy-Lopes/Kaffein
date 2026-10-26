//! Document model tests.

use super::*;

fn document(text: &str) -> Document {
    Document::from_loaded(DocumentId(1), "Example.java", text)
}

#[test]
fn loaded_document_is_clean_and_detects_line_endings() {
    let lf = document("class A {}\n");
    assert!(!lf.is_dirty());
    assert_eq!(lf.line_ending(), LineEnding::Lf);

    let crlf = document("class A {}\r\n");
    assert_eq!(crlf.line_ending(), LineEnding::Crlf);
    assert_eq!(crlf.buffer(), "class A {}\n");
    assert_eq!(crlf.text_for_save(), "class A {}\r\n");
}

#[test]
fn a_new_document_is_dirty_until_the_first_save() {
    let mut doc = Document::new(DocumentId(2), "New.java");

    assert_eq!(doc.buffer(), "");
    assert!(doc.is_dirty());

    doc.mark_saved();

    assert!(!doc.is_dirty());
}

#[test]
fn editing_marks_dirty_and_undo_restores_content() {
    let mut doc = document("abc");
    assert!(doc.replace_text("abcd", 1_000));
    assert!(doc.is_dirty());
    assert!(doc.undo());
    assert_eq!(doc.buffer(), "abc");
    assert!(!doc.is_dirty());
    assert!(doc.redo());
    assert_eq!(doc.buffer(), "abcd");
}

#[test]
fn typing_within_the_window_coalesces_into_one_undo_step() {
    let mut doc = document("");
    assert!(doc.replace_text("a", 1_000));
    assert!(doc.replace_text("ab", 1_100));
    assert!(doc.replace_text("abc", 1_200));

    assert!(doc.undo());
    assert_eq!(doc.buffer(), "");
    assert!(!doc.can_undo());
}

#[test]
fn a_pause_starts_a_new_undo_step() {
    let mut doc = document("");
    assert!(doc.replace_text("a", 1_000));
    assert!(doc.replace_text("ab", 2_000));

    assert!(doc.undo());
    assert_eq!(doc.buffer(), "a");
    assert!(doc.undo());
    assert_eq!(doc.buffer(), "");
}

#[test]
fn a_bulk_replacement_is_its_own_undo_step() {
    let mut doc = document("one");
    assert!(doc.replace_text("two", 1_000));
    assert!(doc.replace_text("three", 1_050));

    assert!(doc.undo());
    assert_eq!(doc.buffer(), "two");
}

#[test]
fn editing_after_undo_clears_redo() {
    let mut doc = document("a");
    assert!(doc.replace_text("ab", 1_000));
    assert!(doc.undo());
    assert!(doc.can_redo());

    assert!(doc.replace_text("ax", 2_000));
    assert!(!doc.can_redo());
}

#[test]
fn marking_saved_clears_dirty_state_but_keeps_history() {
    let mut doc = document("a");
    assert!(doc.replace_text("ab", 1_000));
    doc.mark_saved();

    assert!(!doc.is_dirty());
    assert!(doc.can_undo());
}

#[test]
fn line_count_counts_a_trailing_newline_as_an_empty_line() {
    assert_eq!(document("").line_count(), 1);
    assert_eq!(document("a").line_count(), 1);
    assert_eq!(document("a\n").line_count(), 2);
    assert_eq!(document("a\nb").line_count(), 2);
}
