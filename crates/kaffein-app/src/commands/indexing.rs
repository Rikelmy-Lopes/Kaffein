//! Keeping the symbol index in step with the project and open documents.

use std::path::PathBuf;

use kaffein_domain::ProjectEntry;

use crate::services::Services;
use crate::state::AppState;

/// Rebuilds the index from every Java file in the open project, then overlays
/// open buffers so unsaved declarations are not lost.
pub(crate) fn index_project(state: &mut AppState, services: &Services) {
    state.index_mut().clear();
    let mut files = Vec::new();
    if let Some(project) = state.project() {
        collect_files(project.children(), &mut files);
    }
    for path in files {
        if let Ok(text) = services.fs.read_text(&path) {
            state.index_mut().upsert(path, &text);
        }
    }
    overlay_open_documents(state);
}

pub(crate) fn index_source(state: &mut AppState, path: &std::path::Path, source: &str) {
    state.index_mut().upsert(path, source);
}

fn overlay_open_documents(state: &mut AppState) {
    let open: Vec<(PathBuf, String)> = state
        .documents()
        .iter()
        .filter_map(|document| {
            document
                .path()
                .map(|path| (path.to_path_buf(), document.buffer().to_owned()))
        })
        .collect();
    for (path, text) in open {
        state.index_mut().upsert(path, &text);
    }
}

fn collect_files(entries: &[ProjectEntry], files: &mut Vec<PathBuf>) {
    for entry in entries {
        if entry.is_directory() {
            collect_files(entry.children(), files);
        } else if is_java(entry.path()) {
            files.push(entry.path().to_path_buf());
        }
    }
}

fn is_java(path: &std::path::Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.to_string_lossy().eq_ignore_ascii_case("java"))
}
