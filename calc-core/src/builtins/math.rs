use super::arity;
use crate::error::{CalcError, Pos, Reason, Result};
use crate::registry::Registry;
use crate::value::Value;

fn to_i128_checked(f: f64, pos: Pos) -> Result<i128> {
    // 2^127 округляется вверх за i128::MAX, поэтому верхняя граница строгая.
    if f.is_finite() && f >= -(2f64.powi(127)) && f < 2f64.powi(127) {
        Ok(f as i128)
    } else {
        Err(CalcError::RangeError { msg: Reason::Overflow, pos })
    }
}

fn gcd_i128(a: i128, b: i128) -> Option<i128> {
    if a == i128::MIN || b == i128::MIN {
        return None;
    }
    let mut a = a.abs();
    let mut b = b.abs();
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    Some(a)
}

pub fn register(r: &mut Registry) {
    r.register("Abs", |a, pos| {
        arity(a, 1, "Abs", pos)?;
        match &a[0] {
            Value::Int(i) => i.checked_abs().map(Value::Int).ok_or(CalcError::RangeError { msg: Reason::Overflow, pos }),
            Value::Float(f) => Ok(Value::Float(f.abs())),
            _ => Err(CalcError::RangeError { msg: Reason::ExpectedNumber, pos }),
        }
    });
    r.register("Sqrt", |a, pos| {
        arity(a, 1, "Sqrt", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.sqrt()))
    });
    r.register("Exp", |a, pos| {
        arity(a, 1, "Exp", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.exp()))
    });
    r.register("Ln", |a, pos| {
        arity(a, 1, "Ln", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.ln()))
    });
    r.register("Log2", |a, pos| {
        arity(a, 1, "Log2", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.log2()))
    });
    r.register("Log10", |a, pos| {
        arity(a, 1, "Log10", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.log10()))
    });
    r.register("Frac", |a, pos| {
        arity(a, 1, "Frac", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.fract()))
    });
    r.register("Log", |a, pos| {
        arity(a, 2, "Log", pos)?;
        let x = a[0].as_float(pos)?;
        let base = a[1].as_float(pos)?;
        Ok(Value::Float(x.ln() / base.ln()))
    });
    r.register("Hypot", |a, pos| {
        arity(a, 2, "Hypot", pos)?;
        let x = a[0].as_float(pos)?;
        let y = a[1].as_float(pos)?;
        Ok(Value::Float(x.hypot(y)))
    });
    r.register("Sqr", |a, pos| {
        arity(a, 1, "Sqr", pos)?;
        match &a[0] {
            Value::Int(i) => i.checked_mul(*i).map(Value::Int).ok_or(CalcError::RangeError { msg: Reason::Overflow, pos }),
            Value::Float(f) => Ok(Value::Float(f * f)),
            _ => Err(CalcError::RangeError { msg: Reason::ExpectedNumber, pos }),
        }
    });
    r.register("Pow", |a, pos| {
        arity(a, 2, "Pow", pos)?;
        match (&a[0], &a[1]) {
            (Value::Int(base), Value::Int(exp)) if *exp >= 0 && *exp <= u32::MAX as i128 => {
                base.checked_pow(*exp as u32).map(Value::Int).ok_or(CalcError::RangeError { msg: Reason::Overflow, pos })
            }
            _ => {
                let base = a[0].as_float(pos)?;
                let exp = a[1].as_float(pos)?;
                Ok(Value::Float(base.powf(exp)))
            }
        }
    });
    r.register("Floor", |a, pos| {
        arity(a, 1, "Floor", pos)?;
        to_i128_checked(a[0].as_float(pos)?.floor(), pos).map(Value::Int)
    });
    r.register("Ceil", |a, pos| {
        arity(a, 1, "Ceil", pos)?;
        to_i128_checked(a[0].as_float(pos)?.ceil(), pos).map(Value::Int)
    });
    r.register("Round", |a, pos| {
        arity(a, 1, "Round", pos)?;
        to_i128_checked(a[0].as_float(pos)?.round(), pos).map(Value::Int)
    });
    r.register("Trunc", |a, pos| {
        arity(a, 1, "Trunc", pos)?;
        to_i128_checked(a[0].as_float(pos)?.trunc(), pos).map(Value::Int)
    });
    r.register("Sign", |a, pos| {
        arity(a, 1, "Sign", pos)?;
        let f = a[0].as_float(pos)?;
        let s = if f > 0.0 { 1 } else if f < 0.0 { -1 } else { 0 };
        Ok(Value::Int(s))
    });
    r.register("Min", |a, pos| {
        if a.is_empty() {
            return Err(CalcError::WrongParams { func: "Min".into(), expected: ">=1".into(), got: 0, pos });
        }
        let mut best = &a[0];
        let mut best_f = best.as_float(pos)?;
        for v in &a[1..] {
            let f = v.as_float(pos)?;
            if f < best_f {
                best = v;
                best_f = f;
            }
        }
        Ok(best.clone())
    });
    r.register("Max", |a, pos| {
        if a.is_empty() {
            return Err(CalcError::WrongParams { func: "Max".into(), expected: ">=1".into(), got: 0, pos });
        }
        let mut best = &a[0];
        let mut best_f = best.as_float(pos)?;
        for v in &a[1..] {
            let f = v.as_float(pos)?;
            if f > best_f {
                best = v;
                best_f = f;
            }
        }
        Ok(best.clone())
    });
    r.register("Gcd", |a, pos| {
        arity(a, 2, "Gcd", pos)?;
        let x = a[0].as_int(pos)?;
        let y = a[1].as_int(pos)?;
        let g = gcd_i128(x, y).ok_or(CalcError::RangeError { msg: Reason::Overflow, pos })?;
        Ok(Value::Int(g))
    });
    r.register("Lcm", |a, pos| {
        arity(a, 2, "Lcm", pos)?;
        let x = a[0].as_int(pos)?;
        let y = a[1].as_int(pos)?;
        if x == 0 || y == 0 {
            return Ok(Value::Int(0));
        }
        let g = gcd_i128(x, y).ok_or(CalcError::RangeError { msg: Reason::Overflow, pos })?;
        let xa = x.checked_abs().ok_or(CalcError::RangeError { msg: Reason::Overflow, pos })?;
        let ya = y.checked_abs().ok_or(CalcError::RangeError { msg: Reason::Overflow, pos })?;
        let result = (xa / g).checked_mul(ya).ok_or(CalcError::RangeError { msg: Reason::Overflow, pos })?;
        Ok(Value::Int(result))
    });
    r.register("Fact", |a, pos| {
        arity(a, 1, "Fact", pos)?;
        let n = a[0].as_int(pos)?;
        if n < 0 {
            return Err(CalcError::RangeError { msg: Reason::FactorialNegative, pos });
        }
        let mut result: i128 = 1;
        let mut i: i128 = 2;
        while i <= n {
            result = result.checked_mul(i).ok_or(CalcError::RangeError { msg: Reason::Overflow, pos })?;
            i += 1;
        }
        Ok(Value::Int(result))
    });
    r.register("Pi", |a, pos| {
        arity(a, 0, "Pi", pos)?;
        Ok(Value::Float(std::f64::consts::PI))
    });
    r.register("E", |a, pos| {
        arity(a, 0, "E", pos)?;
        Ok(Value::Float(std::f64::consts::E))
    });
}

