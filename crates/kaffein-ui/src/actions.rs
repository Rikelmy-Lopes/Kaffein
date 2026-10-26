//! Actions emitted by the UI and dispatched by the binary.

use egui::{Context, Key, KeyboardShortcut, Modifiers};
use kaffein_app::{AppState, Command, CommandArgs, CommandRegistry};
use kaffein_i18n::Localizer;

use crate::ui_state::UiState;

/// A command invocation produced by the UI and dispatched by the binary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiAction {
    pub command: &'static str,
    pub args: CommandArgs,
}

pub(crate) fn handle_shortcuts(
    ctx: &Context,
    state: &AppState,
    registry: &CommandRegistry,
    localizer: &Localizer,
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
            trigger(command, state, localizer, ui_state, actions);
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

/// Turns a registry entry into either a pending action or an open path prompt.
pub(crate) fn trigger(
    command: Command,
    state: &AppState,
    localizer: &Localizer,
    ui_state: &mut UiState,
    actions: &mut Vec<UiAction>,
) {
    if command.args.needs_path_prompt() {
        ui_state.open_prompt(command, state, localizer);
    } else {
        actions.push(UiAction {
            command: command.id,
            args: CommandArgs::None,
        });
    }
}
