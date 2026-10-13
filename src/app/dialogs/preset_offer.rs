//! Диалог выбора Anime4K-пресета при старте видео.

use crate::anime4k;
use crate::app::helpers::display_name_of;
use crate::app::state::Anime4KPlayerApp;
use serde_json::json;

impl Anime4KPlayerApp {
    /// Offered when a video starts and the file has no saved preset:
    /// choose an Anime4K preset or disable it entirely. The choice is
    /// remembered per file (`filePresets`); closing without choosing
    /// asks again on the next file.
    pub(crate) fn preset_offer_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_preset_offer {
            return;
        }
        let Some(current) = self.player.current_file().cloned() else {
            self.show_preset_offer = false;
            return;
        };
        let title = self.t("Choose an Anime4K preset");
        let quality_label = self.t("Quality");
        let mode_label = self.t("Mode");
        let apply_label = self.t("Apply");
        let disable_label = self.t("Disable");
        let dont_ask_label = self.t("Don't ask again");
        let name = display_name_of(&current.to_string_lossy());

        // Modes depend on the selected quality (standard 6 vs HQ list).
        let hq_names: Vec<String> = anime4k::ultra_hq_presets()
            .keys()
            .map(|s| s.to_string())
            .collect();
        let mut hq_names = hq_names;
        hq_names.sort();
        let is_hq = self.preset_offer_quality == "HQ";
        if is_hq {
            if !hq_names.contains(&self.preset_offer_mode) {
                self.preset_offer_mode = hq_names.first().cloned().unwrap_or_default();
            }
        } else {
            if !anime4k::MODES.contains(&self.preset_offer_mode.as_str()) {
                self.preset_offer_mode = "A".to_string();
            }
            if !anime4k::QUALITIES.contains(&self.preset_offer_quality.as_str()) {
                self.preset_offer_quality = "M".to_string();
            }
        }

        let mut open = self.show_preset_offer;
        let mut applied = false;
        let mut disabled = false;
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .min_width(380.0)
            .show(ctx, |ui| {
                ui.monospace(&name);
                ui.horizontal(|ui| {
                    ui.label(quality_label);
                    egui::ComboBox::from_id_salt("offer-quality")
                        .selected_text(self.preset_offer_quality.clone())
                        .show_ui(ui, |ui| {
                            for quality in anime4k::QUALITIES
                                .iter()
                                .map(|s| s.to_string())
                                .chain(std::iter::once("HQ".to_string()))
                            {
                                ui.selectable_value(
                                    &mut self.preset_offer_quality,
                                    quality.clone(),
                                    quality,
                                );
                            }
                        });
                    ui.label(mode_label);
                    egui::ComboBox::from_id_salt("offer-mode")
                        .selected_text(self.preset_offer_mode.clone())
                        .show_ui(ui, |ui| {
                            if is_hq {
                                for mode in &hq_names {
                                    ui.selectable_value(
                                        &mut self.preset_offer_mode,
                                        mode.clone(),
                                        mode,
                                    );
                                }
                            } else {
                                for mode in anime4k::MODES {
                                    ui.selectable_value(
                                        &mut self.preset_offer_mode,
                                        (*mode).to_string(),
                                        *mode,
                                    );
                                }
                            }
                        });
                });
                ui.horizontal(|ui| {
                    if ui.button(apply_label).clicked() {
                        applied = true;
                    }
                    if ui.button(disable_label).clicked() {
                        disabled = true;
                    }
                });
                ui.checkbox(&mut self.preset_offer_dont_ask, dont_ask_label);
            });
        if applied {
            if is_hq {
                self.apply_anime4k_hq(&self.preset_offer_mode.clone());
            } else {
                let quality = self.preset_offer_quality.clone();
                let mode = self.preset_offer_mode.clone();
                self.apply_anime4k(&quality, &mode);
            }
            self.finish_preset_offer();
            self.show_preset_offer = false;
        } else if disabled {
            self.disable_anime4k();
            self.finish_preset_offer();
            self.show_preset_offer = false;
        } else {
            self.show_preset_offer = open;
        }
    }

    /// Persists the "don't ask again" opt-out after an explicit choice.
    pub(crate) fn finish_preset_offer(&mut self) {
        if self.preset_offer_dont_ask {
            self.config.set("askPresetOnOpen", json!(false));
            self.flush_config();
        }
    }
}
