use calc_core::{Session, DocLineOutcome};
#[test]
fn document_lines_and_output() {
    let mut s = Session::new();
    let d = s.eval_document("x = 10\nprint(x)\nx * 3\nНету");
    assert_eq!(d.lines[0].line, 1);
    assert!(matches!(d.lines[0].outcome, DocLineOutcome::Value(_)));
    assert_eq!(d.output, "10\n");
    let d2 = s.eval_document("2 +");
    assert!(matches!(d2.lines.last().unwrap().outcome, DocLineOutcome::Error(_)));
}
#[test]
fn document_state_is_fresh_each_call() {
    let mut s = Session::new();
    s.eval_document("y = 99");
    let d = s.eval_document("y");
    assert!(matches!(d.lines[0].outcome, DocLineOutcome::Error(_)));
}
