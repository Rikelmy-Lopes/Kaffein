//! egui presentation for Kaffein.
//!
//! The UI reads [`AppState`], renders it, and emits [`UiAction`] values. It
//! never mutates application or domain state; the binary dispatches the
//! collected actions through [`kaffein_app::Workspace`].

pub mod theme;

use std::path::PathBuf;

use egui::{
    Align, Align2, Button, CentralPanel, Color32, Context, Frame, Key, KeyboardShortcut, Label,
    Layout, Margin, Modifiers, RichText, ScrollArea, SidePanel, TextStyle, TopBottomPanel, Ui,
    Vec2, Window,
};
use kaffein_app::{
    command_id, AppState, ArgKind, CommandArgs, CommandOutcome, CommandRegistry, Enablement,
    MessageSeverity, ThemePreference,
};
use kaffein_domain::{DocumentId, ProjectEntry};

use crate::theme::{
    Theme, ROW_COMPACT, ROW_STANDARD, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5,
    UI_SMALL_FONT_SIZE,
};

/// A command invocation produced by the UI and dispatched by the binary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiAction {
    pub command: &'static str,
    pub args: CommandArgs,
}

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

#[derive(Debug, Default)]
struct EditorBuffer {
    document: Option<DocumentId>,
    revision: Option<u64>,
    text: String,
}

#[derive(Clone, Debug)]
struct PathPrompt {
    command: &'static str,
    title: &'static str,
    kind: ArgKind,
    value: String,
    error: Option<String>,
    request_focus: bool,
}

/// Presentation-only state. It is not part of the application model.
#[derive(Debug, Default)]
pub struct UiState {
    prompt: Option<PathPrompt>,
    bottom_tab: BottomTab,
    editor: EditorBuffer,
    applied_theme: Option<ThemePreference>,
}

impl UiState {
    /// Called by the binary after dispatching an action, so a failed path
    /// prompt can stay open and show the error.
    pub fn after_dispatch(&mut self, command: &str, outcome: &CommandOutcome) {
        let Some(prompt) = self.prompt.as_mut() else {
            return;
        };
        if prompt.command != command {
            return;
        }
        match outcome {
            CommandOutcome::Success { .. } | CommandOutcome::Cancelled => {
                self.prompt = None;
            }
            CommandOutcome::Failed(_) | CommandOutcome::NoOp { .. } => {
                prompt.error = outcome.message().map(str::to_owned);
            }
        }
    }

    pub fn prompt_is_open(&self) -> bool {
        self.prompt.is_some()
    }

    fn open_prompt(&mut self, command: kaffein_app::Command, state: &AppState) {
        let value = default_prompt_path(command.args, state);
        self.prompt = Some(PathPrompt {
            command: command.id,
            title: command.title,
            kind: command.args,
            value,
            error: None,
            request_focus: true,
        });
    }
}

