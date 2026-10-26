//! Kaffein desktop entry point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Kaffein")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Kaffein",
        options,
        Box::new(|_creation_context| Ok(Box::new(app::KaffeinApp::new()))),
    )
}
