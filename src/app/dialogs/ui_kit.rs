//! Общие виджеты диалогов в стиле «Справки» / «О программе».
//!
//! Карточки, секции, подсказки и бейджи: тема (тёмная/светлая) подхватывается
//! автоматически через `ui.visuals()`, как в `crate::reference`.

use egui::{Color32, RichText, Stroke};

/// Обычный текст абзаца.
pub(crate) fn para(ui: &mut egui::Ui, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).size(13.5)).wrap());
    ui.add_space(4.0);
}

/// Мелкая курсивная подсказка слабым цветом.
pub(crate) fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .size(12.5)
            .italics()
            .color(ui.visuals().weak_text_color()),
    );
}

/// Живая подпись поля или группы.
pub(crate) fn field_title(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).strong().size(13.0));
    ui.add_space(2.0);
}

/// Карточка с подложкой (как в справке / «О программе»).
pub(crate) fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(card_fill(ui))
        .inner_margin(10.0)
        .corner_radius(8.0)
        .stroke(Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .show(ui, add);
    ui.add_space(6.0);
}

/// Подложка карточки: чуть плотнее `faint_bg_color` текущей темы.
pub(crate) fn card_fill(ui: &egui::Ui) -> Color32 {
    let base = ui.visuals().faint_bg_color;
    Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), 220)
}

/// Однострочное поле ввода на всю ширину с подсказкой-плейсхолдером.
pub(crate) fn text_field(ui: &mut egui::Ui, text: &mut String, hint_text: &str) {
    ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(hint_text)
            .desired_width(f32::INFINITY),
    );
}

/// Нижняя панель кнопок: разделитель + выравнивание вправо.
/// Внимание: в `right_to_left` первая добавленная кнопка оказывается справа,
/// поэтому основное действие (Сохранить/Выбрать/Применить) идёт первым.
/// `horizontal` вокруг обязателен: голый `with_layout(right_to_left)`
/// забирает `available_size()` (весь остаток окна по высоте) и даёт
/// огромную пустую полосу над кнопками.
pub(crate) fn bottom_bar(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.separator();
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add);
    });
}

/// Цветной бейдж (режимы Anime4K, стек технологий).
pub(crate) fn badge(text: &str, bg: Color32) -> RichText {
    RichText::new(format!(" {text} "))
        .strong()
        .size(13.0)
        .color(Color32::WHITE)
        .background_color(bg)
}

/// Цвет бейджа режима Anime4K (как в справке).
pub(crate) fn mode_color(id: &str) -> Color32 {
    match id {
        "A" => Color32::from_rgb(211, 47, 47),
        "B" => Color32::from_rgb(25, 118, 210),
        "C" => Color32::from_rgb(56, 142, 60),
        "A+A" => Color32::from_rgb(123, 31, 162),
        "B+B" => Color32::from_rgb(0, 121, 107),
        "C+A" => Color32::from_rgb(230, 119, 0),
        _ => Color32::from_rgb(90, 90, 90),
    }
}
