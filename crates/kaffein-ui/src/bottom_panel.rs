//! The bottom tool window with placeholder tabs.

use egui::{Context, Frame, Margin, RichText, Vec2};
use kaffein_i18n::{keys, Localizer};

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
    localizer: &Localizer,
    ui_state: &mut UiState,
    theme: Theme,
) {
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
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            RichText::new(localizer.text(ui_state.bottom_tab.empty_key()))
                                .size(UI_SMALL_FONT_SIZE)
                                .color(theme.text_muted),
                        );
                    });
                });
        });
}
