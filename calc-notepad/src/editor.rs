use crate::document::Document;
use egui::Color32;

/// Рисует редактор (слева) и колонку результатов (справа), выровненную по строкам.
/// Возвращает true, если текст изменился.
///
/// ВНИМАНИЕ: точное визуальное выравнивание колонки результатов относительно
/// внутренней высоты строк `TextEdit` нужно проверять на реальном десктопе —
/// текущая среда headless, поэтому построчное совпадение подтверждено только
/// компиляцией и рассуждением, но не визуально.
pub fn code_with_results(ui: &mut egui::Ui, text: &mut String, doc: &Document, font_size: f32) -> bool {
    let mut changed = false;
    let font = egui::FontId::monospace(font_size);
    let row_h = ui.fonts(|f| f.row_height(&font));
    ui.horizontal_top(|ui| {
        let total = ui.available_width();
        let editor_w = total * 0.72;
        let resp = ui.add_sized(
            [editor_w, ui.available_height()],
            egui::TextEdit::multiline(text)
                .font(egui::FontId::monospace(font_size))
                .desired_width(editor_w)
                .code_editor(),
        );
        changed = resp.changed();
        ui.vertical(|ui| {
            for (i, _line) in text.split('\n').enumerate() {
                let n = i + 1;
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), row_h),
                    egui::Sense::hover(),
                );
                if let Some(res) = doc.result_for_line(n) {
                    let color = if doc.is_error_line(n) {
                        Color32::from_rgb(0xd0, 0x40, 0x40)
                    } else {
                        Color32::from_rgb(0x30, 0x90, 0x30)
                    };
                    let galley =
                        ui.painter()
                            .layout_no_wrap(format!("= {res}"), font.clone(), color);
                    ui.painter().galley(rect.left_top(), galley, color);
                    if ui
                        .interact(rect, ui.id().with(("res", n)), egui::Sense::click())
                        .clicked()
                    {
                        ui.ctx().copy_text(res);
                    }
                }
            }
        });
    });
    changed
}
