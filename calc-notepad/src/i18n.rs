//! Строки интерфейса блокнота на двух языках. Двуязычная справка функций
//! (RU+EN одновременно) — отдельная фича; здесь только «хром» приложения.

use calc_core::Lang;

/// Набор строк интерфейса для выбранного языка.
pub struct Ui {
    pub open: &'static str,
    pub save: &'static str,
    pub clear: &'static str,
    pub help: &'static str,
    pub font_dec: &'static str,
    pub font_inc: &'static str,
    pub on_top: &'static str,
    // окно справки
    pub help_title: &'static str,
    pub search_hint: &'static str,
    pub choose_fn: &'static str,
    pub insert: &'static str,
    pub try_it: &'static str,
    pub example: &'static str,
    // статусы (шаблоны с одним {})
    pub opened: &'static str,
    pub saved: &'static str,
    pub open_err: &'static str,
    pub save_err: &'static str,
}

/// Строки интерфейса для языка.
pub fn ui(lang: Lang) -> Ui {
    match lang {
        Lang::Ru => Ui {
            open: "Открыть",
            save: "Сохранить",
            clear: "Очистить",
            help: "Справка",
            font_dec: "Шрифт −",
            font_inc: "Шрифт +",
            on_top: "поверх окон",
            help_title: "Справка — функции",
            search_hint: "поиск функции",
            choose_fn: "Выберите функцию в списке слева.",
            insert: "Вставить",
            try_it: "Попробовать",
            example: "Пример",
            opened: "Открыто: {}",
            saved: "Сохранено: {}",
            open_err: "Ошибка открытия: {}",
            save_err: "Ошибка сохранения: {}",
        },
        Lang::En => Ui {
            open: "Open",
            save: "Save",
            clear: "Clear",
            help: "Help",
            font_dec: "Font −",
            font_inc: "Font +",
            on_top: "always on top",
            help_title: "Help — functions",
            search_hint: "search function",
            choose_fn: "Select a function in the list on the left.",
            insert: "Insert",
            try_it: "Try",
            example: "Example",
            opened: "Opened: {}",
            saved: "Saved: {}",
            open_err: "Open error: {}",
            save_err: "Save error: {}",
        },
    }
}

/// Подставляет `arg` вместо первого `{}` в шаблоне.
pub fn fill(template: &str, arg: &str) -> String {
    template.replacen("{}", arg, 1)
}
