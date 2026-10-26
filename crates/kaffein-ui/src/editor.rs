//! The editor area: welcome state, tabs, and the text surface.

use egui::{
    Button, CentralPanel, Context, Frame, Margin, RichText, ScrollArea, TextStyle, Ui, Vec2,
};
use kaffein_app::{command_id, AppState, CommandArgs, CommandRegistry};
use kaffein_domain::DocumentId;

use crate::actions::UiAction;
use crate::theme::{self, Theme, ROW_COMPACT, ROW_STANDARD, SPACE_1, SPACE_2, SPACE_3, SPACE_4};
use crate::ui_state::UiState;

#[derive(Debug, Default)]
pub(crate) struct EditorBuffer {
    pub(crate) document: Option<DocumentId>,
    pub(crate) revision: Option<u64>,
    pub(crate) text: String,
}

pub(crate) fn draw_editor_panel(
    ctx: &Context,
    state: &AppState,
    registry: &CommandRegistry,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
    theme: Theme,
    now_ms: u64,
) {
    CentralPanel::default()
        .frame(Frame::none().fill(theme.surface_editor))
        .show(ctx, |ui| {
            if state.documents().is_empty() {
                ui.centered_and_justified(|ui| {
                    let shortcut = registry
                        .get(command_id::FILE_OPEN)
                        .and_then(|command| command.primary_shortcut())
                        .map(|shortcut| format!(" ({})", shortcut.display()))
                        .unwrap_or_default();
                    ui.label(
                        RichText::new(format!(
                            "No file open — use File › Open File…{shortcut} or open a project."
                        ))
                        .size(theme::WELCOME_FONT_SIZE)
                        .color(theme.text_muted),
                    );
                });
                return;
            }

            draw_tabs(ui, state, actions, theme);
            ui.separator();
            draw_editor_text(ui, state, ui_state, actions, theme, now_ms);
        });
}

fn draw_tabs(ui: &mut Ui, state: &AppState, actions: &mut Vec<UiAction>, theme: Theme) {
    Frame::none()
        .fill(theme.surface_panel)
        .inner_margin(Margin::symmetric(SPACE_4, SPACE_1))
        .show(ui, |ui| {
            ScrollArea::horizontal()
                .id_salt("editor_tabs")
                .max_height(ROW_STANDARD)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = SPACE_1;
                        for document in state.documents() {
                            let id = document.id();
                            let active = state.active_id() == Some(id);
                            let mut title = document.display_name();
                            if document.is_dirty() {
                                title.push_str(" •");
                            }
                            let text = RichText::new(title).color(if active {
                                theme.text_primary
                            } else {
                                theme.text_muted
                            });
                            let response = ui.selectable_label(active, text);
                            if response.clicked() && !active {
                                actions.push(UiAction {
                                    command: command_id::EDITOR_ACTIVATE,
                                    args: CommandArgs::Document(id),
                                });
                            }
                            let close = ui
                                .add(
                                    Button::new("×")
                                        .frame(false)
                                        .min_size(Vec2::splat(ROW_COMPACT)),
                                )
                                .on_hover_text("Close file");
                            if close.clicked() || response.middle_clicked() {
                                actions.push(UiAction {
                                    command: command_id::FILE_CLOSE,
                                    args: CommandArgs::Document(id),
                                });
                            }
                            ui.add_space(SPACE_2);
                        }
                    });
                });
        });
}

fn draw_editor_text(
    ui: &mut Ui,
    state: &AppState,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
    theme: Theme,
    now_ms: u64,
) {
    let Some(document) = state.active_document() else {
        return;
    };
    let id = document.id();
    let revision = document.revision();
    let line_count = document.line_count();

    if ui_state.editor.document != Some(id) || ui_state.editor.revision != Some(revision) {
        ui_state.editor.document = Some(id);
        ui_state.editor.revision = Some(revision);
        ui_state.editor.text = document.buffer().to_owned();
    }

    let mut changed = false;
    ScrollArea::vertical()
        .id_salt("editor_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let response = ui.add(
                egui::TextEdit::multiline(&mut ui_state.editor.text)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .desired_rows(line_count.max(1) + 4)
                    .margin(Margin::symmetric(SPACE_4, SPACE_3))
                    .text_color(theme.text_primary)
                    .font(TextStyle::Monospace),
            );
            changed = response.changed();
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
    }
}
