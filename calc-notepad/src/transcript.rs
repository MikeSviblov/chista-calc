//! Единое поле-«блокнот»: один многострочный редактор egui (нативные курсор,
//! стрелки, выделение, ENTER), где строки-результаты вписаны в тот же текст.
//! Пересчёт по ENTER; строки-результаты — на зелёном/красном фоне (через layouter).

use crate::sheet::Sheet;

/// Рисует единое поле. Возвращает true, если по ENTER произошёл пересчёт.
pub fn show(ui: &mut egui::Ui, sheet: &mut Sheet, font_size: f32) -> bool {
    let font = egui::FontId::monospace(font_size);
    // Проверяем ENTER ДО отрисовки: TextEdit его поглотит (вставит перевод строки).
    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
    let error_lines = sheet.error_lines.clone();

    let out = egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut layouter = |ui: &egui::Ui, text: &str, _w: f32| {
                let mut job = crate::highlight::sheet_layout_job(text, font_size, &error_lines);
                job.wrap.max_width = f32::INFINITY;
                ui.fonts(|f| f.layout_job(job))
            };
            egui::TextEdit::multiline(&mut sheet.text)
                .frame(false)
                .desired_width(f32::INFINITY)
                .font(font.clone())
                .layouter(&mut layouter)
                .show(ui)
        })
        .inner;

    if enter_pressed && out.response.has_focus() {
        // Курсор уже после вставленного egui перевода строки.
        let cursor = out
            .state
            .cursor
            .char_range()
            .map(|r| r.primary.index)
            .unwrap_or(0);
        let new_cursor = sheet.recompute_with_cursor(cursor);
        let mut st = out.state.clone();
        st.cursor.set_char_range(Some(egui::text::CCursorRange::one(
            egui::text::CCursor::new(new_cursor),
        )));
        st.store(ui.ctx(), out.response.id);
        return true;
    }
    false
}
