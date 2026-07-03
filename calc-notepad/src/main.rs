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
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([760.0, 620.0])
        .with_title("Чиста-блокнот");
    // Иконка окна (заголовок/панель задач). В .exe иконка ещё и вшивается ресурсом
    // через build.rs. Если PNG не декодировался — просто без иконки.
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png")) {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Чиста-блокнот",
        options,
        Box::new(|cc| Ok(Box::new(app::NotepadApp::new(cc)))),
    )
}
