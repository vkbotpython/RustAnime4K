//! Мелкие виджеты справки (карточки, бейджи, секции).

use egui::{Color32, RichText, Stroke};

pub(crate) fn accent(ui: &egui::Ui) -> Color32 {
    ui.visuals().hyperlink_color
}

pub(crate) fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(10.0);
    ui.label(RichText::new(title).size(15.0).strong().color(accent(ui)));
    ui.separator();
    ui.add_space(2.0);
}

pub(crate) fn para(ui: &mut egui::Ui, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).size(13.5)).wrap());
    ui.add_space(4.0);
}

pub(crate) fn bullets(ui: &mut egui::Ui, items: &[&str]) {
    for item in items {
        ui.add(egui::Label::new(RichText::new(format!("• {item}")).size(13.0)).wrap());
    }
    ui.add_space(4.0);
}

pub(crate) fn badge(text: &str, bg: Color32) -> RichText {
    RichText::new(format!(" {text} "))
        .strong()
        .size(13.0)
        .color(Color32::WHITE)
        .background_color(bg)
}

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

pub(crate) fn card(ui: &mut egui::Ui, fill: Color32, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(fill)
        .inner_margin(10.0)
        .corner_radius(8.0)
        .stroke(Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .show(ui, add);
    ui.add_space(6.0);
}

pub(crate) fn card_fill(ui: &egui::Ui) -> Color32 {
    crate::theme::card_fill_for(ui.visuals().window_fill)
}

pub(crate) fn note_fill(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(64, 50, 12)
    } else {
        Color32::from_rgb(255, 248, 225)
    }
}

// ---------------------------------------------------------------------------
// data
