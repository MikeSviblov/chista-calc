use crate::error::{CalcError, Pos, Result};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value { Int(i128), Float(f64), Str(String), Bool(bool) }

impl Value {
    pub fn as_int(&self, pos: Pos) -> Result<i128> {
        match self {
            Value::Int(i) => Ok(*i),
            Value::Float(f) if f.fract() == 0.0
                && *f >= i128::MIN as f64 && *f <= i128::MAX as f64 =>
                Ok(*f as i128),
            _ => Err(CalcError::RangeError { msg: "ожидалось целое число".into(), pos }),
        }
    }
    pub fn as_float(&self, pos: Pos) -> Result<f64> {
        match self {
            Value::Int(i) => Ok(*i as f64),
            Value::Float(f) => Ok(*f),
            _ => Err(CalcError::RangeError { msg: "ожидалось число".into(), pos }),
        }
    }
    pub fn as_str(&self, pos: Pos) -> Result<&str> {
        match self {
            Value::Str(s) => Ok(s),
            _ => Err(CalcError::RangeError { msg: "ожидалась строка".into(), pos }),
        }
    }
    pub fn truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(x) => write!(f, "{x}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn display_variants() {
        assert_eq!(Value::Int(42).to_string(), "42");
        assert_eq!(Value::Float(1.5).to_string(), "1.5");
        assert_eq!(Value::Str("hi".into()).to_string(), "hi");
        assert_eq!(Value::Bool(true).to_string(), "true");
    }
    #[test]
    fn as_int_promotes_and_rejects() {
        assert_eq!(Value::Int(5).as_int(0).unwrap(), 5);
        assert_eq!(Value::Float(5.0).as_int(0).unwrap(), 5);
        assert!(Value::Float(5.5).as_int(0).is_err());
        assert!(Value::Str("x".into()).as_int(0).is_err());
    }
    #[test]
    fn as_float_promotes_int() {
        assert_eq!(Value::Int(3).as_float(0).unwrap(), 3.0);
        assert_eq!(Value::Float(3.5).as_float(0).unwrap(), 3.5);
    }
    #[test]
    fn as_int_rejects_out_of_range_float() {
        assert!(Value::Float(1e40).as_int(0).is_err());
        assert!(Value::Float(-1e40).as_int(0).is_err());
    }
    #[test]
    fn truthy_semantics() {
        assert!(Value::Bool(true).truthy());
        assert!(!Value::Bool(false).truthy());
        assert!(!Value::Int(0).truthy());
        assert!(Value::Int(3).truthy());
        assert!(!Value::Float(0.0).truthy());
        assert!(Value::Str("x".into()).truthy());
        assert!(!Value::Str("".into()).truthy());
    }
    #[test]
    fn as_str_ok_and_err() {
        assert_eq!(Value::Str("hi".into()).as_str(0).unwrap(), "hi");
        assert!(Value::Int(1).as_str(0).is_err());
    }
}
