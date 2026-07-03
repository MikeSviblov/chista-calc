use super::arity;
use crate::error::{CalcError, Reason};
use crate::registry::Registry;
use crate::value::Value;

pub fn register(r: &mut Registry) {
    r.register("Now", |a, pos| {
        arity(a, 0, "Now", pos)?;
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => Ok(Value::Float(d.as_secs_f64())),
            Err(e) => Err(CalcError::IoError { msg: Reason::SystemTime(e.to_string()) }),
        }
    });
    r.register("FormatFloat", |a, pos| {
        arity(a, 2, "FormatFloat", pos)?;
        let x = a[0].as_float(pos)?;
        let digits = a[1].as_int(pos)?;
        if !(0..=100).contains(&digits) {
            return Err(CalcError::RangeError { msg: Reason::DigitsOutOfRange, pos });
        }
        Ok(Value::Str(format!("{:.*}", digits as usize, x)))
    });
}

#[cfg(test)]
mod tests {
    use crate::registry::Registry; use crate::value::Value;
    fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
    fn err(name: &str, a: &[Value]) -> bool { Registry::with_builtins().get(name).unwrap()(a, 0).is_err() }
    #[test]
    fn now_positive() {
        match call("Now", &[]) { Value::Float(x) => assert!(x > 1_600_000_000.0), _ => panic!() }
    }
    #[test]
    #[allow(clippy::approx_constant)]
    fn format_float() {
        assert_eq!(call("FormatFloat", &[Value::Float(3.14159), Value::Int(2)]), Value::Str("3.14".into()));
        assert_eq!(call("FormatFloat", &[Value::Int(5), Value::Int(3)]), Value::Str("5.000".into()));
    }
    #[test]
    fn format_float_bad_digits() {
        assert!(err("FormatFloat", &[Value::Float(1.0), Value::Int(-1)]));
        assert!(err("FormatFloat", &[Value::Float(1.0), Value::Int(1000)])); // absurd precision
    }
}
