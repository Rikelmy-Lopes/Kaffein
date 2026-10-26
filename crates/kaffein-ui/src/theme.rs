//! Design tokens from `docs/ui-design.md` and their egui mapping.
//!
//! The code is the source of truth for values; the documentation mirrors it.

use egui::{Color32, FontFamily, FontId, Margin, Rounding, Shadow, Stroke, TextStyle, Vec2};
use kaffein_app::MessageSeverity;

/// Spacing scale from the design document.
pub const SPACE_1: f32 = 2.0;
pub const SPACE_2: f32 = 4.0;
pub const SPACE_3: f32 = 6.0;
pub const SPACE_4: f32 = 8.0;
pub const SPACE_5: f32 = 12.0;
pub const SPACE_6: f32 = 16.0;
pub const SPACE_7: f32 = 24.0;

pub const RADIUS_SM: f32 = 3.0;
pub const RADIUS_MD: f32 = 4.0;
pub const RADIUS_LG: f32 = 6.0;

pub const ROW_COMPACT: f32 = 22.0;
pub const ROW_STANDARD: f32 = 28.0;

pub const UI_FONT_SIZE: f32 = 13.0;
pub const UI_SMALL_FONT_SIZE: f32 = 12.0;
pub const CODE_FONT_SIZE: f32 = 13.0;
pub const DIALOG_TITLE_FONT_SIZE: f32 = 15.0;
pub const WELCOME_FONT_SIZE: f32 = 20.0;
pub const BORDER_WIDTH: f32 = 1.0;
pub const FOCUS_WIDTH: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub dark: bool,
    pub surface_editor: Color32,
    pub surface_panel: Color32,
    pub surface_raised: Color32,
    pub surface_sunken: Color32,
    pub border: Color32,
    pub border_subtle: Color32,
    pub border_strong: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub text_disabled: Color32,
    pub accent: Color32,
    pub accent_muted: Color32,
    pub selection_row: Color32,
    pub selection_editor: Color32,
    pub selection_text: Color32,
    pub selection_inactive: Color32,
    pub error: Color32,
    pub warning: Color32,
    pub success: Color32,
    pub info: Color32,
    pub diff_added: Color32,
    pub diff_removed: Color32,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            dark: true,
            surface_editor: rgb(0x1E, 0x1F, 0x22),
            surface_panel: rgb(0x2B, 0x2D, 0x30),
            surface_raised: rgb(0x35, 0x37, 0x3B),
            surface_sunken: rgb(0x23, 0x25, 0x28),
            border: rgb(0x43, 0x45, 0x4A),
            border_subtle: rgb(0x31, 0x33, 0x37),
            border_strong: rgb(0x76, 0x7A, 0x82),
            text_primary: rgb(0xDF, 0xE1, 0xE5),
            text_muted: rgb(0x9D, 0xA0, 0xA8),
            text_disabled: rgb(0x6A, 0x6D, 0x73),
            accent: rgb(0x8A, 0xB4, 0xFF),
            accent_muted: rgb(0x1E, 0x3A, 0x6E),
            selection_row: rgb(0x2E, 0x43, 0x6E),
            selection_editor: rgb(0x2E, 0x43, 0x6E),
            selection_text: rgb(0xE8, 0xEA, 0xED),
            selection_inactive: rgb(0x3A, 0x3D, 0x42),
            error: rgb(0xFF, 0x7A, 0x85),
            warning: rgb(0xE5, 0xA1, 0x3A),
            success: rgb(0x62, 0xB5, 0x43),
            info: rgb(0x6B, 0xA6, 0xFF),
            diff_added: rgb(0x1F, 0x3A, 0x2B),
            diff_removed: rgb(0x45, 0x26, 0x2B),
        }
    }

    pub fn light() -> Self {
        Self {
            dark: false,
            surface_editor: rgb(0xFF, 0xFF, 0xFF),
            surface_panel: rgb(0xF4, 0xF5, 0xF7),
            surface_raised: rgb(0xFF, 0xFF, 0xFF),
            surface_sunken: rgb(0xEC, 0xEE, 0xF1),
            border: rgb(0xD3, 0xD5, 0xDB),
            border_subtle: rgb(0xE4, 0xE6, 0xEA),
            border_strong: rgb(0x7A, 0x7F, 0x88),
            text_primary: rgb(0x1F, 0x23, 0x28),
            text_muted: rgb(0x62, 0x6A, 0x78),
            text_disabled: rgb(0xA3, 0xA7, 0xAE),
            accent: rgb(0x22, 0x58, 0xD8),
            accent_muted: rgb(0xDB, 0xEA, 0xFE),
            selection_row: rgb(0xD6, 0xE4, 0xFF),
            selection_editor: rgb(0xD6, 0xE4, 0xFF),
            selection_text: rgb(0x1F, 0x23, 0x28),
            selection_inactive: rgb(0xE8, 0xEA, 0xEE),
            error: rgb(0xC4, 0x31, 0x4B),
            warning: rgb(0x94, 0x59, 0x0A),
            success: rgb(0x2A, 0x74, 0x30),
            info: rgb(0x1D, 0x4E, 0xD8),
            diff_added: rgb(0xE6, 0xFF, 0xEC),
            diff_removed: rgb(0xFF, 0xEB, 0xE9),
        }
    }

    /// Installs this theme as the active egui style. Called only when the
    /// preference changes, not every frame.
    pub fn apply(&self, ctx: &egui::Context) {
        ctx.set_theme(if self.dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });

        let mut style = egui::Style::default();
        let visuals = &mut style.visuals;
        visuals.dark_mode = self.dark;
        visuals.override_text_color = Some(self.text_primary);
        visuals.panel_fill = self.surface_panel;
        visuals.window_fill = self.surface_raised;
        visuals.extreme_bg_color = self.surface_editor;
        visuals.faint_bg_color = self.surface_sunken;
        visuals.hyperlink_color = self.accent;
        visuals.warn_fg_color = self.warning;
        visuals.error_fg_color = self.error;
        visuals.selection.bg_fill = self.selection_editor;
        visuals.selection.stroke = Stroke::new(BORDER_WIDTH, self.selection_text);
        visuals.text_cursor.stroke = Stroke::new(FOCUS_WIDTH, self.accent);
        visuals.window_rounding = Rounding::same(RADIUS_LG);
        visuals.menu_rounding = Rounding::same(RADIUS_MD);
        visuals.window_shadow = Shadow {
            offset: Vec2::new(0.0, 4.0),
            blur: 12.0,
            spread: 0.0,
            color: Color32::from_black_alpha(64),
        };
        visuals.popup_shadow = visuals.window_shadow;

        let widgets = &mut visuals.widgets;
        widgets.noninteractive.bg_fill = self.surface_panel;
        widgets.noninteractive.weak_bg_fill = self.surface_panel;
        widgets.noninteractive.bg_stroke = Stroke::new(BORDER_WIDTH, self.border_subtle);
        widgets.noninteractive.fg_stroke = Stroke::new(BORDER_WIDTH, self.text_primary);

        widgets.inactive.bg_fill = self.surface_panel;
        widgets.inactive.weak_bg_fill = self.surface_panel;
        widgets.inactive.bg_stroke = Stroke::new(BORDER_WIDTH, self.border_subtle);
        widgets.inactive.fg_stroke = Stroke::new(BORDER_WIDTH, self.text_primary);

        widgets.hovered.bg_fill = self.surface_raised;
        widgets.hovered.weak_bg_fill = self.surface_raised;
        widgets.hovered.bg_stroke = Stroke::new(BORDER_WIDTH, self.border_strong);
        widgets.hovered.fg_stroke = Stroke::new(BORDER_WIDTH, self.text_primary);

        widgets.active.bg_fill = self.accent_muted;
        widgets.active.weak_bg_fill = self.accent_muted;
        widgets.active.bg_stroke = Stroke::new(BORDER_WIDTH, self.accent);
        widgets.active.fg_stroke = Stroke::new(BORDER_WIDTH, self.text_primary);

        widgets.open.bg_fill = self.surface_raised;
        widgets.open.weak_bg_fill = self.surface_raised;
        widgets.open.bg_stroke = Stroke::new(BORDER_WIDTH, self.border_strong);
        widgets.open.fg_stroke = Stroke::new(BORDER_WIDTH, self.text_primary);

        style.spacing.item_spacing = Vec2::new(SPACE_2, SPACE_1);
        style.spacing.button_padding = Vec2::new(SPACE_4, 3.0);
        style.spacing.window_margin = Margin::same(SPACE_4);
        style.spacing.menu_margin = Margin::same(SPACE_2);
        style.spacing.indent = SPACE_5;
        style.spacing.interact_size = Vec2::new(0.0, ROW_COMPACT);

        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(UI_FONT_SIZE, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(UI_FONT_SIZE, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(UI_SMALL_FONT_SIZE, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(CODE_FONT_SIZE, FontFamily::Monospace),
        );
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(DIALOG_TITLE_FONT_SIZE, FontFamily::Proportional),
        );

        ctx.set_style(style);
    }
}

fn rgb(red: u8, green: u8, blue: u8) -> Color32 {
    Color32::from_rgb(red, green, blue)
}

/// Severity colour for surfaces that render application messages.
pub fn severity_color(theme: &Theme, severity: MessageSeverity) -> Color32 {
    match severity {
        MessageSeverity::Info => theme.info,
        MessageSeverity::Warning => theme.warning,
        MessageSeverity::Error => theme.error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_themes_define_every_token() {
        for theme in [Theme::dark(), Theme::light()] {
            // Compile-time exhaustiveness is the real check; this asserts the
            // values survive round-trips through the public struct.
            assert_ne!(theme.text_primary, theme.surface_editor);
            assert_ne!(theme.accent, theme.surface_editor);
        }
    }
}
