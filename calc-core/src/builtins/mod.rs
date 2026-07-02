pub mod bases;
pub mod bits;
pub mod cipher;
pub mod datetime;
pub mod fileio;
pub mod hash;
pub mod math;
pub mod strings;
pub mod trig;
use crate::error::{CalcError, Result};
use crate::registry::Registry;
use crate::value::Value;
pub fn register_all(r: &mut Registry) {
    math::register(r);
    trig::register(r);
    bases::register(r);
    bits::register(r);
    strings::register(r);
    hash::register(r);
    cipher::register(r);
    fileio::register(r);
    datetime::register(r);
    r.register("print", |a, pos| {
        if a.is_empty() {
            return Err(CalcError::WrongParams { func: "print".into(), expected: "≥1".into(), got: 0, pos });
        }
        let line = a.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ");
        println!("{line}");
        Ok(a[0].clone())
    });
}
pub(crate) fn arity(args: &[Value], n: usize, func: &str, pos: usize) -> Result<()> {
    if args.len() != n {
        return Err(CalcError::WrongParams { func: func.into(), expected: n.to_string(), got: args.len(), pos });
    }
    Ok(())
}
