//! Enablement predicates used by the command catalog.

use crate::command::Enablement;
use crate::state::{AppState, ThemePreference};

const NO_DOCUMENT: &str = "No document is open";

pub(super) fn always(_: &AppState) -> Enablement {
    Enablement::Enabled
}

pub(super) fn has_document(state: &AppState) -> Enablement {
    if state.active_document().is_some() {
        Enablement::Enabled
    } else {
        Enablement::Disabled(NO_DOCUMENT)
    }
}

pub(super) fn can_undo(state: &AppState) -> Enablement {
    match state.active_document() {
        Some(document) if document.can_undo() => Enablement::Enabled,
        Some(_) => Enablement::Disabled("Nothing to undo"),
        None => Enablement::Disabled(NO_DOCUMENT),
    }
}

pub(super) fn can_redo(state: &AppState) -> Enablement {
    match state.active_document() {
        Some(document) if document.can_redo() => Enablement::Enabled,
        Some(_) => Enablement::Disabled("Nothing to redo"),
        None => Enablement::Disabled(NO_DOCUMENT),
    }
}

pub(super) fn project_panel_visible(state: &AppState) -> bool {
    state.show_project_panel()
}

pub(super) fn bottom_panel_visible(state: &AppState) -> bool {
    state.show_bottom_panel()
}

pub(super) fn dark_theme(state: &AppState) -> bool {
    state.theme() == ThemePreference::Dark
}
