//! Верхний тулбар и окно-справочник функций (двуязычная справка, как «?» в оригинале).

use crate::i18n;
use calc_core::Lang;

pub struct ToolbarActions {
    pub open: bool,
    pub save: bool,
    pub clear: bool,
    pub toggle_help: bool,
    pub font_delta: f32,
    pub toggle_on_top: bool,
    /// Нажат переключатель языка (RU⇄EN).
    pub toggle_lang: bool,
}

pub fn toolbar(
    ctx: &egui::Context,
    always_on_top: bool,
    status: Option<&str>,
    lang: Lang,
) -> ToolbarActions {
    let t = i18n::ui(lang);
    let mut a = ToolbarActions {
        open: false,
        save: false,
        clear: false,
        toggle_help: false,
        font_delta: 0.0,
        toggle_on_top: false,
        toggle_lang: false,
    };
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button(t.open).clicked() {
                a.open = true;
            }
            if ui.button(t.save).clicked() {
                a.save = true;
            }
            if ui.button(t.clear).clicked() {
                a.clear = true;
            }
            if ui.button(t.help).clicked() {
                a.toggle_help = true;
            }
            ui.separator();
            if ui.button(t.font_dec).clicked() {
                a.font_delta = -1.0;
            }
            if ui.button(t.font_inc).clicked() {
                a.font_delta = 1.0;
            }
            ui.separator();
            let mut on_top = always_on_top;
            if ui.checkbox(&mut on_top, t.on_top).changed() {
                a.toggle_on_top = true;
            }
            ui.separator();
            // Переключатель языка: кнопка показывает текущий язык, клик — переключить.
            let code = match lang {
                Lang::Ru => "RU",
                Lang::En => "EN",
            };
            if ui
                .button(code)
                .on_hover_text("RU ⇄ EN")
                .clicked()
            {
                a.toggle_lang = true;
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
/// статья выбранной функции (двуязычная: RU+EN одновременно). Возвращает текст для
/// вставки новой строкой ввода: «Вставить» даёт `Имя(`, «Попробовать» — готовый пример.
pub fn help_window(ctx: &egui::Context, state: &mut HelpState, lang: Lang) -> Option<String> {
    let t = i18n::ui(lang);
    let mut insert = None;
    let mut open = state.open;
    egui::Window::new(t.help_title)
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
                                .hint_text(t.search_hint)
                                .desired_width(160.0),
                        );
                    });
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(380.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            list_functions(ui, state, lang);
                        });
                });
                ui.separator();
                // Правая колонка: статья.
                ui.vertical(|ui| {
                    insert = detail_pane(ui, state, lang, &t);
                });
            });
        });
    state.open = open;
    insert
}

/// Рисует сгруппированный по категориям список, фильтруя по строке поиска.
fn list_functions(ui: &mut egui::Ui, state: &mut HelpState, lang: Lang) {
    let q = state.query.trim().to_lowercase();
    let matches = |e: &calc_core::help::HelpEntry| -> bool {
        if q.is_empty() {
            return true;
        }
        e.name.to_lowercase().contains(&q)
            || e.summary_ru.to_lowercase().contains(&q)
            || e.summary_en.to_lowercase().contains(&q)
    };
    for (key, ..) in calc_core::help::CATEGORIES {
        let mut in_cat: Vec<&calc_core::help::HelpEntry> = calc_core::help::all()
            .iter()
            .filter(|e| e.category == *key && matches(e))
            .collect();
        if in_cat.is_empty() {
            continue;
        }
        in_cat.sort_by(|a, b| a.name.cmp(b.name));
        ui.add_space(4.0);
        ui.strong(calc_core::help::category_label(key, lang));
        for e in in_cat {
            let selected = state.selected.as_deref() == Some(e.name);
            if ui.selectable_label(selected, e.name).clicked() {
                state.selected = Some(e.name.to_string());
            }
        }
    }
}

/// Рисует правую панель — статью выбранной функции (двуязычную). Возвращает текст
/// для вставки: `Имя(` по «Вставить» или готовый пример по «Попробовать».
fn detail_pane(ui: &mut egui::Ui, state: &HelpState, lang: Lang, t: &i18n::Ui) -> Option<String> {
    let mut insert = None;
    let entry = state.selected.as_deref().and_then(calc_core::help::lookup);
    let Some(e) = entry else {
        ui.weak(t.choose_fn);
        return None;
    };
    ui.horizontal(|ui| {
        ui.heading(e.signature);
        if ui.button(t.insert).clicked() {
            // Незаконченный вызов — курсор внутри скобок, результат молчит.
            insert = Some(format!("{}(", e.name));
        }
    });
    ui.weak(calc_core::help::category_label(e.category, lang));
    ui.add_space(6.0);
    egui::Grid::new("help_detail").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
        ui.strong("RU");
        ui.label(e.summary_ru);
        ui.end_row();
        ui.strong("EN");
        ui.label(e.summary_en);
        ui.end_row();
        ui.strong(t.example);
        ui.horizontal(|ui| {
            ui.monospace(e.example);
            // «Попробовать» — вставляет готовый пример в блокнот и сразу считает.
            if ui.button(t.try_it).clicked() {
                insert = Some(e.example.to_string());
            }
        });
        ui.end_row();
    });
    // Примечание об ошибках/краях — на языке интерфейса.
    let note = e.note(lang);
    if !note.is_empty() {
        ui.add_space(6.0);
        ui.weak(format!("⚠ {note}"));
    }
    insert
}
