# Чиста-блокнот (GUI) — план реализации

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Нативный desktop-«блокнот, который считает» (egui) поверх `calc-core`: живое построчное вычисление с результатами в правой колонке, панель переменных/функций, персистентность, файлы, подсветка.

**Architecture:** Новый workspace-крейт `calc-notepad` (eframe/egui). Вся логика «текст → построчные результаты» — в `document.rs`, поверх трёх добавлений в `calc-core` (перехват `print`, `run_document`/`eval_document`, снапшот состояния). Отрисовка (app/editor/panels) отделена от логики.

**Tech Stack:** Rust, eframe/egui (~0.29), directories (конфиг-путь), rfd (файловые диалоги), serde (настройки). Ядро — существующий `calc-core`.

**Спека:** `docs/superpowers/specs/2026-07-02-notepad-gui-design.md`

## Соглашения
- cargo prefix на этой машине: `source "$HOME/.cargo/env" && PATH="/usr/bin:$PATH" cargo ...`
- TDD там, где логика тестируема (ядро, `document.rs`). egui-рендер — реализация + `cargo build` + (опц.) запуск под Xvfb; настоящий визуальный тест — на реальном десктопе/через собранный exe.
- Строгий clippy: `cargo clippy --workspace --all-targets -- -D warnings` должен быть чист.
- Коммиты: `git -c user.name='Mike' -c user.email='mike@sviblov.com'`, пустая строка, затем `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`.
- Хук Write/Edit блокирует подстроку `eval` — файлы с ней создавать через bash heredoc.
- Ветка: `feature/notepad`.

---

## Фаза 1 — Добавления в `calc-core`

### Task 1.1: Перехват вывода `print` в буфер Evaluator

**Files:** Modify: `calc-core/src/eval.rs`, `calc-core/src/builtins/mod.rs`, `calc-core/src/lib.rs`

- [ ] **Step 1: Падающие тесты** (в `eval.rs`, в `#[cfg(test)] mod tests`)

```rust
    #[test]
    fn print_is_captured_not_stdout() {
        let toks = crate::lexer::tokenize("print(2+2); print(\"hi\")").unwrap();
        let stmts = crate::parser::Parser::new(toks).parse_program().unwrap();
        let mut ev = Evaluator::new();
        ev.run(&stmts).unwrap();
        assert_eq!(ev.take_output(), "4\nhi\n");
        assert_eq!(ev.take_output(), ""); // очищается после забора
    }
    #[test]
    fn print_still_returns_first_arg() {
        assert_eq!(run("print(5)"), Value::Int(5));
    }
    #[test]
    fn print_alias_captured() {
        let toks = crate::lexer::tokenize("alias p = print; p(7)").unwrap();
        let stmts = crate::parser::Parser::new(toks).parse_program().unwrap();
        let mut ev = Evaluator::new();
        ev.run(&stmts).unwrap();
        assert_eq!(ev.take_output(), "7\n");
    }
```

- [ ] **Step 2: Запустить — падает.** `... cargo test -p calc-core eval::tests::print_is_captured` — Expected: не компилируется (`take_output` нет).

- [ ] **Step 3: Реализация.**
В `Evaluator` добавить поле `output: String` (в struct и в `new()` инициализировать `String::new()`):
```rust
pub struct Evaluator { pub env: Env, pub registry: Registry, loop_limit: u64, call_depth: u64, call_limit: u64, expr_depth: u64, expr_limit: u64, output: String }
// new(): ..., output: String::new()
```
Добавить метод:
```rust
    pub fn take_output(&mut self) -> String { std::mem::take(&mut self.output) }
```
В `eval_expr_inner`, в арме `Expr::Call`, СРАЗУ после строки, где вычислен `let vals: Vec<Value> = ...?;` и определён `real`, вставить перехват `print` (до проверки `env.funcs`/`registry`):
```rust
                if real == "print" {
                    if vals.is_empty() {
                        return Err(CalcError::WrongParams { func: "print".into(), expected: "≥1".into(), got: 0, pos: *pos });
                    }
                    let line = vals.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ");
                    self.output.push_str(&line);
                    self.output.push('\n');
                    return Ok(vals[0].clone());
                }
```
В `builtins/mod.rs` УДАЛИТЬ регистрацию `print` (блок `r.register("print", ...)`), т.к. он теперь обрабатывается вычислителем. (Хелпер `arity` и остальные регистрации не трогать.)
В `lib.rs` `Session::eval` — после `run` слить вывод в stdout (сохранить поведение CLI):
```rust
    pub fn eval(&mut self, src: &str) -> error::Result<Value> {
        let toks = lexer::tokenize(src)?;
        let stmts = parser::Parser::new(toks).parse_program()?;
        let v = self.ev.run(&stmts)?;
        let out = self.ev.take_output();
        if !out.is_empty() { print!("{out}"); }
        Ok(v)
    }
```

- [ ] **Step 4: Запустить.** `... cargo test -p calc-core --lib` — Expected: все зелёные (вкл. новые 3). `... cargo test -p calc-cli` — Expected: `run_script_file` по-прежнему видит `42` (вывод слит в stdout из `Session::eval`).

