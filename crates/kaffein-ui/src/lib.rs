//! egui presentation for Kaffein.
//!
//! The UI reads [`AppState`], renders it, and emits [`UiAction`] values. It
//! never mutates application or domain state; the binary dispatches the
//! collected actions through [`kaffein_app::Workspace`].
//!
//! Module layout: this file wires one frame and owns nothing else. Each surface
//! has its own module, and `theme` owns the design tokens.

mod actions;
mod bottom_panel;
mod editor;
mod menu;
mod path_prompt;
mod project_panel;
mod status_bar;
mod syntax;
mod toolbar;
mod ui_state;

pub mod theme;

#[cfg(test)]
mod tests;

pub use actions::UiAction;
pub use bottom_panel::BottomTab;
pub use theme::severity_color;
pub use ui_state::UiState;

use egui::Context;
use kaffein_app::{AppState, CommandRegistry, ThemePreference};
use kaffein_i18n::Localizer;

use crate::theme::Theme;

/// Renders one frame and returns every command the user triggered.
pub fn draw(
    ctx: &Context,
    state: &AppState,
    registry: &CommandRegistry,
    localizer: &Localizer,
    ui_state: &mut UiState,
) -> Vec<UiAction> {
    let theme = theme_for(state.theme());
    apply_theme(ctx, state, ui_state, theme);

    let mut actions = Vec::new();

    if !ui_state.prompt_is_open() {
        actions::handle_shortcuts(ctx, state, registry, localizer, ui_state, &mut actions);
    }

    menu::draw_menu_bar(
        ctx,
        state,
        registry,
        localizer,
        ui_state,
        &mut actions,
        theme,
    );
    toolbar::draw_toolbar(
        ctx,
        state,
        registry,
        localizer,
        ui_state,
        &mut actions,
        theme,
    );
    status_bar::draw_status_bar(ctx, state, localizer, theme);
    if state.show_project_panel() {
        project_panel::draw_project_panel(
            ctx,
            state,
            registry,
            localizer,
            ui_state,
            &mut actions,
            theme,
        );
    }
    if state.show_bottom_panel() {
        let run = state.run_state();
        bottom_panel::draw_bottom_panel(ctx, &run, localizer, ui_state, theme);
    }
    editor::draw_editor_panel(ctx, state, localizer, ui_state, &mut actions, theme);
    path_prompt::draw_path_prompt(ctx, localizer, ui_state, &mut actions);

    actions
}

fn theme_for(preference: ThemePreference) -> Theme {
    match preference {
        ThemePreference::Dark => Theme::dark(),
        ThemePreference::Light => Theme::light(),
    }
}

fn apply_theme(ctx: &Context, state: &AppState, ui_state: &mut UiState, theme: Theme) {
    if ui_state.applied_theme == Some(state.theme()) {
        return;
    }
    theme.apply(ctx);
    ui_state.applied_theme = Some(state.theme());
}
