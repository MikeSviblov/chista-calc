mod app;
#[allow(dead_code)]
mod document;
mod editor;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_title("Чиста-блокнот"),
        ..Default::default()
    };
    eframe::run_native(
        "Чиста-блокнот",
        options,
        Box::new(|cc| Ok(Box::new(app::NotepadApp::new(cc)))),
    )
}
