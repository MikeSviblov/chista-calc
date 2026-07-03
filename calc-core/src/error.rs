use std::fmt;

pub type Pos = usize;

/// Язык пользовательских сообщений (ошибки, справка, UI фронтендов).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Lang {
    /// Русский (по умолчанию).
    #[default]
    Ru,
    /// English.
    En,
}

impl Lang {
    /// Разбор языка из строки (`ru`/`russian`, `en`/`english`), регистр не важен.
    pub fn parse(s: &str) -> Option<Lang> {
        match s.trim().to_lowercase().as_str() {
            "ru" | "rus" | "russian" | "русский" => Some(Lang::Ru),
            "en" | "eng" | "english" => Some(Lang::En),
            _ => None,
        }
    }

    /// Короткий код языка (`"ru"`/`"en"`) — для сохранения в настройках.
    pub fn code(self) -> &'static str {
        match self {
            Lang::Ru => "ru",
            Lang::En => "en",
        }
    }
}

/// Возвращает `ru` или `en` по языку (для коротких двуязычных строк).
fn tr(lang: Lang, ru: &str, en: &str) -> String {
    match lang {
        Lang::Ru => ru,
        Lang::En => en,
    }
    .to_string()
}

/// Причина ошибки диапазона/разбора/ввода-вывода — код вместо готовой строки,
/// чтобы текст можно было отдать на нужном языке (см. [`Reason::text`]). Динамические
/// части (символ, имя алгоритма, путь) хранятся в самих вариантах.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    // --- без параметров ---
    Overflow,
    ExpectedNumber,
    ExpectedInt,
    ExpectedString,
    EmptyString,
    InvalidCharCode,
    InvalidRoman,
    BaseOutOfRange,
    RomanOutOfRange,
    FactorialNegative,
    ExponentTooLarge,
    UnaryMinusNonNumber,
    DigitsOutOfRange,
    StartTooSmall,
    LenNegative,
    KeyLength,
    DecryptFailed,
    DecryptNotUtf8,
    NumberTooLarge,
    UnterminatedString,
    ExprNestingTooDeep,
    ShiftOutOfRange,
    BitIndexOutOfRange,
    // --- с параметрами ---
    InvalidDigitForBase { c: char, base: u32 },
    InvalidCharInRoman(char),
    UnknownHash(String),
    UnknownCipher(String),
    BadKeyHex(String),
    BadCiphertextHex(String),
    SystemTime(String),
    ReadFailed { path: String, err: String },
    WriteFailed { path: String, err: String },
    AppendFailed { path: String, err: String },
    UnknownChar(char),
    InvalidNumber(String),
    ExpectedToken { expected: String, found: String },
    ExpectedIdent(String),
    ExpectedExpression(String),
}

