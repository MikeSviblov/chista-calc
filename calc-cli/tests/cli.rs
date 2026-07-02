use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn one_shot_expression() {
    Command::cargo_bin("calc")
        .unwrap()
        .arg("2 + 2 * 3")
        .assert()
        .success()
        .stdout(contains("8"));
}

#[test]
fn one_shot_error_exits_nonzero() {
    Command::cargo_bin("calc")
        .unwrap()
        .arg("1 +")
        .assert()
        .failure();
}

#[test]
fn run_script_file() {
    let f = std::env::temp_dir().join(format!("calc_script_{}.calc", std::process::id()));
    std::fs::write(&f, "x = 6\nprint(x * 7)\n").unwrap();
    Command::cargo_bin("calc")
        .unwrap()
        .arg("--file")
        .arg(&f)
        .assert()
        .success()
        .stdout(contains("42"));
    let _ = std::fs::remove_file(&f);
}
