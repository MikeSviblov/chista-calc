use super::arity;
use crate::error::CalcError;
use crate::registry::Registry;
use crate::value::Value;

pub fn register(r: &mut Registry) {
    r.register("Length", |a, pos| {
        arity(a, 1, "Length", pos)?;
        let s = a[0].as_str(pos)?;
        Ok(Value::Int(s.chars().count() as i128))
    });
    r.register("Upper", |a, pos| {
        arity(a, 1, "Upper", pos)?;
        Ok(Value::Str(a[0].as_str(pos)?.to_uppercase()))
    });
    r.register("Lower", |a, pos| {
        arity(a, 1, "Lower", pos)?;
        Ok(Value::Str(a[0].as_str(pos)?.to_lowercase()))
    });
    r.register("Trim", |a, pos| {
        arity(a, 1, "Trim", pos)?;
        Ok(Value::Str(a[0].as_str(pos)?.trim().to_string()))
    });
    r.register("TrimLeft", |a, pos| {
        arity(a, 1, "TrimLeft", pos)?;
        Ok(Value::Str(a[0].as_str(pos)?.trim_start().to_string()))
    });
    r.register("TrimRight", |a, pos| {
        arity(a, 1, "TrimRight", pos)?;
        Ok(Value::Str(a[0].as_str(pos)?.trim_end().to_string()))
    });
    r.register("Replace", |a, pos| {
        arity(a, 3, "Replace", pos)?;
        let s = a[0].as_str(pos)?;
        let from = a[1].as_str(pos)?;
        let to = a[2].as_str(pos)?;
        if from.is_empty() {
            return Ok(Value::Str(s.to_string()));
        }
        Ok(Value::Str(s.replace(from, to)))
    });
    r.register("Copy", |a, pos| {
        arity(a, 3, "Copy", pos)?;
        let s = a[0].as_str(pos)?;
        let start = a[1].as_int(pos)?;
        let len = a[2].as_int(pos)?;
        if start < 1 {
            return Err(CalcError::RangeError { msg: "start должен быть >= 1".into(), pos });
        }
        if len < 0 {
            return Err(CalcError::RangeError { msg: "len должен быть >= 0".into(), pos });
        }
        let chars: Vec<char> = s.chars().collect();
        let char_count = chars.len() as i128;
        if start > char_count {
            return Ok(Value::Str(String::new()));
        }
        let start_idx = (start - 1) as usize;
        let end_idx = (start_idx as i128).saturating_add(len).min(char_count) as usize;
        Ok(Value::Str(chars[start_idx..end_idx].iter().collect()))
    });
    r.register("Pos", |a, pos| {
        arity(a, 2, "Pos", pos)?;
        let sub = a[0].as_str(pos)?;
        let s = a[1].as_str(pos)?;
        if sub.is_empty() {
            return Ok(Value::Int(0));
        }
        match s.find(sub) {
            Some(byte_idx) => Ok(Value::Int((s[..byte_idx].chars().count() + 1) as i128)),
            None => Ok(Value::Int(0)),
        }
    });
    r.register("Concat", |a, pos| {
        if a.is_empty() {
            return Err(CalcError::WrongParams { func: "Concat".into(), expected: ">=1".into(), got: 0, pos });
        }
        let mut result = String::new();
        for v in a {
            result.push_str(v.as_str(pos)?);
        }
        Ok(Value::Str(result))
    });
    r.register("Ord", |a, pos| {
        arity(a, 1, "Ord", pos)?;
        let s = a[0].as_str(pos)?;
        match s.chars().next() {
            Some(c) => Ok(Value::Int(c as i128)),
            None => Err(CalcError::RangeError { msg: "пустая строка".into(), pos }),
        }
    });
    r.register("Chr", |a, pos| {
        arity(a, 1, "Chr", pos)?;
        let n = a[0].as_int(pos)?;
        if n < 0 || n > u32::MAX as i128 {
            return Err(CalcError::RangeError { msg: "недопустимый код символа".into(), pos });
        }
        match char::from_u32(n as u32) {
            Some(c) => Ok(Value::Str(c.to_string())),
            None => Err(CalcError::RangeError { msg: "недопустимый код символа".into(), pos }),
        }
    });
    r.register("Compare", |a, pos| {
        arity(a, 2, "Compare", pos)?;
        let x = a[0].as_str(pos)?;
        let y = a[1].as_str(pos)?;
        let c = match x.cmp(y) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        };
        Ok(Value::Int(c))
    });
    r.register("Reverse", |a, pos| {
        arity(a, 1, "Reverse", pos)?;
        Ok(Value::Str(a[0].as_str(pos)?.chars().rev().collect()))
    });
}

