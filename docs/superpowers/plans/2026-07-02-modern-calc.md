# Современный калькулятор (Rust) — план реализации

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Построить современный эквивалент «Чиста калькулятор 2.0» на Rust — библиотека-ядро `calc-core` + CLI `calc-cli` — со скриптовым языком выражений, переменными, пользовательскими функциями, циклами, системами счисления, хешами и шифрами.

**Architecture:** Cargo-workspace. `calc-core`: `lexer -> parser -> ast -> eval`, плюс `value`, `env`, `registry`, `builtins/*`, `error`. Крипта изолирована за диспетчером на базе крейтов RustCrypto. `calc-cli` — тонкая обёртка (REPL, запуск файлов, разовое выражение).

**Tech Stack:** Rust (edition 2021), crates: `clap` (CLI), `rustyline` (REPL), `thiserror` (ошибки), `proptest` (property-тесты, dev), RustCrypto: `md-5 sha1 sha2 sha3 ripemd digest crc32fast adler`, шифры `aes` + `cbc`, `hex`, `base64`.

**Спека:** `docs/superpowers/specs/2026-07-02-modern-calc-design.md`
**Инвентарь функций оригинала:** `reverse/extracted-inventory.md`

---

## Соглашения по всем задачам

- TDD: сначала падающий тест -> минимальная реализация -> зелёный тест -> коммит.
- Каждый коммит атомарный, сообщение на русском, префикс `feat:`/`test:`/`chore:`.
- Тесты юнит-уровня — в `#[cfg(test)] mod tests` внутри модуля; интеграционные — в `calc-core/tests/` и `calc-cli/tests/`.
- Запуск тестов модуля: `cargo test -p calc-core <фильтр>`.
- Сообщения об ошибках для пользователя — на русском; идентификаторы и код — на английском.
- Примечание для исполнителя: слово `eval` встречается как штатное имя метода вычислителя (не опасный вызов). Хук Write помечает его как предупреждение — используйте создание файлов через доступный вам механизм записи.

---

## Фаза 0 — Тулчейн и каркас workspace

### Task 0.1: Установить Rust и создать workspace

**Files:**
- Create: `Cargo.toml` (workspace)
- Create: `calc-core/Cargo.toml`, `calc-core/src/lib.rs`
- Create: `calc-cli/Cargo.toml`, `calc-cli/src/main.rs`
- Modify: `.gitignore`

- [ ] **Step 1: Установить rustup (если cargo отсутствует)**

Run:
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    cargo --version
Expected: печатает версию, напр. `cargo 1.7x.0`.

- [ ] **Step 2: Создать корневой `Cargo.toml`**

    [workspace]
    resolver = "2"
    members = ["calc-core", "calc-cli"]

    [workspace.package]
    edition = "2021"
    version = "0.1.0"
    license = "MIT"

    [workspace.dependencies]
    thiserror = "1"

- [ ] **Step 3: Создать `calc-core/Cargo.toml`**

    [package]
    name = "calc-core"
    edition.workspace = true
    version.workspace = true

    [dependencies]
    thiserror.workspace = true

    [dev-dependencies]
    proptest = "1"

- [ ] **Step 4: Создать `calc-core/src/lib.rs` (пока только error+value; остальные модули включать по мере готовности)**

    pub mod error;
    pub mod value;

    pub use error::CalcError;
    pub use value::Value;

- [ ] **Step 5: Создать `calc-cli/Cargo.toml`**

    [package]
    name = "calc-cli"
    edition.workspace = true
    version.workspace = true

    [[bin]]
    name = "calc"
    path = "src/main.rs"

    [dependencies]
    calc-core = { path = "../calc-core" }

- [ ] **Step 6: Заглушка `calc-cli/src/main.rs`**

    fn main() {
        println!("calc 0.1.0");
    }

- [ ] **Step 7: Обновить `.gitignore`**

    /target
    **/venv/
    *.exe

- [ ] **Step 8: Проверить сборку**

Run: `cargo build`
Expected: `Finished`. (Если `lib.rs` уже ссылается на error/value, а их файлов ещё нет — выполнить Task 1.1/1.2 в паре, либо временно закомментировать `pub mod` строки.)

- [ ] **Step 9: Коммит**

    git add Cargo.toml calc-core calc-cli .gitignore
    git commit -m "chore: каркас cargo workspace (calc-core + calc-cli)"

---

## Фаза 1 — Ошибки и модель значений

### Task 1.1: Тип ошибок `CalcError`

**Files:** Create: `calc-core/src/error.rs`

- [ ] **Step 1: Падающий тест** (в конец файла)

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

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core error::` — Expected: ошибка компиляции.

- [ ] **Step 3: Реализация** (в начало файла)

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

- [ ] **Step 4: Запустить.** Run: `cargo test -p calc-core error::` — Expected: PASS (2).

- [ ] **Step 5: Коммит.** `git add calc-core/src/error.rs && git commit -m "feat: тип ошибок CalcError с русскими сообщениями"`

### Task 1.2: Модель значений `Value`

**Files:** Create: `calc-core/src/value.rs`

- [ ] **Step 1: Падающий тест**

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn display_variants() {
            assert_eq!(Value::Int(42).to_string(), "42");
            assert_eq!(Value::Float(1.5).to_string(), "1.5");
            assert_eq!(Value::Str("hi".into()).to_string(), "hi");
            assert_eq!(Value::Bool(true).to_string(), "true");
        }
        #[test]
        fn as_int_promotes_and_rejects() {
            assert_eq!(Value::Int(5).as_int(0).unwrap(), 5);
            assert_eq!(Value::Float(5.0).as_int(0).unwrap(), 5);
            assert!(Value::Float(5.5).as_int(0).is_err());
            assert!(Value::Str("x".into()).as_int(0).is_err());
        }
        #[test]
        fn as_float_promotes_int() {
            assert_eq!(Value::Int(3).as_float(0).unwrap(), 3.0);
            assert_eq!(Value::Float(3.5).as_float(0).unwrap(), 3.5);
        }
    }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core value::`

- [ ] **Step 3: Реализация**

    use crate::error::{CalcError, Pos, Result};
    use std::fmt;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Value { Int(i128), Float(f64), Str(String), Bool(bool) }

    impl Value {
        pub fn as_int(&self, pos: Pos) -> Result<i128> {
            match self {
                Value::Int(i) => Ok(*i),
                Value::Float(f) if f.fract() == 0.0 => Ok(*f as i128),
                _ => Err(CalcError::RangeError { msg: "ожидалось целое число".into(), pos }),
            }
        }
        pub fn as_float(&self, pos: Pos) -> Result<f64> {
            match self {
                Value::Int(i) => Ok(*i as f64),
                Value::Float(f) => Ok(*f),
                _ => Err(CalcError::RangeError { msg: "ожидалось число".into(), pos }),
            }
        }
        pub fn as_str(&self, pos: Pos) -> Result<&str> {
            match self {
                Value::Str(s) => Ok(s),
                _ => Err(CalcError::RangeError { msg: "ожидалась строка".into(), pos }),
            }
        }
        pub fn truthy(&self) -> bool {
            match self {
                Value::Bool(b) => *b,
                Value::Int(i) => *i != 0,
                Value::Float(f) => *f != 0.0,
                Value::Str(s) => !s.is_empty(),
            }
        }
    }

    impl fmt::Display for Value {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Value::Int(i) => write!(f, "{i}"),
                Value::Float(x) => write!(f, "{x}"),
                Value::Str(s) => write!(f, "{s}"),
                Value::Bool(b) => write!(f, "{b}"),
            }
        }
    }

- [ ] **Step 4: Запустить.** Run: `cargo test -p calc-core value::` — PASS (3). Раскомментировать `pub mod` в lib.rs, `cargo build`.

- [ ] **Step 5: Коммит.** `git add calc-core/src/value.rs calc-core/src/lib.rs && git commit -m "feat: модель значений Value (Int/Float/Str/Bool) с приведениями"`