impl Reason {
    /// Текст причины на заданном языке.
    pub fn text(&self, lang: Lang) -> String {
        use Reason::*;
        match self {
            Overflow => tr(lang, "переполнение", "overflow"),
            ExpectedNumber => tr(lang, "ожидалось число", "expected a number"),
            ExpectedInt => tr(lang, "ожидалось целое число", "expected an integer"),
            ExpectedString => tr(lang, "ожидалась строка", "expected a string"),
            EmptyString => tr(lang, "пустая строка", "empty string"),
            InvalidCharCode => tr(lang, "недопустимый код символа", "invalid character code"),
            InvalidRoman => tr(lang, "недопустимая римская запись", "invalid Roman numeral"),
            BaseOutOfRange => tr(
                lang,
                "база должна быть в диапазоне 2..=36",
                "base must be in the range 2..=36",
            ),
            RomanOutOfRange => tr(
                lang,
                "число должно быть в диапазоне 1..=3999",
                "number must be in the range 1..=3999",
            ),
            FactorialNegative => tr(
                lang,
                "факториал отрицательного числа",
                "factorial of a negative number",
            ),
            ExponentTooLarge => tr(lang, "слишком большая степень", "exponent too large"),
            UnaryMinusNonNumber => tr(
                lang,
                "унарный минус к не-числу",
                "unary minus on a non-number",
            ),
            DigitsOutOfRange => tr(
                lang,
                "digits должен быть в диапазоне 0..=100",
                "digits must be in the range 0..=100",
            ),
            StartTooSmall => tr(lang, "start должен быть >= 1", "start must be >= 1"),
            LenNegative => tr(lang, "len должен быть >= 0", "len must be >= 0"),
            KeyLength => tr(
                lang,
                "длина ключа должна быть 16/24/32 байта",
                "key length must be 16/24/32 bytes",
            ),
            DecryptFailed => tr(lang, "ошибка расшифровки", "decryption failed"),
            DecryptNotUtf8 => tr(
                lang,
                "расшифрованные данные не являются валидным UTF-8",
                "decrypted data is not valid UTF-8",
            ),
            NumberTooLarge => tr(lang, "Число слишком большое", "number too large"),
            UnterminatedString => tr(lang, "Незакрытая строка", "unterminated string"),
            ExprNestingTooDeep => tr(
                lang,
                "Слишком глубокая вложенность выражения",
                "expression nesting too deep",
            ),
            ShiftOutOfRange => tr(
                lang,
                "сдвиг должен быть в диапазоне 0..=127",
                "shift must be in the range 0..=127",
            ),
            BitIndexOutOfRange => tr(
                lang,
                "индекс бита должен быть в диапазоне 0..=127",
                "bit index must be in the range 0..=127",
            ),
            InvalidDigitForBase { c, base } => match lang {
                Lang::Ru => format!("недопустимая цифра '{c}' для базы {base}"),
                Lang::En => format!("invalid digit '{c}' for base {base}"),
            },
            InvalidCharInRoman(c) => match lang {
                Lang::Ru => format!("недопустимый символ '{c}'"),
                Lang::En => format!("invalid character '{c}'"),
            },
            UnknownHash(alg) => match lang {
                Lang::Ru => format!("неизвестный алгоритм хеша '{alg}'"),
                Lang::En => format!("unknown hash algorithm '{alg}'"),
            },
            UnknownCipher(alg) => match lang {
                Lang::Ru => format!("неизвестный/неподдерживаемый шифр '{alg}'"),
                Lang::En => format!("unknown/unsupported cipher '{alg}'"),
            },
            BadKeyHex(e) => match lang {
                Lang::Ru => format!("некорректный hex ключа: {e}"),
                Lang::En => format!("invalid key hex: {e}"),
            },
            BadCiphertextHex(e) => match lang {
                Lang::Ru => format!("некорректный hex шифротекста: {e}"),
                Lang::En => format!("invalid ciphertext hex: {e}"),
            },
            SystemTime(e) => match lang {
                Lang::Ru => format!("не удалось получить текущее время: {e}"),
                Lang::En => format!("failed to get current time: {e}"),
            },
            ReadFailed { path, err } => match lang {
                Lang::Ru => format!("не удалось прочитать '{path}': {err}"),
                Lang::En => format!("failed to read '{path}': {err}"),
            },
            WriteFailed { path, err } => match lang {
                Lang::Ru => format!("не удалось записать '{path}': {err}"),
                Lang::En => format!("failed to write '{path}': {err}"),
            },
            AppendFailed { path, err } => match lang {
                Lang::Ru => format!("не удалось дописать в '{path}': {err}"),
                Lang::En => format!("failed to append to '{path}': {err}"),
            },
            UnknownChar(c) => match lang {
                Lang::Ru => format!("Неизвестный символ '{c}'"),
                Lang::En => format!("unknown character '{c}'"),
            },
            InvalidNumber(s) => match lang {
                Lang::Ru => format!("Некорректное число '{s}'"),
                Lang::En => format!("invalid number '{s}'"),
            },
            ExpectedToken { expected, found } => match lang {
                Lang::Ru => format!("Ожидалось {expected}, но встретилось {found}"),
                Lang::En => format!("expected {expected}, but found {found}"),
            },
            ExpectedIdent(found) => match lang {
                Lang::Ru => format!("Ожидался идентификатор, но встретилось {found}"),
                Lang::En => format!("expected an identifier, but found {found}"),
            },
            ExpectedExpression(found) => match lang {
                Lang::Ru => format!("Ожидалось выражение, но встретилось {found}"),
                Lang::En => format!("expected an expression, but found {found}"),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcError {
    SyntaxError { msg: Reason, pos: Pos },
    ParserError { msg: Reason, pos: Pos },
    UnknownVariable { name: String, pos: Pos },
    UnknownFunction { name: String, pos: Pos },
    WrongParams { func: String, expected: String, got: usize, pos: Pos },
    DivisionByZero { pos: Pos },
    RangeError { msg: Reason, pos: Pos },
    IoError { msg: Reason },
    LoopLimitExceeded { limit: u64 },
    CallDepthExceeded { limit: u64 },
    ExprTooDeep { limit: u64 },
}

impl CalcError {
    /// Человекочитаемое сообщение об ошибке на заданном языке.
    pub fn message(&self, lang: Lang) -> String {
        use CalcError::*;
        match self {
            SyntaxError { msg, pos } => match lang {
                Lang::Ru => format!("Синтаксическая ошибка: {} (позиция {pos})", msg.text(lang)),
                Lang::En => format!("Syntax error: {} (position {pos})", msg.text(lang)),
            },
            ParserError { msg, pos } => match lang {
                Lang::Ru => format!("Ошибка разбора: {} (позиция {pos})", msg.text(lang)),
                Lang::En => format!("Parse error: {} (position {pos})", msg.text(lang)),
            },
            UnknownVariable { name, pos } => match lang {
                Lang::Ru => format!("Неизвестная переменная '{name}' (позиция {pos})"),
                Lang::En => format!("Unknown variable '{name}' (position {pos})"),
            },
            UnknownFunction { name, pos } => match lang {
                Lang::Ru => format!("Неизвестная функция '{name}' (позиция {pos})"),
                Lang::En => format!("Unknown function '{name}' (position {pos})"),
            },
            WrongParams { func, expected, got, pos } => match lang {
                Lang::Ru => format!(
                    "Функция '{func}': неправильные параметры (ожидалось {expected}, получено {got}) (позиция {pos})"
                ),
                Lang::En => format!(
                    "Function '{func}': wrong parameters (expected {expected}, got {got}) (position {pos})"
                ),
            },
            DivisionByZero { pos } => match lang {
                Lang::Ru => format!("Деление на ноль (позиция {pos})"),
                Lang::En => format!("Division by zero (position {pos})"),
            },
            RangeError { msg, pos } => match lang {
                Lang::Ru => format!("Ошибка диапазона: {} (позиция {pos})", msg.text(lang)),
                Lang::En => format!("Range error: {} (position {pos})", msg.text(lang)),
            },
            IoError { msg } => match lang {
                Lang::Ru => format!("Ошибка ввода-вывода: {}", msg.text(lang)),
                Lang::En => format!("I/O error: {}", msg.text(lang)),
            },
            LoopLimitExceeded { limit } => match lang {
                Lang::Ru => format!("Превышен лимит итераций цикла ({limit})"),
                Lang::En => format!("Loop iteration limit exceeded ({limit})"),
            },
            CallDepthExceeded { limit } => match lang {
                Lang::Ru => format!("Превышена глубина вызовов функций ({limit})"),
                Lang::En => format!("Function call depth exceeded ({limit})"),
            },
            ExprTooDeep { limit } => match lang {
                Lang::Ru => format!("Слишком глубокое выражение (предел {limit})"),
                Lang::En => format!("Expression too deep (limit {limit})"),
            },
        }
    }
}

/// По умолчанию — русский (обратная совместимость: фронтенды зовут `message(lang)`).
impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message(Lang::Ru))
    }
}

impl std::error::Error for CalcError {}

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
    fn division_by_zero_message_in_english() {
        let e = CalcError::DivisionByZero { pos: 4 };
        assert_eq!(e.message(Lang::En), "Division by zero (position 4)");
    }
    #[test]
    fn unknown_function_carries_name() {
        let e = CalcError::UnknownFunction { name: "Foo".into(), pos: 0 };
        assert!(e.to_string().contains("Foo"));
        assert!(e.message(Lang::En).contains("Foo"));
    }
    #[test]
    fn range_error_reason_switches_language() {
        let e = CalcError::RangeError { msg: Reason::Overflow, pos: 2 };
        assert_eq!(e.message(Lang::Ru), "Ошибка диапазона: переполнение (позиция 2)");
        assert_eq!(e.message(Lang::En), "Range error: overflow (position 2)");
    }
}
