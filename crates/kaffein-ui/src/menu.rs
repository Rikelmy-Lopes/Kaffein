//! The menu bar, generated from the command registry.

use egui::{Button, Context, Frame, Margin, Vec2};
use kaffein_app::{AppState, CommandRegistry, Enablement, Menu};

use crate::actions::{trigger, UiAction};
use crate::theme::{Theme, ROW_STANDARD, SPACE_1, SPACE_4};
use crate::ui_state::UiState;

pub(crate) fn draw_menu_bar(
    ctx: &Context,
    state: &AppState,
    registry: &CommandRegistry,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
    theme: Theme,
) {
    egui::TopBottomPanel::top("menu_bar")
        .exact_height(ROW_STANDARD)
        .frame(
            Frame::none()
                .fill(theme.surface_panel)
                .inner_margin(Margin::symmetric(SPACE_4, 0.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().button_padding = Vec2::new(SPACE_4, SPACE_1);
                for menu in Menu::ALL {
                    ui.menu_button(menu.title(), |ui| {
                        ui.set_min_width(220.0);
                        for command in registry.menu_commands(menu) {
                            if command
                                .menu
                                .is_some_and(|placement| placement.separator_before)
                            {
                                ui.separator();
                            }
                            let enablement = command.enablement(state);
                            let checked = command.checked.is_some_and(|predicate| predicate(state));
                            let prefix = if checked { "• " } else { "   " };
                            let mut button =
                                Button::new(format!("{prefix}{}", command.title)).frame(false);
                            if let Some(shortcut) = command.primary_shortcut() {
                                button = button.shortcut_text(shortcut.display());
                            }
                            let response = ui.add_enabled(enablement.is_enabled(), button);
                            let response = match enablement {
                                Enablement::Disabled(reason) => {
                                    response.on_disabled_hover_text(reason)
                                }
                                Enablement::Enabled => response,
                            };
                            if response.clicked() {
                                trigger(command, state, ui_state, actions);
                                ui.close_menu();
                            }
                        }
                    });
                }
            });
        });
}
