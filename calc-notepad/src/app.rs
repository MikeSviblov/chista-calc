use crate::sheet::Sheet;

pub struct NotepadApp {
    sheet: Sheet,
    font_size: f32,
    always_on_top: bool,
    first_frame: bool,
    status: Option<String>,
    help: crate::panels::HelpState,
    complete: crate::complete::CompleteState,
}

impl NotepadApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let st = crate::settings::load();
        let seed = "цена = 1990\nштук = 12\nцена * штук\nIntToRoman(2024)".to_string();
        let text = if st.text.is_empty() { seed } else { st.text };
        NotepadApp {
            sheet: Sheet::from_input(&text),
            font_size: st.font_size,
            always_on_top: st.always_on_top,
            first_frame: true,
            status: None,
            help: crate::panels::HelpState::default(),
            complete: crate::complete::CompleteState::default(),
        }
    }

    fn persist(&self) {
        crate::settings::save(&crate::settings::Settings {
            font_size: self.font_size,
            always_on_top: self.always_on_top,
            text: self.sheet.input(), // храним только ввод (результаты пересчитаются)
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
                if self.always_on_top { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal },
            ));
            self.persist();
        }
        if acts.toggle_help {
            self.help.open = !self.help.open;
        }
        if acts.clear {
            self.sheet = Sheet::from_input("");
            self.persist();
        }
        if acts.open {
            if let Some(path) = rfd::FileDialog::new().add_filter("calc", &["calc", "txt"]).pick_file() {
                match std::fs::read_to_string(&path) {
                    Ok(s) => {
                        self.sheet = Sheet::from_input(&s);
                        self.persist();
                        self.status = Some(format!("Открыто: {}", path.display()));
                    }
                    Err(e) => self.status = Some(format!("Ошибка открытия: {e}")),
                }
            }
        }
        if acts.save {
            if let Some(path) = rfd::FileDialog::new().add_filter("calc", &["calc"]).save_file() {
                self.status = Some(match std::fs::write(&path, self.sheet.input()) {
                    Ok(()) => format!("Сохранено: {}", path.display()),
                    Err(e) => format!("Ошибка сохранения: {e}"),
                });
            }
        }

        // Окно справки добавляет строку ввода: «Вставить» — `Имя(`, «Попробовать» —
        // готовый пример (Sheet::from_input сразу его вычислит).
        if let Some(text) = crate::panels::help_window(ctx, &mut self.help) {
            let mut inp = self.sheet.input();
            if !inp.is_empty() {
                inp.push('\n');
            }
            inp.push_str(&text);
            self.sheet = Sheet::from_input(&inp);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if crate::transcript::show(ui, &mut self.sheet, self.font_size, &mut self.complete) {
                self.persist();
            }
        });
    }
}
