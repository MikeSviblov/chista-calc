use crate::ast::Expr;
use crate::value::Value;
use std::collections::HashMap;
#[derive(Clone)]
pub struct UserFn { pub params: Vec<String>, pub body: Expr }
pub struct Env {
    scopes: Vec<HashMap<String, Value>>,
    pub funcs: HashMap<String, UserFn>,
    pub aliases: HashMap<String, String>,
}
impl Env {
    pub fn new() -> Self { Env { scopes: vec![HashMap::new()], funcs: HashMap::new(), aliases: HashMap::new() } }
    pub fn push_scope(&mut self) { self.scopes.push(HashMap::new()); }
    pub fn pop_scope(&mut self) { if self.scopes.len() > 1 { self.scopes.pop(); } }
    pub fn set_var(&mut self, name: &str, v: Value) { self.scopes.last_mut().unwrap().insert(name.to_string(), v); }
    pub fn assign(&mut self, name: &str, v: Value) {
        for s in self.scopes.iter_mut().rev() { if s.contains_key(name) { s.insert(name.to_string(), v); return; } }
        self.scopes.last_mut().unwrap().insert(name.to_string(), v);
    }
    pub fn get_var(&self, name: &str) -> Option<Value> {
        for s in self.scopes.iter().rev() { if let Some(v) = s.get(name) { return Some(v.clone()); } }
        None
    }
    pub fn globals(&self) -> Vec<(String, Value)> {
        self.scopes.first().map(|s| s.iter().map(|(k, v)| (k.clone(), v.clone())).collect()).unwrap_or_default()
    }
}
impl Default for Env { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    #[test]
    fn set_get_variable() {
        let mut env = Env::new();
        env.set_var("x", Value::Int(5));
        assert_eq!(env.get_var("x"), Some(Value::Int(5)));
        assert_eq!(env.get_var("y"), None);
    }
    #[test]
    fn scopes_shadow_and_pop() {
        let mut env = Env::new();
        env.set_var("x", Value::Int(1));
        env.push_scope();
        env.set_var("x", Value::Int(2));
        assert_eq!(env.get_var("x"), Some(Value::Int(2)));
        env.pop_scope();
        assert_eq!(env.get_var("x"), Some(Value::Int(1)));
    }
    #[test]
    fn assign_updates_outer_scope() {
        let mut env = Env::new();
        env.set_var("x", Value::Int(1));
        env.push_scope();
        env.assign("x", Value::Int(9));
        env.pop_scope();
        assert_eq!(env.get_var("x"), Some(Value::Int(9)));
    }
    #[test]
    fn globals_lists_top_scope() {
        let mut env = Env::new();
        env.set_var("a", Value::Int(1));
        env.set_var("b", Value::Str("x".into()));
        let mut g = env.globals(); g.sort_by(|x,y| x.0.cmp(&y.0));
        assert_eq!(g, vec![("a".to_string(), Value::Int(1)), ("b".to_string(), Value::Str("x".into()))]);
    }
}
