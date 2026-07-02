use super::arity;
use crate::error::{CalcError, Result};
use crate::registry::Registry;
use crate::value::Value;

fn check_shift(n: i128, pos: usize) -> Result<u32> {
    if !(0..=127).contains(&n) {
        return Err(CalcError::RangeError { msg: "сдвиг должен быть в диапазоне 0..=127".into(), pos });
    }
    Ok(n as u32)
}

fn check_bit_index(n: i128, pos: usize) -> Result<u32> {
    if !(0..=127).contains(&n) {
        return Err(CalcError::RangeError { msg: "индекс бита должен быть в диапазоне 0..=127".into(), pos });
    }
    Ok(n as u32)
}

pub fn register(r: &mut Registry) {
    r.register("And", |a, pos| {
        arity(a, 2, "And", pos)?;
        Ok(Value::Int(a[0].as_int(pos)? & a[1].as_int(pos)?))
    });
    r.register("Or", |a, pos| {
        arity(a, 2, "Or", pos)?;
        Ok(Value::Int(a[0].as_int(pos)? | a[1].as_int(pos)?))
    });
    r.register("Xor", |a, pos| {
        arity(a, 2, "Xor", pos)?;
        Ok(Value::Int(a[0].as_int(pos)? ^ a[1].as_int(pos)?))
    });
    r.register("Not", |a, pos| {
        arity(a, 1, "Not", pos)?;
        Ok(Value::Int(!a[0].as_int(pos)?))
    });
    r.register("Shl", |a, pos| {
        arity(a, 2, "Shl", pos)?;
        let v = a[0].as_int(pos)?;
        let s = check_shift(a[1].as_int(pos)?, pos)?;
        Ok(Value::Int(v << s))
    });
    r.register("Shr", |a, pos| {
        arity(a, 2, "Shr", pos)?;
        let v = a[0].as_int(pos)?;
        let s = check_shift(a[1].as_int(pos)?, pos)?;
        Ok(Value::Int(v >> s))
    });
    r.register("BitTest", |a, pos| {
        arity(a, 2, "BitTest", pos)?;
        let v = a[0].as_int(pos)?;
        let n = check_bit_index(a[1].as_int(pos)?, pos)?;
        Ok(Value::Bool((v >> n) & 1 == 1))
    });
    r.register("BitSet", |a, pos| {
        arity(a, 2, "BitSet", pos)?;
        let v = a[0].as_int(pos)?;
        let n = check_bit_index(a[1].as_int(pos)?, pos)?;
        Ok(Value::Int(v | (1i128 << n)))
    });
    r.register("BitClear", |a, pos| {
        arity(a, 2, "BitClear", pos)?;
        let v = a[0].as_int(pos)?;
        let n = check_bit_index(a[1].as_int(pos)?, pos)?;
        Ok(Value::Int(v & !(1i128 << n)))
    });
    r.register("BitToggle", |a, pos| {
        arity(a, 2, "BitToggle", pos)?;
        let v = a[0].as_int(pos)?;
        let n = check_bit_index(a[1].as_int(pos)?, pos)?;
        Ok(Value::Int(v ^ (1i128 << n)))
    });
}

#[cfg(test)]
mod tests {
    use crate::registry::Registry;
    use crate::value::Value;
    fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
    fn err(name: &str, a: &[Value]) -> bool { Registry::with_builtins().get(name).unwrap()(a, 0).is_err() }
    #[test]
    fn bit_ops() {
        assert_eq!(call("And", &[Value::Int(0b1100), Value::Int(0b1010)]), Value::Int(0b1000));
        assert_eq!(call("Or",  &[Value::Int(0b1100), Value::Int(0b1010)]), Value::Int(0b1110));
        assert_eq!(call("Xor", &[Value::Int(0b1100), Value::Int(0b1010)]), Value::Int(0b0110));
        assert_eq!(call("Not", &[Value::Int(0)]), Value::Int(-1));
        assert_eq!(call("Shl", &[Value::Int(1), Value::Int(4)]), Value::Int(16));
        assert_eq!(call("Shr", &[Value::Int(16), Value::Int(4)]), Value::Int(1));
        assert_eq!(call("BitSet", &[Value::Int(0), Value::Int(3)]), Value::Int(8));
        assert_eq!(call("BitClear", &[Value::Int(8), Value::Int(3)]), Value::Int(0));
        assert_eq!(call("BitToggle", &[Value::Int(0), Value::Int(3)]), Value::Int(8));
        assert_eq!(call("BitTest", &[Value::Int(8), Value::Int(3)]), Value::Bool(true));
        assert_eq!(call("BitTest", &[Value::Int(8), Value::Int(2)]), Value::Bool(false));
    }
    #[test]
    fn shift_and_bit_index_bounds() {
        assert!(err("Shl", &[Value::Int(1), Value::Int(200)]));   // shift >= 128
        assert!(err("Shl", &[Value::Int(1), Value::Int(-1)]));    // negative shift
        assert!(err("BitSet", &[Value::Int(0), Value::Int(200)])); // bit index out of 0..=127
        assert!(err("BitTest", &[Value::Int(0), Value::Int(-1)]));
    }
}
