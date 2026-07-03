//! Единое поле-«блокнот»: один многострочный редактор egui (нативные курсор,
//! стрелки, выделение, ENTER), где строки-результаты вписаны в тот же текст.
//! Пересчёт по ENTER; строки-результаты — на зелёном/красном фоне (через layouter).
//! Плюс автоподстановка имён функций по Tab (всплывающий список у курсора).

use crate::complete::{self, CompleteState};
use crate::sheet::Sheet;

/// Рисует единое поле. Возвращает true, если по ENTER произошёл пересчёт.
pub fn show(ui: &mut egui::Ui, sheet: &mut Sheet, font_size: f32, comp: &mut CompleteState) -> bool {
    let font = egui::FontId::monospace(font_size);

    // --- перехват клавиш ДО TextEdit, чтобы он их не обработал ---
    // Tab перехватываем всегда: он не должен попадать в буфер (там '\t' — маркер
    // строк-результатов) и не должен уводить фокус.
    let tab = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab));
    let mut accept = false;
    let mut open_request = false;
    if comp.open {
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)) {
            comp.selected = comp.selected.saturating_add(1);
        }
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)) {
            comp.selected = comp.selected.saturating_sub(1);
        }
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            comp.open = false;
        }
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
            accept = true;
        }
        if tab {
            accept = true;
        }
    } else if tab {
        open_request = true;
    }

    // ENTER читаем ПОСЛЕ перехвата: если список съел Enter, здесь будет false.
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

    let cursor = out
        .state
        .cursor
        .char_range()
        .map(|r| r.primary.index)
        .unwrap_or(0);

    // --- открытие списка по Tab ---
    if open_request {
        if let Some((start, prefix)) = complete::current_prefix(&sheet.text, cursor) {
            let m = complete::matches(&prefix);
            if m.len() == 1 {
                apply_completion(sheet, &out, ui, start, cursor, m[0].name);
                return false;
            } else if m.len() > 1 {
                comp.open = true;
                comp.selected = 0;
            }
        }
    }

    // --- работа с открытым списком: рисуем/принимаем ---
    if comp.open {
        match complete::current_prefix(&sheet.text, cursor) {
            Some((start, prefix)) => {
                let m = complete::matches(&prefix);
                if m.is_empty() {
                    comp.open = false;
                } else {
                    if comp.selected >= m.len() {
                        comp.selected = m.len() - 1;
                    }
                    let clicked = draw_popup(ui, &out, cursor, &m, comp.selected);
                    let chosen = if accept { Some(comp.selected) } else { clicked };
                    if let Some(i) = chosen {
                        apply_completion(sheet, &out, ui, start, cursor, m[i].name);
                        comp.open = false;
                        return false;
                    }
                }
            }
            None => comp.open = false,
        }
    }

    // --- пересчёт по ENTER (как раньше), если список не перехватил Enter ---
    if enter_pressed && out.response.has_focus() {
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

/// Вставляет `name(` вместо префикса и переносит курсор внутрь скобок.
fn apply_completion(
    sheet: &mut Sheet,
    out: &egui::text_edit::TextEditOutput,
    ui: &egui::Ui,
    start: usize,
    cursor: usize,
    name: &str,
) {
    let (new_text, new_cursor) = complete::apply(&sheet.text, start, cursor, name);
    sheet.text = new_text;
    let mut st = out.state.clone();
    st.cursor.set_char_range(Some(egui::text::CCursorRange::one(
        egui::text::CCursor::new(new_cursor),
    )));
    st.store(ui.ctx(), out.response.id);
}

/// Рисует всплывающий список кандидатов у курсора. Возвращает Some(index), если по
/// кандидату кликнули мышью.
fn draw_popup(
    ui: &egui::Ui,
    out: &egui::text_edit::TextEditOutput,
    cursor: usize,
    matches: &[&'static calc_core::help::HelpEntry],
    selected: usize,
) -> Option<usize> {
    let ccursor = egui::text::CCursor::new(cursor);
    let cur = out.galley.from_ccursor(ccursor);
    let rect = out.galley.pos_from_cursor(&cur);
    let pos = out.galley_pos + rect.left_bottom().to_vec2();

    let mut clicked = None;
    egui::Area::new(egui::Id::new("calc_autocomplete"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (i, e) in matches.iter().enumerate() {
                            let resp =
                                ui.selectable_label(i == selected, egui::RichText::new(e.signature).monospace());
                            if i == selected {
                                resp.scroll_to_me(Some(egui::Align::Center));
                            }
                            if resp.clicked() {
                                clicked = Some(i);
                            }
                        }
                    });
            });
        });
    clicked
}
