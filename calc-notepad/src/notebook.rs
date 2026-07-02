//! Логика единого поля-блокнота, независимая от egui (тестируемая):
//! строки-выражения, их результаты, пересчёт и обработка ENTER.

use crate::document::Document;

pub struct Notebook {
    pub entries: Vec<String>,
    /// Для каждой строки: (текст результата, это ошибка). None — нет результата.
    pub results: Vec<Option<(String, bool)>>,
    /// Строка, которую нужно сфокусировать на следующем кадре.
    pub focus: Option<usize>,
}

fn split_entries(text: &str) -> Vec<String> {
    let v: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
    if v.is_empty() {
        vec![String::new()]
    } else {
        v
    }
}

impl Notebook {
    pub fn from_text(text: &str) -> Self {
        let mut nb = Notebook {
            entries: split_entries(text),
            results: Vec::new(),
            focus: None,
        };
        nb.recompute();
        nb
    }

    /// Весь документ как текст (строки через перевод строки).
    pub fn text(&self) -> String {
        self.entries.join("\n")
    }

    /// Пересчитать результаты всех строк с чистого состояния.
    pub fn recompute(&mut self) {
        let doc = Document::evaluate(&self.entries.join("\n"));
        self.results = (0..self.entries.len())
            .map(|i| doc.result_for_line(i + 1).map(|t| (t, doc.is_error_line(i + 1))))
            .collect();
    }

    /// ENTER в строке i: пересчитать документ; если строка последняя и непустая —
    /// добавить новую пустую строку и сфокусировать её; иначе перейти на следующую.
    pub fn handle_enter(&mut self, i: usize) {
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

    /// Заменить весь текст (открытие файла) и пересчитать.
    pub fn set_text(&mut self, text: &str) {
        self.entries = split_entries(text);
        self.recompute();
    }

    /// Очистить до одной пустой строки.
    pub fn clear(&mut self) {
        self.entries = vec![String::new()];
        self.focus = Some(0);
        self.recompute();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_text_computes_results_per_line() {
        let nb = Notebook::from_text("цена = 1990\nцена * 12\nIntToRoman(2024)");
        assert_eq!(nb.results[0], Some(("1990".into(), false)));
        assert_eq!(nb.results[1], Some(("23880".into(), false)));
        assert_eq!(nb.results[2], Some(("MMXXIV".into(), false)));
    }

    #[test]
    fn enter_on_last_nonempty_adds_line_and_focuses_it() {
        let mut nb = Notebook::from_text("2 + 3");
        assert_eq!(nb.entries.len(), 1);
        nb.handle_enter(0);
        assert_eq!(nb.entries.len(), 2); // добавлена пустая строка
        assert_eq!(nb.entries[1], "");
        assert_eq!(nb.focus, Some(1)); // фокус на новой строке
        assert_eq!(nb.results[0], Some(("5".into(), false))); // результат под выражением
    }

    #[test]
    fn enter_recomputes_after_editing_earlier_line() {
        let mut nb = Notebook::from_text("x = 2\nx * 10");
        assert_eq!(nb.results[1], Some(("20".into(), false)));
        nb.entries[0] = "x = 5".to_string(); // отредактировали первую строку
        nb.handle_enter(0);
        assert_eq!(nb.results[1], Some(("50".into(), false))); // зависящий результат обновился
    }

    #[test]
    fn error_line_is_flagged() {
        let nb = Notebook::from_text("1/0");
        let (text, is_err) = nb.results[0].clone().unwrap();
        assert!(is_err);
        assert!(text.contains("ноль"));
    }

    #[test]
    fn enter_on_empty_last_line_does_not_grow() {
        let mut nb = Notebook::from_text("2+2\n");
        // вторая строка пустая; ENTER на ней не должен плодить строки
        let n = nb.entries.len();
        nb.handle_enter(n - 1);
        assert_eq!(nb.entries.len(), n);
    }

    #[test]
    fn clear_resets_to_single_empty_line() {
        let mut nb = Notebook::from_text("a\nb\nc");
        nb.clear();
        assert_eq!(nb.entries, vec![String::new()]);
        assert_eq!(nb.focus, Some(0));
    }
}
