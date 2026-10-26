//! Kaffein desktop entry point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use std::sync::Arc;

use eframe::egui;
use kaffein_i18n::{keys, Localizer};
use kaffein_infra::FileLocaleSource;

fn main() -> eframe::Result {
    let localizer = Localizer::with_source(Arc::new(FileLocaleSource::discover()));
    let title = localizer.text(keys::APP_TITLE).to_owned();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title.clone())
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(move |_creation_context| Ok(Box::new(app::KaffeinApp::new(localizer)))),
    )
}
