use crate::ast::{BinOp, Expr, UnOp};
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
                        Value::Int(i) => i.checked_neg().map(Value::Int).ok_or(CalcError::RangeError { msg: "переполнение".into(), pos: *pos }),
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
                    Add => a.checked_add(b).map(Value::Int).ok_or(CalcError::RangeError { msg: "переполнение".into(), pos }),
                    Sub => a.checked_sub(b).map(Value::Int).ok_or(CalcError::RangeError { msg: "переполнение".into(), pos }),
                    Mul => a.checked_mul(b).map(Value::Int).ok_or(CalcError::RangeError { msg: "переполнение".into(), pos }),
                    Rem => { if b == 0 { return Err(CalcError::DivisionByZero { pos }); } Ok(Value::Int(a % b)) }
                    Div => {
                        if b == 0 { return Err(CalcError::DivisionByZero { pos }); }
                        if a % b == 0 { Ok(Value::Int(a / b)) } else { Ok(Value::Float(a as f64 / b as f64)) }
                    }
                    Pow => { if b < 0 { Ok(Value::Float((a as f64).powf(b as f64))) } else if b > u32::MAX as i128 { Err(CalcError::RangeError { msg: "слишком большая степень".into(), pos }) } else { a.checked_pow(b as u32).map(Value::Int).ok_or(CalcError::RangeError { msg: "переполнение".into(), pos }) } }
                    _ => unreachable!(),
                }
            } else {
                let (a, b) = (l.as_float(pos)?, r.as_float(pos)?);
                match op {
                    Add => Ok(Value::Float(a + b)), Sub => Ok(Value::Float(a - b)), Mul => Ok(Value::Float(a * b)),
                    Div => { if b == 0.0 { return Err(CalcError::DivisionByZero { pos }); } Ok(Value::Float(a / b)) }
                    Rem => Ok(Value::Float(a % b)), Pow => Ok(Value::Float(a.powf(b))),
                    _ => unreachable!(),
                }
            }
        }
        Eq | Ne | Lt | Le | Gt | Ge => {
            let res = match (&l, &r) {
                (Value::Str(x), Value::Str(y)) => match op { Eq => x==y, Ne => x!=y, Lt => x<y, Le => x<=y, Gt => x>y, Ge => x>=y, _ => unreachable!() },
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
}
