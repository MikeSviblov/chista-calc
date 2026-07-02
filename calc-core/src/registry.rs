use crate::error::{Pos, Result};
use crate::value::Value;
use std::collections::HashMap;
pub type BuiltinFn = fn(&[Value], Pos) -> Result<Value>;
pub struct Registry { map: HashMap<&'static str, BuiltinFn> }
impl Registry {
    pub fn new() -> Self { Registry { map: HashMap::new() } }
    pub fn register(&mut self, name: &'static str, f: BuiltinFn) { self.map.insert(name, f); }
    pub fn get(&self, name: &str) -> Option<BuiltinFn> { self.map.get(name).copied() }
    pub fn with_builtins() -> Self { let mut r = Registry::new(); crate::builtins::register_all(&mut r); r }
}
impl Default for Registry { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    #[test]
    fn lookup_and_call() {
        let reg = Registry::with_builtins();
        let f = reg.get("Abs").expect("Abs зарегистрирована");
        assert_eq!(f(&[Value::Int(-3)], 0).unwrap(), Value::Int(3));
    }
    #[test]
    fn unknown_returns_none() { assert!(Registry::with_builtins().get("Нету").is_none()); }
    #[test]
    fn abs_i128_min_does_not_panic_and_errors() {
        let reg = Registry::with_builtins();
        let f = reg.get("Abs").unwrap();
        assert!(f(&[Value::Int(i128::MIN)], 0).is_err());
    }
}
