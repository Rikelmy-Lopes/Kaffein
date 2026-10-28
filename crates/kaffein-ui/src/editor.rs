//! The editor area: welcome state, tabs, and the text surface.

mod empty;
mod text;

use egui::{Button, CentralPanel, Context, Frame, Margin, RichText, ScrollArea, Ui, Vec2};
use kaffein_app::{command_id, AppState, CommandArgs};
use kaffein_domain::DocumentId;
use kaffein_i18n::{keys, Localizer};

use crate::actions::UiAction;
use crate::theme::{Theme, ROW_COMPACT, ROW_STANDARD, SPACE_1, SPACE_2, SPACE_4};
use crate::ui_state::UiState;

pub(crate) use text::{draw_editor_text, EditorTextContext};

#[derive(Debug, Default)]
pub(crate) struct EditorBuffer {
    pub(crate) document: Option<DocumentId>,
    pub(crate) revision: Option<u64>,
    pub(crate) text: String,
}

pub(crate) fn draw_editor_panel(
    ctx: &Context,
    state: &AppState,
    localizer: &Localizer,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
    theme: Theme,
) {
    let now_ms = ctx.input(|input| (input.time * 1000.0) as u64);
    CentralPanel::default()
        .frame(Frame::none().fill(theme.surface_editor))
        .show(ctx, |ui| {
            if state.documents().is_empty() {
                empty::draw_empty_editor(ui, theme, ui_state);
                return;
            }

            draw_tabs(ui, state, localizer, actions, theme);
            ui.separator();
            let context = EditorTextContext {
                ctx,
                state,
                localizer,
                theme,
                now_ms,
            };
            draw_editor_text(ui, &context, ui_state, actions);
        });
}

fn draw_tabs(
    ui: &mut Ui,
    state: &AppState,
    localizer: &Localizer,
    actions: &mut Vec<UiAction>,
    theme: Theme,
) {
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
                                .on_hover_text(localizer.text(keys::EDITOR_CLOSE_FILE));
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