/// Renders one frame and returns every command the user triggered.
pub fn draw(
    ctx: &Context,
    state: &AppState,
    registry: &CommandRegistry,
    ui_state: &mut UiState,
) -> Vec<UiAction> {
    let theme = theme_for(state.theme());
    apply_theme(ctx, state, ui_state, theme);

    let now_ms = ctx.input(|input| (input.time * 1000.0) as u64);
    let mut actions = Vec::new();

    if !ui_state.prompt_is_open() {
        handle_shortcuts(ctx, state, registry, ui_state, &mut actions);
    }

    draw_menu_bar(ctx, state, registry, ui_state, &mut actions, theme);
    draw_toolbar(ctx, state, registry, ui_state, &mut actions, theme);
    draw_status_bar(ctx, state, theme);
    if state.show_project_panel() {
        draw_project_panel(ctx, state, registry, ui_state, &mut actions, theme);
    }
    if state.show_bottom_panel() {
        draw_bottom_panel(ctx, ui_state, theme);
    }
    draw_editor_panel(ctx, state, registry, ui_state, &mut actions, theme, now_ms);
    draw_path_prompt(ctx, ui_state, theme, &mut actions);

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

fn handle_shortcuts(
    ctx: &Context,
    state: &AppState,
    registry: &CommandRegistry,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
) {
    for command in registry.with_shortcuts() {
        let matched = command.shortcuts.iter().any(|binding| {
            let shortcut = KeyboardShortcut::new(
                if binding.ctrl {
                    Modifiers::CTRL
                } else {
                    Modifiers::NONE
                } | if binding.shift {
                    Modifiers::SHIFT
                } else {
                    Modifiers::NONE
                } | if binding.alt {
                    Modifiers::ALT
                } else {
                    Modifiers::NONE
                },
                egui_key(binding.key),
            );
            ctx.input_mut(|input| input.consume_shortcut(&shortcut))
        });

        if matched {
            trigger(command, state, ui_state, actions);
        }
    }
}

fn egui_key(key: kaffein_app::Key) -> Key {
    match key {
        kaffein_app::Key::N => Key::N,
        kaffein_app::Key::O => Key::O,
        kaffein_app::Key::S => Key::S,
        kaffein_app::Key::Z => Key::Z,
        kaffein_app::Key::Y => Key::Y,
        kaffein_app::Key::Q => Key::Q,
    }
}

fn trigger(
    command: kaffein_app::Command,
    state: &AppState,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
) {
    if command.args.needs_path_prompt() {
        ui_state.open_prompt(command, state);
    } else {
        actions.push(UiAction {
            command: command.id,
            args: CommandArgs::None,
        });
    }
}

fn draw_menu_bar(
    ctx: &Context,
    state: &AppState,
    registry: &CommandRegistry,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
    theme: Theme,
) {
    TopBottomPanel::top("menu_bar")
        .exact_height(ROW_STANDARD)
        .frame(
            Frame::none()
                .fill(theme.surface_panel)
                .inner_margin(Margin::symmetric(SPACE_4, 0.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().button_padding = Vec2::new(SPACE_4, SPACE_1);
                for menu in kaffein_app::Menu::ALL {
                    ui.menu_button(menu.title(), |ui| {
                        ui.set_min_width(220.0);
                        for command in registry.menu_commands(menu) {
                            if command
                                .menu
                                .is_some_and(|placement| placement.separator_before)
                            {
                                ui.separator();
                            }
                            let enablement = command.enablement(state);
                            let checked = command.checked.is_some_and(|predicate| predicate(state));
                            let prefix = if checked { "• " } else { "   " };
                            let mut button =
                                Button::new(format!("{prefix}{}", command.title)).frame(false);
                            if let Some(shortcut) = command.primary_shortcut() {
                                button = button.shortcut_text(shortcut.display());
                            }
                            let response = ui.add_enabled(enablement.is_enabled(), button);
                            let response = match enablement {
                                Enablement::Disabled(reason) => {
                                    response.on_disabled_hover_text(reason)
                                }
                                Enablement::Enabled => response,
                            };
                            if response.clicked() {
                                trigger(command, state, ui_state, actions);
                                ui.close_menu();
                            }
                        }
                    });
                }
            });
        });
}

fn draw_toolbar(
    ctx: &Context,
    state: &AppState,
    registry: &CommandRegistry,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
    theme: Theme,
) {
    TopBottomPanel::top("toolbar")
        .exact_height(ROW_STANDARD + SPACE_3)
        .frame(
            Frame::none()
                .fill(theme.surface_panel)
                .inner_margin(Margin::symmetric(SPACE_4, SPACE_2)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().button_padding = Vec2::new(SPACE_4, 2.0);
                for command in registry.toolbar_commands() {
                    let Some(placement) = command.toolbar else {
                        continue;
                    };
                    let enablement = command.enablement(state);
                    let response = ui.add_enabled(
                        enablement.is_enabled(),
                        Button::new(placement.label).min_size(Vec2::new(0.0, ROW_COMPACT)),
                    );
                    let tooltip = match command.primary_shortcut() {
                        Some(shortcut) => {
                            format!("{}\n{}", command.title, shortcut.display())
                        }
                        None => command.title.to_owned(),
                    };
                    let response = match enablement {
                        Enablement::Disabled(reason) => response.on_disabled_hover_text(reason),
                        Enablement::Enabled => response.on_hover_text(tooltip),
                    };
                    if response.clicked() {
                        trigger(command, state, ui_state, actions);
                    }
                }
            });
        });
}