Правила приведения (реализуются в Task 4.3 apply_binop): `Int op Int -> Int`, кроме `/` с нецелым результатом и `^` с отрицательной степенью -> `Float`; любой `Float` -> `Float`; битовые и base-функции требуют `Int`, иначе ошибка.

---

## Фаза 2 — Лексер

### Task 2.1: Токены и `tokenize`

**Files:** Create: `calc-core/src/lexer.rs`; Modify: `lib.rs` (`pub mod lexer;`)

- [ ] **Step 1: Падающий тест**

    #[cfg(test)]
    mod tests {
        use super::*;
        fn kinds(src: &str) -> Vec<TokenKind> {
            tokenize(src).unwrap().into_iter().map(|t| t.kind).collect()
        }
        #[test]
        fn numbers_in_all_bases() {
            assert_eq!(kinds("10"), vec![TokenKind::Int(10), TokenKind::Eof]);
            assert_eq!(kinds("0x1F"), vec![TokenKind::Int(31), TokenKind::Eof]);
            assert_eq!(kinds("0b1010"), vec![TokenKind::Int(10), TokenKind::Eof]);
            assert_eq!(kinds("0o17"), vec![TokenKind::Int(15), TokenKind::Eof]);
            assert_eq!(kinds("1.5"), vec![TokenKind::Float(1.5), TokenKind::Eof]);
            assert_eq!(kinds("1e3"), vec![TokenKind::Float(1000.0), TokenKind::Eof]);
        }
        #[test]
        fn string_with_escapes() {
            assert_eq!(kinds("\"a\\nb\""), vec![TokenKind::Str("a\nb".into()), TokenKind::Eof]);
        }
        #[test]
        fn operators_idents_comments() {
            assert_eq!(kinds("x = f(1) # hi"), vec![
                TokenKind::Ident("x".into()), TokenKind::Eq, TokenKind::Ident("f".into()),
                TokenKind::LParen, TokenKind::Int(1), TokenKind::RParen, TokenKind::Eof,
            ]);
        }
        #[test]
        fn multi_char_operators() {
            assert_eq!(kinds("=="), vec![TokenKind::EqEq, TokenKind::Eof]);
            assert_eq!(kinds("&&"), vec![TokenKind::AndAnd, TokenKind::Eof]);
            assert_eq!(kinds("<="), vec![TokenKind::Le, TokenKind::Eof]);
        }
    }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core lexer::`

- [ ] **Step 3: Реализация.** Определить:

    #[derive(Debug, Clone, PartialEq)]
    pub enum TokenKind {
        Int(i128), Float(f64), Str(String), Ident(String), True, False,
        Plus, Minus, Star, Slash, Percent, Caret,
        EqEq, Ne, Lt, Le, Gt, Ge, AndAnd, OrOr, Bang,
        Eq, LParen, RParen, LBrace, RBrace, Comma, Semicolon, Newline,
        KwFn, KwAlias, KwWhile, KwRepeat, Eof,
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Token { pub kind: TokenKind, pub pos: usize }
    pub fn tokenize(src: &str) -> crate::error::Result<Vec<Token>> { /* см. ниже */ }

Логика сканера по `Vec<char>` с индексом-позицией:
- пробелы/табы пропускать; `#` — до конца строки (комментарий); `\n` -> эмитить `Newline` (нужно позже для скриптов; в однострочных тестах не появляется).
- число: `0x`/`0b`/`0o` -> парсить в базе 16/2/8 в `Int`; иначе десятичное: если есть `.` или `e/E` -> `Float` (через `str::parse::<f64>`), иначе `Int`.
- строка `"..."`: экранирование `\n \t \" \\`; незакрытая -> `SyntaxError`.
- идентификатор `[A-Za-z_]` + `[A-Za-z0-9_]` (кириллица допускается через `char::is_alphabetic`): ключевые слова `fn/alias/while/repeat/true/false`, иначе `Ident`.
- операторы: сперва двусимвольные `== != <= >= && ||`, затем односимвольные `+ - * / % ^ = ( ) { } , ; < > !`.
- в конце — `Eof` с позицией `src.len()`.
- неизвестный символ -> `SyntaxError { msg: format!("Неизвестный символ '{c}'"), pos }`.

- [ ] **Step 4: Запустить.** Run: `cargo test -p calc-core lexer::` — PASS (4).

- [ ] **Step 5: Коммит.** `git add calc-core/src/lexer.rs calc-core/src/lib.rs && git commit -m "feat: лексер (числа в 4 базах, строки, операторы, комментарии)"`

---

## Фаза 3 — AST и парсер

### Task 3.1: AST

**Files:** Create: `calc-core/src/ast.rs`; Modify: `lib.rs`

- [ ] **Step 1: Реализация + тест-заглушка**

    #[derive(Debug, Clone, PartialEq)]
    pub enum Expr {
        Int(i128, usize), Float(f64, usize), Str(String, usize), Bool(bool, usize),
        Var(String, usize),
        Unary { op: UnOp, rhs: Box<Expr>, pos: usize },
        Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr>, pos: usize },
        Call { name: String, args: Vec<Expr>, pos: usize },
        Assign { name: String, value: Box<Expr>, pos: usize },
    }
    #[derive(Debug, Clone, PartialEq)]
    pub enum Stmt {
        Expr(Expr),
        FnDef { name: String, params: Vec<String>, body: Expr, pos: usize },
        Alias { name: String, target: String, pos: usize },
        While { cond: Expr, body: Vec<Stmt>, pos: usize },
        Repeat { count: Expr, body: Vec<Stmt>, pos: usize },
    }
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum UnOp { Neg, Not }
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum BinOp { Add, Sub, Mul, Div, Rem, Pow, Eq, Ne, Lt, Le, Gt, Ge, And, Or }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn build_expr() {
            let e = Expr::Binary { op: BinOp::Add, lhs: Box::new(Expr::Int(1,0)), rhs: Box::new(Expr::Int(2,2)), pos: 1 };
            assert!(matches!(e, Expr::Binary { op: BinOp::Add, .. }));
        }
    }

- [ ] **Step 2: Запустить.** Run: `cargo test -p calc-core ast::` — PASS.
- [ ] **Step 3: Коммит.** `git add calc-core/src/ast.rs calc-core/src/lib.rs && git commit -m "feat: типы AST (Expr, Stmt, операторы)"`

### Task 3.2: Парсер (Pratt) выражений и инструкций

**Files:** Create: `calc-core/src/parser.rs`; Modify: `lib.rs`

