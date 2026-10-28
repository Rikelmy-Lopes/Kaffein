//! Tests for untrusted console output rendering.

use egui::Context;
use kaffein_app::RunState;
use kaffein_i18n::Localizer;

use crate::theme::Theme;
use crate::ui_state::UiState;

use super::{draw_bottom_panel, sanitize_output, BottomTab};

#[test]
fn a_new_run_selects_the_console_tab() {
    let run = RunState::Running {
        id: 7,
        source_name: "Scratch.java".to_owned(),
    };
    let mut ui_state = UiState::default();
    let localizer = Localizer::english();
    let ctx = Context::default();

    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        draw_bottom_panel(ctx, &run, &localizer, &mut ui_state, Theme::dark());
    });

    assert_eq!(ui_state.bottom_tab, BottomTab::Console);
    assert_eq!(ui_state.seen_run_id, 7);
}

#[test]
fn terminal_escape_sequences_are_removed() {
    let output = "\u{1b}[31mred\u{1b}[0m\n";

    assert_eq!(sanitize_output(output), "red\n");
}

#[test]
fn terminal_osc_sequences_are_removed() {
    let output = "before\u{1b}]0;title\u{7}after";
    assert_eq!(sanitize_output(output), "beforeafter");

    let hyperlink = "before\u{1b}]8;;https://example.com\u{1b}\\after";
    assert_eq!(sanitize_output(hyperlink), "beforeafter");
}

#[test]
fn ordinary_text_and_newlines_are_preserved() {
    let output = "line one\n\tline two\r\n";

    assert_eq!(sanitize_output(output), output);
}

#[test]
fn other_control_characters_are_removed() {
    let output = "before\u{7}after\u{0}";

    assert_eq!(sanitize_output(output), "beforeafter");
}
