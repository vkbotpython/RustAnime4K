//! Диалог «О программе».

use crate::app::state::Anime4KPlayerApp;
use crate::app::state::{APP_NAME, DONATE_CARD_PLAIN, DONATE_CARD_SPACED, VERSION};
use crate::i18n::Lang;
use std::time::{Duration, Instant};

use super::ui_kit::{badge, card};

/// Badge backgrounds for the engine/license values (same badge language as
/// the tech stack above; engine reuses the libmpv purple).
const ENGINE_BADGE: egui::Color32 = egui::Color32::from_rgb(123, 31, 162);
const LICENSE_BADGE: egui::Color32 = egui::Color32::from_rgb(90, 90, 90);

impl Anime4KPlayerApp {
    pub(crate) fn about_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_about {
            return;
        }
        let title = self.t("About");
        let about_text = self.t("About program");
        let copy_label = self.t("Copy");
        let mpv_version = self.mpv.mpv_version();
        let russian = self.lang == Lang::Russian;
        let copied_label = if russian {
            "Скопировано"
        } else {
            "Copied"
        };
        let copied =
            matches!(self.about_copied_at, Some(t) if t.elapsed() < Duration::from_secs(2));
        let mut open = self.show_about;
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .min_width(460.0)
            .show(ctx, |ui| {
                // --- шапка: иконка, название, версия, слоган ---
                ui.vertical_centered(|ui| {
                    ui.add(
                        egui::Image::from_bytes(
                            "bytes://anime4k-player-icon",
                            crate::icon::ICON_PNG,
                        )
                        .max_width(96.0),
                    );
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(APP_NAME).size(22.0).strong());
                    ui.label(
                        egui::RichText::new(format!(" v{VERSION} "))
                            .size(13.0)
                            .strong()
                            .color(egui::Color32::WHITE)
                            .background_color(ui.visuals().hyperlink_color),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new("Rust · egui · libmpv · Anime4K")
                            .size(12.5)
                            .italics()
                            .color(ui.visuals().weak_text_color()),
                    );
                });
                ui.add_space(4.0);
                // --- стек технологий ---
                ui.vertical_centered(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        for (name, color) in [
                            ("Rust", egui::Color32::from_rgb(206, 92, 0)),
                            ("egui", egui::Color32::from_rgb(25, 118, 210)),
                            ("libmpv", egui::Color32::from_rgb(123, 31, 162)),
                            ("Anime4K", egui::Color32::from_rgb(211, 47, 47)),
                        ] {
                            ui.label(
                                egui::RichText::new(format!(" {name} "))
                                    .size(12.5)
                                    .strong()
                                    .color(egui::Color32::WHITE)
                                    .background_color(color),
                            );
                        }
                    });
                });
                ui.separator();
                // --- описание ---
                ui.add(egui::Label::new(egui::RichText::new(about_text).size(13.5)).wrap());
                ui.add_space(4.0);
                // --- движок и лицензия ---
                egui::Grid::new("about_info")
                    .num_columns(2)
                    .spacing([12.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(if russian { "Движок" } else { "Engine" })
                                .strong()
                                .size(12.5),
                        );
                        ui.monospace(badge(&mpv_version, ENGINE_BADGE));
                        ui.end_row();
                        ui.label(
                            egui::RichText::new(if russian {
                                "Лицензия"
                            } else {
                                "License"
                            })
                            .strong()
                            .size(12.5),
                        );
                        ui.monospace(badge("GPL-3.0-or-later", LICENSE_BADGE));
                        ui.end_row();
                    });
                ui.add_space(6.0);
                // --- связь и исходный код ---
                card(ui, |ui| {
                    ui.label(
                        egui::RichText::new(if russian {
                            "Связь и исходный код"
                        } else {
                            "Contact & source"
                        })
                        .strong()
                        .size(13.0),
                    );
                    ui.horizontal_wrapped(|ui| {
                        ui.label("YouTube:");
                        ui.hyperlink("https://www.youtube.com/@avencores/");
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Telegram:");
                        ui.hyperlink("https://t.me/avencoresyt");
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label("VK:");
                        ui.hyperlink("https://vk.ru/avencoresreuploads");
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(if russian { "Дзен:" } else { "Dzen:" });
                        ui.hyperlink("https://dzen.ru/avencores");
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label("GitLab:");
                        ui.hyperlink("https://gitlab.com/avencores/RustAnime4KPlayer");
                    });
                });
                // --- донат ---
                card(ui, |ui| {
                    ui.label(
                        egui::RichText::new(if russian {
                            "Поддержать автора"
                        } else {
                            "Support the author"
                        })
                        .strong()
                        .size(13.0),
                    );
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new(" SBER ")
                                .strong()
                                .size(12.5)
                                .color(egui::Color32::WHITE)
                                .background_color(egui::Color32::from_rgb(46, 125, 50)),
                        );
                        ui.monospace(DONATE_CARD_SPACED);
                        if ui.button(copy_label).clicked() {
                            // Bank apps paste card numbers without spaces.
                            ui.ctx().copy_text(DONATE_CARD_PLAIN.to_string());
                            self.about_copied_at = Some(Instant::now());
                        }
                        if copied {
                            ui.label(
                                egui::RichText::new(copied_label)
                                    .strong()
                                    .size(12.5)
                                    .color(egui::Color32::from_rgb(46, 125, 50)),
                            );
                        }
                    });
                });
            });
        self.show_about = open;
    }
}
