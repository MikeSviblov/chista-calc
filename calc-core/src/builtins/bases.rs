use super::arity;
use crate::error::{CalcError, Pos, Result};
use crate::registry::Registry;
use crate::value::Value;

const ALPHA: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

const ROMAN_TABLE: &[(i128, &str)] = &[
    (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"), (100, "C"), (90, "XC"),
    (50, "L"), (40, "XL"), (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I"),
];

fn int_to_base(n: i128, base: i128, pos: Pos) -> Result<Value> {
    if !(2..=36).contains(&base) {
        return Err(CalcError::RangeError { msg: "база должна быть в диапазоне 2..=36".into(), pos });
    }
    if n == 0 {
        return Ok(Value::Str("0".into()));
    }
    let neg = n < 0;
    let mut m = n;
    let mut digits = Vec::new();
    while m != 0 {
        let rem = (m % base).unsigned_abs() as usize;
        digits.push(ALPHA[rem] as char);
        m /= base;
    }
    if neg {
        digits.push('-');
    }
    digits.reverse();
    Ok(Value::Str(digits.into_iter().collect()))
}

fn base_to_int(s: &str, base: i128, pos: Pos) -> Result<Value> {
    if !(2..=36).contains(&base) {
        return Err(CalcError::RangeError { msg: "база должна быть в диапазоне 2..=36".into(), pos });
    }
    let (neg, digits) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    if digits.is_empty() {
        return Err(CalcError::RangeError { msg: "пустая строка".into(), pos });
    }
    let mut value: i128 = 0;
    for c in digits.chars() {
        let cu = c.to_ascii_uppercase() as u8;
        let d = ALPHA[..base as usize]
            .iter()
            .position(|&b| b == cu)
            .ok_or(CalcError::RangeError { msg: format!("недопустимая цифра '{c}' для базы {base}"), pos })?
            as i128;
        value = value
            .checked_mul(base)
            .and_then(|v| v.checked_add(d))
            .ok_or(CalcError::RangeError { msg: "переполнение".into(), pos })?;
    }
    Ok(Value::Int(if neg { -value } else { value }))
}

fn int_to_roman(mut n: i128) -> String {
    let mut s = String::new();
    for &(v, sym) in ROMAN_TABLE {
        while n >= v {
            s.push_str(sym);
            n -= v;
        }
    }
    s
}

fn roman_value(c: char) -> Option<i128> {
    match c {
        'I' => Some(1),
        'V' => Some(5),
        'X' => Some(10),
        'L' => Some(50),
        'C' => Some(100),
        'D' => Some(500),
        'M' => Some(1000),
        _ => None,
    }
}

pub fn register(r: &mut Registry) {
    r.register("IntToBase", |a, pos| {
        arity(a, 2, "IntToBase", pos)?;
        let n = a[0].as_int(pos)?;
        let base = a[1].as_int(pos)?;
        int_to_base(n, base, pos)
    });
    r.register("BaseToInt", |a, pos| {
        arity(a, 2, "BaseToInt", pos)?;
        let s = a[0].as_str(pos)?;
        let base = a[1].as_int(pos)?;
        base_to_int(s, base, pos)
    });
    r.register("IntToHex", |a, pos| {
        arity(a, 1, "IntToHex", pos)?;
        int_to_base(a[0].as_int(pos)?, 16, pos)
    });
    r.register("HexToInt", |a, pos| {
        arity(a, 1, "HexToInt", pos)?;
        base_to_int(a[0].as_str(pos)?, 16, pos)
    });
    r.register("IntToBin", |a, pos| {
        arity(a, 1, "IntToBin", pos)?;
        int_to_base(a[0].as_int(pos)?, 2, pos)
    });
    r.register("BinToInt", |a, pos| {
        arity(a, 1, "BinToInt", pos)?;
        base_to_int(a[0].as_str(pos)?, 2, pos)
    });
    r.register("IntToOct", |a, pos| {
        arity(a, 1, "IntToOct", pos)?;
        int_to_base(a[0].as_int(pos)?, 8, pos)
    });
    r.register("OctToInt", |a, pos| {
        arity(a, 1, "OctToInt", pos)?;
        base_to_int(a[0].as_str(pos)?, 8, pos)
    });
    r.register("IntToRoman", |a, pos| {
        arity(a, 1, "IntToRoman", pos)?;
        let n = a[0].as_int(pos)?;
        if !(1..=3999).contains(&n) {
            return Err(CalcError::RangeError { msg: "число должно быть в диапазоне 1..=3999".into(), pos });
        }
        Ok(Value::Str(int_to_roman(n)))
    });
    r.register("RomanToInt", |a, pos| {
        arity(a, 1, "RomanToInt", pos)?;
        let s = a[0].as_str(pos)?.to_uppercase();
        let chars: Vec<char> = s.chars().collect();
        let mut total: i128 = 0;
        let mut i = 0;
        while i < chars.len() {
            let v = roman_value(chars[i])
                .ok_or(CalcError::RangeError { msg: format!("недопустимый символ '{}'", chars[i]), pos })?;
            if i + 1 < chars.len() {
                let next = roman_value(chars[i + 1])
                    .ok_or(CalcError::RangeError { msg: format!("недопустимый символ '{}'", chars[i + 1]), pos })?;
                if v < next {
                    total -= v;
                    i += 1;
                    continue;
                }
            }
            total += v;
            i += 1;
        }
        if !(1..=3999).contains(&total) || int_to_roman(total) != s {
            return Err(CalcError::RangeError { msg: "недопустимая римская запись".into(), pos });
        }
        Ok(Value::Int(total))
    });
}

#[cfg(test)]
mod tests {
    use crate::registry::Registry;
    use crate::value::Value;
    fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
    fn err(name: &str, a: &[Value]) -> bool { Registry::with_builtins().get(name).unwrap()(a, 0).is_err() }
    #[test]
    fn roman_roundtrip() {
        assert_eq!(call("IntToRoman", &[Value::Int(14)]), Value::Str("XIV".into()));
        assert_eq!(call("IntToRoman", &[Value::Int(2024)]), Value::Str("MMXXIV".into()));
        assert_eq!(call("RomanToInt", &[Value::Str("MCMXCIV".into())]), Value::Int(1994));
    }
    #[test]
    fn roman_out_of_range_and_invalid() {
        assert!(err("IntToRoman", &[Value::Int(0)]));
        assert!(err("IntToRoman", &[Value::Int(4000)]));
        assert!(err("RomanToInt", &[Value::Str("IIII".into())])); // invalid form
        assert!(err("RomanToInt", &[Value::Str("Q".into())])); // invalid char
    }
    #[test]
    fn base_roundtrip() {
        assert_eq!(call("IntToBase", &[Value::Int(255), Value::Int(16)]), Value::Str("FF".into()));
        assert_eq!(call("BaseToInt", &[Value::Str("FF".into()), Value::Int(16)]), Value::Int(255));
        assert_eq!(call("IntToBase", &[Value::Int(-255), Value::Int(16)]), Value::Str("-FF".into()));
        assert_eq!(call("BaseToInt", &[Value::Str("-FF".into()), Value::Int(16)]), Value::Int(-255));
        assert_eq!(call("IntToHex", &[Value::Int(255)]), Value::Str("FF".into()));
        assert_eq!(call("HexToInt", &[Value::Str("ff".into())]), Value::Int(255));
        assert_eq!(call("IntToBin", &[Value::Int(10)]), Value::Str("1010".into()));
        assert_eq!(call("BinToInt", &[Value::Str("1010".into())]), Value::Int(10));
        assert_eq!(call("IntToOct", &[Value::Int(8)]), Value::Str("10".into()));
        assert_eq!(call("OctToInt", &[Value::Str("17".into())]), Value::Int(15));
    }
    #[test]
    fn base_errors() {
        assert!(err("IntToBase", &[Value::Int(5), Value::Int(99)])); // base out of 2..=36
        assert!(err("IntToBase", &[Value::Int(5), Value::Int(1)]));
        assert!(err("BaseToInt", &[Value::Str("Z".into()), Value::Int(10)])); // digit invalid for base
        assert!(err("BaseToInt", &[Value::Str("".into()), Value::Int(16)])); // empty
    }
}

#[cfg(test)]
mod prop {
    use crate::registry::Registry;
    use crate::value::Value;
    use proptest::prelude::*;
    fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
    proptest! {
        #[test]
        fn base_roundtrip_prop(n in 0i128..1_000_000, base in 2i128..=36) {
            let s = call("IntToBase", &[Value::Int(n), Value::Int(base)]);
            prop_assert_eq!(call("BaseToInt", &[s, Value::Int(base)]), Value::Int(n));
        }
        #[test]
        fn roman_roundtrip_prop(n in 1i128..=3999) {
            let s = call("IntToRoman", &[Value::Int(n)]);
            prop_assert_eq!(call("RomanToInt", &[s]), Value::Int(n));
        }
    }
}
