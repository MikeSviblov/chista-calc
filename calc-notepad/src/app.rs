use crate::document::Document;

pub struct NotepadApp {
    text: String,
    doc: Document,
}

impl NotepadApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let text = "цена = 1990\nштук = 12\nцена * штук\nIntToRoman(2024)\n".to_string();
        let doc = Document::evaluate(&text);
        NotepadApp { text, doc }
    }
}

impl eframe::App for NotepadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let font_size = 14.0;
        egui::CentralPanel::default().show(ctx, |ui| {
            if crate::editor::code_with_results(ui, &mut self.text, &self.doc, font_size) {
                self.doc = Document::evaluate(&self.text);
            }
        });
    }
}
