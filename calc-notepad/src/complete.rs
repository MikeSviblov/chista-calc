//! Автоподстановка имён функций по Tab. Чистая логика без egui:
//! извлечение префикса-идентификатора под курсором и подбор функций из справки.

use calc_core::help::HelpEntry;

/// Состояние всплывающего списка автоподстановки.
#[derive(Default)]
pub struct CompleteState {
    /// Показан ли список.
    pub open: bool,
    /// Индекс выбранного кандидата в текущем списке совпадений.
    pub selected: usize,
}

/// Символ — часть идентификатора функции (имена встроенных функций ASCII).
fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Префикс-идентификатор, заканчивающийся на позиции курсора (индекс в символах).
/// Возвращает `(индекс начала, префикс)`. `None`, если префикса нет или он не
/// начинается с буквы (числа и кириллицу-переменные не дополняем).
pub fn current_prefix(text: &str, cursor: usize) -> Option<(usize, String)> {
    let chars: Vec<char> = text.chars().collect();
    let cur = cursor.min(chars.len());
    let mut start = cur;
    while start > 0 && is_ident(chars[start - 1]) {
        start -= 1;
    }
    if start == cur {
        return None;
    }
    let prefix: String = chars[start..cur].iter().collect();
    if !prefix.chars().next().unwrap().is_ascii_alphabetic() {
        return None;
    }
    Some((start, prefix))
}

/// Функции, чьи имена начинаются с префикса (регистронезависимо), по алфавиту.
pub fn matches(prefix: &str) -> Vec<&'static HelpEntry> {
    let p = prefix.to_lowercase();
    let mut v: Vec<&'static HelpEntry> = calc_core::help::all()
        .iter()
        .filter(|e| e.name.to_lowercase().starts_with(&p))
        .collect();
    v.sort_by(|a, b| a.name.cmp(b.name));
    v
}

/// Заменяет префикс `[start..cursor]` на `name(`. Возвращает `(новый текст, новый курсор)`.
pub fn apply(text: &str, start: usize, cursor: usize, name: &str) -> (String, usize) {
    let chars: Vec<char> = text.chars().collect();
    let cur = cursor.min(chars.len());
    let start = start.min(cur);
    let mut out: String = chars[..start].iter().collect();
    out.push_str(name);
    out.push('(');
    let new_cursor = start + name.chars().count() + 1;
    let tail: String = chars[cur..].iter().collect();
    out.push_str(&tail);
    (out, new_cursor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_at_end() {
        assert_eq!(current_prefix("Sq", 2), Some((0, "Sq".to_string())));
    }

    #[test]
    fn prefix_inside_expression() {
        // "цена = Sq" — курсор после Sq (индекс 9 в символах)
        let t = "цена = Sq";
        let cur = t.chars().count();
        assert_eq!(current_prefix(t, cur), Some((7, "Sq".to_string())));
    }

    #[test]
    fn no_prefix_after_space() {
        assert_eq!(current_prefix("a = ", 4), None);
    }

    #[test]
    fn numbers_are_not_a_prefix() {
        assert_eq!(current_prefix("123", 3), None);
    }

    #[test]
    fn cyrillic_identifier_is_not_completed() {
        // кириллическое имя переменной не дополняем (все встроенные — ASCII)
        let t = "цена";
        assert_eq!(current_prefix(t, t.chars().count()), None);
    }

    #[test]
    fn matches_case_insensitive_prefix() {
        let names: Vec<&str> = matches("intto").iter().map(|e| e.name).collect();
        assert_eq!(
            names,
            vec!["IntToBase", "IntToBin", "IntToHex", "IntToOct", "IntToRoman"]
        );
    }

    #[test]
    fn matches_sq() {
        let names: Vec<&str> = matches("Sq").iter().map(|e| e.name).collect();
        assert_eq!(names, vec!["Sqr", "Sqrt"]);
    }

    #[test]
    fn apply_replaces_prefix_with_call() {
        let (text, cursor) = apply("x = Sq", 4, 6, "Sqrt");
        assert_eq!(text, "x = Sqrt(");
        assert_eq!(cursor, 9);
    }

    #[test]
    fn apply_keeps_tail_after_cursor() {
        // курсор между "Sq" и " + 1"
        let (text, cursor) = apply("Sq + 1", 0, 2, "Sqrt");
        assert_eq!(text, "Sqrt( + 1");
        assert_eq!(cursor, 5);
    }
}