- [ ] **Step 5: Коммит.** `git add calc-core/src/eval.rs calc-core/src/builtins/mod.rs calc-core/src/lib.rs && git commit -m "feat(core): перехват вывода print в буфер Evaluator"`

### Task 1.2: Помощники позиции инструкции/выражения

**Files:** Modify: `calc-core/src/ast.rs`

- [ ] **Step 1: Падающий тест** (в `ast.rs`)

```rust
    #[test]
    fn expr_and_stmt_pos() {
        let e = Expr::Binary { op: BinOp::Add, lhs: Box::new(Expr::Int(1,3)), rhs: Box::new(Expr::Int(2,7)), pos: 5 };
        assert_eq!(expr_pos(&e), 5);
        let s = Stmt::Expr(Expr::Int(9, 42));
        assert_eq!(stmt_pos(&s), 42);
        let d = Stmt::FnDef { name: "f".into(), params: vec![], body: Expr::Int(1,0), pos: 11 };
        assert_eq!(stmt_pos(&d), 11);
    }
```

- [ ] **Step 2: Запустить — падает.** `... cargo test -p calc-core ast::tests::expr_and_stmt_pos`

- [ ] **Step 3: Реализация** (в `ast.rs`, публичные функции):
```rust
pub fn expr_pos(e: &Expr) -> usize {
    match e {
        Expr::Int(_, p) | Expr::Float(_, p) | Expr::Str(_, p) | Expr::Bool(_, p) | Expr::Var(_, p) => *p,
        Expr::Unary { pos, .. } | Expr::Binary { pos, .. } | Expr::Call { pos, .. } | Expr::Assign { pos, .. } => *pos,
    }
}
pub fn stmt_pos(s: &Stmt) -> usize {
    match s {
        Stmt::Expr(e) => expr_pos(e),
        Stmt::FnDef { pos, .. } | Stmt::Alias { pos, .. } | Stmt::While { pos, .. } | Stmt::Repeat { pos, .. } => *pos,
    }
}
```

- [ ] **Step 4: Запустить.** `... cargo test -p calc-core ast::` — PASS.
- [ ] **Step 5: Коммит.** `git add calc-core/src/ast.rs && git commit -m "feat(core): помощники expr_pos/stmt_pos"`

### Task 1.3: Построчное выполнение документа

**Files:** Modify: `calc-core/src/eval.rs`

- [ ] **Step 1: Падающие тесты** (в `eval.rs`)

```rust
    #[test]
    fn run_document_per_statement() {
        use crate::eval::StmtOutcome;
        let toks = crate::lexer::tokenize("x = 5\nx * 2\nfn f(n) = n\n1/0\nx + 1").unwrap();
        let stmts = crate::parser::Parser::new(toks).parse_program().unwrap();
        let mut ev = Evaluator::new();
        let res = ev.run_document(&stmts);
        assert_eq!(res.len(), 5);
        assert!(matches!(res[0].1, StmtOutcome::Value(Value::Int(5))));   // x = 5
        assert!(matches!(res[1].1, StmtOutcome::Value(Value::Int(10))));  // x*2
        assert!(matches!(res[2].1, StmtOutcome::Defined));                // fn
        assert!(matches!(res[3].1, StmtOutcome::Error(_)));               // 1/0
        assert!(matches!(res[4].1, StmtOutcome::Value(Value::Int(6))));   // x+1 — состояние выжило
    }
```

- [ ] **Step 2: Запустить — падает.** `... cargo test -p calc-core eval::tests::run_document`

- [ ] **Step 3: Реализация** (в `eval.rs`):
```rust
#[derive(Debug)]
pub enum StmtOutcome { Value(Value), Defined, Error(CalcError) }
```
Метод на `impl Evaluator`:
```rust
    /// Выполняет инструкции по очереди, best-effort: ошибка одной не останавливает остальные.
    /// Возвращает (позиция инструкции, исход). fn/alias -> Defined; выражение/присваивание -> Value; ошибка -> Error.
    pub fn run_document(&mut self, stmts: &[Stmt]) -> Vec<(usize, StmtOutcome)> {
        let mut out = Vec::with_capacity(stmts.len());
        for s in stmts {
            let pos = crate::ast::stmt_pos(s);
            let outcome = match s {
                Stmt::FnDef { .. } | Stmt::Alias { .. } => match self.eval_stmt(s) {
                    Ok(_) => StmtOutcome::Defined,
                    Err(e) => StmtOutcome::Error(e),
                },
                _ => match self.eval_stmt(s) {
                    Ok(v) => StmtOutcome::Value(v),
                    Err(e) => StmtOutcome::Error(e),
                },
            };
            out.push((pos, outcome));
        }
        out
    }
```

- [ ] **Step 4: Запустить.** `... cargo test -p calc-core eval::` — PASS.
- [ ] **Step 5: Коммит.** `git add calc-core/src/eval.rs && git commit -m "feat(core): run_document — построчное best-effort выполнение"`

