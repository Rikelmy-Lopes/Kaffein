//! The status bar: project, active document, last message, and theme.

use egui::{Align, Context, Frame, Label, Layout, Margin, RichText};
use kaffein_app::{AppState, MessageSeverity};
use kaffein_domain::LineEnding;
use kaffein_i18n::{keys, Localizer};

use crate::theme::{Theme, ROW_COMPACT, SPACE_1, SPACE_4, UI_SMALL_FONT_SIZE};

pub(crate) fn draw_status_bar(
    ctx: &Context,
    state: &AppState,
    localizer: &Localizer,
    theme: Theme,
) {
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(ROW_COMPACT + SPACE_1)
        .frame(
            Frame::none()
                .fill(theme.surface_sunken)
                .inner_margin(Margin::symmetric(SPACE_4, 0.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let project_name = state
                    .project()
                    .map(kaffein_domain::ProjectTree::name)
                    .unwrap_or_else(|| localizer.text(keys::STATUS_NO_PROJECT));
                ui.label(
                    RichText::new(project_name)
                        .size(UI_SMALL_FONT_SIZE)
                        .color(theme.text_muted),
                );
                ui.separator();

                if let Some(document) = state.active_document() {
                    let mut name = document.display_name();
                    if document.is_dirty() {
                        name.push_str(" •");
                    }
                    ui.label(RichText::new(name).size(UI_SMALL_FONT_SIZE));
                    ui.separator();
                    ui.label(
                        RichText::new(
                            localizer.count(keys::STATUS_LINE_COUNT, document.line_count()),
                        )
                        .size(UI_SMALL_FONT_SIZE)
                        .color(theme.text_muted),
                    );
                    ui.label(
                        RichText::new(localizer.text(keys::STATUS_ENCODING_UTF8))
                            .size(UI_SMALL_FONT_SIZE)
                            .color(theme.text_muted),
                    );
                    let line_ending = match document.line_ending() {
                        LineEnding::Lf => keys::STATUS_LINE_ENDING_LF,
                        LineEnding::Crlf => keys::STATUS_LINE_ENDING_CRLF,
                    };
                    ui.label(
                        RichText::new(localizer.text(line_ending))
                            .size(UI_SMALL_FONT_SIZE)
                            .color(theme.text_muted),
                    );
                }

                if let Some(status) = state.status() {
                    ui.separator();
                    let color = match status.severity {
                        MessageSeverity::Info => theme.info,
                        MessageSeverity::Warning => theme.warning,
                        MessageSeverity::Error => theme.error,
                    };
                    let text = status.text.resolve(localizer);
                    ui.add(
                        Label::new(RichText::new(&text).size(UI_SMALL_FONT_SIZE).color(color))
                            .truncate(),
                    )
                    .on_hover_text(&text);
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(localizer.text(state.theme().label_key()))
                            .size(UI_SMALL_FONT_SIZE)
                            .color(theme.text_muted),
                    );
                    ui.separator();
                    ui.label(
                        RichText::new(localizer.active_locale().name())
                            .size(UI_SMALL_FONT_SIZE)
                            .color(theme.text_muted),
                    );
                });
            });
        });
}
