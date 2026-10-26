//! eframe adapter: owns the workspace, draws a frame, and dispatches the
//! actions the UI emitted.

use eframe::egui;
use kaffein_app::{command_id, CommandArgs, Services, ThemePreference, Workspace};
use kaffein_infra::{FsProjectLoader, StdFileSystem};
use kaffein_ui::UiState;

pub(crate) struct KaffeinApp {
    workspace: Workspace,
    ui_state: UiState,
}

impl KaffeinApp {
    pub(crate) fn new() -> Self {
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