### Task 1.4: Снапшот состояния (переменные, имена функций)

**Files:** Modify: `calc-core/src/env.rs`, `calc-core/src/registry.rs`

- [ ] **Step 1: Падающие тесты**
В `env.rs`:
```rust
    #[test]
    fn globals_lists_top_scope() {
        let mut env = Env::new();
        env.set_var("a", Value::Int(1));
        env.set_var("b", Value::Str("x".into()));
        let mut g = env.globals();
        g.sort_by(|x,y| x.0.cmp(&y.0));
        assert_eq!(g, vec![("a".to_string(), Value::Int(1)), ("b".to_string(), Value::Str("x".into()))]);
    }
```
В `registry.rs`:
```rust
    #[test]
    fn names_contains_builtins() {
        let r = Registry::with_builtins();
        let names = r.names();
        assert!(names.contains(&"Sqrt"));
        assert!(names.contains(&"Sha256"));
    }
```

- [ ] **Step 2: Запустить — падает.** `... cargo test -p calc-core env::tests::globals_lists`; `... cargo test -p calc-core registry::tests::names_contains`

- [ ] **Step 3: Реализация.**
В `env.rs` (scopes приватный `Vec<HashMap<String,Value>>`) добавить:
```rust
    /// Переменные глобального (внешнего) скоупа.
    pub fn globals(&self) -> Vec<(String, Value)> {
        self.scopes.first().map(|m| m.iter().map(|(k,v)| (k.clone(), v.clone())).collect()).unwrap_or_default()
    }
```
В `registry.rs` (map: `HashMap<&'static str, BuiltinFn>`) добавить:
```rust
    pub fn names(&self) -> Vec<&'static str> { self.map.keys().copied().collect() }
```

- [ ] **Step 4: Запустить.** `... cargo test -p calc-core --lib` — PASS.
- [ ] **Step 5: Коммит.** `git add calc-core/src/env.rs calc-core/src/registry.rs && git commit -m "feat(core): снапшот переменных (Env::globals) и имён (Registry::names)"`

### Task 1.5: Фасад `Session::eval_document` + типы результата

**Files:** Modify: `calc-core/src/lib.rs`; Create: `calc-core/tests/document.rs`

- [ ] **Step 1: Падающий интеграционный тест** (`calc-core/tests/document.rs`)
```rust
use calc_core::{Session, DocLineOutcome};

#[test]
fn document_lines_and_output() {
    let mut s = Session::new();
    let d = s.eval_document("x = 10\nprint(x)\nx * 3\nбред$");
    // строки нумеруются с 1; последняя строка — синтаксическая ошибка
    assert_eq!(d.lines[0].line, 1);
    assert!(matches!(d.lines[0].outcome, DocLineOutcome::Value(_)));
    assert_eq!(d.output, "10\n");
    // при синтаксической ошибке документа — одна запись-ошибка с номером строки
    // (парсер не даёт частичный разбор), проверяем что вернулась ошибка
    let d2 = s.eval_document("2 +");
    assert!(matches!(d2.lines.last().unwrap().outcome, DocLineOutcome::Error(_)));
}

#[test]
fn document_state_is_fresh_each_call() {
    let mut s = Session::new();
    s.eval_document("y = 99");
    let d = s.eval_document("y"); // y не должна сохраниться между вызовами
    assert!(matches!(d.lines[0].outcome, DocLineOutcome::Error(_)));
}
```

- [ ] **Step 2: Запустить — падает.** `... cargo test -p calc-core --test document`