fn draw_status_bar(ctx: &Context, state: &AppState, theme: Theme) {
    TopBottomPanel::bottom("status_bar")
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
                    .unwrap_or("No project");
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
                        RichText::new(format!("{} lines", document.line_count()))
                            .size(UI_SMALL_FONT_SIZE)
                            .color(theme.text_muted),
                    );
                    ui.label(
                        RichText::new("UTF-8")
                            .size(UI_SMALL_FONT_SIZE)
                            .color(theme.text_muted),
                    );
                    ui.label(
                        RichText::new(document.line_ending().label())
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
                    ui.add(
                        Label::new(
                            RichText::new(&status.text)
                                .size(UI_SMALL_FONT_SIZE)
                                .color(color),
                        )
                        .truncate(),
                    )
                    .on_hover_text(&status.text);
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(state.theme().label())
                            .size(UI_SMALL_FONT_SIZE)
                            .color(theme.text_muted),
                    );
                });
            });
        });
}

fn draw_project_panel(
    ctx: &Context,
    state: &AppState,
    registry: &CommandRegistry,
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
                    RichText::new("PROJECT")
                        .size(UI_SMALL_FONT_SIZE)
                        .color(theme.text_muted)
                        .strong(),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(command) = registry.get(command_id::PROJECT_OPEN) {
                        if ui
                            .small_button("Open…")
                            .on_hover_text("Open a project folder")
                            .clicked()
                        {
                            trigger(command, state, ui_state, actions);
                        }
                    }
                });
            });
            ui.separator();

            let Some(project) = state.project() else {
                ui.add_space(SPACE_5);
                ui.label(
                    RichText::new("No project open")
                        .color(theme.text_muted)
                        .size(UI_SMALL_FONT_SIZE),
                );
                ui.add_space(SPACE_2);
                if let Some(command) = registry.get(command_id::PROJECT_OPEN) {
                    if ui.button("Open Project…").clicked() {
                        trigger(command, state, ui_state, actions);
                    }
                }
                return;
            };

            let active_path = state.active_document().map(|document| document.path());
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
        egui::CollapsingHeader::new(RichText::new(entry.name()).color(theme.text_primary))
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
        response.on_hover_text(kaffein_app::display_path(entry.path()));
    }
}

