//! Встроенный двуязычный (RU/EN) справочник по функциям.
//!
//! Единый источник правды: используется и GUI-блокнотом (панель справки),
//! и CLI (команда `help`). Каждая запись описывает одну встроенную функцию:
//! сигнатуру, назначение на двух языках, пример вызова и примечание об
//! ошибках/крайних случаях.

/// Одна статья справки по функции.
pub struct HelpEntry {
    /// Имя функции ровно как в реестре (`Sqrt`, `IntToRoman`, ...).
    pub name: &'static str,
    /// Ключ категории (`math`, `trig`, ...); см. [`category_label`].
    pub category: &'static str,
    /// Сигнатура для показа, напр. `Log(x, base)`.
    pub signature: &'static str,
    /// Назначение (одна фраза, русский).
    pub summary_ru: &'static str,
    /// Назначение (одна фраза, английский).
    pub summary_en: &'static str,
    /// Пример вызова, вычислимый калькулятором, напр. `Sqrt(2)`.
    pub example: &'static str,
    /// Примечание об ошибках/краях (русский); пусто — если нечего сказать.
    pub note_ru: &'static str,
    /// Примечание об ошибках/краях (английский); пусто — если нечего сказать.
    pub note_en: &'static str,
}

/// Порядок категорий для показа и их русские заголовки.
pub const CATEGORIES: &[(&str, &str)] = &[
    ("math", "Математика"),
    ("trig", "Тригонометрия"),
    ("bits", "Биты"),
    ("logic", "Логика"),
    ("bases", "Системы счисления"),
    ("strings", "Строки"),
    ("hash", "Хеши"),
    ("cipher", "Шифрование"),
    ("datetime", "Дата и время"),
    ("fileio", "Файлы"),
    ("io", "Ввод-вывод"),
];

/// Русский заголовок категории по её ключу (или заглушка, если незнаком).
pub fn category_label(key: &str) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, label)| *label)
        .unwrap_or("Прочее")
}

/// Все статьи справки (в порядке объявления).
pub fn all() -> &'static [HelpEntry] {
    ENTRIES
}

/// Статья по имени функции (без учёта регистра).
pub fn lookup(name: &str) -> Option<&'static HelpEntry> {
    ENTRIES.iter().find(|e| e.name.eq_ignore_ascii_case(name))
}

include!("help_data.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;

    #[test]
    fn every_builtin_has_help() {
        let names = Session::new().builtin_names();
        let missing: Vec<String> = names
            .iter()
            .filter(|n| lookup(n).is_none())
            .cloned()
            .collect();
        assert!(missing.is_empty(), "нет справки для: {missing:?}");
    }

    #[test]
    fn no_orphan_help_entries() {
        let names = Session::new().builtin_names();
        let orphans: Vec<&str> = ENTRIES
            .iter()
            .map(|e| e.name)
            .filter(|n| !names.iter().any(|b| b == n))
            .collect();
        assert!(orphans.is_empty(), "справка для несуществующих: {orphans:?}");
    }

    #[test]
    fn every_category_is_known() {
        for e in ENTRIES {
            assert!(
                CATEGORIES.iter().any(|(k, _)| *k == e.category),
                "{}: неизвестная категория {}",
                e.name,
                e.category
            );
        }
    }

    #[test]
    fn examples_run_without_error() {
        // Примеры, читающие/пишущие файлы, пропускаем — они трогают ФС.
        let skip_fs = ["FileToStr", "StrToFile", "AppendFile"];
        for e in ENTRIES {
            if skip_fs.contains(&e.name) {
                continue;
            }
            let mut sess = Session::new();
            let r = sess.eval(e.example);
            assert!(
                r.is_ok(),
                "{}: пример `{}` не вычислился: {:?}",
                e.name,
                e.example,
                r.err()
            );
        }
    }
}