- [ ] **Step 3: Реализация** (в `lib.rs`):
Ре-экспорт и типы:
```rust
pub use eval::StmtOutcome;

#[derive(Debug)]
pub enum DocLineOutcome { Value(Value), Defined, Error(CalcError) }

#[derive(Debug)]
pub struct DocLine { pub line: usize, pub outcome: DocLineOutcome }

#[derive(Debug, Default)]
pub struct DocResult { pub lines: Vec<DocLine>, pub output: String }
```
Хелпер номера строки по char-позиции (лексер использует char-индексы):
```rust
fn char_pos_to_line(src: &str, pos: usize) -> usize {
    src.chars().take(pos).filter(|c| *c == '\n').count() + 1
}
fn err_pos(e: &CalcError) -> usize {
    use CalcError::*;
    match e {
        SyntaxError { pos, .. } | ParserError { pos, .. } | UnknownVariable { pos, .. }
        | UnknownFunction { pos, .. } | WrongParams { pos, .. } | DivisionByZero { pos }
        | RangeError { pos, .. } => *pos,
        IoError { .. } | LoopLimitExceeded { .. } | CallDepthExceeded { .. } | ExprTooDeep { .. } => 0,
    }
}
```
(Проверить точные варианты `CalcError` в `error.rs` и покрыть все — компилятор заставит.)
Метод:
```rust
    pub fn eval_document(&mut self, src: &str) -> DocResult {
        // Свежее состояние на каждый прогон документа.
        self.ev = eval::Evaluator::new();
        let toks = match lexer::tokenize(src) {
            Ok(t) => t,
            Err(e) => return DocResult { lines: vec![DocLine { line: char_pos_to_line(src, err_pos(&e)), outcome: DocLineOutcome::Error(e) }], output: String::new() },
        };
        let stmts = match parser::Parser::new(toks).parse_program() {
            Ok(s) => s,
            Err(e) => return DocResult { lines: vec![DocLine { line: char_pos_to_line(src, err_pos(&e)), outcome: DocLineOutcome::Error(e) }], output: String::new() },
        };
        let raw = self.ev.run_document(&stmts);
        let lines = raw.into_iter().map(|(pos, o)| {
            let outcome = match o {
                StmtOutcome::Value(v) => DocLineOutcome::Value(v),
                StmtOutcome::Defined => DocLineOutcome::Defined,
                StmtOutcome::Error(e) => DocLineOutcome::Error(e),
            };
            DocLine { line: char_pos_to_line(src, pos), outcome }
        }).collect();
        DocResult { lines, output: self.ev.take_output() }
    }
    pub fn variables(&self) -> Vec<(String, Value)> { self.ev.env.globals() }
    pub fn builtin_names(&self) -> Vec<String> {
        let mut n: Vec<String> = self.ev.registry.names().into_iter().map(|s| s.to_string()).collect();
        n.push("print".to_string());
        n.sort();
        n
    }
```
(`registry` — поле `Evaluator`; при необходимости сделать доступ через геттер `evaluator()` или прямое поле — `env`/`registry` у Evaluator публичны.)

- [ ] **Step 4: Запустить.** `... cargo test -p calc-core` — все зелёные. `... cargo clippy --workspace --all-targets -- -D warnings` — чисто.
- [ ] **Step 5: Коммит.** `git add calc-core/src/lib.rs calc-core/tests/document.rs && git commit -m "feat(core): Session::eval_document + variables/builtin_names"`

---

## Фаза 2 — Каркас `calc-notepad` и модель документа

### Task 2.1: Крейт `calc-notepad` в workspace

**Files:** Modify: `Cargo.toml` (workspace members); Create: `calc-notepad/Cargo.toml`, `calc-notepad/src/main.rs`

- [ ] **Step 1: Добавить в корневой `Cargo.toml`** член `"calc-notepad"` в `members`.

- [ ] **Step 2: `calc-notepad/Cargo.toml`:**
```toml
[package]
name = "calc-notepad"
edition.workspace = true
version.workspace = true

[[bin]]
name = "calc-notepad"
path = "src/main.rs"

[dependencies]
calc-core = { path = "../calc-core" }
eframe = "0.29"
egui = "0.29"
directories = "5"
rfd = "0.15"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```
(Если `0.29` не резолвится — взять актуальную 0.2x и синхронно eframe+egui.)

- [ ] **Step 3: Заглушка `calc-notepad/src/main.rs`:**
```rust
fn main() {
    println!("calc-notepad (заглушка)");
}
```

- [ ] **Step 4: Сборка.** `... cargo build -p calc-notepad` — Expected: `Finished` (скачает eframe/egui).
- [ ] **Step 5: Коммит.** `git add Cargo.toml calc-notepad Cargo.lock && git commit -m "chore(notepad): каркас крейта calc-notepad (eframe/egui)"`

### Task 2.2: Модель документа `document.rs` (без egui)

**Files:** Create: `calc-notepad/src/document.rs`; Modify: `calc-notepad/src/main.rs` (объявить `mod document;`)

- [ ] **Step 1: Падающие тесты** (в `document.rs`)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rows_map_results_to_line_numbers() {
        let doc = Document::evaluate("цена = 1990\nштук = 12\nцена * штук\nIntToRoman(2024)");
        // result_for_line(n) -> Option<String> текст результата для строки n (1-индекс)
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
```

- [ ] **Step 2: Запустить — падает.** `... cargo test -p calc-notepad document::`

- [ ] **Step 3: Реализация** (`document.rs`):
```rust
use calc_core::{DocLineOutcome, Session};
use std::collections::HashMap;

/// Результаты вычисления документа, разложенные по номерам строк для отрисовки.
pub struct Document {
    results: HashMap<usize, String>,   // строка -> текст результата ("= X")
    errors: std::collections::HashSet<usize>,
    pub output: String,
    pub variables: Vec<(String, String)>, // имя -> строковое значение (для панели)
}

impl Document {
    pub fn evaluate(src: &str) -> Self {
        let mut s = Session::new();
        let d = s.eval_document(src);
        let mut results = HashMap::new();
        let mut errors = std::collections::HashSet::new();
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
```
В `main.rs` добавить `mod document;`.

- [ ] **Step 4: Запустить.** `... cargo test -p calc-notepad` — PASS. `... cargo clippy -p calc-notepad --all-targets -- -D warnings` — чисто.
- [ ] **Step 5: Коммит.** `git add calc-notepad/src/document.rs calc-notepad/src/main.rs && git commit -m "feat(notepad): модель документа (текст -> построчные результаты)"`

---

## Фаза 3 — egui-приложение: редактор + живые результаты

### Task 3.1: Скелет окна eframe + редактор

**Files:** Create: `calc-notepad/src/app.rs`; Modify: `calc-notepad/src/main.rs`

- [ ] **Step 1: Реализация main + app** (нет юнит-теста — проверка сборкой/запуском).
`main.rs`:
```rust
mod app;
mod document;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]).with_title("Чиста-блокнот"),
        ..Default::default()
    };
    eframe::run_native("Чиста-блокнот", options, Box::new(|cc| Ok(Box::new(app::NotepadApp::new(cc)))))
}
```
`app.rs`:
```rust
use crate::document::Document;