fn draw_bottom_panel(ctx: &Context, ui_state: &mut UiState, theme: Theme) {
    TopBottomPanel::bottom("bottom_tools")
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

fn draw_editor_panel(
    ctx: &Context,
    state: &AppState,
    registry: &CommandRegistry,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
    theme: Theme,
    now_ms: u64,
) {
    CentralPanel::default()
        .frame(Frame::none().fill(theme.surface_editor))
        .show(ctx, |ui| {
            if state.documents().is_empty() {
                ui.centered_and_justified(|ui| {
                    let shortcut = registry
                        .get(command_id::FILE_OPEN)
                        .and_then(|command| command.primary_shortcut())
                        .map(|shortcut| format!(" ({})", shortcut.display()))
                        .unwrap_or_default();
                    ui.label(
                        RichText::new(format!(
                            "No file open — use File › Open File…{shortcut} or open a project."
                        ))
                        .size(theme::WELCOME_FONT_SIZE)
                        .color(theme.text_muted),
                    );
                });
                return;
            }

            draw_tabs(ui, state, actions, theme);
            ui.separator();
            draw_editor_text(ui, state, ui_state, actions, theme, now_ms);
        });
}

fn draw_tabs(ui: &mut Ui, state: &AppState, actions: &mut Vec<UiAction>, theme: Theme) {
    Frame::none()
        .fill(theme.surface_panel)
        .inner_margin(Margin::symmetric(SPACE_4, SPACE_1))
        .show(ui, |ui| {
            ScrollArea::horizontal()
                .id_salt("editor_tabs")
                .max_height(ROW_STANDARD)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = SPACE_1;
                        for document in state.documents() {
                            let id = document.id();
                            let active = state.active_id() == Some(id);
                            let mut title = document.display_name();
                            if document.is_dirty() {
                                title.push_str(" •");
                            }
                            let text = RichText::new(title).color(if active {
                                theme.text_primary
                            } else {
                                theme.text_muted
                            });
                            let response = ui.selectable_label(active, text);
                            if response.clicked() && !active {
                                actions.push(UiAction {
                                    command: command_id::EDITOR_ACTIVATE,
                                    args: CommandArgs::Document(id),
                                });
                            }
                            let close = ui
                                .add(
                                    Button::new("×")
                                        .frame(false)
                                        .min_size(Vec2::splat(ROW_COMPACT)),
                                )
                                .on_hover_text("Close file");
                            if close.clicked() || response.middle_clicked() {
                                actions.push(UiAction {
                                    command: command_id::FILE_CLOSE,
                                    args: CommandArgs::Document(id),
                                });
                            }
                            ui.add_space(SPACE_2);
                        }
                    });
                });
        });
}

fn draw_editor_text(
    ui: &mut Ui,
    state: &AppState,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
    theme: Theme,
    now_ms: u64,
) {
    let Some(document) = state.active_document() else {
        return;
    };
    let id = document.id();
    let revision = document.revision();
    let line_count = document.line_count();

    if ui_state.editor.document != Some(id) || ui_state.editor.revision != Some(revision) {
        ui_state.editor.document = Some(id);
        ui_state.editor.revision = Some(revision);
        ui_state.editor.text = document.buffer().to_owned();
    }

    let mut changed = false;
    ScrollArea::vertical()
        .id_salt("editor_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let response = ui.add(
                egui::TextEdit::multiline(&mut ui_state.editor.text)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .desired_rows(line_count.max(1) + 4)
                    .margin(Margin::symmetric(SPACE_4, SPACE_3))
                    .text_color(theme.text_primary)
                    .font(TextStyle::Monospace),
            );
            changed = response.changed();
        });

    if changed {
        ui_state.editor.revision = None;
        actions.push(UiAction {
            command: command_id::EDITOR_REPLACE_TEXT,
            args: CommandArgs::ReplaceText {
                document: id,
                text: ui_state.editor.text.clone(),
                at_ms: now_ms,
            },
        });
    }
}

fn draw_path_prompt(
    ctx: &Context,
    ui_state: &mut UiState,
    _theme: Theme,
    actions: &mut Vec<UiAction>,
) {
    let Some(prompt) = ui_state.prompt.as_mut() else {
        return;
    };

    let mut submit = false;
    let mut cancel = false;

    Window::new(prompt.title)
        .id(egui::Id::new("path_prompt"))
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .min_width(560.0)
        .max_width(560.0)
        .show(ctx, |ui| {
            let (label, hint, action_label) = match prompt.kind {
                ArgKind::Directory => (
                    "Project folder",
                    "Paste a folder path, for example C:\\work\\project",
                    "Open Project",
                ),
                ArgKind::NewFile => (
                    "File path",
                    "Paste a new file path, for example C:\\work\\project\\src\\New.java",
                    "Create File",
                ),
                _ => (
                    "File path",
                    "Paste a file path, for example C:\\work\\project\\src\\App.java",
                    "Open File",
                ),
            };
            ui.label(label);
            let response = ui.add(
                egui::TextEdit::singleline(&mut prompt.value)
                    .hint_text(hint)
                    .desired_width(f32::INFINITY),
            );
            if prompt.request_focus {
                response.request_focus();
                prompt.request_focus = false;
            }
            if response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter)) {
                submit = true;
            }
            if let Some(error) = &prompt.error {
                ui.add_space(SPACE_2);
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
            ui.add_space(SPACE_4);
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }
                if ui.button(action_label).clicked() {
                    submit = true;
                }
            });
            if ui.input(|input| input.key_pressed(Key::Escape)) {
                cancel = true;
            }
        });

    if cancel {
        ui_state.prompt = None;
        return;
    }

    if submit {
        let value = prompt.value.trim().to_owned();
        if value.is_empty() {
            prompt.error = Some("Enter a path.".to_owned());
            return;
        }
        actions.push(UiAction {
            command: prompt.command,
            args: CommandArgs::path(PathBuf::from(value)),
        });
    }
}

