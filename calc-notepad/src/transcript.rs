//! Единое поле-«блокнот»: стек строк-выражений, под каждой — результат (зелёный).
//! Ввод и вывод в одном поле, пересчёт по ENTER — как в оригинале «Чиста калькулятор».

/// Что произошло за кадр.
pub struct Action {
    /// Текст какой-либо строки изменился.
    pub changed: bool,
    /// В строке с этим индексом нажали ENTER (нужно пересчитать/добавить строку).
    pub entered: Option<usize>,
}

/// Рисует единое поле: для каждой строки — редактируемое выражение и под ним,
/// если есть, результат на зелёном (ошибка — на красном) фоне.
pub fn show(
    ui: &mut egui::Ui,
    entries: &mut [String],
    results: &[Option<(String, bool)>],
    font_size: f32,
    focus: &mut Option<usize>,
) -> Action {
    let mut action = Action { changed: false, entered: None };
    let font = egui::FontId::monospace(font_size);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            for (i, entry) in entries.iter_mut().enumerate() {
                // Подсветка синтаксиса выражения тем же лексером ядра.
                let mut layouter = |ui: &egui::Ui, text: &str, _w: f32| {
                    let mut job = crate::highlight::layout_job(text, font_size);
                    job.wrap.max_width = f32::INFINITY;
                    ui.fonts(|f| f.layout_job(job))
                };
                let resp = ui.add(
                    egui::TextEdit::singleline(entry)
                        .frame(false)
                        .desired_width(f32::INFINITY)
                        .font(font.clone())
                        .layouter(&mut layouter),
                );
                if resp.changed() {
                    action.changed = true;
                }
                if *focus == Some(i) {
                    resp.request_focus();
                    *focus = None;
                }
                if resp.lost_focus() && ui.input(|inp| inp.key_pressed(egui::Key::Enter)) {
                    action.entered = Some(i);
                }

                // Строка-результат под выражением (как зелёный бокс оригинала).
                if let Some((text, is_err)) = results.get(i).and_then(|o| o.as_ref()) {
                    let (bg, fg) = if *is_err {
                        (
                            egui::Color32::from_rgb(0x4a, 0x1c, 0x1c),
                            egui::Color32::from_rgb(0xff, 0xb3, 0xb3),
                        )
                    } else {
                        (
                            egui::Color32::from_rgb(0x2f, 0x7d, 0x3f),
                            egui::Color32::from_rgb(0xe8, 0xff, 0xe8),
                        )
                    };
                    ui.horizontal(|ui| {
                        ui.add_space(font_size); // небольшой отступ слева, как в оригинале
                        egui::Frame::none()
                            .fill(bg)
                            .inner_margin(egui::Margin::symmetric(6.0, 1.0))
                            .rounding(egui::Rounding::same(2.0))
                            .show(ui, |ui| {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(text).monospace().size(font_size).color(fg),
                                    )
                                    .wrap(),
                                );
                            });
                    });
                }
            }
        });

    action
}
