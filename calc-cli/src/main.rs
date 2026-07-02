use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "calc", version, about = "Калькулятор — выражения, переменные, функции, циклы, хеши, шифры")]
struct Args {
    /// Выражение для однократного вычисления
    expr: Option<String>,
    /// Выполнить скрипт из файла
    #[arg(long)]
    file: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let mut sess = calc_core::Session::new();
    if let Some(path) = args.file {
        match std::fs::read_to_string(&path) {
            Ok(src) => match sess.eval(&src) {
                Ok(v) => println!("{v}"),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Не удалось прочитать файл: {e}");
                std::process::exit(1);
            }
        }
    } else if let Some(expr) = args.expr {
        match sess.eval(&expr) {
            Ok(v) => println!("{v}"),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    } else {
        repl(&mut sess);
    }
}

fn repl(sess: &mut calc_core::Session) {
    use rustyline::DefaultEditor;
    let mut rl = match DefaultEditor::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    println!("Калькулятор. Ctrl-D для выхода.");
    // Цикл прерывается на Ctrl-D / Ctrl-C, когда readline возвращает Err.
    while let Ok(line) = rl.readline("> ") {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let _ = rl.add_history_entry(line);
        match sess.eval(line) {
            Ok(v) => println!("= {v}"),
            Err(e) => eprintln!("{e}"),
        }
    }
}