fn default_prompt_path(kind: ArgKind, state: &AppState) -> String {
    let directory = match kind {
        ArgKind::Directory => state.project().map(|project| project.root().to_path_buf()),
        ArgKind::NewFile | ArgKind::File => state
            .active_document()
            .and_then(|document| document.path().parent().map(std::path::Path::to_path_buf))
            .or_else(|| state.project().map(|project| project.root().to_path_buf())),
        _ => None,
    };
    let path = match kind {
        ArgKind::NewFile => directory.map(|directory| directory.join("Untitled.java")),
        _ => directory,
    };
    path.map(|path| kaffein_app::display_path(&path))
        .unwrap_or_default()
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
    use kaffein_app::{AppState, CommandRegistry, ThemePreference};
    use kaffein_domain::{Document, DocumentId};

    fn draw_frame(state: &AppState, ui_state: &mut UiState) -> Vec<UiAction> {
        let ctx = Context::default();
        let registry = CommandRegistry::new();
        let mut actions = Vec::new();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            actions = draw(ctx, state, &registry, ui_state);
        });
        actions
    }

    #[test]
    fn an_empty_workspace_assembles_without_actions() {
        let state = AppState::new(ThemePreference::Dark);
        let mut ui_state = UiState::default();

        let actions = draw_frame(&state, &mut ui_state);

        assert!(actions.is_empty());
        assert!(!ui_state.prompt_is_open());
    }

    #[test]
    fn an_open_document_renders_and_emits_no_actions() {
        let mut state = AppState::new(ThemePreference::Dark);
        state.add_document(Document::from_loaded(
            DocumentId(1),
            "C:/project/App.java",
            "class App {}\n",
        ));
        let mut ui_state = UiState::default();

        let actions = draw_frame(&state, &mut ui_state);

        assert!(actions.is_empty());
    }

    #[test]
    fn a_prompt_closes_after_a_successful_dispatch() {
        let state = AppState::new(ThemePreference::Dark);
        let mut ui_state = UiState::default();
        let registry = CommandRegistry::new();
        let command = registry.get(command_id::FILE_OPEN).expect("command");
        ui_state.open_prompt(command, &state);

        ui_state.after_dispatch(
            command_id::FILE_OPEN,
            &CommandOutcome::success("Opened App.java"),
        );

        assert!(!ui_state.prompt_is_open());
    }

    #[test]
    fn a_prompt_stays_open_and_shows_a_failure() {
        let state = AppState::new(ThemePreference::Dark);
        let mut ui_state = UiState::default();
        let registry = CommandRegistry::new();
        let command = registry.get(command_id::FILE_OPEN).expect("command");
        ui_state.open_prompt(command, &state);

        ui_state.after_dispatch(
            command_id::FILE_OPEN,
            &CommandOutcome::failed(kaffein_app::ErrorKind::UserInput, "missing file"),
        );

        assert!(ui_state.prompt_is_open());
        assert_eq!(
            ui_state
                .prompt
                .as_ref()
                .and_then(|prompt| prompt.error.as_deref()),
            Some("missing file")
        );
    }
}
