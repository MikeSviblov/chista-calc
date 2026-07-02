use super::arity;
use crate::error::CalcError;
use crate::registry::Registry;
use crate::value::Value;

pub fn register(r: &mut Registry) {
    r.register("FileToStr", |a, pos| {
        arity(a, 1, "FileToStr", pos)?;
        let path = a[0].as_str(pos)?;
        match std::fs::read_to_string(path) {
            Ok(data) => Ok(Value::Str(data)),
            Err(e) => Err(CalcError::IoError { msg: format!("не удалось прочитать '{path}': {e}") }),
        }
    });
    r.register("StrToFile", |a, pos| {
        arity(a, 2, "StrToFile", pos)?;
        let path = a[0].as_str(pos)?;
        let data = a[1].as_str(pos)?;
        match std::fs::write(path, data) {
            Ok(()) => Ok(Value::Str(data.to_string())),
            Err(e) => Err(CalcError::IoError { msg: format!("не удалось записать '{path}': {e}") }),
        }
    });
    r.register("AppendFile", |a, pos| {
        arity(a, 2, "AppendFile", pos)?;
        let path = a[0].as_str(pos)?;
        let data = a[1].as_str(pos)?;
        use std::io::Write;
        let result = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut f| f.write_all(data.as_bytes()));
        match result {
            Ok(()) => Ok(Value::Str(data.to_string())),
            Err(e) => Err(CalcError::IoError { msg: format!("не удалось дописать в '{path}': {e}") }),
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::registry::Registry; use crate::value::Value;
    fn call(name: &str, a: &[Value]) -> Value { Registry::with_builtins().get(name).unwrap()(a, 0).unwrap() }
    fn err(name: &str, a: &[Value]) -> bool { Registry::with_builtins().get(name).unwrap()(a, 0).is_err() }
    fn s(x: &str) -> Value { Value::Str(x.into()) }
    #[test]
    fn write_read_append() {
        let path = std::env::temp_dir().join(format!("calc_io_{}.txt", std::process::id()));
        let p = s(path.to_str().unwrap());
        call("StrToFile", &[p.clone(), s("данные")]);
        assert_eq!(call("FileToStr", std::slice::from_ref(&p)), s("данные"));
        call("AppendFile", &[p.clone(), s("+ещё")]);
        assert_eq!(call("FileToStr", std::slice::from_ref(&p)), s("данные+ещё"));
        let _ = std::fs::remove_file(&path);
    }
    #[test]
    fn missing_file_errors() {
        assert!(err("FileToStr", &[s("/nonexistent/definitely/missing_xyz")]));
    }
    #[test]
    fn wrong_types_error() {
        assert!(err("FileToStr", &[Value::Int(5)]));
        assert!(err("StrToFile", &[Value::Int(5), s("x")]));
    }
}
