//! The toolbar, generated from the command registry.

use egui::{Button, Context, Frame, Margin, Vec2};
use kaffein_app::{AppState, CommandRegistry, Enablement};

use crate::actions::{trigger, UiAction};
use crate::theme::{Theme, ROW_COMPACT, ROW_STANDARD, SPACE_2, SPACE_3, SPACE_4};
use crate::ui_state::UiState;

pub(crate) fn draw_toolbar(
    ctx: &Context,
    state: &AppState,
    registry: &CommandRegistry,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
    theme: Theme,
) {
    egui::TopBottomPanel::top("toolbar")
        .exact_height(ROW_STANDARD + SPACE_3)
        .frame(
            Frame::none()
                .fill(theme.surface_panel)
                .inner_margin(Margin::symmetric(SPACE_4, SPACE_2)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().button_padding = Vec2::new(SPACE_4, 2.0);
                for command in registry.toolbar_commands() {
                    let Some(placement) = command.toolbar else {
                        continue;
                    };
                    let enablement = command.enablement(state);
                    let response = ui.add_enabled(
                        enablement.is_enabled(),
                        Button::new(placement.label).min_size(Vec2::new(0.0, ROW_COMPACT)),
                    );
                    let tooltip = match command.primary_shortcut() {
                        Some(shortcut) => {
                            format!("{}\n{}", command.title, shortcut.display())
                        }
                        None => command.title.to_owned(),
                    };
                    let response = match enablement {
                        Enablement::Disabled(reason) => response.on_disabled_hover_text(reason),
                        Enablement::Enabled => response.on_hover_text(tooltip),
                    };
                    if response.clicked() {
                        trigger(command, state, ui_state, actions);
                    }
                }
            });
        });
}