pub struct NotepadApp {
    text: String,
    doc: Document,
}

impl NotepadApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let text = "цена = 1990\nштук = 12\nцена * штук\nIntToRoman(2024)\n".to_string();
        let doc = Document::evaluate(&text);
        NotepadApp { text, doc }
    }
}

impl eframe::App for NotepadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let resp = ui.add(
                egui::TextEdit::multiline(&mut self.text)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .code_editor(),
            );
            if resp.changed() {
                self.doc = Document::evaluate(&self.text);
            }
        });
    }
}
```

- [ ] **Step 2: Сборка.** `... cargo build -p calc-notepad` — Expected: `Finished`.
- [ ] **Step 3: (Опц.) дымовой запуск под Xvfb** (если доступен): `xvfb-run -a ./target/debug/calc-notepad` — окно создаётся без паники (закрыть). Если Xvfb нет — пропустить, проверка визуально на десктопе.
- [ ] **Step 4: Коммит.** `git add calc-notepad/src/app.rs calc-notepad/src/main.rs && git commit -m "feat(notepad): окно eframe + редактор с живым пересчётом"`

### Task 3.2: Колонка результатов, выровненная построчно

**Files:** Modify: `calc-notepad/src/app.rs`; Create: `calc-notepad/src/editor.rs`

Реализовать двухколоночную раскладку: редактор слева, результаты справа. Для выравнивания использовать постоянную высоту моноширинной строки.

- [ ] **Step 1: Реализация** `editor.rs` — функция отрисовки:
```rust
use crate::document::Document;
use egui::{Color32, RichText};

/// Рисует редактор (слева) и колонку результатов (справа), выровненную по строкам.
/// Возвращает true, если текст изменился.
pub fn code_with_results(ui: &mut egui::Ui, text: &mut String, doc: &Document, font_size: f32) -> bool {
    let mut changed = false;
    ui.horizontal_top(|ui| {
        // Левая колонка: редактор
        let editor_w = ui.available_width() * 0.72;
        let resp = ui.add_sized(
            [editor_w, ui.available_height()],
            egui::TextEdit::multiline(text)
                .font(egui::FontId::monospace(font_size))
                .desired_width(editor_w)
                .code_editor(),
        );
        changed = resp.changed();
        // Правая колонка: результаты, по одной на строку исходника
        ui.vertical(|ui| {
            let row_h = font_size * 1.30; // высота строки моноширинного текста в egui code_editor
            for (i, _line) in text.split('\n').enumerate() {
                let n = i + 1;
                let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), row_h), egui::Sense::hover());
                if let Some(res) = doc.result_for_line(n) {
                    let color = if doc.is_error_line(n) { Color32::from_rgb(0xd0, 0x40, 0x40) } else { Color32::from_rgb(0x30, 0x90, 0x30) };
                    let text = format!("= {res}");
                    let galley = ui.painter().layout_no_wrap(text.clone(), egui::FontId::monospace(font_size), color);
                    ui.painter().galley(rect.left_top(), galley, color);
                    // клик по результату — копирование
                    if ui.interact(rect, ui.id().with(("res", n)), egui::Sense::click()).clicked() {
                        ui.ctx().copy_text(res);
                    }
                }
            }
        });
    });
    changed
}
```
В `app.rs` `update` — заменить прямой TextEdit на:
```rust
        let font_size = 14.0;
        egui::CentralPanel::default().show(ctx, |ui| {
            if crate::editor::code_with_results(ui, &mut self.text, &self.doc, font_size) {
                self.doc = Document::evaluate(&self.text);
            }
        });
```
В `main.rs` добавить `mod editor;`.

> Примечание по выравниванию: `row_h` подобрать так, чтобы совпадало с межстрочным интервалом egui `code_editor` при данном размере шрифта; при рассинхроне — вычислять высоту строки из `ui.fonts(|f| f.row_height(&FontId::monospace(font_size)))`. Реализовать через `row_height`, а множитель 1.30 — фолбэк.

- [ ] **Step 2: Сборка + (опц.) Xvfb-запуск.** `... cargo build -p calc-notepad`; при наличии дисплея — визуально проверить, что результаты стоят напротив своих строк.
- [ ] **Step 3: clippy.** `... cargo clippy -p calc-notepad --all-targets -- -D warnings` — чисто.
- [ ] **Step 4: Коммит.** `git add calc-notepad/src/editor.rs calc-notepad/src/app.rs calc-notepad/src/main.rs && git commit -m "feat(notepad): колонка результатов с построчным выравниванием + копирование"`

---

## Фаза 4 — Панели: переменные/функции и вывод

### Task 4.1: Левая панель (переменные + справочник функций) и нижняя панель вывода

**Files:** Create: `calc-notepad/src/panels.rs`; Modify: `calc-notepad/src/app.rs`, `calc-notepad/src/main.rs`

- [ ] **Step 1: Реализация** `panels.rs`:
```rust
use crate::document::Document;

