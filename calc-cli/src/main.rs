use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "calc", version, about = "Калькулятор — выражения, переменные, функции, циклы, хеши, шифры")]
struct Args {
    /// Выражение для вычисления, либо `help` для справки по функциям
    #[arg(conflicts_with = "file")]
    expr: Option<String>,
    /// Имя функции для `calc help <имя>`
    #[arg(conflicts_with = "file")]
    topic: Option<String>,
    /// Выполнить скрипт из файла
    #[arg(long)]
    file: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let mut sess = calc_core::Session::new();

    // `calc help` / `calc help <имя>` — справочник функций.
    if args.expr.as_deref() == Some("help") {
        print_help(&sess, args.topic.as_deref());
        return;
    }

    if let Some(path) = args.file {
        match std::fs::read_to_string(&path) {
            // Вывод скрипта — только из явных print(...); финальное значение не эхоим.
            Ok(src) => match sess.eval(&src) {
                Ok(_) => {}
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

/// Печатает справку: без имени — список по категориям, с именем — статью.
fn print_help(sess: &calc_core::Session, topic: Option<&str>) {
    if let Some(name) = topic {
        match sess.help_for(name) {
            Some(e) => print!("{}", render_entry(e)),
            None => {
                eprintln!("Нет функции с именем «{name}».");
                std::process::exit(1);
            }
        }
        return;
    }
    for (key, label) in calc_core::help::CATEGORIES {
        let mut in_cat: Vec<_> = sess.help_all().iter().filter(|e| e.category == *key).collect();
        if in_cat.is_empty() {
            continue;
        }
        in_cat.sort_by(|a, b| a.name.cmp(b.name));
        println!("\n=== {label} ===");
        for e in in_cat {
            println!("  {:<24} {}", e.signature, e.summary_ru);
        }
    }
    println!("\nПодробно: calc help <имя функции>");
}

/// Многострочная статья для одной функции (двуязычная).
fn render_entry(e: &calc_core::help::HelpEntry) -> String {
    let mut s = format!(
        "{}   [{}]\n  RU: {}\n  EN: {}\n  Пример: {}\n",
        e.signature,
        calc_core::help::category_label(e.category),
        e.summary_ru,
        e.summary_en,
        e.example,
    );
    if !e.note_ru.is_empty() || !e.note_en.is_empty() {
        s.push_str(&format!("  ! {} / {}\n", e.note_ru, e.note_en));
    }
    s
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
    println!("Калькулятор. `help` — справка, `help <имя>` — по функции. Ctrl-D для выхода.");
    // Цикл прерывается на Ctrl-D / Ctrl-C, когда readline возвращает Err.
    while let Ok(line) = rl.readline("> ") {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let _ = rl.add_history_entry(line);
        // `help` и `help <имя>` внутри REPL.
        if line == "help" || line.starts_with("help ") {
            let topic = line.strip_prefix("help").map(str::trim).filter(|s| !s.is_empty());
            print_help(sess, topic);
            continue;
        }
        match sess.eval(line) {
            Ok(v) => println!("= {v}"),
            Err(e) => eprintln!("{e}"),
        }
    }
}
