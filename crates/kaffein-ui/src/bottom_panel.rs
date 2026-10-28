//! The bottom tool window: placeholder tabs and the Java run console.

#[cfg(test)]
mod tests;

use egui::{Context, Frame, Margin, RichText, ScrollArea, Vec2};
use kaffein_app::RunState;
use kaffein_domain::JavaRunStage;
use kaffein_i18n::{keys, Localizer, Text};

use crate::theme::{Theme, SPACE_1, SPACE_2, SPACE_4, SPACE_5, UI_SMALL_FONT_SIZE};
use crate::ui_state::UiState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BottomTab {
    #[default]
    Problems,
    Build,
    Console,
}

impl BottomTab {
    fn title_key(self) -> &'static str {
        match self {
            Self::Problems => keys::BOTTOM_PANEL_TAB_PROBLEMS,
            Self::Build => keys::BOTTOM_PANEL_TAB_BUILD,
            Self::Console => keys::BOTTOM_PANEL_TAB_CONSOLE,
        }
    }

    fn empty_key(self) -> &'static str {
        match self {
            Self::Problems => keys::BOTTOM_PANEL_EMPTY_PROBLEMS,
            Self::Build => keys::BOTTOM_PANEL_EMPTY_BUILD,
            Self::Console => keys::BOTTOM_PANEL_EMPTY_CONSOLE,
        }
    }

    const ALL: [Self; 3] = [Self::Problems, Self::Build, Self::Console];
}

pub(crate) fn draw_bottom_panel(
    ctx: &Context,
    run: &RunState,
    localizer: &Localizer,
    ui_state: &mut UiState,
    theme: Theme,
) {
    if run.id() != 0 && run.id() != ui_state.seen_run_id {
        ui_state.seen_run_id = run.id();
        ui_state.bottom_tab = BottomTab::Console;
    }

    egui::TopBottomPanel::bottom("bottom_tools")
        .resizable(true)
        .default_height(160.0)
        .min_height(120.0)
        .max_height((ctx.screen_rect().height() * 0.5).max(160.0))
        .frame(Frame::none().fill(theme.surface_panel))
        .show(ctx, |ui| {
            Frame::none()
                .inner_margin(Margin::symmetric(SPACE_4, SPACE_1))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().button_padding = Vec2::new(SPACE_4, SPACE_2);
                        for tab in BottomTab::ALL {
                            ui.selectable_value(
                                &mut ui_state.bottom_tab,
                                tab,
                                localizer.text(tab.title_key()),
                            );
                        }
                    });
                });
            ui.separator();
            Frame::none()
                .fill(theme.surface_editor)
                .inner_margin(Margin::same(SPACE_5))
                .show(ui, |ui| {
                    ui.set_min_height(ui.available_height());
                    match ui_state.bottom_tab {
                        BottomTab::Console => draw_console(ui, run, localizer, theme),
                        tab => centered_empty(ui, localizer.text(tab.empty_key()), theme),
                    }
                });
        });
}

fn centered_empty(ui: &mut egui::Ui, text: &str, theme: Theme) {
    ui.centered_and_justified(|ui| {
        ui.label(
            RichText::new(text)
                .size(UI_SMALL_FONT_SIZE)
                .color(theme.text_muted),
        );
    });
}

