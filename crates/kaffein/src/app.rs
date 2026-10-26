//! eframe adapter: owns the workspace, draws a frame, and dispatches the
//! actions the UI emitted.

use eframe::egui;
use kaffein_app::{command_id, CommandArgs, Services, ThemePreference, Workspace};
use kaffein_i18n::Localizer;
use kaffein_infra::{FsProjectLoader, StdFileSystem};
use kaffein_ui::UiState;

pub(crate) struct KaffeinApp {
    workspace: Workspace,
    ui_state: UiState,
    title_locale: String,
}

impl KaffeinApp {
    pub(crate) fn new(localizer: Localizer) -> Self {
        let title_locale = localizer.active_locale().code().to_owned();
        let services = Services::new(
            Box::new(StdFileSystem::new()),
            Box::new(FsProjectLoader::new()),
            localizer,
        );
        Self {
            workspace: Workspace::new(services, ThemePreference::Dark),
            ui_state: UiState::default(),
            title_locale,
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
            self.workspace.localizer(),
            &mut self.ui_state,
        );

        for action in actions {
            let outcome = self.workspace.dispatch(action.command, action.args);
            self.ui_state.after_dispatch(action.command, &outcome);
        }

        self.sync_window_title(ctx);

        if self.workspace.state().quit_requested() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl KaffeinApp {
    /// Text rendered by egui follows the localizer automatically; the native
    /// window title is outside the frame and must be pushed explicitly.
    fn sync_window_title(&mut self, ctx: &egui::Context) {
        let locale = self.workspace.localizer().active_locale();
        if locale.code() == self.title_locale {
            return;
        }
        self.title_locale = locale.code().to_owned();
        let title = self
            .workspace
            .localizer()
            .text(kaffein_i18n::keys::APP_TITLE)
            .to_owned();
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }
}
