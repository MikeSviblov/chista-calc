#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod complete;
mod document;
mod highlight;
mod i18n;
mod panels;
mod settings;
mod sheet;
mod transcript;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([760.0, 620.0])
            .with_title("Чиста-блокнот"),
        ..Default::default()
    };
    eframe::run_native(
        "Чиста-блокнот",
        options,
        Box::new(|cc| Ok(Box::new(app::NotepadApp::new(cc)))),
    )
}
