//! Frame-level tests: the UI assembles and reports actions.

use egui::Context;
use kaffein_app::{command_id, AppState, CommandOutcome, CommandRegistry, ThemePreference};
use kaffein_domain::{Document, DocumentId};

use crate::{draw, UiAction, UiState};

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