- [ ] **Step 1: Падающие тесты**

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::ast::{Expr, BinOp};
        fn parse_expr(src: &str) -> Expr {
            let toks = crate::lexer::tokenize(src).unwrap();
            Parser::new(toks).parse_single_expr().unwrap()
        }
        #[test]
        fn precedence_mul_over_add() {
            match parse_expr("1 + 2 * 3") {
                Expr::Binary { op: BinOp::Add, rhs, .. } => assert!(matches!(*rhs, Expr::Binary { op: BinOp::Mul, .. })),
                _ => panic!("ожидался Add на вершине"),
            }
        }
        #[test]
        fn pow_is_right_assoc() {
            match parse_expr("2 ^ 3 ^ 2") {
                Expr::Binary { op: BinOp::Pow, rhs, .. } => assert!(matches!(*rhs, Expr::Binary { op: BinOp::Pow, .. })),
                _ => panic!(),
            }
        }
        #[test]
        fn call_with_args() {
            match parse_expr("Max(1, 2, 3)") {
                Expr::Call { name, args, .. } => { assert_eq!(name, "Max"); assert_eq!(args.len(), 3); }
                _ => panic!(),
            }
        }
        #[test]
        fn unary_minus_parses() { let _ = parse_expr("-2 ^ 2"); }
    }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core parser::`

- [ ] **Step 3: Реализация.** `pub struct Parser { tokens: Vec<Token>, i: usize }`:
- `new`, `peek() -> &TokenKind`, `advance() -> Token`, `pos()`, `expect(kind)` (несовпадение -> `SyntaxError { msg: format!("Ожидалось {:?}, но встретилось {:?}", want, got), pos }`).
- `parse_single_expr(&mut) -> Result<Expr>`: `parse_expr(0)` затем `expect(Eof)`.
- `parse_program(&mut) -> Result<Vec<Stmt>>`: цикл; пропускать разделители `Semicolon`/`Newline`; на `Eof` стоп; иначе `parse_stmt`.
- `parse_stmt`: по первому токену — `KwFn` -> FnDef (`fn Ident ( ident,* ) = expr`); `KwAlias` -> Alias (`alias Ident = Ident`); `KwWhile` -> While (`while ( expr ) { block }`); `KwRepeat` -> Repeat (`repeat expr { block }`); иначе `Expr(parse_expr(0))`.
- `parse_block`: `expect(LBrace)`; собирать инструкции до `RBrace`, пропуская разделители; `expect(RBrace)`.
- Pratt `parse_expr(min_bp)`:
  - nud: `Int/Float/Str/True/False` -> литерал; `Ident` -> если следующий `LParen` — `Call` (парсить args через запятую), иначе `Var`; `LParen expr RParen`; префикс `Minus` -> `Unary{Neg}` (bp правого операнда 5), `Bang` -> `Unary{Not}`.
  - led по binding power (левый,правый): `OrOr`(1,2) `AndAnd`(3,4) сравнения `EqEq Ne Lt Le Gt Ge`(5,6) `Plus Minus`(7,8) `Star Slash Percent`(9,10) `Caret`(правоассоц. 12,11). Унарный минус выше `*`, ниже `^`.
  - присваивание: после nud, если получили `Var(name)` и текущий токен `Eq` — прочитать `=`, распарсить rhs `parse_expr(0)`, вернуть `Assign`.

- [ ] **Step 4: Запустить.** Run: `cargo test -p calc-core parser::` — PASS (4).
- [ ] **Step 5: Коммит.** `git add calc-core/src/parser.rs calc-core/src/lib.rs && git commit -m "feat: Pratt-парсер выражений и инструкций"`

---

## Фаза 4 — Окружение, реестр, вычислитель

### Task 4.1: `Env`

**Files:** Create: `calc-core/src/env.rs`; Modify: `lib.rs`

- [ ] **Step 1: Падающий тест**

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::value::Value;
        #[test]
        fn set_get_variable() {
            let mut env = Env::new();
            env.set_var("x", Value::Int(5));
            assert_eq!(env.get_var("x"), Some(Value::Int(5)));
            assert_eq!(env.get_var("y"), None);
        }
        #[test]
        fn scopes_shadow_and_pop() {
            let mut env = Env::new();
            env.set_var("x", Value::Int(1));
            env.push_scope();
            env.set_var("x", Value::Int(2));
            assert_eq!(env.get_var("x"), Some(Value::Int(2)));
            env.pop_scope();
            assert_eq!(env.get_var("x"), Some(Value::Int(1)));
        }
        #[test]
        fn assign_updates_outer_scope() {
            let mut env = Env::new();
            env.set_var("x", Value::Int(1));
            env.push_scope();
            env.assign("x", Value::Int(9));
            env.pop_scope();
            assert_eq!(env.get_var("x"), Some(Value::Int(9)));
        }
    }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core env::`

- [ ] **Step 3: Реализация**

    use crate::ast::Expr;
    use crate::value::Value;
    use std::collections::HashMap;

    #[derive(Clone)]
    pub struct UserFn { pub params: Vec<String>, pub body: Expr }

    pub struct Env {
        scopes: Vec<HashMap<String, Value>>,
        pub funcs: HashMap<String, UserFn>,
        pub aliases: HashMap<String, String>,
    }
    impl Env {
        pub fn new() -> Self { Env { scopes: vec![HashMap::new()], funcs: HashMap::new(), aliases: HashMap::new() } }
        pub fn push_scope(&mut self) { self.scopes.push(HashMap::new()); }
        pub fn pop_scope(&mut self) { if self.scopes.len() > 1 { self.scopes.pop(); } }
        pub fn set_var(&mut self, name: &str, v: Value) { self.scopes.last_mut().unwrap().insert(name.to_string(), v); }
        pub fn assign(&mut self, name: &str, v: Value) {
            for s in self.scopes.iter_mut().rev() {
                if s.contains_key(name) { s.insert(name.to_string(), v); return; }
            }
            self.scopes.last_mut().unwrap().insert(name.to_string(), v);
        }
        pub fn get_var(&self, name: &str) -> Option<Value> {
            for s in self.scopes.iter().rev() { if let Some(v) = s.get(name) { return Some(v.clone()); } }
            None
        }
    }
    impl Default for Env { fn default() -> Self { Self::new() } }

- [ ] **Step 4: Запустить.** Run: `cargo test -p calc-core env::` — PASS (3).
- [ ] **Step 5: Коммит.** `git add calc-core/src/env.rs calc-core/src/lib.rs && git commit -m "feat: Env — переменные (скоупы, assign), функции, псевдонимы"`

### Task 4.2: `Registry` + первый builtin

**Files:** Create: `calc-core/src/registry.rs`, `calc-core/src/builtins/mod.rs`; Modify: `lib.rs`

- [ ] **Step 1: Падающий тест**

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::value::Value;
        #[test]
        fn lookup_and_call() {
            let reg = Registry::with_builtins();
            let f = reg.get("Abs").expect("Abs зарегистрирована");
            assert_eq!(f(&[Value::Int(-3)], 0).unwrap(), Value::Int(3));
        }
        #[test]
        fn unknown_returns_none() {
            assert!(Registry::with_builtins().get("Нету").is_none());
        }
    }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core registry::`

- [ ] **Step 3: Реализация.** `registry.rs`:

    use crate::error::{Pos, Result};
    use crate::value::Value;
    use std::collections::HashMap;
    pub type BuiltinFn = fn(&[Value], Pos) -> Result<Value>;
    pub struct Registry { map: HashMap<&'static str, BuiltinFn> }
    impl Registry {
        pub fn new() -> Self { Registry { map: HashMap::new() } }
        pub fn register(&mut self, name: &'static str, f: BuiltinFn) { self.map.insert(name, f); }
        pub fn get(&self, name: &str) -> Option<BuiltinFn> { self.map.get(name).copied() }
        pub fn with_builtins() -> Self { let mut r = Registry::new(); crate::builtins::register_all(&mut r); r }
    }
    impl Default for Registry { fn default() -> Self { Self::new() } }

`builtins/mod.rs`:

    use crate::error::{CalcError, Result};
    use crate::registry::Registry;
    use crate::value::Value;
    pub fn register_all(r: &mut Registry) {
        r.register("Abs", |a, pos| {
            arity(a, 1, "Abs", pos)?;
            match &a[0] {
                Value::Int(i) => Ok(Value::Int(i.abs())),
                Value::Float(f) => Ok(Value::Float(f.abs())),
                _ => Err(CalcError::WrongParams { func: "Abs".into(), expected: "число".into(), got: a.len(), pos }),
            }
        });
    }
    pub(crate) fn arity(args: &[Value], n: usize, func: &str, pos: usize) -> Result<()> {
        if args.len() != n {
            return Err(CalcError::WrongParams { func: func.into(), expected: n.to_string(), got: args.len(), pos });
        }
        Ok(())
    }

- [ ] **Step 4: Запустить.** Run: `cargo test -p calc-core registry::` — PASS (2).
- [ ] **Step 5: Коммит.** `git add calc-core/src/registry.rs calc-core/src/builtins/mod.rs calc-core/src/lib.rs && git commit -m "feat: реестр встроенных функций + Abs"`

### Task 4.3: `Evaluator` (выражения)

**Files:** Create: `calc-core/src/eval.rs`; Modify: `lib.rs`

- [ ] **Step 1: Падающие тесты**

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::value::Value;
        fn eval_str(src: &str) -> Value {
            let toks = crate::lexer::tokenize(src).unwrap();
            let expr = crate::parser::Parser::new(toks).parse_single_expr().unwrap();
            Evaluator::new().eval_expr(&expr).unwrap()
        }
        #[test]
        fn arithmetic_precedence() {
            assert_eq!(eval_str("1 + 2 * 3"), Value::Int(7));
            assert_eq!(eval_str("2 ^ 3 ^ 2"), Value::Int(512));
            assert_eq!(eval_str("-2 ^ 2"), Value::Int(-4));
        }
        #[test]
        fn int_div_promotes() {
            assert_eq!(eval_str("7 / 2"), Value::Float(3.5));
            assert_eq!(eval_str("6 / 2"), Value::Int(3));
        }
        #[test]
        fn comparisons_and_logic() {
            assert_eq!(eval_str("2 < 3 && 3 <= 3"), Value::Bool(true));
            assert_eq!(eval_str("1 == 2 || 5 > 4"), Value::Bool(true));
        }
        #[test]
        fn builtin_call() { assert_eq!(eval_str("Abs(-9)"), Value::Int(9)); }
        #[test]
        fn division_by_zero_errors() {
            let toks = crate::lexer::tokenize("1/0").unwrap();
            let expr = crate::parser::Parser::new(toks).parse_single_expr().unwrap();
            assert!(matches!(Evaluator::new().eval_expr(&expr),
                Err(crate::error::CalcError::DivisionByZero { .. })));
        }
    }

Примечание: `eval_expr` берёт `&mut self` (присваивание меняет env). В тесте `Evaluator::new().eval_expr(...)` — временный `mut` через `let mut ev = ...`.

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core eval::`

