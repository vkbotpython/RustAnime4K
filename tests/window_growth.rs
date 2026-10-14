//! Regression test: a dialog window with a fill-remaining-space multiline
//! editor laid out LAST stays a stable height frame over frame, in both
//! default and spacious styles. (A fill-sized editor with widgets after it,
//! or a `bottom_up` region, fed back into the auto-sized window and
//! ballooned it to the screen edge.)

use std::cell::RefCell;

fn window_heights(frames: usize, spacious: bool) -> Vec<f32> {
    let ctx = egui::Context::default();
    if spacious {
        ctx.all_styles_mut(|style| {
            style.spacing.button_padding = egui::vec2(10.0, 6.0);
            style.spacing.item_spacing = egui::vec2(10.0, 6.0);
            style.spacing.menu_margin = egui::Margin::same(8);
        });
    }
    let mut text = "volume=50\nhwdec=auto\naudio-delay=0.15\n".to_string();
    let rect: RefCell<Option<egui::Rect>> = RefCell::new(None);
    let mut out = Vec::with_capacity(frames);
    let mut t = 0.0;
    for _ in 0..frames {
        t += 1.0 / 60.0;
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 800.0),
            )),
            time: Some(t),
            predicted_dt: 1.0 / 60.0,
            ..Default::default()
        };
        ctx.begin_pass(raw);
        let mut open = true;
        // Mirrors `launch_params_dialog`: card, buttons, fill editor last.
        let resp = egui::Window::new("test")
            .open(&mut open)
            .collapsible(false)
            .min_width(440.0)
            .default_size([540.0, 420.0])
            .show(&ctx, |ui| {
                egui::Frame::new()
                    .inner_margin(10.0)
                    .corner_radius(8.0)
                    .show(ui, |ui| {
                        ui.label("Используйте руководство mpv https://mpv.io/manual/");
                        ui.label("Вводите по одной паре имя=значение на строку.");
                        ui.monospace("volume=50");
                        ui.label("name=value");
                    });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let _ = ui.button("Применить");
                    let _ = ui.button("Сохранить");
                    let _ = ui.button("Отмена");
                });
                ui.add_space(4.0);
                let avail = ui.available_size();
                ui.add_sized(
                    egui::vec2(avail.x, avail.y.max(120.0)),
                    egui::TextEdit::multiline(&mut text)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("name=value"),
                );
            });
        *rect.borrow_mut() = resp.map(|r| r.response.rect);
        out.push(rect.borrow().map(|r| r.height()).unwrap_or(-1.0));
        let _ = ctx.end_pass();
    }
    out
}

#[test]
fn fill_last_editor_dialog_height_is_stable() {
    for spacious in [false, true] {
        let hs = window_heights(500, spacious);
        assert!(hs.iter().all(|&h| h > 0.0), "window never shown");
        let tail = &hs[hs.len() / 2..];
        let min = tail.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = tail.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        assert!(
            max - min < 1.0,
            "spacious={spacious}: window height drifts ({min:.1}..{max:.1})"
        );
    }
}
