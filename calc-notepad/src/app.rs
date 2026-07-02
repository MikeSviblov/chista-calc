use crate::document::Document;

pub struct NotepadApp {
    text: String,
    doc: Document,
    builtins: Vec<String>,
}

impl NotepadApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let text = "цена = 1990\nштук = 12\nцена * штук\nIntToRoman(2024)\n".to_string();
        let doc = Document::evaluate(&text);
        let builtins = calc_core::Session::new().builtin_names();
        NotepadApp { text, doc, builtins }
    }
}

impl eframe::App for NotepadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let font_size = 14.0;
        if let Some(name) = crate::panels::side_panel(ctx, &self.doc, &self.builtins) {
            self.text.push_str(&name);
            self.text.push('(');
            self.doc = Document::evaluate(&self.text);
        }
        crate::panels::output_panel(ctx, &self.doc.output);
        egui::CentralPanel::default().show(ctx, |ui| {
            if crate::editor::code_with_results(ui, &mut self.text, &self.doc, font_size) {
                self.doc = Document::evaluate(&self.text);
            }
        });
    }
}