- [ ] **Step 3: Реализация.**

    use crate::ast::{BinOp, Expr, Stmt, UnOp};
    use crate::env::Env;
    use crate::error::{CalcError, Result};
    use crate::registry::Registry;
    use crate::value::Value;

    pub struct Evaluator { pub env: Env, pub registry: Registry, loop_limit: u64 }
    impl Evaluator {
        pub fn new() -> Self { Evaluator { env: Env::new(), registry: Registry::with_builtins(), loop_limit: 1_000_000 } }
        pub fn set_loop_limit(&mut self, n: u64) { self.loop_limit = n; }

        pub fn eval_expr(&mut self, e: &Expr) -> Result<Value> {
            match e {
                Expr::Int(i, _) => Ok(Value::Int(*i)),
                Expr::Float(f, _) => Ok(Value::Float(*f)),
                Expr::Str(s, _) => Ok(Value::Str(s.clone())),
                Expr::Bool(b, _) => Ok(Value::Bool(*b)),
                Expr::Var(n, pos) => self.env.get_var(n)
                    .ok_or_else(|| CalcError::UnknownVariable { name: n.clone(), pos: *pos }),
                Expr::Assign { name, value, .. } => { let v = self.eval_expr(value)?; self.env.assign(name, v.clone()); Ok(v) }
                Expr::Unary { op, rhs, pos } => {
                    let v = self.eval_expr(rhs)?;
                    match op {
                        UnOp::Neg => match v { Value::Int(i) => Ok(Value::Int(-i)), Value::Float(f) => Ok(Value::Float(-f)),
                            _ => Err(CalcError::RangeError { msg: "унарный минус к не-числу".into(), pos: *pos }) },
                        UnOp::Not => Ok(Value::Bool(!v.truthy())),
                    }
                }
                Expr::Binary { op, lhs, rhs, pos } => {
                    let l = self.eval_expr(lhs)?; let r = self.eval_expr(rhs)?;
                    apply_binop(*op, l, r, *pos)
                }
                Expr::Call { name, args, pos } => {
                    let real = self.env.aliases.get(name).cloned().unwrap_or_else(|| name.clone());
                    let vals: Vec<Value> = args.iter().map(|a| self.eval_expr(a)).collect::<Result<_>>()?;
                    if let Some(uf) = self.env.funcs.get(&real).cloned() {
                        if uf.params.len() != vals.len() {
                            return Err(CalcError::WrongParams { func: real, expected: uf.params.len().to_string(), got: vals.len(), pos: *pos });
                        }
                        self.env.push_scope();
                        for (p, v) in uf.params.iter().zip(vals) { self.env.set_var(p, v); }
                        let out = self.eval_expr(&uf.body);
                        self.env.pop_scope();
                        out
                    } else if let Some(f) = self.registry.get(&real) {
                        f(&vals, *pos)
                    } else {
                        Err(CalcError::UnknownFunction { name: real, pos: *pos })
                    }
                }
            }
        }
    }
    impl Default for Evaluator { fn default() -> Self { Self::new() } }

    fn apply_binop(op: BinOp, l: Value, r: Value, pos: usize) -> Result<Value> {
        use BinOp::*;
        let both_int = matches!((&l, &r), (Value::Int(_), Value::Int(_)));
        match op {
            Add | Sub | Mul | Div | Rem | Pow => {
                if both_int {
                    let (a, b) = (l.as_int(pos)?, r.as_int(pos)?);
                    match op {
                        Add => Ok(Value::Int(a + b)),
                        Sub => Ok(Value::Int(a - b)),
                        Mul => Ok(Value::Int(a * b)),
                        Rem => { if b == 0 { return Err(CalcError::DivisionByZero { pos }); } Ok(Value::Int(a % b)) }
                        Div => { if b == 0 { return Err(CalcError::DivisionByZero { pos }); }
                                 if a % b == 0 { Ok(Value::Int(a / b)) } else { Ok(Value::Float(a as f64 / b as f64)) } }
                        Pow => { if b < 0 { Ok(Value::Float((a as f64).powf(b as f64))) }
                                 else { Ok(Value::Int((a as i128).pow(b as u32))) } }
                        _ => unreachable!(),
                    }
                } else {
                    let (a, b) = (l.as_float(pos)?, r.as_float(pos)?);
                    match op {
                        Add => Ok(Value::Float(a + b)), Sub => Ok(Value::Float(a - b)),
                        Mul => Ok(Value::Float(a * b)),
                        Div => { if b == 0.0 { return Err(CalcError::DivisionByZero { pos }); } Ok(Value::Float(a / b)) }
                        Rem => Ok(Value::Float(a % b)), Pow => Ok(Value::Float(a.powf(b))),
                        _ => unreachable!(),
                    }
                }
            }
            Eq | Ne | Lt | Le | Gt | Ge => {
                let res = match (&l, &r) {
                    (Value::Str(x), Value::Str(y)) => match op { Eq => x == y, Ne => x != y, Lt => x < y, Le => x <= y, Gt => x > y, Ge => x >= y, _ => unreachable!() },
                    _ => { let (a, b) = (l.as_float(pos)?, r.as_float(pos)?);
                           match op { Eq => a == b, Ne => a != b, Lt => a < b, Le => a <= b, Gt => a > b, Ge => a >= b, _ => unreachable!() } }
                };
                Ok(Value::Bool(res))
            }
            And => Ok(Value::Bool(l.truthy() && r.truthy())),
            Or => Ok(Value::Bool(l.truthy() || r.truthy())),
        }
    }

- [ ] **Step 4: Запустить.** Run: `cargo test -p calc-core eval::` — PASS (5).
- [ ] **Step 5: Коммит.** `git add calc-core/src/eval.rs calc-core/src/lib.rs && git commit -m "feat: вычислитель выражений"`

---

## Фаза 5 — Математика и тригонометрия

### Task 5.1: `builtins/math.rs`

