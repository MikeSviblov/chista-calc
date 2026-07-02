use calc_core::{Session, Value};

#[test]
fn session_eval_line() {
    let mut sess = Session::new();
    assert_eq!(sess.eval("2 + 2").unwrap(), Value::Int(4));
    sess.eval("x = 10").unwrap();
    assert_eq!(sess.eval("x * 2").unwrap(), Value::Int(20));
}

#[test]
fn session_multiline() {
    let mut sess = Session::new();
    assert_eq!(
        sess.eval("a = 3\nb = 4\nSqrt(a*a + b*b)").unwrap(),
        Value::Float(5.0)
    );
}

#[test]
fn session_error_is_recoverable() {
    let mut sess = Session::new();
    assert!(sess.eval("1 +").is_err()); // syntax error
    assert_eq!(sess.eval("2 + 3").unwrap(), Value::Int(5)); // still works after
}
