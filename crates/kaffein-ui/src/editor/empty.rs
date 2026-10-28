//! Read-only Java sample shown when no document is open.

use egui::{Margin, ScrollArea, TextStyle, Ui};
use kaffein_app::DEFAULT_JAVA_SOURCE;

use crate::syntax::TrackedBuffer;
use crate::theme::{Theme, SPACE_3, SPACE_4};
use crate::ui_state::UiState;

/// Renders a read-only sample Java file when no document is open, so the editor
/// always shows code instead of an empty surface.
pub(super) fn draw_empty_editor(ui: &mut Ui, theme: Theme, ui_state: &mut UiState) {
    let mut text = DEFAULT_JAVA_SOURCE.to_owned();
    let syntax = &mut ui_state.syntax;
    let generation = syntax.generation();
    let syntax_theme = theme.syntax;
    let mut buffer = TrackedBuffer::new(&mut text, generation.clone());
    let mut layouter = move |ui: &Ui, text: &str, wrap_width: f32| {
        syntax.layout(ui, text, wrap_width, generation.get(), &syntax_theme)
    };

    ScrollArea::vertical()
        .id_salt("empty_editor_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_enabled(
                false,
                egui::TextEdit::multiline(&mut buffer)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .desired_rows(10)
                    .margin(Margin::symmetric(SPACE_4, SPACE_3))
                    .font(TextStyle::Monospace)
                    .layouter(&mut layouter),
            );
        });
}
