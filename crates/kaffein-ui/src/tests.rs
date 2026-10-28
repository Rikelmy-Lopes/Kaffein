//! Frame-level tests: the UI assembles and reports actions.

use egui::Context;
use kaffein_app::{command_id, AppState, CommandOutcome, CommandRegistry, ThemePreference};
use kaffein_domain::{Document, DocumentId};
use kaffein_i18n::{keys, Localizer, Text};

use crate::{draw, UiAction, UiState};

fn draw_frame(state: &AppState, ui_state: &mut UiState) -> Vec<UiAction> {
    draw_frame_with(state, ui_state, &Localizer::bundled())
}

fn draw_frame_with(
    state: &AppState,
    ui_state: &mut UiState,
    localizer: &Localizer,
) -> Vec<UiAction> {
    let ctx = Context::default();
    let registry = CommandRegistry::new();
    let mut actions = Vec::new();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        actions = draw(ctx, state, &registry, localizer, ui_state);
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
fn an_empty_editor_shows_default_java_code() {
    let state = AppState::new(ThemePreference::Dark);
    let mut ui_state = UiState::default();

    let actions = draw_frame(&state, &mut ui_state);

    assert!(actions.is_empty());
    assert_eq!(ui_state.syntax.line_count_for_test(), 5);
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
fn switching_between_clean_documents_updates_the_editor_text() {
    let mut state = AppState::new(ThemePreference::Dark);
    state.add_document(Document::from_loaded(
        DocumentId(1),
        "C:/project/A.java",
        "class A {}\n",
    ));
    state.add_document(Document::from_loaded(
        DocumentId(2),
        "C:/project/B.java",
        "class B {}\n",
    ));
    let mut ui_state = UiState::default();

    draw_frame(&state, &mut ui_state);
    assert_eq!(ui_state.editor.text, "class B {}\n");

    state.activate(DocumentId(1));
    draw_frame(&state, &mut ui_state);
    assert_eq!(ui_state.editor.text, "class A {}\n");
}

#[test]
fn a_java_document_with_all_token_families_renders() {
    let mut state = AppState::new(ThemePreference::Dark);
    state.add_document(Document::from_loaded(
        DocumentId(1),
        "C:/project/App.java",
        "/** doc */\npublic class App {\n    // note\n    String s = \"hi\";\n    int n = 0x1f;\n    boolean b = n >>> 2 > 0;\n}\n",
    ));
    let mut ui_state = UiState::default();

    let actions = draw_frame(&state, &mut ui_state);

    assert!(actions.is_empty());
}

#[test]
fn the_frame_renders_in_a_non_english_locale() {
    let state = AppState::new(ThemePreference::Dark);
    let mut ui_state = UiState::default();
    let mut localizer = Localizer::bundled();
    localizer.set_locale("pt-BR").expect("bundled locale");

    let actions = draw_frame_with(&state, &mut ui_state, &localizer);

    assert!(actions.is_empty());
    assert_eq!(localizer.text(keys::MENU_FILE), "Arquivo");
}

#[test]
fn a_prompt_closes_after_a_successful_dispatch() {
    let state = AppState::new(ThemePreference::Dark);
    let mut ui_state = UiState::default();
    let registry = CommandRegistry::new();
    let command = registry.get(command_id::FILE_OPEN).expect("command");
    let localizer = Localizer::bundled();
    ui_state.open_prompt(command, &state, &localizer);

    ui_state.after_dispatch(
        command_id::FILE_OPEN,
        &CommandOutcome::success(Text::literal("Opened App.java")),
    );

    assert!(!ui_state.prompt_is_open());
}

#[test]
fn a_prompt_stays_open_and_shows_a_failure() {
    let state = AppState::new(ThemePreference::Dark);
    let mut ui_state = UiState::default();
    let registry = CommandRegistry::new();
    let command = registry.get(command_id::FILE_OPEN).expect("command");
    let localizer = Localizer::bundled();
    ui_state.open_prompt(command, &state, &localizer);

    ui_state.after_dispatch(
        command_id::FILE_OPEN,
        &CommandOutcome::failed(
            kaffein_app::ErrorKind::UserInput,
            Text::literal("missing file"),
        ),
    );

    assert!(ui_state.prompt_is_open());
    assert_eq!(
        ui_state
            .prompt
            .as_ref()
            .and_then(|prompt| prompt.error.as_ref())
            .map(|error| error.resolve(&localizer)),
        Some("missing file".to_owned())
    );
}
