//! The path prompt dialog: a small window that collects a file or folder path
//! before the command is dispatched.

use std::path::PathBuf;

use egui::{Align2, Context, Key, Vec2, Window};
use kaffein_app::{display_path, AppState, ArgKind, Command, CommandArgs};
use kaffein_i18n::{keys, Localizer, Text};

use crate::actions::UiAction;
use crate::theme::{SPACE_2, SPACE_4};
use crate::ui_state::UiState;

#[derive(Clone, Debug)]
pub(crate) struct PathPrompt {
    pub(crate) command: &'static str,
    pub(crate) title_key: &'static str,
    pub(crate) kind: ArgKind,
    pub(crate) value: String,
    pub(crate) error: Option<Text>,
    pub(crate) request_focus: bool,
}

impl PathPrompt {
    pub(crate) fn for_command(command: Command, state: &AppState, localizer: &Localizer) -> Self {
        Self {
            command: command.id,
            title_key: command.title_key,
            kind: command.args,
            value: default_prompt_path(command.args, state, localizer),
            error: None,
            request_focus: true,
        }
    }
}

pub(crate) fn draw_path_prompt(
    ctx: &Context,
    localizer: &Localizer,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
) {
    let Some(prompt) = ui_state.prompt.as_mut() else {
        return;
    };

    let mut submit = false;
    let mut cancel = false;

    Window::new(localizer.text(prompt.title_key))
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
                    localizer.text(keys::PROMPT_DIRECTORY_LABEL),
                    localizer.text(keys::PROMPT_DIRECTORY_HINT),
                    localizer.text(keys::PROMPT_DIRECTORY_ACTION),
                ),
                ArgKind::NewFile => (
                    localizer.text(keys::PROMPT_NEW_FILE_LABEL),
                    localizer.text(keys::PROMPT_NEW_FILE_HINT),
                    localizer.text(keys::PROMPT_NEW_FILE_ACTION),
                ),
                _ => (
                    localizer.text(keys::PROMPT_FILE_LABEL),
                    localizer.text(keys::PROMPT_FILE_HINT),
                    localizer.text(keys::PROMPT_FILE_ACTION),
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
                ui.colored_label(ui.visuals().error_fg_color, error.resolve(localizer));
            }
            ui.add_space(SPACE_4);
            ui.horizontal(|ui| {
                if ui.button(localizer.text(keys::COMMON_CANCEL)).clicked() {
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
            prompt.error = Some(Text::key(keys::PROMPT_ERROR_PATH_REQUIRED));
            return;
        }
        actions.push(UiAction {
            command: prompt.command,
            args: CommandArgs::path(PathBuf::from(value)),
        });
    }
}

fn default_prompt_path(kind: ArgKind, state: &AppState, localizer: &Localizer) -> String {
    let directory = match kind {
        ArgKind::Directory => state.project().map(|project| project.root().to_path_buf()),
        ArgKind::NewFile | ArgKind::File => state
            .active_document()
            .and_then(|document| document.path())
            .and_then(std::path::Path::parent)
            .map(std::path::Path::to_path_buf)
            .or_else(|| state.project().map(|project| project.root().to_path_buf())),
        _ => None,
    };
    let path = match kind {
        ArgKind::NewFile => directory
            .map(|directory| directory.join(localizer.text(keys::PROMPT_DEFAULT_NEW_FILE_NAME))),
        _ => directory,
    };
    path.map(|path| display_path(&path)).unwrap_or_default()
}
