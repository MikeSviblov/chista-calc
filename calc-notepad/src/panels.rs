//! Верхний тулбар и отдельное окно-справочник функций (как «Справка» в оригинале).

pub struct ToolbarActions {
    pub open: bool,
    pub save: bool,
    pub clear: bool,
    pub toggle_help: bool,
    pub font_delta: f32,
    pub toggle_on_top: bool,
}

pub fn toolbar(ctx: &egui::Context, always_on_top: bool, status: Option<&str>) -> ToolbarActions {
    let mut a = ToolbarActions {
        open: false,
        save: false,
        clear: false,
        toggle_help: false,
        font_delta: 0.0,
        toggle_on_top: false,
    };
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("Открыть").clicked() {
                a.open = true;
            }
            if ui.button("Сохранить").clicked() {
                a.save = true;
            }
            if ui.button("Очистить").clicked() {
                a.clear = true;
            }
            if ui.button("Справка").clicked() {
                a.toggle_help = true;
            }
            ui.separator();
            if ui.button("Шрифт −").clicked() {
                a.font_delta = -1.0;
            }
            if ui.button("Шрифт +").clicked() {
                a.font_delta = 1.0;
            }
            ui.separator();
            let mut on_top = always_on_top;
            if ui.checkbox(&mut on_top, "поверх окон").changed() {
                a.toggle_on_top = true;
            }
            if let Some(s) = status {
                ui.separator();
                ui.weak(s);
            }
        });
    });
    a
}

/// Отдельное окно-справочник встроенных функций. Возвращает Some(name), если
/// пользователь кликнул по имени (для вставки в текущую строку).
pub fn help_window(ctx: &egui::Context, open: &mut bool, builtins: &[String]) -> Option<String> {
    let mut insert = None;
    egui::Window::new("Справка — функции")
        .open(open)
        .default_width(220.0)
        .show(ctx, |ui| {
            ui.label("Встроенные функции (клик — вставить):");
            egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                for name in builtins {
                    if ui.link(name).clicked() {
                        insert = Some(name.clone());
                    }
                }
            });
        });
    insert
}
