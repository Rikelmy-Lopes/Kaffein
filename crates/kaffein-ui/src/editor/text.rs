//! The editable text surface, caret-to-completion wiring, and popup anchoring.

use egui::{Context, Key, Margin, Modifiers, ScrollArea, TextStyle, Ui};
use kaffein_app::{command_id, AppState, CommandArgs};

use crate::actions::UiAction;
use crate::completion::draw_completion_popup;
use crate::syntax::TrackedBuffer;
use crate::theme::{Theme, SPACE_3, SPACE_4};
use crate::ui_state::UiState;
use kaffein_i18n::Localizer;

use super::EditorBuffer;

pub(crate) struct EditorTextContext<'a> {
    pub(crate) ctx: &'a Context,
    pub(crate) state: &'a AppState,
    pub(crate) localizer: &'a Localizer,
    pub(crate) theme: Theme,
    pub(crate) now_ms: u64,
}

pub(crate) fn draw_editor_text(
    ui: &mut Ui,
    context: &EditorTextContext<'_>,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
) {
    let EditorTextContext {
        ctx,
        state,
        localizer,
        theme,
        now_ms,
    } = *context;
    let Some(document) = state.active_document() else {
        return;
    };
    let id = document.id();
    let revision = document.revision();
    let line_count = document.line_count();

    if ui_state.editor.document != Some(id) {
        ui_state.editor.document = Some(id);
        ui_state.editor.revision = Some(revision);
        ui_state.editor.text = document.buffer().to_owned();
        ui_state.syntax.reset();
    } else if ui_state.editor.revision != Some(revision) {
        ui_state.editor.revision = Some(revision);
        ui_state.editor.text = document.buffer().to_owned();
        ui_state.syntax.invalidate();
    }

    completion_keys(ctx, state, actions, now_ms);

    let mut changed = false;
    let mut anchor = None;
    let mut caret = None;
    ScrollArea::vertical()
        .id_salt("editor_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let EditorBuffer { text, .. } = &mut ui_state.editor;
            let syntax = &mut ui_state.syntax;
            let generation = syntax.generation();
            let syntax_theme = theme.syntax;
            let mut buffer = TrackedBuffer::new(text, generation.clone());
            let mut layouter = move |ui: &Ui, text: &str, wrap_width: f32| {
                syntax.layout(ui, text, wrap_width, generation.get(), &syntax_theme)
            };
            let output = egui::TextEdit::multiline(&mut buffer)
                .code_editor()
                .desired_width(f32::INFINITY)
                .desired_rows(line_count.max(1) + 4)
                .margin(Margin::symmetric(SPACE_4, SPACE_3))
                .text_color(theme.text_primary)
                .font(TextStyle::Monospace)
                .layouter(&mut layouter)
                .show(ui);
            changed = output.response.changed();
            if let Some(range) = output.cursor_range {
                let cursor = range.primary;
                let rect = output.galley.pos_from_cursor(&cursor);
                anchor =
                    Some(output.galley_pos + rect.left_bottom().to_vec2() + egui::vec2(0.0, 2.0));
                caret = Some(byte_offset(text, cursor.ccursor.index));
            }
        });
    if changed {
        ui_state.editor.revision = None;
        actions.push(UiAction {
            command: command_id::EDITOR_REPLACE_TEXT,
            args: CommandArgs::ReplaceText {
                document: id,
                text: ui_state.editor.text.clone(),
                at_ms: now_ms,
            },
        });
        if let Some(caret) = caret {
            actions.push(UiAction {
                command: command_id::EDITOR_COMPLETE,
                args: CommandArgs::Complete {
                    document: id,
                    caret,
                },
            });
        }
    }

    if let Some(anchor) = anchor {
        draw_completion_popup(ctx, state, localizer, actions, theme, now_ms, anchor);
    }
}

fn completion_keys(ctx: &Context, state: &AppState, actions: &mut Vec<UiAction>, now_ms: u64) {
    let Some(session) = state.completion() else {
        return;
    };
    let document = session.document;
    let action = ctx.input_mut(|input| {
        if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
            Some(KeyAction::Select(1))
        } else if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
            Some(KeyAction::Select(-1))
        } else if input.consume_key(Modifiers::NONE, Key::Enter)
            || input.consume_key(Modifiers::NONE, Key::Tab)
        {
            Some(KeyAction::Accept)
        } else if input.consume_key(Modifiers::NONE, Key::Escape) {
            Some(KeyAction::Dismiss)
        } else {
            None
        }
    });

    match action {
        Some(KeyAction::Select(delta)) => actions.push(UiAction {
            command: command_id::EDITOR_COMPLETE_SELECT,
            args: CommandArgs::CompleteSelect { delta },
        }),
        Some(KeyAction::Accept) => actions.push(UiAction {
            command: command_id::EDITOR_COMPLETE_ACCEPT,
            args: CommandArgs::CompleteAccept {
                document,
                at_ms: now_ms,
                item: None,
            },
        }),
        Some(KeyAction::Dismiss) => actions.push(UiAction {
            command: command_id::EDITOR_COMPLETE_DISMISS,
            args: CommandArgs::None,
        }),
        None => {}
    }
}

fn byte_offset(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map_or(text.len(), |(offset, _)| offset)
}

enum KeyAction {
    Select(isize),
    Accept,
    Dismiss,
}
