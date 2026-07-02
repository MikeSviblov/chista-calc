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
                    if ui.link(name.clone()).clicked() { insert = Some(name.clone()); }
                }
            });
        });
    });
    insert
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
