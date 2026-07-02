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
            Expr::Var(n, pos) => self.env.get_var(n).ok_or_else(|| CalcError::UnknownVariable { name: n.clone(), pos: *pos }),
            Expr::Assign { name, value, .. } => { let v = self.eval_expr(value)?; self.env.assign(name, v.clone()); Ok(v) }
            Expr::Unary { op, rhs, pos } => {
                let v = self.eval_expr(rhs)?;
                match op {
                    UnOp::Neg => match v {
                        Value::Int(i) => checked_int(i.checked_neg(), *pos),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(CalcError::RangeError { msg: "унарный минус к не-числу".into(), pos: *pos }),
                    },
                    UnOp::Not => Ok(Value::Bool(!v.truthy())),
                }
            }
            Expr::Binary { op, lhs, rhs, pos } => { let l = self.eval_expr(lhs)?; let r = self.eval_expr(rhs)?; apply_binop(*op, l, r, *pos) }
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
    // temporary stub until Task 12.1:
    fn eval_loop(&mut self, _s: &Stmt) -> Result<Value> { Ok(Value::Bool(false)) }
}
impl Default for Evaluator { fn default() -> Self { Self::new() } }
fn checked_int(v: Option<i128>, pos: usize) -> Result<Value> {
    v.map(Value::Int).ok_or(CalcError::RangeError { msg: "переполнение".into(), pos })
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
                        None => Err(CalcError::RangeError { msg: "переполнение".into(), pos }),
                    },
                    Div => match a.checked_rem(b) {
                        None if b == 0 => Err(CalcError::DivisionByZero { pos }),
                        None => Err(CalcError::RangeError { msg: "переполнение".into(), pos }),
                        Some(0) => checked_int(a.checked_div(b), pos),
                        Some(_) => Ok(Value::Float(a as f64 / b as f64)),
                    },
                    Pow => { if b < 0 { Ok(Value::Float((a as f64).powf(b as f64))) } else if b > u32::MAX as i128 { Err(CalcError::RangeError { msg: "слишком большая степень".into(), pos }) } else { checked_int(a.checked_pow(b as u32), pos) } }
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
}
