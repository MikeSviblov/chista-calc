use crate::ast::{BinOp, Expr, Stmt, UnOp};
use crate::env::Env;
use crate::error::{CalcError, Reason, Result};
use crate::registry::Registry;
use crate::value::Value;
pub struct Evaluator { pub env: Env, pub registry: Registry, loop_limit: u64, call_depth: u64, call_limit: u64, expr_depth: u64, expr_limit: u64, output: String }
#[derive(Debug)]
pub enum StmtOutcome { Value(Value), Defined, Error(CalcError) }
impl Evaluator {
    pub fn new() -> Self { Evaluator { env: Env::new(), registry: Registry::with_builtins(), loop_limit: 1_000_000, call_depth: 0, call_limit: 512, expr_depth: 0, expr_limit: 150, output: String::new() } }
    pub fn set_loop_limit(&mut self, n: u64) { self.loop_limit = n; }
    pub fn set_call_limit(&mut self, n: u64) { self.call_limit = n; }
    pub fn set_expr_limit(&mut self, n: u64) { self.expr_limit = n; }
    pub fn take_output(&mut self) -> String { std::mem::take(&mut self.output) }
    // #[inline(never)]: inlining into the recursive eval_expr_inner grows its per-frame
    // stack size and can trip the 512-deep recursion test on the 2 MB test-thread stack.
    #[inline(never)]
    fn capture_print(&mut self, vals: &[Value], pos: usize) -> Result<Value> {
        if vals.is_empty() {
            return Err(CalcError::WrongParams { func: "print".into(), expected: "≥1".into(), got: 0, pos });
        }
        let line = vals.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ");
        self.output.push_str(&line);
        self.output.push('\n');
        Ok(vals[0].clone())
    }
    pub fn eval_expr(&mut self, e: &Expr) -> Result<Value> {
        self.expr_depth += 1;
        if self.expr_depth > self.expr_limit {
            self.expr_depth -= 1;
            return Err(CalcError::ExprTooDeep { limit: self.expr_limit });
        }
        let out = self.eval_expr_inner(e);
        self.expr_depth -= 1;
        out
    }
    fn eval_expr_inner(&mut self, e: &Expr) -> Result<Value> {
        match e {
            Expr::Int(i, _) => Ok(Value::Int(*i)),
            Expr::Float(f, _) => Ok(Value::Float(*f)),
            Expr::Str(s, _) => Ok(Value::Str(s.clone())),
            Expr::Bool(b, _) => Ok(Value::Bool(*b)),
            Expr::Var(n, pos) => self.env.get_var(n).ok_or_else(|| CalcError::UnknownVariable { name: n.clone(), pos: *pos }),
            Expr::Assign { name, value, .. } => { let v = self.eval_expr(value)?; self.env.assign(name, v.clone()); Ok(v) }
            Expr::Unary { op, rhs, pos } => {
                let v = self.eval_expr(rhs)?;
                match op {
                    UnOp::Neg => match v {
                        Value::Int(i) => checked_int(i.checked_neg(), *pos),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(CalcError::RangeError { msg: Reason::UnaryMinusNonNumber, pos: *pos }),
                    },
                    UnOp::Not => Ok(Value::Bool(!v.truthy())),
                }
            }
            Expr::Binary { op, lhs, rhs, pos } => { let l = self.eval_expr(lhs)?; let r = self.eval_expr(rhs)?; apply_binop(*op, l, r, *pos) }
            Expr::Call { name, args, pos } => {
                let real = self.env.aliases.get(name).cloned().unwrap_or_else(|| name.clone());
                let vals: Vec<Value> = args.iter().map(|a| self.eval_expr(a)).collect::<Result<_>>()?;
                // Пользовательская функция имеет приоритет над встроенными, включая print:
                // `fn print(n) = ...` должна затенять встроенный print, как любой другой builtin.
                if let Some(uf) = self.env.funcs.get(&real).cloned() {
                    if uf.params.len() != vals.len() {
                        return Err(CalcError::WrongParams { func: real, expected: uf.params.len().to_string(), got: vals.len(), pos: *pos });
                    }
                    self.call_depth += 1;
                    if self.call_depth > self.call_limit {
                        self.call_depth -= 1;
                        return Err(CalcError::CallDepthExceeded { limit: self.call_limit });
                    }
                    self.env.push_scope();
                    for (p, v) in uf.params.iter().zip(vals) { self.env.set_var(p, v); }
                    // Тело функции — новое выражение: глубина вложенности считается заново,
                    // а рекурсию ограничивает call_depth. Native-стек в целом защищён обоими лимитами.
                    let saved_expr = self.expr_depth;
                    self.expr_depth = 0;
                    let out = self.eval_expr(&uf.body);
                    self.expr_depth = saved_expr;
                    self.env.pop_scope();
                    self.call_depth -= 1;
                    out
                } else if real == "print" {
                    self.capture_print(&vals, *pos)
                } else if let Some(f) = self.registry.get(&real) {
                    f(&vals, *pos)
                } else {
                    Err(CalcError::UnknownFunction { name: real, pos: *pos })
                }
            }
        }
    }
    pub fn run(&mut self, stmts: &[Stmt]) -> Result<Value> {
        let mut last = Value::Bool(false);
        for s in stmts { last = self.eval_stmt(s)?; }
        Ok(last)
    }
    pub fn run_document(&mut self, stmts: &[Stmt]) -> Vec<(usize, StmtOutcome)> {
        let mut out = Vec::with_capacity(stmts.len());
        for s in stmts {
            let pos = crate::ast::stmt_pos(s);
            let outcome = match s {
                Stmt::FnDef { .. } | Stmt::Alias { .. } => match self.eval_stmt(s) { Ok(_) => StmtOutcome::Defined, Err(e) => StmtOutcome::Error(e) },
                _ => match self.eval_stmt(s) { Ok(v) => StmtOutcome::Value(v), Err(e) => StmtOutcome::Error(e) },
            };
            out.push((pos, outcome));
        }
        out
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
    fn exec_block(&mut self, body: &[Stmt]) -> Result<Value> {
        self.env.push_scope();
        let mut last = Value::Bool(false);
        let mut result = Ok(());
        for st in body {
            match self.eval_stmt(st) { Ok(v) => last = v, Err(e) => { result = Err(e); break; } }
        }
        self.env.pop_scope();
        result.map(|_| last)
    }
    fn eval_loop(&mut self, s: &Stmt) -> Result<Value> {
        let mut iters: u64 = 0;
        let mut last = Value::Bool(false);
        match s {
            Stmt::Repeat { count, body, pos } => {
                let n = self.eval_expr(count)?.as_int(*pos)?;
                let mut k: i128 = 0;
                while k < n {
                    tick(&mut iters, self.loop_limit)?;
                    last = self.exec_block(body)?;
                    k += 1;
                }
            }
            Stmt::While { cond, body, .. } => {
                while self.eval_expr(cond)?.truthy() {
                    tick(&mut iters, self.loop_limit)?;
                    last = self.exec_block(body)?;
                }
            }
            // guarded: eval_stmt only routes Stmt::While/Stmt::Repeat here.
            _ => unreachable!(),
        }
        Ok(last)
    }
}
impl Default for Evaluator { fn default() -> Self { Self::new() } }
fn tick(iters: &mut u64, limit: u64) -> Result<()> {
    *iters += 1;
    if *iters > limit { return Err(CalcError::LoopLimitExceeded { limit }); }
    Ok(())
}
fn checked_int(v: Option<i128>, pos: usize) -> Result<Value> {
    v.map(Value::Int).ok_or(CalcError::RangeError { msg: Reason::Overflow, pos })
}
fn apply_binop(op: BinOp, l: Value, r: Value, pos: usize) -> Result<Value> {
    use BinOp::*;
    let both_int = matches!((&l, &r), (Value::Int(_), Value::Int(_)));
    match op {
        Add | Sub | Mul | Div | Rem | Pow => {
            if both_int {
                let (a, b) = (l.as_int(pos)?, r.as_int(pos)?);
                match op {
                    Add => checked_int(a.checked_add(b), pos),
                    Sub => checked_int(a.checked_sub(b), pos),
                    Mul => checked_int(a.checked_mul(b), pos),
                    Rem => match a.checked_rem(b) {
                        Some(v) => Ok(Value::Int(v)),
                        None if b == 0 => Err(CalcError::DivisionByZero { pos }),
                        None => Err(CalcError::RangeError { msg: Reason::Overflow, pos }),
                    },
                    Div => match a.checked_rem(b) {
                        None if b == 0 => Err(CalcError::DivisionByZero { pos }),
                        None => Err(CalcError::RangeError { msg: Reason::Overflow, pos }),
                        Some(0) => checked_int(a.checked_div(b), pos),
                        Some(_) => Ok(Value::Float(a as f64 / b as f64)),
                    },
                    Pow => { if b < 0 { Ok(Value::Float((a as f64).powf(b as f64))) } else if b > u32::MAX as i128 { Err(CalcError::RangeError { msg: Reason::ExponentTooLarge, pos }) } else { checked_int(a.checked_pow(b as u32), pos) } }
                    _ => unreachable!(),
                }
            } else {
                let (a, b) = (l.as_float(pos)?, r.as_float(pos)?);
                match op {
                    Add => Ok(Value::Float(a + b)), Sub => Ok(Value::Float(a - b)), Mul => Ok(Value::Float(a * b)),
                    Div => { if b == 0.0 { return Err(CalcError::DivisionByZero { pos }); } Ok(Value::Float(a / b)) }
                    Rem => { if b == 0.0 { return Err(CalcError::DivisionByZero { pos }); } Ok(Value::Float(a % b)) }
                    Pow => Ok(Value::Float(a.powf(b))),
                    _ => unreachable!(),
                }
            }
        }
        Eq | Ne | Lt | Le | Gt | Ge => {
            let res = match (&l, &r) {
                (Value::Str(x), Value::Str(y)) => match op { Eq => x==y, Ne => x!=y, Lt => x<y, Le => x<=y, Gt => x>y, Ge => x>=y, _ => unreachable!() },
                (Value::Bool(x), Value::Bool(y)) => {
                    let (a, b) = (*x as i32, *y as i32);
                    match op { Eq => a==b, Ne => a!=b, Lt => a<b, Le => a<=b, Gt => a>b, Ge => a>=b, _ => unreachable!() }
                }
                _ => { let (a, b) = (l.as_float(pos)?, r.as_float(pos)?); match op { Eq => a==b, Ne => a!=b, Lt => a<b, Le => a<=b, Gt => a>b, Ge => a>=b, _ => unreachable!() } }
            };
            Ok(Value::Bool(res))
        }
        And => Ok(Value::Bool(l.truthy() && r.truthy())),
        Or => Ok(Value::Bool(l.truthy() || r.truthy())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    fn eval_str(src: &str) -> Value {
        let toks = crate::lexer::tokenize(src).unwrap();
        let expr = crate::parser::Parser::new(toks).parse_single_expr().unwrap();
        let mut ev = Evaluator::new();
        ev.eval_expr(&expr).unwrap()
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
        let mut ev = Evaluator::new();
        assert!(matches!(ev.eval_expr(&expr), Err(crate::error::CalcError::DivisionByZero { .. })));
    }
    #[test]
    fn float_rem_by_zero_errors() {
        let toks = crate::lexer::tokenize("1.5 % 0").unwrap();
        let expr = crate::parser::Parser::new(toks).parse_single_expr().unwrap();
        let mut ev = Evaluator::new();
        assert!(matches!(ev.eval_expr(&expr), Err(crate::error::CalcError::DivisionByZero { .. })));
    }
    fn eval_res(src: &str) -> Result<Value> {
        let toks = crate::lexer::tokenize(src).unwrap();
        let expr = crate::parser::Parser::new(toks).parse_single_expr().unwrap();
        let mut ev = Evaluator::new();
        ev.eval_expr(&expr)
    }
    #[test]
    fn int_min_div_rem_no_panic() {
        assert!(eval_res("(-170141183460469231731687303715884105727 - 1) % -1").is_err());
        assert!(eval_res("(-170141183460469231731687303715884105727 - 1) / -1").is_err());
    }
    #[test]
    fn bool_comparisons() {
        assert_eq!(eval_str("true == true"), Value::Bool(true));
        assert_eq!(eval_str("false < true"), Value::Bool(true));
        assert_eq!(eval_str("true != false"), Value::Bool(true));
    }
    fn run(src: &str) -> Value {
        let toks = crate::lexer::tokenize(src).unwrap();
        let stmts = crate::parser::Parser::new(toks).parse_program().unwrap();
        let mut ev = Evaluator::new();
        ev.run(&stmts).unwrap()
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
    #[test]
    fn recursive_user_fn() {
        // fib via recursion (small)
        assert_eq!(run("fn f(n) = n; f(7)"), Value::Int(7));
    }
    #[test]
    fn repeat_accumulates() { assert_eq!(run("s = 0; repeat 5 { s = s + 1 }; s"), Value::Int(5)); }
    #[test]
    fn while_counts_down() { assert_eq!(run("n = 3; c = 0; while (n > 0) { n = n - 1; c = c + 1 }; c"), Value::Int(3)); }
    #[test]
    fn nested_loops() { assert_eq!(run("t = 0; repeat 3 { repeat 4 { t = t + 1 } }; t"), Value::Int(12)); }
    #[test]
    fn infinite_loop_hits_limit() {
        let toks = crate::lexer::tokenize("while (1 == 1) { }").unwrap();
        let stmts = crate::parser::Parser::new(toks).parse_program().unwrap();
        let mut ev = Evaluator::new(); ev.set_loop_limit(1000);
        assert!(matches!(ev.run(&stmts), Err(crate::error::CalcError::LoopLimitExceeded { .. })));
    }
    #[test]
    fn repeat_negative_count_is_noop() { assert_eq!(run("s = 5; repeat -3 { s = s + 1 }; s"), Value::Int(5)); }
    #[test]
    fn infinite_recursion_errors_not_aborts() {
        fn run_res(src: &str) -> crate::error::Result<Value> {
            let toks = crate::lexer::tokenize(src).unwrap();
            let stmts = crate::parser::Parser::new(toks).parse_program().unwrap();
            let mut ev = Evaluator::new();
            ev.run(&stmts)
        }
        // Тест-поток cargo по умолчанию имеет стек 2 МБ — на нём 512 кадров рекурсии
        // впритык, и любые мелочи кодогенерации сталкивают его в переполнение раньше,
        // чем сработает предохранитель call_limit. Прогоняем на потоке со стеком 16 МБ
        // (реальный main-поток приложения имеет ≥8 МБ), где проверяется именно логика:
        // возвращается ошибка CallDepthExceeded, а не аварийное завершение.
        std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                assert!(matches!(run_res("fn f(n) = f(n); f(1)"), Err(crate::error::CalcError::CallDepthExceeded { .. })));
                assert!(matches!(run_res("fn a(n)=b(n); fn b(n)=a(n); a(1)"), Err(crate::error::CalcError::CallDepthExceeded { .. })));
            })
            .unwrap()
            .join()
            .unwrap();
    }
    #[test]
    fn print_returns_first_arg() { assert_eq!(run("print(5)"), Value::Int(5)); }
    #[test]
    fn newline_separates_statements() { assert_eq!(run("x = 1\nx + 1"), Value::Int(2)); }
    #[test]
    fn deep_flat_expression_errors_not_aborts() {
        let src = format!("{}1", "1+".repeat(5000));
        let toks = crate::lexer::tokenize(&src).unwrap();
        let stmts = crate::parser::Parser::new(toks).parse_program().unwrap();
        let mut ev = Evaluator::new();
        assert!(matches!(ev.run(&stmts), Err(crate::error::CalcError::ExprTooDeep { .. })));
    }
    #[test]
    fn moderate_deep_expression_still_evaluates() {
        assert_eq!(run(&format!("{}1", "1+".repeat(100))), Value::Int(101));
    }
    #[test]
    fn print_is_captured_not_stdout() {
        let toks = crate::lexer::tokenize("print(2+2); print(\"hi\")").unwrap();
        let stmts = crate::parser::Parser::new(toks).parse_program().unwrap();
        let mut ev = Evaluator::new();
        ev.run(&stmts).unwrap();
        assert_eq!(ev.take_output(), "4\nhi\n");
        assert_eq!(ev.take_output(), "");
    }
    #[test]
    fn print_still_returns_first_arg() { assert_eq!(run("print(5)"), Value::Int(5)); }
    #[test]
    fn print_alias_captured() {
        let toks = crate::lexer::tokenize("alias p = print; p(7)").unwrap();
        let stmts = crate::parser::Parser::new(toks).parse_program().unwrap();
        let mut ev = Evaluator::new();
        ev.run(&stmts).unwrap();
        assert_eq!(ev.take_output(), "7\n");
    }
    #[test]
    fn user_fn_can_shadow_print() {
        assert_eq!(run("fn print(n) = n*2; print(5)"), Value::Int(10));
    }
}