/// Левая боковая панель: текущие переменные и справочник встроенных функций.
/// Возвращает Some(name), если пользователь кликнул по имени функции (для вставки).
pub fn side_panel(ctx: &egui::Context, doc: &Document, builtins: &[String]) -> Option<String> {
    let mut insert = None;
    egui::SidePanel::left("side").resizable(true).default_width(180.0).show(ctx, |ui| {
        ui.heading("Переменные");
        if doc.variables.is_empty() {
            ui.weak("— нет —");
        } else {
            for (k, v) in &doc.variables {
                ui.monospace(format!("{k} = {v}"));
            }
        }
        ui.separator();
        egui::CollapsingHeader::new("Функции").default_open(false).show(ui, |ui| {
            for name in builtins {
                if ui.link(name).clicked() { insert = Some(name.clone()); }
            }
        });
    });
    insert
}

/// Нижняя панель вывода print/циклов.
pub fn output_panel(ctx: &egui::Context, output: &str) {
    egui::TopBottomPanel::bottom("output").resizable(true).default_height(120.0).show(ctx, |ui| {
        ui.label("Вывод:");
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(egui::TextEdit::multiline(&mut output.to_string()).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY).interactive(false));
        });
    });
}
```
В `app.rs`: в `NotepadApp` добавить поле `builtins: Vec<String>` (заполнить в `new` через `calc_core::Session::new().builtin_names()`). В `update` — вызвать `panels::side_panel` и `panels::output_panel` ДО `CentralPanel`; если вернулось `Some(name)`, добавить `name + "("` в конец `self.text` и пересчитать. Порядок в egui: SidePanel/TopBottomPanel объявляются до CentralPanel.
В `main.rs` — `mod panels;`.

- [ ] **Step 2: Сборка + clippy.** `... cargo build -p calc-notepad`; `... cargo clippy -p calc-notepad --all-targets -- -D warnings`.
- [ ] **Step 3: Коммит.** `git add calc-notepad/src/panels.rs calc-notepad/src/app.rs calc-notepad/src/main.rs && git commit -m "feat(notepad): панель переменных/функций и панель вывода"`

---

## Фаза 5 — Персистентность, настройки, файлы

### Task 5.1: Настройки и автосохранение текста

**Files:** Create: `calc-notepad/src/settings.rs`; Modify: `calc-notepad/src/app.rs`, `calc-notepad/src/main.rs`

- [ ] **Step 1: Падающий тест** (`settings.rs`) — сериализация round-trip:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_roundtrip() {
        let s = Settings { font_size: 16.0, always_on_top: true, text: "x=1".into() };
        let j = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&j).unwrap();
        assert_eq!(back.font_size, 16.0);
        assert!(back.always_on_top);
        assert_eq!(back.text, "x=1");
    }
}
```

- [ ] **Step 2: Запустить — падает.** `... cargo test -p calc-notepad settings::`

- [ ] **Step 3: Реализация** (`settings.rs`):
```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub font_size: f32,
    pub always_on_top: bool,
    pub text: String,
}
impl Default for Settings {
    fn default() -> Self { Settings { font_size: 14.0, always_on_top: false, text: String::new() } }
}
fn config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("irish", "green", "chista-notepad")
        .map(|d| d.config_dir().join("state.json"))
}
pub fn load() -> Settings {
    config_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
pub fn save(s: &Settings) {
    if let Some(p) = config_path() {
        if let Some(dir) = p.parent() { let _ = std::fs::create_dir_all(dir); }
        if let Ok(j) = serde_json::to_string_pretty(s) { let _ = std::fs::write(p, j); }
    }
}
```
В `app.rs`: `NotepadApp` получает поля `font_size: f32`, `always_on_top: bool`. В `new` — `let st = crate::settings::load();` инициализировать текст/настройки из него (если `st.text` пуст — оставить демо-текст). Реализовать `impl eframe::App` метод `fn save(&mut self, _storage: &mut dyn eframe::Storage)` НЕ используем; вместо этого сохранять по изменению и на выходе: в `update` при `resp.changed()` или периодически вызывать `settings::save(&Settings{...})`. Простейше: сохранять на каждое изменение текста и настроек (файл маленький). В `main.rs` — `mod settings;`.

