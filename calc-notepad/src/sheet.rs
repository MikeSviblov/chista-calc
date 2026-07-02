//! Единое текстовое поле-«лист»: один буфер, где строки-ввод чередуются со
//! строками-результатами (с ведущим маркером-табом). Редактируется как обычный
//! многострочный редактор egui (курсор, стрелки, выделение — нативно), результаты
//! пересчитываются по ENTER и подставляются в тот же текст.

use crate::document::Document;
use std::collections::HashSet;

/// Маркер строки-результата (ведущий символ). Даёт и отступ, и признак «это результат».
pub const MARK: char = '\t';

pub struct Sheet {
    /// Полный буфер: строки-ввод + строки-результаты (каждая начинается с MARK).
    pub text: String,
    /// Индексы строк (в text) — результатов-ошибок (для красной подсветки).
    pub error_lines: HashSet<usize>,
}

fn is_result_line(line: &str) -> bool {
    line.starts_with(MARK)
}

impl Sheet {
    pub fn from_input(input: &str) -> Self {
        let mut s = Sheet { text: String::new(), error_lines: HashSet::new() };
        s.rebuild_from(input);
        s
    }

    /// Только строки-ввод (без строк-результатов) — то, что редактирует пользователь.
    pub fn input(&self) -> String {
        self.text
            .split('\n')
            .filter(|l| !is_result_line(l))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn rebuild_from(&mut self, input: &str) {
        let doc = Document::evaluate(input);
        let mut out = String::new();
        let mut errors = HashSet::new();
        for (i, line) in input.split('\n').enumerate() {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(line);
            if let Some(res) = doc.result_for_line(i + 1) {
                out.push('\n');
                let line_idx = out.matches('\n').count(); // индекс добавляемой строки
                out.push(MARK);
                out.push_str(&res);
                if doc.is_error_line(i + 1) {
                    errors.insert(line_idx);
                }
            }
        }
        self.text = out;
        self.error_lines = errors;
    }

    /// Пересчитать результаты по текущему вводу (курсор не трогаем).
    pub fn recompute(&mut self) {
        let input = self.input();
        self.rebuild_from(&input);
    }

    /// Пересчитать после ENTER: курсор был на символьной позиции `cursor` в
    /// текущем `text`. Возвращает новую позицию курсора (начало той же строки-ввода).
    pub fn recompute_with_cursor(&mut self, cursor: usize) -> usize {
        let input_idx = self.input_line_index_at(cursor);
        self.recompute();
        self.input_line_start(input_idx)
    }

    /// Индекс строки-ВВОДА (среди строк-ввода), на которой стоит курсор.
    fn input_line_index_at(&self, cursor: usize) -> usize {
        let mut chars = 0usize;
        let mut input_idx = 0usize;
        for line in self.text.split('\n') {
            let len = line.chars().count();
            if cursor <= chars + len {
                return input_idx;
            }
            if !is_result_line(line) {
                input_idx += 1;
            }
            chars += len + 1;
        }
        input_idx
    }

    /// Символьная позиция начала k-й строки-ввода в `text`.
    fn input_line_start(&self, k: usize) -> usize {
        let mut chars = 0usize;
        let mut input_idx = 0usize;
        for line in self.text.split('\n') {
            if !is_result_line(line) {
                if input_idx == k {
                    return chars;
                }
                input_idx += 1;
            }
            chars += line.chars().count() + 1;
        }
        self.text.chars().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebuild_interleaves_only_for_expressions() {
        // присваивание молчит, выражение даёт строку-результат с маркером
        let s = Sheet::from_input("x = 2\nx * 3");
        let lines: Vec<&str> = s.text.split('\n').collect();
        assert_eq!(lines[0], "x = 2");
        assert_eq!(lines[1], "x * 3");
        assert_eq!(lines[2], "\t6"); // результат под выражением
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn input_strips_result_lines() {
        let s = Sheet::from_input("a = 1\na + 4");
        // text содержит строку-результат, а input() — нет
        assert!(s.text.contains("\t5"));
        assert_eq!(s.input(), "a = 1\na + 4");
    }

    #[test]
    fn error_line_is_marked_red() {
        let s = Sheet::from_input("1/0");
        // строка 0 — ввод, строка 1 — результат-ошибка
        assert!(s.error_lines.contains(&1));
    }

    #[test]
    fn recompute_after_edit_updates_results() {
        let mut s = Sheet::from_input("x = 2\nx * 10");
        assert!(s.text.contains("\t20"));
        // пользователь отредактировал ввод (эмулируем: подменяем input и пересчёт)
        s.text = "x = 5\nx * 10".to_string();
        s.recompute();
        assert!(s.text.contains("\t50"));
        assert!(!s.text.contains("\t20"));
    }

    #[test]
    fn cursor_maps_to_start_of_same_input_line() {
        // ввод: "2+2"(0) \t4(1) "5+5"(2). Курсор на строке-ввода "5+5".
        let mut s = Sheet::from_input("2+2\n5+5");
        // позиция начала строки "5+5" в text: "2+2\n\t4\n5+5" → индекс 8
        let pos = s.text.find("5+5").unwrap();
        let pos_chars = s.text[..pos].chars().count();
        let new_cursor = s.recompute_with_cursor(pos_chars + 1); // курсор внутри "5+5"
        // после пересчёта та же строка-ввода начинается там же
        let np = s.text.find("5+5").unwrap();
        assert_eq!(new_cursor, s.text[..np].chars().count());
    }
}
