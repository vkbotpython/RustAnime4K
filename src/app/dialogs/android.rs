//! Диалог генератора конфига для Android.

use crate::anime4k;
use crate::app::state::Anime4KPlayerApp;
use crate::i18n::Lang;
use serde_json::json;

use super::ui_kit::{card, card_fill, field_title, hint, para, text_field};

impl Anime4KPlayerApp {
    pub(crate) fn android_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_android {
            return;
        }
        let title = self.t("Create config for Android");
        let intro = self.t(
            "You can use this config to use the Anime4K algorithm in the mpv video player on android",
        );
        let path_label = self.t("Enter the path to the shaders");
        let preset_label = self.t("Select the algorithm configuration");
        let selected_label = self.t("Selected");
        let all_label = self.t("All");
        let russian = self.lang == Lang::Russian;
        let mut combos: Vec<(String, String, String)> = Vec::new();
        for quality in anime4k::QUALITIES {
            for mode in anime4k::MODES {
                combos.push((
                    format!("{} {mode} ({quality})", self.t("Mode")),
                    quality.to_string(),
                    mode.to_string(),
                ));
            }
        }
        if self.android_index >= combos.len() {
            self.android_index = 0;
        }
        let mut open = self.show_android;
        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .min_width(460.0)
            .default_size([540.0, 500.0])
            .show(ctx, |ui| {
                card(ui, |ui| {
                    para(ui, intro);
                });
                card(ui, |ui| {
                    field_title(ui, path_label);
                    text_field(
                        ui,
                        &mut self.android_path,
                        "/storage/emulated/0/mpv/shaders/",
                    );
                });
                card(ui, |ui| {
                    field_title(ui, preset_label);
                    egui::ComboBox::from_id_salt("android-preset")
                        .selected_text(&combos[self.android_index].0)
                        .show_ui(ui, |ui| {
                            for (i, (label, _, _)) in combos.iter().enumerate() {
                                ui.selectable_value(&mut self.android_index, i, label);
                            }
                        });
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        if ui.button(selected_label).clicked() {
                            let selected_text = combos[self.android_index].0.clone();
                            let quality = combos[self.android_index].1.clone();
                            let mode = combos[self.android_index].2.clone();
                            let single = anime4k::create_preset(&quality, &mode)
                                .map(|p| {
                                    anime4k::android_config_line(&p, &self.android_path.clone())
                                })
                                .unwrap_or_default();
                            ui.ctx().copy_text(format!("# {selected_text}\n{single}"));
                        }
                        if ui.button(all_label).clicked() {
                            let mut blocks = Vec::new();
                            for (label, q, m) in &combos {
                                if let Ok(p) = anime4k::create_preset(q, m) {
                                    blocks.push(format!(
                                        "# {label}\n# {}",
                                        anime4k::android_config_line(&p, &self.android_path)
                                    ));
                                }
                            }
                            ui.ctx().copy_text(blocks.join("\n\n"));
                        }
                    });
                    hint(
                        ui,
                        if russian {
                            "Кнопки копируют готовые строки для mpv.conf в буфер обмена."
                        } else {
                            "The buttons copy ready mpv.conf lines to the clipboard."
                        },
                    );
                });
                let selected_text = combos[self.android_index].0.clone();
                let quality = combos[self.android_index].1.clone();
                let mode = combos[self.android_index].2.clone();
                let single = anime4k::create_preset(&quality, &mode)
                    .map(|p| anime4k::android_config_line(&p, &self.android_path.clone()))
                    .unwrap_or_default();
                // Same look as `card()`, but with NO trailing space: this
                // fill-sized card is the last element of an auto-sized
                // window, and any widget after the fill (even 6px of space)
                // makes the content larger than the window every frame, so
                // the window grows without bound.
                egui::Frame::new()
                    .fill(card_fill(ui))
                    .inner_margin(10.0)
                    .corner_radius(8.0)
                    .stroke(egui::Stroke::new(
                        1.0,
                        ui.visuals().widgets.noninteractive.bg_stroke.color,
                    ))
                    .show(ui, |ui| {
                        field_title(ui, "mpv.conf");
                        // Fill, don't cap: the scroll area takes exactly the
                        // remaining window space, so the window is stable on
                        // open and follows a manual resize (a `max_height`
                        // cap makes a taller drag snap back on release).
                        egui::ScrollArea::both()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.monospace(format!("# {selected_text}\n{single}"));
                            });
                    });
            });
        // The path field has no Save button (actions only copy to the
        // clipboard): remember it on close so it survives a restart.
        if self.show_android && !open {
            self.config
                .set("androidPath", json!(self.android_path.clone()));
            self.flush_config();
        }
        self.show_android = open;
    }
}