- [ ] **Step 4: Запустить тест + сборка.** `... cargo test -p calc-notepad settings::` — PASS. `... cargo build -p calc-notepad`.
- [ ] **Step 5: Коммит.** `git add calc-notepad/src/settings.rs calc-notepad/src/app.rs calc-notepad/src/main.rs && git commit -m "feat(notepad): настройки + автосохранение текста (StoreText)"`

### Task 5.2: Тулбар (шрифт, поверх окон) + Открыть/Сохранить файл

**Files:** Modify: `calc-notepad/src/app.rs`, `calc-notepad/src/panels.rs`

- [ ] **Step 1: Реализация.** В `panels.rs` добавить тулбар:
```rust
pub struct ToolbarActions { pub open: bool, pub save: bool, pub font_delta: f32, pub toggle_on_top: bool }

pub fn toolbar(ctx: &egui::Context, always_on_top: bool) -> ToolbarActions {
    let mut a = ToolbarActions { open: false, save: false, font_delta: 0.0, toggle_on_top: false };
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("Открыть").clicked() { a.open = true; }
            if ui.button("Сохранить").clicked() { a.save = true; }
            ui.separator();
            if ui.button("Шрифт −").clicked() { a.font_delta = -1.0; }
            if ui.button("Шрифт +").clicked() { a.font_delta = 1.0; }
            ui.separator();
            let mut on_top = always_on_top;
            if ui.checkbox(&mut on_top, "поверх окон").changed() { a.toggle_on_top = true; }
        });
    });
    a
}
```
В `app.rs` `update` (в начале, до других панелей): вызвать `toolbar`, обработать:
- `font_delta` → `self.font_size = (self.font_size + d).clamp(8.0, 40.0)`;
- `toggle_on_top` → `self.always_on_top = !self.always_on_top;` и применить `ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(if self.always_on_top { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal }));`
- `open` → `rfd::FileDialog::new().add_filter("calc", &["calc","txt"]).pick_file()` → прочитать в `self.text`, пересчитать;
- `save` → `rfd::FileDialog::new().add_filter("calc", &["calc"]).save_file()` → записать `self.text`.
Применять `always_on_top` также при старте (в `new`/первом кадре).

- [ ] **Step 2: Сборка + clippy.** `... cargo build -p calc-notepad`; `... cargo clippy -p calc-notepad --all-targets -- -D warnings`.
- [ ] **Step 3: Коммит.** `git add calc-notepad/src/panels.rs calc-notepad/src/app.rs && git commit -m "feat(notepad): тулбар (шрифт, поверх окон) + Открыть/Сохранить .calc"`

---

## Фаза 6 — Подсветка синтаксиса

### Task 6.1: Layouter на лексере calc-core

**Files:** Create: `calc-notepad/src/highlight.rs`; Modify: `calc-notepad/src/editor.rs`, `calc-notepad/src/main.rs`

- [ ] **Step 1: Падающий тест** (в `highlight.rs`) — классификация токенов:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classifies_tokens() {
        // spans(src) -> Vec<(диапазон, вид)> по строке; проверяем виды
        let kinds: Vec<Kind> = spans("Sqrt(2) + 0x1F").into_iter().map(|s| s.kind).collect();
        assert!(kinds.contains(&Kind::Func));    // Sqrt перед (
        assert!(kinds.contains(&Kind::Number));  // 0x1F
        assert!(kinds.contains(&Kind::Op));      // +
    }
    #[test]
    fn bad_input_does_not_panic() {
        let _ = spans("\"незакрытая");
        let _ = spans("0xZZ");
    }
}
```

- [ ] **Step 2: Запустить — падает.** `... cargo test -p calc-notepad highlight::`

- [ ] **Step 3: Реализация** (`highlight.rs`): токенизировать через `calc_core::lexer::tokenize`; если ошибка — вернуть пустой список (текст покрасится дефолтно). Определить:
```rust
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind { Number, Str, Func, Ident, Op, Keyword, Comment }
pub struct Span { pub start: usize, pub end: usize, pub kind: Kind } // char-индексы

pub fn spans(src: &str) -> Vec<Span> { /* по токенам lexer + правило Func: Ident, за которым LParen */ }
```
Правила: `Int/Float` → Number; `Str` → Str; `Ident` перед `LParen` → Func, иначе Ident; операторы/пунктуация → Op; ключевые слова (`fn/alias/while/repeat/true/false`) → Keyword. Комментарии лексер не выдаёт токеном (съедает) — Comment можно опустить в v1 или вычислить отдельно (найти `#` вне строк). Для v1 — без Comment (YAGNI), убрать из enum если не используется.
В `editor.rs` — задать `layouter` для `TextEdit`:
```rust
let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
    let mut job = crate::highlight::layout_job(text, font_size);
    job.wrap.max_width = wrap_width;
    ui.fonts(|f| f.layout_job(job))
};
// .layouter(&mut layouter)
```
и функцию `highlight::layout_job(text, font_size) -> egui::text::LayoutJob`, красящую по `spans` (числа — синий, строки — зелёный, функции — фиолетовый, ключевые — оранжевый, операторы — серый, прочее — дефолт). Цвета — из `egui::Color32`.

