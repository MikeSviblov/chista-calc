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
pub use value::Value;

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
        self.ev.run(&stmts)
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
