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
        .stdout("42\n");
    let _ = std::fs::remove_file(&f);
}

#[test]
fn file_flushes_print_before_error() {
    let f = std::env::temp_dir().join(format!("calc_flush_{}.calc", std::process::id()));
    std::fs::write(&f, "print(1)\nprint(2)\n1/0\n").unwrap();
    Command::cargo_bin("calc")
        .unwrap()
        .arg("--file")
        .arg(&f)
        .assert()
        .failure()
        .stdout("1\n2\n");
    let _ = std::fs::remove_file(&f);
}

#[test]
fn one_shot_echoes_value() {
    Command::cargo_bin("calc")
        .unwrap()
        .arg("IntToRoman(2024)")
        .assert()
        .success()
        .stdout("MMXXIV\n");
}