**Files:** Create: `calc-core/src/builtins/math.rs`; Modify: `builtins/mod.rs`

- [ ] **Step 1: Падающие тесты**

    #[cfg(test)]
    mod tests {
        use crate::registry::Registry; use crate::value::Value;
        fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
        #[test]
        fn basic_math() {
            assert_eq!(call("Sqrt", &[Value::Float(9.0)]), Value::Float(3.0));
            assert_eq!(call("Min", &[Value::Int(3), Value::Int(1), Value::Int(2)]), Value::Int(1));
            assert_eq!(call("Max", &[Value::Int(3), Value::Int(1)]), Value::Int(3));
            assert_eq!(call("Floor", &[Value::Float(2.7)]), Value::Int(2));
            assert_eq!(call("Gcd", &[Value::Int(12), Value::Int(8)]), Value::Int(4));
            assert_eq!(call("Fact", &[Value::Int(5)]), Value::Int(120));
        }
        #[test]
        fn pi_constant() {
            match call("Pi", &[]) { Value::Float(x) => assert!((x - std::f64::consts::PI).abs() < 1e-12), _ => panic!() }
        }
    }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core builtins::math`

- [ ] **Step 3: Реализация.** `pub fn register(r: &mut Registry)` регистрирует:
`Abs Sqrt Sqr Pow Exp Ln Log Log2 Log10 Floor Ceil Round Trunc Frac Sign Min Max Gcd Lcm Fact Pi E Hypot`.
- `Floor/Ceil/Round/Trunc(x)` -> `Value::Int` (округлённое из `as_float`).
- `Sqrt/Exp/Ln/Log2/Log10/Frac(x)` -> `Value::Float`; `Log(x, base)` арность 2 -> `Float`.
- `Sqr(x)=x*x` (сохраняет тип); `Pow(a,b)` -> Int если оба Int и b>=0, иначе Float; `Hypot(a,b)` -> Float.
- `Sign(x)` -> Int (-1/0/1); `Min/Max` арность>=1, сохраняют Int если все Int (сравнение через as_float, возврат исходного элемента).
- `Gcd/Lcm(a,b)` через `as_int`; `Fact(n)` через `as_int` (n>=0, иначе RangeError).
- `Pi/E` арность 0 -> Float.
Перенести `Abs` из `mod.rs` в `math.rs`. В `builtins/mod.rs`:

    pub mod math;
    pub fn register_all(r: &mut Registry) { math::register(r); }

(и удалить прежнюю inline-регистрацию Abs; хелпер `arity` оставить в `mod.rs`, сделать `pub(crate)`).

- [ ] **Step 4: Запустить.** Run: `cargo test -p calc-core builtins::math` — PASS.
- [ ] **Step 5: Коммит.** `git add calc-core/src/builtins/ && git commit -m "feat: математические встроенные функции"`

### Task 5.2: `builtins/trig.rs`

**Files:** Create: `calc-core/src/builtins/trig.rs`; Modify: `builtins/mod.rs`

- [ ] **Step 1: Падающие тесты**

    #[cfg(test)]
    mod tests {
        use crate::registry::Registry; use crate::value::Value;
        fn callf(name: &str, x: f64) -> f64 {
            match Registry::with_builtins().get(name).unwrap()(&[Value::Float(x)], 0).unwrap() { Value::Float(v) => v, _ => panic!() }
        }
        #[test]
        fn trig_values() {
            assert!(callf("Sin", 0.0).abs() < 1e-12);
            assert!((callf("Cos", 0.0) - 1.0).abs() < 1e-12);
            assert!((callf("DegToRad", 180.0) - std::f64::consts::PI).abs() < 1e-12);
        }
    }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core builtins::trig`
- [ ] **Step 3: Реализация.** `register` для `Sin Cos Tan Cotan ArcSin ArcCos ArcTan SinH CosH TanH ArcSinH ArcCosH ArcTanH DegToRad RadToDeg` — арность 1, `as_float` -> `Float`. `Cotan(x)=1.0/tan(x)`, `DegToRad=x*PI/180`, `RadToDeg=x*180/PI`. Подключить `trig::register(r)`.
- [ ] **Step 4: Запустить.** PASS.
- [ ] **Step 5: Коммит.** `git add calc-core/src/builtins/ && git commit -m "feat: тригонометрические функции"`

---

## Фаза 6 — Системы счисления и биты

### Task 6.1: `builtins/bases.rs`

**Files:** Create: `calc-core/src/builtins/bases.rs`; Modify: `builtins/mod.rs`

- [ ] **Step 1: Падающие тесты**

    #[cfg(test)]
    mod tests {
        use crate::registry::Registry; use crate::value::Value;
        fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
        #[test]
        fn roman_roundtrip() {
            assert_eq!(call("IntToRoman", &[Value::Int(14)]), Value::Str("XIV".into()));
            assert_eq!(call("RomanToInt", &[Value::Str("MCMXCIV".into())]), Value::Int(1994));
        }
        #[test]
        fn base_roundtrip() {
            assert_eq!(call("IntToBase", &[Value::Int(255), Value::Int(16)]), Value::Str("FF".into()));
            assert_eq!(call("BaseToInt", &[Value::Str("FF".into()), Value::Int(16)]), Value::Int(255));
            assert_eq!(call("IntToHex", &[Value::Int(255)]), Value::Str("FF".into()));
            assert_eq!(call("BinToInt", &[Value::Str("1010".into())]), Value::Int(10));
        }
        #[test]
        fn base_out_of_range_errors() {
            assert!(Registry::with_builtins().get("IntToBase").unwrap()(&[Value::Int(5), Value::Int(99)], 0).is_err());
        }
    }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core builtins::bases`
- [ ] **Step 3: Реализация.** `IntToRoman RomanToInt IntToBase BaseToInt IntToHex HexToInt IntToBin BinToInt IntToOct OctToInt`.
- Алфавит `const ALPHA: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";`. `IntToBase(n, base)`: base в 2..=36 иначе `RangeError`; отрицательные с ведущим `-`.
- `BaseToInt(s, base)`: регистронезависимо; неверный символ -> `RangeError`.
- `IntToHex/Bin/Oct` = base 16/2/8; `HexToInt/BinToInt/OctToInt` симметрично.
- Римские 1..=3999 (пары значение/символ по убыванию), вне диапазона -> `RangeError`; `RomanToInt` — стандартный разбор с проверкой.
Подключить `bases::register(r)`.
- [ ] **Step 4: Запустить.** PASS.
- [ ] **Step 5: Property-тест**

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn base_roundtrip_prop(n in 0i128..1_000_000, base in 2i128..=36) {
            let s = call("IntToBase", &[Value::Int(n), Value::Int(base)]);
            prop_assert_eq!(call("BaseToInt", &[s, Value::Int(base)]), Value::Int(n));
        }
    }

Run: `cargo test -p calc-core builtins::bases` — PASS.
- [ ] **Step 6: Коммит.** `git add calc-core/src/builtins/ && git commit -m "feat: системы счисления и римские числа + property-тесты"`

### Task 6.2: `builtins/bits.rs`

**Files:** Create: `calc-core/src/builtins/bits.rs`; Modify: `builtins/mod.rs`

