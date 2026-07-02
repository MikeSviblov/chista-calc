use calc_core::{DocLineOutcome, Session};
use std::collections::{HashMap, HashSet};

/// Результаты вычисления документа, разложенные по номерам строк для отрисовки.
pub struct Document {
    results: HashMap<usize, String>,
    errors: HashSet<usize>,
    pub output: String,
    pub variables: Vec<(String, String)>,
}

impl Document {
    pub fn evaluate(src: &str) -> Self {
        let mut s = Session::new();
        let d = s.eval_document(src);
        let mut results = HashMap::new();
        let mut errors = HashSet::new();
        for l in &d.lines {
            match &l.outcome {
                DocLineOutcome::Value(v) => { results.insert(l.line, v.to_string()); }
                DocLineOutcome::Error(e) => { results.insert(l.line, e.to_string()); errors.insert(l.line); }
                DocLineOutcome::Defined => {}
            }
        }
        let variables = s.variables().into_iter().map(|(k, v)| (k, v.to_string())).collect();
        Document { results, errors, output: d.output, variables }
    }
    pub fn result_for_line(&self, line: usize) -> Option<String> { self.results.get(&line).cloned() }
    pub fn is_error_line(&self, line: usize) -> bool { self.errors.contains(&line) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rows_map_results_to_line_numbers() {
        let doc = Document::evaluate("цена = 1990\nштук = 12\nцена * штук\nIntToRoman(2024)");
        assert_eq!(doc.result_for_line(1).as_deref(), Some("1990"));
        assert_eq!(doc.result_for_line(3).as_deref(), Some("23880"));
        assert_eq!(doc.result_for_line(4).as_deref(), Some("MMXXIV"));
        assert_eq!(doc.output, "");
    }
    #[test]
    fn error_line_has_error_text_and_is_flagged() {
        let doc = Document::evaluate("1/0");
        let r = doc.result_for_line(1).unwrap();
        assert!(r.contains("ноль"));
        assert!(doc.is_error_line(1));
    }
    #[test]
    fn print_goes_to_output_panel() {
        let doc = Document::evaluate("print(2+2)");
        assert_eq!(doc.output, "4\n");
    }
    #[test]
    fn defined_lines_have_no_result() {
        let doc = Document::evaluate("fn f(n) = n*n");
        assert_eq!(doc.result_for_line(1), None);
    }
}