- [ ] **Step 4: Запустить + сборка.** `... cargo test -p calc-notepad highlight::` — PASS. `... cargo build -p calc-notepad`. `... cargo clippy -p calc-notepad --all-targets -- -D warnings`.
- [ ] **Step 5: Коммит.** `git add calc-notepad/src/highlight.rs calc-notepad/src/editor.rs calc-notepad/src/main.rs && git commit -m "feat(notepad): подсветка синтаксиса на лексере calc-core"`

---

## Фаза 7 — CI и документация

### Task 7.1: Сборка GUI-бинарника в release.yml

**Files:** Modify: `.gitea/workflows/release.yml`

- [ ] **Step 1: Добавить сборку calc-notepad** в шаги release.yml после сборки CLI:
```yaml
      - name: Build Linux notepad
        run: cargo build --release -p calc-notepad
      - name: Build Windows notepad
        run: cargo build --release --target x86_64-pc-windows-gnu -p calc-notepad
```
и в шаг «Collect artifacts» добавить копирование:
```bash
          cp target/release/calc-notepad dist/calc-notepad-linux-x64
          cp target/x86_64-pc-windows-gnu/release/calc-notepad.exe dist/calc-notepad.exe
```
и в цикле публикации перечислить эти два файла.

> Примечание: eframe под windows-gnu может потребовать доп. системные пакеты в раннере (например `libgtk`-зависимости для `rfd` на Linux-сборке — на Linux `rfd` тянет GTK; для headless-сборки использовать `rfd` с features `xdg-portal` или собрать без GUI-диалогов в CI). На этапе выполнения: если Linux-сборка `calc-notepad` падает из-за GTK, переключить `rfd` на `default-features = false, features = ["xdg-portal", "tokio"]` или задокументировать необходимые apt-пакеты (`libgtk-3-dev`) и добавить их в шаг установки. Зафиксировать рабочий вариант.

- [ ] **Step 2: Локальная проверка кросс-сборки.** `... cargo build --release --target x86_64-pc-windows-gnu -p calc-notepad` — Expected: `Finished` (или зафиксировать и устранить проблемы линковки eframe/glow под mingw).
- [ ] **Step 3: Коммит.** `git add .gitea/workflows/release.yml && git commit -m "ci: сборка calc-notepad (Windows/Linux) в релиз"`

### Task 7.2: README — раздел про блокнот

**Files:** Modify: `README.md`

- [ ] **Step 1: Дополнить README** разделом «GUI-блокнот»: что это (живой inline-калькулятор), как запустить (`calc-notepad`), скриншот-описание раскладки, персистентность/файлы, что бинарник в релизах. Обновить архитектуру (три бинарника: `calc`, `calc-notepad`; ядро `calc-core`).
- [ ] **Step 2: Финальная проверка.** `... cargo build --release` (весь workspace), `... cargo test`, `... cargo clippy --workspace --all-targets -- -D warnings` — всё зелёное.
- [ ] **Step 3: Коммит.** `git add README.md && git commit -m "docs: README — GUI-блокнот"`

---

## Self-review (покрытие спеки)

- **§2 Крейты/архитектура** → Фаза 2 (крейт + document.rs), Фаза 3-6 (модули app/editor/panels/settings/highlight).
- **§3.1 Построчное вычисление** → Task 1.3 (run_document), Task 1.5 (eval_document/DocResult), Task 2.2 (Document).
- **§3.2 Перехват print** → Task 1.1.
- **§3.3 Снапшот состояния** → Task 1.4 (globals/names), Task 1.5 (variables/builtin_names).
- **§4 Раскладка** → Task 3.1/3.2 (редактор+результаты), Task 4.1 (панели), Task 5.2 (тулбар).
- **§5 Ошибки построчно** → Task 1.3 (best-effort), Task 2.2 (is_error_line), Task 3.2 (красный результат).
- **§6 Персистентность/файлы** → Task 5.1 (StoreText+настройки), Task 5.2 (Открыть/Сохранить, on-top, шрифт).
- **§7 Подсветка** → Фаза 6.
- **§8 Тестирование** → тесты ядра (Фаза 1), document.rs (2.2), settings/highlight (5.1/6.1); UI — сборка+ручной.
- **§9 Дистрибуция** → Task 7.1.
- **§10 Открытые уточнения** → выравнивание результатов (Task 3.2, через row_height), кросс-сборка eframe/rfd под mingw (Task 7.1, зафиксировать флаги/пакеты).

Осознанные риски (вынести в исполнение):
- Выравнивание правой колонки с TextEdit — если ручной рендер по `row_height` расходится, перейти на пер-строчные виджеты; проверить на первом визуальном прогоне.
- `rfd` на Linux тянет GTK — при проблемах CI переключить features или добавить apt-пакеты (Task 7.1).
- Версии eframe/egui — синхронно; если 0.29 API отличается (`copy_text`, `ViewportCommand::WindowLevel`, layouter-сигнатура) — свериться с докой этой версии при реализации.
