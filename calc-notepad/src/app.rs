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
        egui::CentralPanel::default().show(ctx, |ui| {
            let resp = ui.add(
                egui::TextEdit::multiline(&mut self.text)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .code_editor(),
            );
            if resp.changed() {
                self.doc = Document::evaluate(&self.text);
            }
        });
    }
}
