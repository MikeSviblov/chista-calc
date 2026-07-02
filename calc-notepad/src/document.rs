//! Построчное вычисление документа (как оригинал «Чиста калькулятор»):
//! каждая строка разбирается и считается независимо, состояние переменных
//! накапливается между строками. Присваивания и определения результата не
//! показывают — только чистые выражения.

use calc_core::ast::{Expr, Stmt};
use calc_core::{lexer, parser, Evaluator};
use std::collections::{HashMap, HashSet};

pub struct Document {
    results: HashMap<usize, String>,
    errors: HashSet<usize>,
    /// Накопленный вывод print (пока не отображается отдельно).
    #[allow(dead_code)]
    pub output: String,
}

/// Показывать результат строки нужно, только если её последняя инструкция —
/// чистое выражение (не присваивание, не fn/alias, не цикл).
fn shows_result(stmts: &[Stmt]) -> bool {
    matches!(stmts.last(), Some(Stmt::Expr(e)) if !matches!(e, Expr::Assign { .. }))
}

impl Document {
    pub fn evaluate(src: &str) -> Self {
        let mut ev = Evaluator::new();
        let mut results = HashMap::new();
        let mut errors = HashSet::new();
        let mut output = String::new();

        for (i, line) in src.split('\n').enumerate() {
            let n = i + 1;
            let parsed =
                lexer::tokenize(line).and_then(|t| parser::Parser::new(t).parse_program());
            match parsed {
                Err(e) => {
                    results.insert(n, e.to_string());
                    errors.insert(n);
                }
                Ok(stmts) => {
                    let show = shows_result(&stmts);
                    match ev.run(&stmts) {
                        Ok(v) => {
                            if show {
                                results.insert(n, v.to_string());
                            }
                        }
                        Err(e) => {
                            results.insert(n, e.to_string());
                            errors.insert(n);
                        }
                    }
                    output.push_str(&ev.take_output());
                }
            }
        }

        Document { results, errors, output }
    }

    pub fn result_for_line(&self, line: usize) -> Option<String> {
        self.results.get(&line).cloned()
    }
    pub fn is_error_line(&self, line: usize) -> bool {
        self.errors.contains(&line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignments_are_silent_only_expressions_show() {
        let doc = Document::evaluate("цена = 1990\nштук = 12\nцена * штук\nIntToRoman(2024)");
        assert_eq!(doc.result_for_line(1), None); // присваивание — молчит
        assert_eq!(doc.result_for_line(2), None); // присваивание — молчит
        assert_eq!(doc.result_for_line(3).as_deref(), Some("23880")); // выражение
        assert_eq!(doc.result_for_line(4).as_deref(), Some("MMXXIV")); // выражение
    }

    #[test]
    fn state_accumulates_across_lines() {
        let doc = Document::evaluate("test1 = 1\ntest2 = 2\nresult = test1 + test2\nresult");
        assert_eq!(doc.result_for_line(1), None);
        assert_eq!(doc.result_for_line(3), None); // тоже присваивание
        assert_eq!(doc.result_for_line(4).as_deref(), Some("3")); // голое выражение
    }

    #[test]
    fn broken_line_does_not_blank_the_others() {
        // Последняя строка — незаконченное присваивание (синтаксическая ошибка),
        // но предыдущие строки должны посчитаться (как в оригинале).
        let doc = Document::evaluate("a = 10\na + 5\nb =");
        assert_eq!(doc.result_for_line(2).as_deref(), Some("15")); // не обнулилось
        assert!(doc.is_error_line(3)); // ошибка только на своей строке
        assert!(!doc.is_error_line(2));
    }

    #[test]
    fn error_line_has_error_text_and_is_flagged() {
        let doc = Document::evaluate("1/0");
        let r = doc.result_for_line(1).unwrap();
        assert!(r.contains("ноль"));
        assert!(doc.is_error_line(1));
    }

    #[test]
    fn last_statement_on_line_determines_result() {
        let doc = Document::evaluate("x = 1; x + 1");
        assert_eq!(doc.result_for_line(1).as_deref(), Some("2"));
    }

    #[test]
    fn defined_lines_have_no_result() {
        let doc = Document::evaluate("fn f(n) = n*n");
        assert_eq!(doc.result_for_line(1), None);
    }

    #[test]
    fn print_captured_in_output() {
        let doc = Document::evaluate("print(2+2)");
        assert_eq!(doc.output, "4\n");
    }
}
