use crate::document::Document;

pub struct NotepadApp {
    entries: Vec<String>,
    results: Vec<Option<(String, bool)>>,
    builtins: Vec<String>,
    font_size: f32,
    always_on_top: bool,
    first_frame: bool,
    status: Option<String>,
    focus: Option<usize>,
    show_help: bool,
}

impl NotepadApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let st = crate::settings::load();
        let seed = "цена = 1990\nштук = 12\nцена * штук\nIntToRoman(2024)".to_string();
        let text = if st.text.is_empty() { seed } else { st.text };
        let entries = split_entries(&text);
        let mut app = NotepadApp {
            entries,
            results: Vec::new(),
            builtins: calc_core::Session::new().builtin_names(),
            font_size: st.font_size,
            always_on_top: st.always_on_top,
            first_frame: true,
            status: None,
            focus: None,
            show_help: false,
        };
        app.recompute();
        app
    }

    fn recompute(&mut self) {
        let doc = Document::evaluate(&self.entries.join("\n"));
        self.results = (0..self.entries.len())
            .map(|i| doc.result_for_line(i + 1).map(|t| (t, doc.is_error_line(i + 1))))
            .collect();
        self.persist();
    }

    fn persist(&self) {
        crate::settings::save(&crate::settings::Settings {
            font_size: self.font_size,
            always_on_top: self.always_on_top,
            text: self.entries.join("\n"),
        });
    }
}

fn split_entries(text: &str) -> Vec<String> {
    let v: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
    if v.is_empty() { vec![String::new()] } else { v }
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
                if self.always_on_top { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal },
            ));
            self.persist();
        }
        if acts.toggle_help {
            self.show_help = !self.show_help;
        }
        if acts.clear {
            self.entries = vec![String::new()];
            self.focus = Some(0);
            self.recompute();
        }
        if acts.open {
            if let Some(path) = rfd::FileDialog::new().add_filter("calc", &["calc", "txt"]).pick_file() {
                match std::fs::read_to_string(&path) {
                    Ok(s) => {
                        self.entries = split_entries(&s);
                        self.recompute();
                        self.status = Some(format!("Открыто: {}", path.display()));
                    }
                    Err(e) => self.status = Some(format!("Ошибка открытия: {e}")),
                }
            }
        }
        if acts.save {
            if let Some(path) = rfd::FileDialog::new().add_filter("calc", &["calc"]).save_file() {
                self.status = Some(match std::fs::write(&path, self.entries.join("\n")) {
                    Ok(()) => format!("Сохранено: {}", path.display()),
                    Err(e) => format!("Ошибка сохранения: {e}"),
                });
            }
        }

        // Окно-справочник функций; клик вставляет имя в последнюю строку.
        if let Some(name) = crate::panels::help_window(ctx, &mut self.show_help, &self.builtins) {
            if let Some(last) = self.entries.last_mut() {
                last.push_str(&name);
                last.push('(');
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let action = crate::transcript::show(
                ui,
                &mut self.entries,
                &self.results,
                self.font_size,
                &mut self.focus,
            );
            if action.changed {
                self.persist();
            }
            if let Some(i) = action.entered {
                // Пересчитать весь документ, при необходимости добавить пустую строку.
                let at_last = i + 1 == self.entries.len();
                let non_empty = self.entries.get(i).map(|s| !s.trim().is_empty()).unwrap_or(false);
                if at_last && non_empty {
                    self.entries.push(String::new());
                    self.focus = Some(self.entries.len() - 1);
                } else if i + 1 < self.entries.len() {
                    self.focus = Some(i + 1);
                }
                self.recompute();
            }
        });
    }
}
