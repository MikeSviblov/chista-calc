use thiserror::Error;
pub type Pos = usize;

#[derive(Debug, Error, PartialEq)]
pub enum CalcError {
    #[error("Синтаксическая ошибка: {msg} (позиция {pos})")]
    SyntaxError { msg: String, pos: Pos },
    #[error("Ошибка разбора: {msg} (позиция {pos})")]
    ParserError { msg: String, pos: Pos },
    #[error("Неизвестная переменная '{name}' (позиция {pos})")]
    UnknownVariable { name: String, pos: Pos },
    #[error("Неизвестная функция '{name}' (позиция {pos})")]
    UnknownFunction { name: String, pos: Pos },
    #[error("Функция '{func}': неправильные параметры (ожидалось {expected}, получено {got}) (позиция {pos})")]
    WrongParams { func: String, expected: String, got: usize, pos: Pos },
    #[error("Деление на ноль (позиция {pos})")]
    DivisionByZero { pos: Pos },
    #[error("Ошибка диапазона: {msg} (позиция {pos})")]
    RangeError { msg: String, pos: Pos },
    #[error("Ошибка ввода-вывода: {msg}")]
    IoError { msg: String },
    #[error("Превышен лимит итераций цикла ({limit})")]
    LoopLimitExceeded { limit: u64 },
}
pub type Result<T> = std::result::Result<T, CalcError>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn division_by_zero_message_is_russian() {
        let e = CalcError::DivisionByZero { pos: 4 };
        assert_eq!(e.to_string(), "Деление на ноль (позиция 4)");
    }
    #[test]
    fn unknown_function_carries_name() {
        let e = CalcError::UnknownFunction { name: "Foo".into(), pos: 0 };
        assert!(e.to_string().contains("Foo"));
    }
}
