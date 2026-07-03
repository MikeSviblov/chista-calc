fn main() {
    // Вшиваем иконку в .exe только при сборке под Windows (в т.ч. кросс-компиляция
    // x86_64-pc-windows-gnu через mingw: winresource найдёт x86_64-w64-mingw32-windres).
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        if let Err(e) = res.compile() {
            // Не валим сборку, если windres недоступен — просто предупреждаем.
            println!("cargo:warning=winresource: не удалось вшить иконку: {e}");
        }
    }
    println!("cargo:rerun-if-changed=assets/icon.ico");
}