#[cfg(test)]
mod tests {
    use crate::registry::Registry;
    use crate::value::Value;
    fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
    #[test]
    fn basic_math() {
        assert_eq!(call("Abs", &[Value::Int(-3)]), Value::Int(3));
        assert_eq!(call("Sqrt", &[Value::Float(9.0)]), Value::Float(3.0));
        assert_eq!(call("Min", &[Value::Int(3), Value::Int(1), Value::Int(2)]), Value::Int(1));
        assert_eq!(call("Max", &[Value::Int(3), Value::Int(1)]), Value::Int(3));
        assert_eq!(call("Floor", &[Value::Float(2.7)]), Value::Int(2));
        assert_eq!(call("Ceil", &[Value::Float(2.1)]), Value::Int(3));
        assert_eq!(call("Round", &[Value::Float(2.5)]), Value::Int(3));
        assert_eq!(call("Trunc", &[Value::Float(2.9)]), Value::Int(2));
        assert_eq!(call("Gcd", &[Value::Int(12), Value::Int(8)]), Value::Int(4));
        assert_eq!(call("Lcm", &[Value::Int(4), Value::Int(6)]), Value::Int(12));
        assert_eq!(call("Fact", &[Value::Int(5)]), Value::Int(120));
        assert_eq!(call("Sign", &[Value::Int(-7)]), Value::Int(-1));
        assert_eq!(call("Sqr", &[Value::Int(4)]), Value::Int(16));
    }
    #[test]
    fn constants_and_pow() {
        match call("Pi", &[]) { Value::Float(x) => assert!((x - std::f64::consts::PI).abs() < 1e-12), _ => panic!() }
        match call("E", &[]) { Value::Float(x) => assert!((x - std::f64::consts::E).abs() < 1e-12), _ => panic!() }
        assert_eq!(call("Pow", &[Value::Int(2), Value::Int(10)]), Value::Int(1024));
        match call("Hypot", &[Value::Float(3.0), Value::Float(4.0)]) { Value::Float(x) => assert!((x-5.0).abs()<1e-12), _=>panic!() }
    }
    #[test]
    fn fact_negative_and_overflow_error() {
        assert!(Registry::with_builtins().get("Fact").unwrap()(&[Value::Int(-1)], 0).is_err());
        assert!(Registry::with_builtins().get("Fact").unwrap()(&[Value::Int(100)], 0).is_err()); // overflows i128
    }
    #[test]
    fn no_panic_edge_cases() {
        fn err(name: &str, a: &[Value]) -> bool { Registry::with_builtins().get(name).unwrap()(a, 0).is_err() }
        assert!(err("Sqr", &[Value::Int(i128::MAX)]));            // overflow
        assert!(err("Pow", &[Value::Int(10), Value::Int(200)]));  // overflow
        assert!(err("Gcd", &[Value::Int(i128::MIN), Value::Int(5)]));
        assert!(err("Lcm", &[Value::Int(i128::MIN), Value::Int(5)]));
        assert!(err("Floor", &[Value::Float(1e40)]));             // out of i128 range
        assert!(err("Floor", &[Value::Float(2f64.powi(127))]));   // 2^127 округляется за i128::MAX
        assert!(err("Min", &[]));                                  // zero args
        assert!(err("Max", &[]));
    }
    #[test]
    fn more_math_values() {
        fn f(name: &str, a: &[Value]) -> f64 { match Registry::with_builtins().get(name).unwrap()(a,0).unwrap() { Value::Float(v)=>v, _=>panic!() } }
        assert!((f("Ln", &[Value::Float(std::f64::consts::E)]) - 1.0).abs() < 1e-12);
        assert!((f("Log2", &[Value::Float(8.0)]) - 3.0).abs() < 1e-12);
        assert!((f("Log10", &[Value::Float(1000.0)]) - 3.0).abs() < 1e-12);
        assert!((f("Log", &[Value::Float(81.0), Value::Float(3.0)]) - 4.0).abs() < 1e-12);
        assert!((f("Exp", &[Value::Float(0.0)]) - 1.0).abs() < 1e-12);
        assert!((f("Frac", &[Value::Float(2.25)]) - 0.25).abs() < 1e-12);
    }
}
