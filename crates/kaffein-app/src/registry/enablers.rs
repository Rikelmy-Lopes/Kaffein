//! Enablement predicates used by the command catalog.

use kaffein_i18n::keys;

use crate::command::Enablement;
use crate::state::{AppState, ThemePreference};

pub(super) fn always(_: &AppState) -> Enablement {
    Enablement::Enabled
}

pub(super) fn has_document(state: &AppState) -> Enablement {
    if state.active_document().is_some() {
        Enablement::Enabled
    } else {
        Enablement::Disabled(keys::ENABLEMENT_NO_DOCUMENT)
    }
}

pub(super) fn can_undo(state: &AppState) -> Enablement {
    match state.active_document() {
        Some(document) if document.can_undo() => Enablement::Enabled,
        Some(_) => Enablement::Disabled(keys::ENABLEMENT_NOTHING_TO_UNDO),
        None => Enablement::Disabled(keys::ENABLEMENT_NO_DOCUMENT),
    }
}

pub(super) fn can_redo(state: &AppState) -> Enablement {
    match state.active_document() {
        Some(document) if document.can_redo() => Enablement::Enabled,
        Some(_) => Enablement::Disabled(keys::ENABLEMENT_NOTHING_TO_REDO),
        None => Enablement::Disabled(keys::ENABLEMENT_NO_DOCUMENT),
    }
}

pub(super) fn can_run(state: &AppState) -> Enablement {
    if state.run_state().is_running() {
        return Enablement::Disabled(keys::ENABLEMENT_RUN_IN_PROGRESS);
    }
    match state.active_document() {
        Some(document) if crate::commands::is_java_document(document) => Enablement::Enabled,
        Some(_) => Enablement::Disabled(keys::ENABLEMENT_NOT_JAVA),
        None => Enablement::Disabled(keys::ENABLEMENT_NO_DOCUMENT),
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
