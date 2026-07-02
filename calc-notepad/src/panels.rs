use crate::document::Document;

/// Левая панель: текущие переменные + справочник встроенных функций.
/// Возвращает Some(name), если кликнули по имени функции (для вставки в редактор).
pub fn side_panel(ctx: &egui::Context, doc: &Document, builtins: &[String]) -> Option<String> {
    let mut insert = None;
    egui::SidePanel::left("side").resizable(true).default_width(180.0).show(ctx, |ui| {
        ui.heading("Переменные");
        if doc.variables.is_empty() {
            ui.weak("— нет —");
        } else {
            for (k, v) in &doc.variables {
                ui.monospace(format!("{k} = {v}"));
            }
        }
        ui.separator();
        egui::CollapsingHeader::new("Функции").default_open(false).show(ui, |ui| {
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                for name in builtins {
                    if ui.link(name).clicked() { insert = Some(name.clone()); }
                }
            });
        });
    });
    insert
}

pub struct ToolbarActions { pub open: bool, pub save: bool, pub font_delta: f32, pub toggle_on_top: bool }

/// Верхняя панель: открыть/сохранить файл, изменить размер шрифта, переключить "поверх окон".
pub fn toolbar(ctx: &egui::Context, always_on_top: bool) -> ToolbarActions {
    let mut a = ToolbarActions { open: false, save: false, font_delta: 0.0, toggle_on_top: false };
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("Открыть").clicked() { a.open = true; }
            if ui.button("Сохранить").clicked() { a.save = true; }
            ui.separator();
            if ui.button("Шрифт −").clicked() { a.font_delta = -1.0; }
            if ui.button("Шрифт +").clicked() { a.font_delta = 1.0; }
            ui.separator();
            let mut on_top = always_on_top;
            if ui.checkbox(&mut on_top, "поверх окон").changed() { a.toggle_on_top = true; }
        });
    });
    a
}

/// Нижняя панель вывода print/циклов.
pub fn output_panel(ctx: &egui::Context, output: &str) {
    egui::TopBottomPanel::bottom("output").resizable(true).default_height(120.0).show(ctx, |ui| {
        ui.label("Вывод:");
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut text = output.to_string();
            ui.add(egui::TextEdit::multiline(&mut text)
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .interactive(false));
        });
    });
}
