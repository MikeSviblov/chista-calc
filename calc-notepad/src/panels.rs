//! Верхний тулбар и окно-справочник функций (двуязычная справка, как «?» в оригинале).

pub struct ToolbarActions {
    pub open: bool,
    pub save: bool,
    pub clear: bool,
    pub toggle_help: bool,
    pub font_delta: f32,
    pub toggle_on_top: bool,
}

pub fn toolbar(ctx: &egui::Context, always_on_top: bool, status: Option<&str>) -> ToolbarActions {
    let mut a = ToolbarActions {
        open: false,
        save: false,
        clear: false,
        toggle_help: false,
        font_delta: 0.0,
        toggle_on_top: false,
    };
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("Открыть").clicked() {
                a.open = true;
            }
            if ui.button("Сохранить").clicked() {
                a.save = true;
            }
            if ui.button("Очистить").clicked() {
                a.clear = true;
            }
            if ui.button("Справка").clicked() {
                a.toggle_help = true;
            }
            ui.separator();
            if ui.button("Шрифт −").clicked() {
                a.font_delta = -1.0;
            }
            if ui.button("Шрифт +").clicked() {
                a.font_delta = 1.0;
            }
            ui.separator();
            let mut on_top = always_on_top;
            if ui.checkbox(&mut on_top, "поверх окон").changed() {
                a.toggle_on_top = true;
            }
            if let Some(s) = status {
                ui.separator();
                ui.weak(s);
            }
        });
    });
    a
}

/// Состояние окна справки: открытость, строка поиска, выбранная функция.
#[derive(Default)]
pub struct HelpState {
    pub open: bool,
    pub query: String,
    pub selected: Option<String>,
}

/// Двухпанельное окно справки. Слева — поиск и список по категориям, справа —
/// статья выбранной функции. Возвращает Some(name), если нажата «Вставить».
pub fn help_window(ctx: &egui::Context, state: &mut HelpState) -> Option<String> {
    let mut insert = None;
    let mut open = state.open;
    egui::Window::new("Справка — функции")
        .open(&mut open)
        .default_size([580.0, 440.0])
        .min_width(420.0)
        .show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                // Левая колонка: поиск + список.
                ui.vertical(|ui| {
                    ui.set_width(210.0);
                    ui.horizontal(|ui| {
                        ui.label("🔍");
                        ui.add(
                            egui::TextEdit::singleline(&mut state.query)
                                .hint_text("поиск функции")
                                .desired_width(160.0),
                        );
                    });
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(380.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            list_functions(ui, state);
                        });
                });
                ui.separator();
                // Правая колонка: статья.
                ui.vertical(|ui| {
                    insert = detail_pane(ui, state);
                });
            });
        });
    state.open = open;
    insert
}

/// Рисует сгруппированный по категориям список, фильтруя по строке поиска.
fn list_functions(ui: &mut egui::Ui, state: &mut HelpState) {
    let q = state.query.trim().to_lowercase();
    let matches = |e: &calc_core::help::HelpEntry| -> bool {
        if q.is_empty() {
            return true;
        }
        e.name.to_lowercase().contains(&q)
            || e.summary_ru.to_lowercase().contains(&q)
            || e.summary_en.to_lowercase().contains(&q)
    };
    for (key, label) in calc_core::help::CATEGORIES {
        let mut in_cat: Vec<&calc_core::help::HelpEntry> = calc_core::help::all()
            .iter()
            .filter(|e| e.category == *key && matches(e))
            .collect();
        if in_cat.is_empty() {
            continue;
        }
        in_cat.sort_by(|a, b| a.name.cmp(b.name));
        ui.add_space(4.0);
        ui.strong(*label);
        for e in in_cat {
            let selected = state.selected.as_deref() == Some(e.name);
            if ui.selectable_label(selected, e.name).clicked() {
                state.selected = Some(e.name.to_string());
            }
        }
    }
}

/// Рисует правую панель — статью выбранной функции. Возвращает текст для вставки:
/// `Имя(` по «Вставить» или готовый пример по «Попробовать».
fn detail_pane(ui: &mut egui::Ui, state: &HelpState) -> Option<String> {
    let mut insert = None;
    let entry = state
        .selected
        .as_deref()
        .and_then(calc_core::help::lookup);
    let Some(e) = entry else {
        ui.weak("Выберите функцию в списке слева.");
        return None;
    };
    ui.horizontal(|ui| {
        ui.heading(e.signature);
        if ui.button("Вставить").clicked() {
            // Незаконченный вызов — курсор внутри скобок, результат молчит.
            insert = Some(format!("{}(", e.name));
        }
    });
    ui.weak(calc_core::help::category_label(e.category));
    ui.add_space(6.0);
    egui::Grid::new("help_detail").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
        ui.strong("RU");
        ui.label(e.summary_ru);
        ui.end_row();
        ui.strong("EN");
        ui.label(e.summary_en);
        ui.end_row();
        ui.strong("Пример");
        ui.horizontal(|ui| {
            ui.monospace(e.example);
            // «Попробовать» — вставляет готовый пример в блокнот и сразу считает.
            if ui.button("Попробовать").clicked() {
                insert = Some(e.example.to_string());
            }
        });
        ui.end_row();
    });
    if !e.note_ru.is_empty() || !e.note_en.is_empty() {
        ui.add_space(6.0);
        if !e.note_ru.is_empty() {
            ui.weak(format!("⚠ {}", e.note_ru));
        }
        if !e.note_en.is_empty() {
            ui.weak(format!("⚠ {}", e.note_en));
        }
    }
    insert
}
