//! Kaffein desktop entry point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use kaffein_app::{command_id, CommandArgs, Services, ThemePreference, Workspace};
use kaffein_infra::{FsProjectLoader, StdFileSystem};
use kaffein_ui::UiState;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Kaffein")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Kaffein",
        options,
        Box::new(|_creation_context| Ok(Box::new(KaffeinApp::new()))),
    )
}

struct KaffeinApp {
    workspace: Workspace,
    ui_state: UiState,
}

impl KaffeinApp {
    fn new() -> Self {
        let services = Services::new(Box::new(StdFileSystem), Box::new(FsProjectLoader));
        Self {
            workspace: Workspace::new(services, ThemePreference::Dark),
            ui_state: UiState::default(),
        }
    }
}

impl eframe::App for KaffeinApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // The window close button must not discard unsaved documents. Route it
        // through the same command the menu uses.
        if ctx.input(|input| input.viewport().close_requested())
            && self.workspace.state().has_dirty_documents()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.workspace
                .dispatch(command_id::APP_QUIT, CommandArgs::None);
            ctx.request_repaint();
        }

        let actions = kaffein_ui::draw(
            ctx,
            self.workspace.state(),
            self.workspace.registry(),
            &mut self.ui_state,
        );

        for action in actions {
            let outcome = self.workspace.dispatch(action.command, action.args);
            self.ui_state.after_dispatch(action.command, &outcome);
        }

        if self.workspace.state().quit_requested() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
