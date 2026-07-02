pub mod ast;
pub mod builtins;
pub mod env;
pub mod error;
pub mod eval;
pub mod lexer;
pub mod parser;
pub mod registry;
pub mod value;

pub use error::CalcError;
pub use eval::Evaluator;
pub use eval::StmtOutcome;
pub use value::Value;

#[derive(Debug)]
pub enum DocLineOutcome {
    Value(Value),
    Defined,
    Error(CalcError),
}

#[derive(Debug)]
pub struct DocLine {
    pub line: usize,
    pub outcome: DocLineOutcome,
}

#[derive(Debug, Default)]
pub struct DocResult {
    pub lines: Vec<DocLine>,
    pub output: String,
}

fn char_pos_to_line(src: &str, pos: usize) -> usize {
    src.chars().take(pos).filter(|c| *c == '\n').count() + 1
}

fn err_pos(e: &CalcError) -> usize {
    match e {
        CalcError::SyntaxError { pos, .. }
        | CalcError::ParserError { pos, .. }
        | CalcError::UnknownVariable { pos, .. }
        | CalcError::UnknownFunction { pos, .. }
        | CalcError::WrongParams { pos, .. }
        | CalcError::DivisionByZero { pos }
        | CalcError::RangeError { pos, .. } => *pos,
        CalcError::IoError { .. }
        | CalcError::LoopLimitExceeded { .. }
        | CalcError::CallDepthExceeded { .. }
        | CalcError::ExprTooDeep { .. } => 0,
    }
}

pub struct Session {
    ev: eval::Evaluator,
}

impl Session {
    pub fn new() -> Self {
        Session {
            ev: eval::Evaluator::new(),
        }
    }

    /// Вычислить исходный текст (одна или несколько инструкций через `;` или перенос строки).
    pub fn eval(&mut self, src: &str) -> error::Result<Value> {
        let toks = lexer::tokenize(src)?;
        let stmts = parser::Parser::new(toks).parse_program()?;
        // Выполнить, затем ВСЕГДA сбросить накопленный print-вывод — даже если
        // одна из инструкций упала с ошибкой (частичный вывод не должен теряться).
        let result = self.ev.run(&stmts);
        let out = self.ev.take_output();
        if !out.is_empty() {
            print!("{out}");
        }
        result
    }

    /// Построчно выполнить документ с чистого состояния (best-effort).
    pub fn eval_document(&mut self, src: &str) -> DocResult {
        self.ev = eval::Evaluator::new();
        let toks = match lexer::tokenize(src) {
            Ok(t) => t,
            Err(e) => {
                return DocResult {
                    lines: vec![DocLine { line: char_pos_to_line(src, err_pos(&e)), outcome: DocLineOutcome::Error(e) }],
                    output: String::new(),
                };
            }
        };
        let stmts = match parser::Parser::new(toks).parse_program() {
            Ok(s) => s,
            Err(e) => {
                return DocResult {
                    lines: vec![DocLine { line: char_pos_to_line(src, err_pos(&e)), outcome: DocLineOutcome::Error(e) }],
                    output: String::new(),
                };
            }
        };
        let raw = self.ev.run_document(&stmts);
        let lines = raw
            .into_iter()
            .map(|(pos, outcome)| DocLine {
                line: char_pos_to_line(src, pos),
                outcome: match outcome {
                    StmtOutcome::Value(v) => DocLineOutcome::Value(v),
                    StmtOutcome::Defined => DocLineOutcome::Defined,
                    StmtOutcome::Error(e) => DocLineOutcome::Error(e),
                },
            })
            .collect();
        let output = self.ev.take_output();
        DocResult { lines, output }
    }

    /// Снимок глобальных переменных.
    pub fn variables(&self) -> Vec<(String, Value)> {
        self.ev.env.globals()
    }

    /// Имена всех доступных функций (встроенные + print).
    pub fn builtin_names(&self) -> Vec<String> {
        let mut n: Vec<String> = self.ev.registry.names().into_iter().map(|s| s.to_string()).collect();
        n.push("print".to_string());
        n.sort();
        n
    }

    /// Доступ к вычислителю (для будущих фронтендов).
    pub fn evaluator(&mut self) -> &mut eval::Evaluator {
        &mut self.ev
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
