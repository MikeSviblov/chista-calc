use crate::document::Document;

pub struct NotepadApp {
    text: String,
    doc: Document,
    builtins: Vec<String>,
    font_size: f32,
    always_on_top: bool,
    first_frame: bool,
    status: Option<String>,
}

impl NotepadApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let demo = "цена = 1990\nштук = 12\nцена * штук\nIntToRoman(2024)\n".to_string();
        let st = crate::settings::load();
        let text = if st.text.is_empty() { demo } else { st.text };
        let doc = Document::evaluate(&text);
        let builtins = calc_core::Session::new().builtin_names();
        NotepadApp {
            text,
            doc,
            builtins,
            font_size: st.font_size,
            always_on_top: st.always_on_top,
            first_frame: true,
            status: None,
        }
    }

    fn persist(&self) {
        crate::settings::save(&crate::settings::Settings {
            font_size: self.font_size,
            always_on_top: self.always_on_top,
            text: self.text.clone(),
        });
    }
}

impl eframe::App for NotepadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.first_frame {
            self.first_frame = false;
            if self.always_on_top {
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
            }
        }

        let acts = crate::panels::toolbar(ctx, self.always_on_top, self.status.as_deref());
        if acts.font_delta != 0.0 {
            self.font_size = (self.font_size + acts.font_delta).clamp(8.0, 40.0);
            self.persist();
        }
        if acts.toggle_on_top {
            self.always_on_top = !self.always_on_top;
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                if self.always_on_top { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal }
            ));
            self.persist();
        }
        if acts.open {
            if let Some(path) = rfd::FileDialog::new().add_filter("calc", &["calc", "txt"]).pick_file() {
                match std::fs::read_to_string(&path) {
                    Ok(s) => {
                        self.text = s;
                        self.doc = Document::evaluate(&self.text);
                        self.persist();
                        self.status = Some(format!("Открыто: {}", path.display()));
                    }
                    Err(e) => { self.status = Some(format!("Ошибка открытия: {e}")); }
                }
            }
        }
        if acts.save {
            if let Some(path) = rfd::FileDialog::new().add_filter("calc", &["calc"]).save_file() {
                self.status = Some(match std::fs::write(&path, &self.text) {
                    Ok(()) => format!("Сохранено: {}", path.display()),
                    Err(e) => format!("Ошибка сохранения: {e}"),
                });
            }
        }

        let font_size = self.font_size;
        if let Some(name) = crate::panels::side_panel(ctx, &self.doc, &self.builtins) {
            self.text.push_str(&name);
            self.text.push('(');
            self.doc = Document::evaluate(&self.text);
            self.persist();
        }
        crate::panels::output_panel(ctx, &self.doc.output);
        egui::CentralPanel::default().show(ctx, |ui| {
            if crate::editor::code_with_results(ui, &mut self.text, &self.doc, font_size) {
                self.doc = Document::evaluate(&self.text);
                self.persist();
            }
        });
    }
}
