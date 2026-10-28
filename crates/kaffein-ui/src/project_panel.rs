//! The project tree panel.

use egui::{
    Align, CollapsingHeader, Context, Frame, Layout, Margin, RichText, ScrollArea, SidePanel, Ui,
};
use kaffein_app::{command_id, display_path, AppState, CommandArgs, CommandRegistry};
use kaffein_domain::ProjectEntry;
use kaffein_i18n::{keys, Localizer};

use crate::actions::{trigger, UiAction};
use crate::theme::{Theme, SPACE_2, SPACE_3, SPACE_4, SPACE_5, UI_SMALL_FONT_SIZE};
use crate::ui_state::UiState;

pub(crate) fn draw_project_panel(
    ctx: &Context,
    state: &AppState,
    registry: &CommandRegistry,
    localizer: &Localizer,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
    theme: Theme,
) {
    let max_width = (ctx.screen_rect().width() * 0.45).max(220.0);
    SidePanel::left("project_panel")
        .resizable(true)
        .default_width(280.0)
        .width_range(180.0..=max_width)
        .frame(
            Frame::none()
                .fill(theme.surface_panel)
                .inner_margin(Margin::symmetric(SPACE_4, SPACE_3)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(localizer.text(keys::PROJECT_PANEL_HEADER))
                        .size(UI_SMALL_FONT_SIZE)
                        .color(theme.text_muted)
                        .strong(),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(command) = registry.get(command_id::PROJECT_OPEN) {
                        if ui
                            .small_button(localizer.text(keys::PROJECT_PANEL_OPEN))
                            .on_hover_text(localizer.text(keys::PROJECT_PANEL_OPEN_TOOLTIP))
                            .clicked()
                        {
                            trigger(command, state, localizer, ui_state, actions);
                        }
                    }
                });
            });
            ui.separator();

            let Some(project) = state.project() else {
                ui.add_space(SPACE_5);
                ui.label(
                    RichText::new(localizer.text(keys::PROJECT_PANEL_EMPTY))
                        .color(theme.text_muted)
                        .size(UI_SMALL_FONT_SIZE),
                );
                ui.add_space(SPACE_2);
                if let Some(command) = registry.get(command_id::PROJECT_OPEN) {
                    if ui
                        .button(localizer.text(keys::PROJECT_PANEL_OPEN_PROJECT))
                        .clicked()
                    {
                        trigger(command, state, localizer, ui_state, actions);
                    }
                }
                return;
            };

            let active_path = state.active_document().and_then(|document| document.path());
            ScrollArea::vertical()
                .id_salt("project_tree")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.label(RichText::new(project.name()).strong());
                    for entry in project.children() {
                        draw_project_entry(ui, entry, active_path, actions, theme, 0);
                    }
                });
        });
}

fn draw_project_entry(
    ui: &mut Ui,
    entry: &ProjectEntry,
    active_path: Option<&std::path::Path>,
    actions: &mut Vec<UiAction>,
    theme: Theme,
    depth: usize,
) {
    if entry.is_directory() {
        CollapsingHeader::new(RichText::new(entry.name()).color(theme.text_primary))
            .id_salt(entry.path())
            .default_open(depth == 0)
            .show(ui, |ui| {
                for child in entry.children() {
                    draw_project_entry(ui, child, active_path, actions, theme, depth + 1);
                }
            });
    } else {
        let selected = active_path == Some(entry.path());
        let response = ui.selectable_label(selected, entry.name());
        if response.clicked() {
            actions.push(UiAction {
                command: command_id::FILE_OPEN,
                args: CommandArgs::path(entry.path()),
            });
        }
        response.on_hover_text(display_path(entry.path()));
    }
}