#[cfg(test)]
mod tests {
    use crate::registry::Registry;
    use crate::value::Value;
    fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
    fn err(name: &str, a: &[Value]) -> bool { Registry::with_builtins().get(name).unwrap()(a, 0).is_err() }
    fn s(x: &str) -> Value { Value::Str(x.into()) }
    #[test]
    fn string_ops() {
        assert_eq!(call("Length", &[s("abc")]), Value::Int(3));
        assert_eq!(call("Length", &[s("абв")]), Value::Int(3));       // unicode by char
        assert_eq!(call("Upper", &[s("aБв")]), s("AБВ"));
        assert_eq!(call("Lower", &[s("AБВ")]), s("aбв"));
        assert_eq!(call("Trim", &[s("  hi  ")]), s("hi"));
        assert_eq!(call("TrimLeft", &[s("  hi  ")]), s("hi  "));
        assert_eq!(call("TrimRight", &[s("  hi  ")]), s("  hi"));
        assert_eq!(call("Replace", &[s("a-b-c"), s("-"), s("+")]), s("a+b+c"));
        assert_eq!(call("Copy", &[s("abcdef"), Value::Int(2), Value::Int(3)]), s("bcd")); // 1-indexed
        assert_eq!(call("Copy", &[s("абвгд"), Value::Int(2), Value::Int(2)]), s("бв")); // unicode
        assert_eq!(call("Pos", &[s("cd"), s("abcdef")]), Value::Int(3));  // 1-indexed
        assert_eq!(call("Pos", &[s("xy"), s("abcdef")]), Value::Int(0));  // not found
        assert_eq!(call("Concat", &[s("a"), s("b"), s("c")]), s("abc"));
        assert_eq!(call("Ord", &[s("A")]), Value::Int(65));
        assert_eq!(call("Ord", &[s("Я")]), Value::Int(1071));
        assert_eq!(call("Chr", &[Value::Int(65)]), s("A"));
        assert_eq!(call("Chr", &[Value::Int(1071)]), s("Я"));
        assert_eq!(call("Compare", &[s("a"), s("b")]), Value::Int(-1));
        assert_eq!(call("Compare", &[s("b"), s("a")]), Value::Int(1));
        assert_eq!(call("Compare", &[s("a"), s("a")]), Value::Int(0));
        assert_eq!(call("Reverse", &[s("abc")]), s("cba"));
        assert_eq!(call("Reverse", &[s("абв")]), s("вба"));
    }
    #[test]
    fn string_edge_cases() {
        assert!(err("Ord", &[s("")]));                       // empty → no first char
        assert!(err("Chr", &[Value::Int(-1)]));              // invalid code point
        assert!(err("Chr", &[Value::Int(0x110000)]));        // > max unicode
        assert_eq!(call("Copy", &[s("abc"), Value::Int(2), Value::Int(100)]), s("bc")); // len clamps
        assert_eq!(call("Copy", &[s("abc"), Value::Int(10), Value::Int(3)]), s(""));    // start past end → empty
        assert!(err("Copy", &[s("abc"), Value::Int(0), Value::Int(1)]));                // start < 1 invalid
    }
}