fn draw_console(ui: &mut egui::Ui, run: &RunState, localizer: &Localizer, theme: Theme) {
    ScrollArea::vertical()
        .id_salt("run_console")
        .auto_shrink([false, false])
        .show(ui, |ui| match run {
            RunState::Idle => {
                centered_empty(ui, localizer.text(keys::BOTTOM_PANEL_EMPTY_CONSOLE), theme)
            }
            RunState::Running { source_name, .. } => {
                let source_name = sanitize_output(source_name);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(
                        Text::message(keys::BOTTOM_PANEL_CONSOLE_RUNNING)
                            .arg("name", source_name)
                            .resolve(localizer),
                    );
                });
            }
            RunState::Failed { detail, .. } => {
                let detail = sanitize_output(detail);
                ui.label(
                    RichText::new(
                        Text::message(keys::BOTTOM_PANEL_CONSOLE_RUN_FAILED)
                            .arg("error", detail)
                            .resolve(localizer),
                    )
                    .color(theme.error),
                );
            }
            RunState::Finished { output, .. } => {
                let (summary, color) = run_summary(
                    output.stage,
                    output.exit_code,
                    output.timed_out,
                    localizer,
                    theme,
                );
                ui.label(RichText::new(summary).color(color));
                if output.stdout.is_empty() && output.stderr.is_empty() {
                    ui.label(
                        RichText::new(localizer.text(keys::BOTTOM_PANEL_CONSOLE_NO_OUTPUT))
                            .size(UI_SMALL_FONT_SIZE)
                            .color(theme.text_muted),
                    );
                } else {
                    output_section(
                        ui,
                        localizer.text(keys::BOTTOM_PANEL_CONSOLE_STDOUT),
                        &output.stdout,
                        theme.text_primary,
                        theme,
                    );
                    output_section(
                        ui,
                        localizer.text(keys::BOTTOM_PANEL_CONSOLE_STDERR),
                        &output.stderr,
                        theme.error,
                        theme,
                    );
                }
                if output.stdout_truncated || output.stderr_truncated {
                    ui.label(
                        RichText::new(localizer.text(keys::BOTTOM_PANEL_CONSOLE_TRUNCATED))
                            .size(UI_SMALL_FONT_SIZE)
                            .color(theme.warning),
                    );
                }
            }
        });
}

fn run_summary(
    stage: JavaRunStage,
    exit_code: Option<i32>,
    timed_out: bool,
    localizer: &Localizer,
    theme: Theme,
) -> (String, egui::Color32) {
    match stage {
        JavaRunStage::Compile if timed_out => (
            localizer
                .text(keys::BOTTOM_PANEL_CONSOLE_COMPILE_TIMED_OUT)
                .to_owned(),
            theme.error,
        ),
        JavaRunStage::Compile => (
            Text::message(keys::BOTTOM_PANEL_CONSOLE_COMPILE_FAILED)
                .arg("code", exit_code.map(i64::from).unwrap_or(-1))
                .resolve(localizer),
            theme.error,
        ),
        JavaRunStage::Run if timed_out => (
            localizer
                .text(keys::BOTTOM_PANEL_CONSOLE_RUN_TIMED_OUT)
                .to_owned(),
            theme.warning,
        ),
        JavaRunStage::Run if exit_code == Some(0) => (
            localizer
                .text(keys::BOTTOM_PANEL_CONSOLE_RUN_FINISHED)
                .to_owned(),
            theme.success,
        ),
        JavaRunStage::Run => (
            Text::message(keys::BOTTOM_PANEL_CONSOLE_RUN_EXIT)
                .arg("code", exit_code.map(i64::from).unwrap_or(-1))
                .resolve(localizer),
            theme.error,
        ),
    }
}

fn output_section(ui: &mut egui::Ui, title: &str, text: &str, color: egui::Color32, theme: Theme) {
    if text.is_empty() {
        return;
    }
    let text = sanitize_output(text);
    ui.add_space(SPACE_1);
    ui.label(
        RichText::new(title)
            .size(UI_SMALL_FONT_SIZE)
            .color(theme.text_muted),
    );
    ui.add(egui::Label::new(RichText::new(text).monospace().color(color)).selectable(true));
}

/// Child-process output is untrusted data. Strip terminal escape sequences and
/// other control characters before egui can render them.
fn sanitize_output(text: &str) -> String {
    let mut sanitized = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(character) = chars.next() {
        if character == '\u{1b}' {
            match chars.peek() {
                Some('[') => {
                    chars.next();
                    for next in chars.by_ref() {
                        if ('@'..='~').contains(&next) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    while let Some(next) = chars.next() {
                        if next == '\u{7}' {
                            break;
                        }
                        if next == '\u{1b}' {
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                    }
                }
                _ => {}
            }
            continue;
        }
        if matches!(character, '\n' | '\r' | '\t') || !character.is_control() {
            sanitized.push(character);
        }
    }

    sanitized
}
