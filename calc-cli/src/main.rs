use calc_core::Lang;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "calc", version, about = "Калькулятор — выражения, переменные, функции, циклы, хеши, шифры / Calculator")]
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
    /// Язык сообщений: ru | en (по умолчанию ru; можно задать переменной CALC_LANG)
    #[arg(long)]
    lang: Option<String>,
}

/// Язык: флаг `--lang` > переменная CALC_LANG > по умолчанию (ru).
fn resolve_lang(cli: &Option<String>) -> Lang {
    if let Some(s) = cli {
        if let Some(l) = Lang::parse(s) {
            return l;
        }
    }
    if let Ok(s) = std::env::var("CALC_LANG") {
        if let Some(l) = Lang::parse(&s) {
            return l;
        }
    }
    Lang::default()
}

/// Выбор строки по языку.
fn tr(lang: Lang, ru: &str, en: &str) -> String {
    match lang {
        Lang::Ru => ru,
        Lang::En => en,
    }
    .to_string()
}

fn main() {
    let args = Args::parse();
    let lang = resolve_lang(&args.lang);
    let mut sess = calc_core::Session::new();
    sess.set_lang(lang);

    // `calc help` / `calc help <имя>` — справочник функций.
    if args.expr.as_deref() == Some("help") {
        print_help(&sess, args.topic.as_deref(), lang);
        return;
    }

    if let Some(path) = args.file {
        match std::fs::read_to_string(&path) {
            // Вывод скрипта — только из явных print(...); финальное значение не эхоим.
            Ok(src) => match sess.eval(&src) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e.message(lang));
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!(
                    "{}",
                    match lang {
                        Lang::Ru => format!("Не удалось прочитать файл: {e}"),
                        Lang::En => format!("Failed to read file: {e}"),
                    }
                );
                std::process::exit(1);
            }
        }
    } else if let Some(expr) = args.expr {
        match sess.eval(&expr) {
            Ok(v) => println!("{v}"),
            Err(e) => {
                eprintln!("{}", e.message(lang));
                std::process::exit(1);
            }
        }
    } else {
        repl(&mut sess, lang);
    }
}

/// Печатает справку: без имени — список по категориям, с именем — статью.
fn print_help(sess: &calc_core::Session, topic: Option<&str>, lang: Lang) {
    if let Some(name) = topic {
        match sess.help_for(name) {
            Some(e) => print!("{}", render_entry(e, lang)),
            None => {
                eprintln!(
                    "{}",
                    match lang {
                        Lang::Ru => format!("Нет функции с именем «{name}»."),
                        Lang::En => format!("No function named \"{name}\"."),
                    }
                );
                std::process::exit(1);
            }
        }
        return;
    }
    for (key, ..) in calc_core::help::CATEGORIES {
        let mut in_cat: Vec<_> = sess.help_all().iter().filter(|e| e.category == *key).collect();
        if in_cat.is_empty() {
            continue;
        }
        in_cat.sort_by(|a, b| a.name.cmp(b.name));
        println!("\n=== {} ===", calc_core::help::category_label(key, lang));
        for e in in_cat {
            println!("  {:<24} {}", e.signature, e.summary(lang));
        }
    }
    println!(
        "\n{}",
        tr(lang, "Подробно: calc help <имя функции>", "Details: calc help <function name>")
    );
}

/// Статья для одной функции на выбранном языке.
fn render_entry(e: &calc_core::help::HelpEntry, lang: Lang) -> String {
    let mut s = format!(
        "{}   [{}]\n  {}\n  {} {}\n",
        e.signature,
        calc_core::help::category_label(e.category, lang),
        e.summary(lang),
        tr(lang, "Пример:", "Example:"),
        e.example,
    );
    let note = e.note(lang);
    if !note.is_empty() {
        s.push_str(&format!("  ! {note}\n"));
    }
    s
}

fn repl(sess: &mut calc_core::Session, lang: Lang) {
    use rustyline::DefaultEditor;
    let mut rl = match DefaultEditor::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    println!(
        "{}",
        tr(
            lang,
            "Калькулятор. `help` — справка, `help <имя>` — по функции. Ctrl-D для выхода.",
            "Calculator. `help` for the function list, `help <name>` for one. Ctrl-D to exit.",
        )
    );
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
            print_help(sess, topic, lang);
            continue;
        }
        match sess.eval(line) {
            Ok(v) => println!("= {v}"),
            Err(e) => eprintln!("{}", e.message(lang)),
        }
    }
}
