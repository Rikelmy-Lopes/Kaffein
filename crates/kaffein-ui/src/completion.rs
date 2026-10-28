//! The completion popup: a caret-anchored, keyboard- and mouse-operable list.

use egui::{Area, Context, Frame, Id, Order, Pos2, RichText, ScrollArea, Stroke, TextStyle};
use kaffein_app::{command_id, AppState, CommandArgs};
use kaffein_i18n::{keys, Localizer};

use crate::actions::UiAction;
use crate::theme::{Theme, SPACE_1, SPACE_2, SPACE_4};

const POPUP_MIN_WIDTH: f32 = 280.0;
const POPUP_MAX_HEIGHT: f32 = 220.0;

pub(crate) fn draw_completion_popup(
    ctx: &Context,
    state: &AppState,
    localizer: &Localizer,
    actions: &mut Vec<UiAction>,
    theme: Theme,
    now_ms: u64,
    anchor: Pos2,
) {
    let Some(session) = state.completion() else {
        return;
    };
    let mut chosen = None;

    Area::new(Id::new("completion_popup"))
        .order(Order::Foreground)
        .fixed_pos(anchor)
        .constrain(true)
        .show(ctx, |ui| {
            Frame::popup(&ctx.style())
                .fill(theme.surface_raised)
                .stroke(Stroke::new(1.0_f32, theme.border))
                .inner_margin(SPACE_2)
                .show(ui, |ui| {
                    ui.set_min_width(POPUP_MIN_WIDTH);
                    ScrollArea::vertical()
                        .id_salt("completion_popup_scroll")
                        .max_height(POPUP_MAX_HEIGHT)
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = SPACE_1;
                            for (index, item) in session.items.iter().enumerate() {
                                let selected = index == session.selected;
                                let label = if item.detail.is_empty() {
                                    item.label.clone()
                                } else {
                                    format!("{}    {}", item.label, item.detail)
                                };
                                let text = RichText::new(label)
                                    .text_style(TextStyle::Monospace)
                                    .color(if selected {
                                        theme.selection_text
                                    } else {
                                        theme.text_primary
                                    });
                                let response = ui.selectable_label(selected, text);
                                if response.clicked() {
                                    chosen = Some(index);
                                }
                            }
                        });
                    ui.add_space(SPACE_4);
                    ui.label(
                        RichText::new(localizer.text(keys::COMPLETION_HINT))
                            .small()
                            .color(theme.text_muted),
                    );
                });
        });

    if let Some(index) = chosen {
        actions.push(UiAction {
            command: command_id::EDITOR_COMPLETE_ACCEPT,
            args: CommandArgs::CompleteAccept {
                document: session.document,
                at_ms: now_ms,
                item: Some(index),
            },
        });
    }
}
