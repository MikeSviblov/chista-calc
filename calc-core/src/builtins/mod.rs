use crate::error::{CalcError, Result};
use crate::registry::Registry;
use crate::value::Value;
pub fn register_all(r: &mut Registry) {
    r.register("Abs", |a, pos| {
        arity(a, 1, "Abs", pos)?;
        match &a[0] {
            Value::Int(i) => i.checked_abs().map(Value::Int).ok_or(CalcError::RangeError { msg: "переполнение".into(), pos }),
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
