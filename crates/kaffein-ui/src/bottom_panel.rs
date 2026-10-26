//! The bottom tool window with placeholder tabs.

use egui::{Context, Frame, Margin, RichText, Vec2};

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
    fn title(self) -> &'static str {
        match self {
            Self::Problems => "Problems",
            Self::Build => "Build",
            Self::Console => "Console",
        }
    }

    const ALL: [Self; 3] = [Self::Problems, Self::Build, Self::Console];
}

pub(crate) fn draw_bottom_panel(ctx: &Context, ui_state: &mut UiState, theme: Theme) {
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
                            ui.selectable_value(&mut ui_state.bottom_tab, tab, tab.title());
                        }
                    });
                });
            ui.separator();
            Frame::none()
                .fill(theme.surface_editor)
                .inner_margin(Margin::same(SPACE_5))
                .show(ui, |ui| {
                    ui.set_min_height(ui.available_height());
                    ui.centered_and_justified(|ui| {
                        let message = match ui_state.bottom_tab {
                            BottomTab::Problems => {
                                "No problems. Diagnostics appear once Java analysis is available."
                            }
                            BottomTab::Build => {
                                "Build output will appear here once build integration is implemented."
                            }
                            BottomTab::Console => {
                                "Process output will appear here once run integration is implemented."
                            }
                        };
                        ui.label(
                            RichText::new(message)
                                .size(UI_SMALL_FONT_SIZE)
                                .color(theme.text_muted),
                        );
                    });
                });
        });
}