- [ ] **Step 1: Падающие тесты**

    #[cfg(test)]
    mod tests {
        use crate::registry::Registry; use crate::value::Value;
        fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
        #[test]
        fn bit_ops() {
            assert_eq!(call("And", &[Value::Int(0b1100), Value::Int(0b1010)]), Value::Int(0b1000));
            assert_eq!(call("Or", &[Value::Int(0b1100), Value::Int(0b1010)]), Value::Int(0b1110));
            assert_eq!(call("Xor", &[Value::Int(0b1100), Value::Int(0b1010)]), Value::Int(0b0110));
            assert_eq!(call("Shl", &[Value::Int(1), Value::Int(4)]), Value::Int(16));
            assert_eq!(call("BitSet", &[Value::Int(0), Value::Int(3)]), Value::Int(8));
            assert_eq!(call("BitTest", &[Value::Int(8), Value::Int(3)]), Value::Bool(true));
        }
    }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core builtins::bits`
- [ ] **Step 3: Реализация.** `And Or Xor Not Shl Shr BitTest BitSet BitClear BitToggle` через `as_int`. `Not` — `!v`. `BitTest(v,n)->Bool`; `BitSet/BitClear/BitToggle(v,n)->Int`. Подключить.
- [ ] **Step 4: Запустить.** PASS.
- [ ] **Step 5: Коммит.** `git add calc-core/src/builtins/ && git commit -m "feat: битовые операции"`

---

## Фаза 7 — Строки

### Task 7.1: `builtins/strings.rs`

**Files:** Create: `calc-core/src/builtins/strings.rs`; Modify: `builtins/mod.rs`

- [ ] **Step 1: Падающие тесты**

    #[cfg(test)]
    mod tests {
        use crate::registry::Registry; use crate::value::Value;
        fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
        fn s(x: &str) -> Value { Value::Str(x.into()) }
        #[test]
        fn string_ops() {
            assert_eq!(call("Length", &[s("abc")]), Value::Int(3));
            assert_eq!(call("Upper", &[s("aБв")]), s("AБВ"));
            assert_eq!(call("Lower", &[s("AБВ")]), s("aбв"));
            assert_eq!(call("Trim", &[s("  hi  ")]), s("hi"));
            assert_eq!(call("Replace", &[s("a-b-c"), s("-"), s("+")]), s("a+b+c"));
            assert_eq!(call("Copy", &[s("abcdef"), Value::Int(2), Value::Int(3)]), s("bcd"));
            assert_eq!(call("Pos", &[s("cd"), s("abcdef")]), Value::Int(3));
            assert_eq!(call("Concat", &[s("a"), s("b"), s("c")]), s("abc"));
            assert_eq!(call("Ord", &[s("A")]), Value::Int(65));
            assert_eq!(call("Chr", &[Value::Int(65)]), s("A"));
        }
    }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core builtins::strings`
- [ ] **Step 3: Реализация.** `Length Copy Pos Replace Trim TrimLeft TrimRight Upper Lower Ord Chr Concat Compare Reverse`.
- Юникод по символам (`chars().collect::<Vec<char>>()`). `Copy(s, start, len)` — 1-индексация. `Pos(sub, s)` — 1-индексация, 0 если нет. `Concat` — арность>=1, склейка `Display`. `Ord` — код первого символа (`u32`); `Chr(n)` -> символ. `Compare(a,b)` -> Int(-1/0/1).
- [ ] **Step 4: Запустить.** PASS.
- [ ] **Step 5: Коммит.** `git add calc-core/src/builtins/ && git commit -m "feat: строковые функции"`

---

## Фаза 8 — Хеши

### Task 8.1: `builtins/hash.rs`

**Files:** Create: `calc-core/src/builtins/hash.rs`; Modify: `builtins/mod.rs`, `calc-core/Cargo.toml`

- [ ] **Step 1: Зависимости** (в `[dependencies]`):

    digest = "0.10"
    md-5 = "0.10"
    sha1 = "0.10"
    sha2 = "0.10"
    sha3 = "0.10"
    ripemd = "0.10"
    crc32fast = "1"
    adler = "1"
    hex = "0.4"

