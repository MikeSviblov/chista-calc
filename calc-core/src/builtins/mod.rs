pub mod math;
pub mod trig;
use crate::error::{CalcError, Result};
use crate::registry::Registry;
use crate::value::Value;
pub fn register_all(r: &mut Registry) {
    math::register(r);
    trig::register(r);
}
pub(crate) fn arity(args: &[Value], n: usize, func: &str, pos: usize) -> Result<()> {
    if args.len() != n {
        return Err(CalcError::WrongParams { func: func.into(), expected: n.to_string(), got: args.len(), pos });
    }
    Ok(())
}