- [ ] **Step 2: Падающие тесты (известные векторы)**

    #[cfg(test)]
    mod tests {
        use crate::registry::Registry; use crate::value::Value;
        fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
        fn s(x: &str) -> Value { Value::Str(x.into()) }
        #[test]
        fn known_hashes() {
            assert_eq!(call("Md5", &[s("")]), s("d41d8cd98f00b204e9800998ecf8427e"));
            assert_eq!(call("Sha1", &[s("abc")]), s("a9993e364706816aba3e25717850c26c9cd0d89d"));
            assert_eq!(call("Sha256", &[s("abc")]), s("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
            assert_eq!(call("Hash", &[s("sha256"), s("abc")]), s("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
        }
        #[test]
        fn unknown_algo_errors() {
            assert!(Registry::with_builtins().get("Hash").unwrap()(&[s("nope"), s("x")], 0).is_err());
        }
    }

- [ ] **Step 3: Запустить — падает.** Run: `cargo test -p calc-core builtins::hash`
- [ ] **Step 4: Реализация.**
- `fn hash_bytes(alg: &str, data: &[u8], pos: usize) -> Result<String>`: `match alg.to_lowercase().as_str()`:
  `"md5"` (md5::Md5), `"sha1"`, `"sha256"`/`"sha384"`/`"sha512"` (sha2), `"sha3_256"` (sha3), `"ripemd160"` — через `Digest::digest`, hex; `"crc32"` (crc32fast, `{:08x}`); `"adler32"` (adler); иначе `RangeError { msg: "неизвестный алгоритм хеша" }`.
- `register(r)`: `Hash(alg, data)` арность 2; обёртки-алиасы `Md5 Sha1 Sha256 Sha384 Sha512 Sha3 RipeMD160 Crc32 Adler32` арность 1 (фиксируют alg). Все аргументы-строки через `as_str`, байты `.as_bytes()`. Подключить.
> Примечание: `Gost`/`Tiger` — только при наличии стабильного крейта; иначе не включать (YAGNI) и указать в README.
- [ ] **Step 5: Запустить.** PASS.
- [ ] **Step 6: Коммит.** `git add calc-core/src/builtins/ calc-core/Cargo.toml Cargo.lock && git commit -m "feat: хеши через Hash() + алиасы, тест-векторы"`

---

## Фаза 9 — Шифры

### Task 9.1: `builtins/cipher.rs`

**Files:** Create: `calc-core/src/builtins/cipher.rs`; Modify: `builtins/mod.rs`, `calc-core/Cargo.toml`

- [ ] **Step 1: Зависимости** (в `[dependencies]`):

    aes = "0.8"
    cbc = { version = "0.1", features = ["alloc"] }
    cipher = { version = "0.4", features = ["block-padding", "alloc"] }
    base64 = "0.22"

(Доп. шифры blowfish/des/twofish/serpent/cast5/idea — добавлять по мере включения; в v1 обязателен AES, остальные — по возможности, отсутствующие задокументировать.)

- [ ] **Step 2: Падающий тест (round-trip)**

    #[cfg(test)]
    mod tests {
        use crate::registry::Registry; use crate::value::Value;
        fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
        fn s(x: &str) -> Value { Value::Str(x.into()) }
        #[test]
        fn aes_roundtrip() {
            let key = s("0123456789abcdef0123456789abcdef"); // 32 hex = 16 байт
            let enc = call("Encrypt", &[s("aes"), key.clone(), s("hello world")]);
            let dec = call("Decrypt", &[s("aes"), key, enc]);
            assert_eq!(dec, s("hello world"));
        }
        #[test]
        fn bad_key_errors() {
            assert!(Registry::with_builtins().get("Encrypt").unwrap()(&[s("aes"), s("short"), s("data")], 0).is_err());
        }
    }

- [ ] **Step 3: Запустить — падает.** Run: `cargo test -p calc-core builtins::cipher`
- [ ] **Step 4: Реализация.** AES-128-CBC + PKCS7. Ключ — hex (16/24/32 байта). IV — фиксированный нулевой (детерминизм v1; задокументировать как ограничение). Вывод шифра — hex.
- `Encrypt(alg, key_hex, plaintext) -> Str(hex)`; `Decrypt(alg, key_hex, hex) -> Str(plaintext)`.
- `match alg.to_lowercase()`: `"aes"|"rijndael"` -> AES; иначе `RangeError { msg: "неизвестный/неподдерживаемый шифр" }`.
- Неверная длина ключа/нечётный hex -> `WrongParams`/`RangeError`; ошибка расшифровки паддинга -> `RangeError`.
- Тип: `cbc::Encryptor<aes::Aes128>` / `cbc::Decryptor<aes::Aes128>` с `block_padding::Pkcs7`.
- [ ] **Step 5: Запустить.** PASS.
- [ ] **Step 6: Коммит.** `git add calc-core/src/builtins/ calc-core/Cargo.toml Cargo.lock && git commit -m "feat: шифрование Encrypt/Decrypt (AES-CBC)"`

---

## Фаза 10 — Дата/время и файлы

### Task 10.1: `builtins/fileio.rs`

**Files:** Create: `calc-core/src/builtins/fileio.rs`; Modify: `builtins/mod.rs`

- [ ] **Step 1: Падающий тест**

    #[cfg(test)]
    mod tests {
        use crate::registry::Registry; use crate::value::Value;
        fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
        fn s(x: &str) -> Value { Value::Str(x.into()) }
        #[test]
        fn write_then_read() {
            let path = std::env::temp_dir().join("calc_test_io.txt");
            let p = s(path.to_str().unwrap());
            call("StrToFile", &[p.clone(), s("данные")]);
            assert_eq!(call("FileToStr", &[p]), s("данные"));
        }
        #[test]
        fn missing_file_errors() {
            assert!(Registry::with_builtins().get("FileToStr").unwrap()(&[s("/nonexistent/xxx")], 0).is_err());
        }
    }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core builtins::fileio`
- [ ] **Step 3: Реализация.** `FileToStr(path)->Str`, `StrToFile(path,data)->Str(data)`, `AppendFile(path,data)`. Ошибки -> `CalcError::IoError { msg: e.to_string() }`. Подключить.
- [ ] **Step 4: Запустить.** PASS.
- [ ] **Step 5: Коммит.** `git add calc-core/src/builtins/ && git commit -m "feat: файловые функции"`

### Task 10.2: `builtins/datetime.rs`

**Files:** Create: `calc-core/src/builtins/datetime.rs`; Modify: `builtins/mod.rs`

- [ ] **Step 1: Падающий тест**

    #[cfg(test)]
    mod tests {
        use crate::registry::Registry; use crate::value::Value;
        #[test]
        fn now_returns_positive_float() {
            match Registry::with_builtins().get("Now").unwrap()(&[], 0).unwrap() {
                Value::Float(x) => assert!(x > 0.0), _ => panic!(),
            }
        }
    }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core builtins::datetime`
- [ ] **Step 3: Реализация.** `Now()->Float` (Unix-секунды через `SystemTime::now().duration_since(UNIX_EPOCH)`). `FormatFloat(x, digits)->Str` (фиксированная точность через `format!("{:.*}", d, x)`). Подключить.
- [ ] **Step 4: Запустить.** PASS.
- [ ] **Step 5: Коммит.** `git add calc-core/src/builtins/ && git commit -m "feat: минимальные дата/время (Now, FormatFloat)"`

---

## Фаза 11 — Инструкции: функции, псевдонимы

### Task 11.1: `Evaluator::run` и `eval_stmt`

**Files:** Modify: `calc-core/src/eval.rs`

- [ ] **Step 1: Падающие тесты** (добавить в `eval::tests`)

    fn run(src: &str) -> Value {
        let toks = crate::lexer::tokenize(src).unwrap();
        let stmts = crate::parser::Parser::new(toks).parse_program().unwrap();
        Evaluator::new().run(&stmts).unwrap()
    }
    #[test]
    fn user_function_and_alias() {
        assert_eq!(run("fn sq(x) = x*x; sq(5)"), Value::Int(25));
        assert_eq!(run("alias root = Sqrt; root(16.0)"), Value::Float(4.0));
    }
    #[test]
    fn variables_persist_across_stmts() {
        assert_eq!(run("x = 3; y = 4; Sqrt(x*x + y*y)"), Value::Float(5.0));
    }

(в `run` использовать `let mut ev = Evaluator::new();`)

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core eval::tests::user_function_and_alias`
- [ ] **Step 3: Реализация** (методы `Evaluator`):

    pub fn run(&mut self, stmts: &[Stmt]) -> Result<Value> {
        let mut last = Value::Bool(false);
        for s in stmts { last = self.eval_stmt(s)?; }
        Ok(last)
    }
    fn eval_stmt(&mut self, s: &Stmt) -> Result<Value> {
        match s {
            Stmt::Expr(e) => self.eval_expr(e),
            Stmt::FnDef { name, params, body, .. } => {
                self.env.funcs.insert(name.clone(), crate::env::UserFn { params: params.clone(), body: body.clone() });
                Ok(Value::Bool(true))
            }
            Stmt::Alias { name, target, .. } => { self.env.aliases.insert(name.clone(), target.clone()); Ok(Value::Bool(true)) }
            Stmt::While { .. } | Stmt::Repeat { .. } => self.eval_loop(s),
        }
    }
    // временная заглушка до Task 12.1:
    fn eval_loop(&mut self, _s: &Stmt) -> Result<Value> { Ok(Value::Bool(false)) }

- [ ] **Step 4: Запустить.** Run: `cargo test -p calc-core eval::` — PASS.
- [ ] **Step 5: Коммит.** `git add calc-core/src/eval.rs && git commit -m "feat: исполнение инструкций (fn, alias, последовательности)"`

---

## Фаза 12 — Циклы и предохранители

### Task 12.1: `eval_loop` (while/repeat + лимит)

**Files:** Modify: `calc-core/src/eval.rs`

- [ ] **Step 1: Падающие тесты**

    #[test]
    fn repeat_accumulates() { assert_eq!(run("s = 0; repeat 5 { s = s + 1 }; s"), Value::Int(5)); }
    #[test]
    fn while_counts_down() { assert_eq!(run("n = 3; c = 0; while (n > 0) { n = n - 1; c = c + 1 }; c"), Value::Int(3)); }
    #[test]
    fn infinite_loop_hits_limit() {
        let toks = crate::lexer::tokenize("while (1 == 1) { }").unwrap();
        let stmts = crate::parser::Parser::new(toks).parse_program().unwrap();
        let mut ev = Evaluator::new(); ev.set_loop_limit(1000);
        assert!(matches!(ev.run(&stmts), Err(crate::error::CalcError::LoopLimitExceeded { .. })));
    }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core eval::tests::repeat_accumulates`
- [ ] **Step 3: Реализация** (заменить заглушку `eval_loop`):

    fn eval_loop(&mut self, s: &Stmt) -> Result<Value> {
        let mut iters: u64 = 0;
        let mut last = Value::Bool(false);
        match s {
            Stmt::Repeat { count, body, pos } => {
                let n = self.eval_expr(count)?.as_int(*pos)?;
                for _ in 0..n.max(0) {
                    iters += 1;
                    if iters > self.loop_limit { return Err(CalcError::LoopLimitExceeded { limit: self.loop_limit }); }
                    self.env.push_scope();
                    for st in body { last = self.eval_stmt(st)?; }
                    self.env.pop_scope();
                }
            }
            Stmt::While { cond, body, .. } => {
                while self.eval_expr(cond)?.truthy() {
                    iters += 1;
                    if iters > self.loop_limit { return Err(CalcError::LoopLimitExceeded { limit: self.loop_limit }); }
                    self.env.push_scope();
                    for st in body { last = self.eval_stmt(st)?; }
                    self.env.pop_scope();
                }
            }
            _ => unreachable!(),
        }
        Ok(last)
    }

Семантика присваивания во внешнюю переменную обеспечена `env.assign` (Task 4.1) и его использованием в `Assign` (Task 4.3).

- [ ] **Step 4: Запустить.** Run: `cargo test -p calc-core eval::` — PASS.
- [ ] **Step 5: Коммит.** `git add calc-core/src/eval.rs && git commit -m "feat: циклы while/repeat с предохранителем итераций"`

---

## Фаза 13 — CLI

### Task 13.1: Фасад `Session`

**Files:** Modify: `calc-core/src/lib.rs`; Create: `calc-core/tests/api.rs`

- [ ] **Step 1: Падающий интеграционный тест** (`calc-core/tests/api.rs`)

    use calc_core::{Session, Value};
    #[test]
    fn session_eval_line() {
        let mut sess = Session::new();
        assert_eq!(sess.eval("2 + 2").unwrap(), Value::Int(4));
        sess.eval("x = 10").unwrap();
        assert_eq!(sess.eval("x * 2").unwrap(), Value::Int(20));
    }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core --test api`
- [ ] **Step 3: Реализация.** Полный `lib.rs`:

    pub mod error;
    pub mod value;
    pub mod lexer;
    pub mod ast;
    pub mod parser;
    pub mod env;
    pub mod registry;
    pub mod builtins;
    pub mod eval;

    pub use error::CalcError;
    pub use value::Value;

    pub struct Session { ev: eval::Evaluator }
    impl Session {
        pub fn new() -> Self { Session { ev: eval::Evaluator::new() } }
        pub fn eval(&mut self, src: &str) -> error::Result<Value> {
            let toks = lexer::tokenize(src)?;
            let stmts = parser::Parser::new(toks).parse_program()?;
            self.ev.run(&stmts)
        }
    }
    impl Default for Session { fn default() -> Self { Self::new() } }

- [ ] **Step 4: Запустить.** Run: `cargo test -p calc-core --test api` — PASS.
- [ ] **Step 5: Коммит.** `git add calc-core/src/lib.rs calc-core/tests/api.rs && git commit -m "feat: публичный фасад Session"`

### Task 13.2: `print` + перенос строки как разделитель

**Files:** Modify: `calc-core/src/builtins/mod.rs`, `lexer.rs`, `parser.rs`, `eval.rs` (тесты)

- [ ] **Step 1: Падающие тесты** (в `eval::tests`)

    #[test]
    fn newline_separates_statements() { assert_eq!(run("x = 1\nx + 1"), Value::Int(2)); }
    #[test]
    fn print_returns_value() { assert_eq!(run("print(5)"), Value::Int(5)); }

- [ ] **Step 2: Запустить — падает.** Run: `cargo test -p calc-core eval::tests::newline_separates_statements`
- [ ] **Step 3: Реализация.**
- Лексер уже эмитит `Newline` на `\n` (Task 2.1). Убедиться, что комментарий `#` поглощает до `\n`, а сам `\n` эмитится.
- Парсер `parse_program`/`parse_block`: `Semicolon` и `Newline` — эквивалентные разделители, пустые пропускать (уже заложено; проверить).
- `print`: арность>=1, печатает значения через пробел + `\n` в stdout, возвращает первый аргумент. Зарегистрировать в `register_all` (можно inline в `mod.rs` или в новом `io.rs`).
- [ ] **Step 4: Запустить всё ядро.** Run: `cargo test -p calc-core` — PASS.
- [ ] **Step 5: Коммит.** `git add calc-core/src/ && git commit -m "feat: print + перенос строки как разделитель инструкций"`

### Task 13.3: CLI (REPL, файл, разовое выражение)

**Files:** Modify: `calc-cli/src/main.rs`, `calc-cli/Cargo.toml`; Create: `calc-cli/tests/cli.rs`

- [ ] **Step 1: Зависимости.** `[dependencies]`:

    clap = { version = "4", features = ["derive"] }
    rustyline = "14"

`[dev-dependencies]`:

    assert_cmd = "2"
    predicates = "3"

- [ ] **Step 2: Падающий интеграционный тест** (`calc-cli/tests/cli.rs`)

    use assert_cmd::Command;
    use predicates::str::contains;
    #[test]
    fn one_shot_expression() {
        Command::cargo_bin("calc").unwrap().arg("2 + 2 * 3").assert().success().stdout(contains("8"));
    }
    #[test]
    fn run_script_file() {
        let f = std::env::temp_dir().join("calc_script.calc");
        std::fs::write(&f, "x = 6\nprint(x * 7)\n").unwrap();
        Command::cargo_bin("calc").unwrap().arg("--file").arg(&f).assert().success().stdout(contains("42"));
    }

- [ ] **Step 3: Запустить — падает.** Run: `cargo test -p calc-cli --test cli`
- [ ] **Step 4: Реализация** `main.rs`:
- clap derive: `struct Args { expr: Option<String>, #[arg(long)] file: Option<PathBuf> }`.
- `--file`: прочитать файл целиком, `Session::eval(text)` (перенос строки — разделитель).
- позиционный `expr`: `Session::eval(expr)`, напечатать результат; при ошибке — в stderr, `std::process::exit(1)`.
- иначе REPL (rustyline): цикл `readline("> ")`; на строке — `Session::eval`, печать `= {результат}`; ошибка — печать, продолжить; `Ctrl-D`/`Eof` — выход.
- [ ] **Step 5: Запустить.** Run: `cargo test -p calc-cli` — PASS.
- [ ] **Step 6: Коммит.** `git add calc-cli/ && git commit -m "feat: CLI — разовое выражение, запуск файла, REPL"`

---

## Фаза 14 — Документация и релиз

### Task 14.1: README и релизная сборка

**Files:** Create: `README.md`

- [ ] **Step 1: README** по проектному шаблону: описание; где работает (CLI, кроссплатформенно); архитектура (`calc-core`+`calc-cli`); стек; сборка (`cargo build --release`); использование (REPL / `--file` / разовое выражение с примерами); полный список встроенных функций по категориям; известные ограничения (нет bignum; фиксированный нулевой IV в шифрах v1; отсутствующие экзотические алгоритмы Gost/Tiger/Ice/MARS/Misty1/Haval/Cast256; минимальные дата/время); статус.

- [ ] **Step 2: Релизная сборка + весь тест-сьют**

Run:
    cargo build --release
    cargo test
Expected: `Finished release`, все тесты PASS.

- [ ] **Step 3: Дымовой прогон**

Run:
    ./target/release/calc "IntToRoman(2024)"
    ./target/release/calc 'Sha256("abc")'
Expected: `MMXXIV`; `ba7816bf...20015ad`.

- [ ] **Step 4: Коммит.** `git add README.md && git commit -m "docs: README (использование, список функций, ограничения)"`

---

## Self-review (покрытие спеки)

- **§2 Архитектура** -> Фазы 0–4, 13 (workspace, модули, фасад Session).
- **§3 Модель значений** (i128/f64/Str/Bool, приведения) -> Task 1.2; правила приведения -> Task 4.3 `apply_binop`.
- **§4 Синтаксис** (числа 4 баз, строки, комментарии, переменные, польз. функции, псевдонимы, циклы, print) -> Фазы 2,3,11,12; Task 13.2.
- **§5 Встроенные функции** (math, trig, bases+roman, bits, strings, datetime, file, Hash-диспетчер, Encrypt/Decrypt-диспетчер) -> Фазы 5–10.
- **§6 Ошибки** (категории, позиции, русские сообщения) -> Task 1.1; использование по всем builtins/eval.
- **§7 Тестирование** (юнит, тест-векторы крипты, парсер, property round-trip, интеграция) -> тесты в каждой задаче; property -> Task 6.1; интеграция -> Task 13.1–13.3.
- **§8 Дистрибуция** (release-бинарник, README) -> Фаза 14.
- **§9 Открытые уточнения** -> режим шифра/IV зафиксированы (Task 9.1: AES-CBC, нулевой IV, документировать); лимиты предохранителей -> Task 12.1 (`loop_limit`, default 1_000_000, настраивается `set_loop_limit`).

Осознанные компромиссы v1 (в README): экзотические хеши/шифры включаются только при наличии стабильного крейта; фиксированный нулевой IV детерминирован, но небезопасен для реального шифрования — отметить.
