//! Диалог настроек.

use crate::app::helpers::{get_bool, get_str};
use crate::app::state::Anime4KPlayerApp;
use crate::i18n::{self, Lang};
use crate::theme::{ColorScheme, Palette};
use serde_json::{Value, json};

impl Anime4KPlayerApp {
    /// Copies saved config into the settings drafts and opens the dialog.
    pub(crate) fn open_settings(&mut self) {
        self.settings_lang = Lang::from_config_value(get_str(&self.config, "language").as_deref());
        self.settings_open_last = get_bool(&self.config, "onOpenLastFile", true);
        self.settings_pos_last = get_bool(&self.config, "onPosLastFile", true);
        self.settings_volume_plus = get_bool(&self.config, "volumePlus", false);
        self.settings_svp = get_bool(&self.config, "SVP", false);
        self.settings_fixed_panel = get_bool(&self.config, "fixedControlPanel", false);
        self.settings_ask_preset = get_bool(&self.config, "askPresetOnOpen", true);
        self.settings_sound_offset = self
            .config
            .get("soundOffset")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        self.settings_palette = Palette::from_name(
            get_str(&self.config, "palette")
                .as_deref()
                .unwrap_or("System"),
        );
        self.settings_style = crate::theme::style::normalize(
            get_str(&self.config, "style")
                .as_deref()
                .unwrap_or("default"),
        )
        .to_string();
        self.settings_scheme = ColorScheme::from_name(
            get_str(&self.config, "colorScheme")
                .as_deref()
                .unwrap_or("System"),
        );
        self.show_settings = true;
    }

    pub(crate) fn settings_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_settings {
            return;
        }
        let title = self.t("Settings");
        let language_label = self.t("Language selection");
        let open_last_label = self.t("On startup open the last opened file");
        let pos_last_label = self.t("Set the position of the last opened file");
        let volume_plus_label = self.t("Increase maximum volume up to 150%");
        let svp_label = self.t("Activate SVP");
        let fixed_label = self.t("Fixed multimedia control panel");
        let ask_preset_label = self.t("Offer to choose an Anime4K preset when opening a file");
        let offset_label = self.t("Sound offset");
        let palette_label = self.t("Palette");
        let style_label = self.t("Theme style");
        let scheme_label = self.t("Color scheme");
        let save_label = self.t("Save");
        let cancel_label = self.t("Cancel");
        let scheme_system = self.t("System");
        let scheme_dark = self.t("Dark");
        let scheme_light = self.t("Light");
        let mut open = self.show_settings;
        let mut save = false;
        let mut cancel = false;
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .min_width(300.0)
            .show(ctx, |ui| {
                ui.label(language_label);
                egui::ComboBox::from_id_salt("lang")
                    .selected_text(self.settings_lang.config_value())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.settings_lang,
                            Lang::Auto,
                            Lang::Auto.config_value(),
                        );
                        for lang in Lang::all() {
                            ui.selectable_value(
                                &mut self.settings_lang,
                                *lang,
                                lang.config_value(),
                            );
                        }
                    });
                ui.checkbox(&mut self.settings_open_last, open_last_label);
                ui.checkbox(&mut self.settings_pos_last, pos_last_label);
                ui.checkbox(&mut self.settings_volume_plus, volume_plus_label);
                ui.checkbox(&mut self.settings_svp, svp_label);
                ui.checkbox(&mut self.settings_fixed_panel, fixed_label);
                ui.checkbox(&mut self.settings_ask_preset, ask_preset_label);
                ui.horizontal(|ui| {
                    ui.label(offset_label);
                    // Sound offset: -100..100 s, step 0.01.
                    ui.add(
                        egui::DragValue::new(&mut self.settings_sound_offset)
                            .speed(0.01)
                            .range(-100.0..=100.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(palette_label);
                    egui::ComboBox::from_id_salt("palette")
                        .selected_text(self.settings_palette.name())
                        .show_ui(ui, |ui| {
                            for name in Palette::all_names() {
                                ui.selectable_value(
                                    &mut self.settings_palette,
                                    Palette::from_name(name),
                                    *name,
                                );
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label(scheme_label);
                    egui::ComboBox::from_id_salt("scheme")
                        .selected_text(self.settings_scheme.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.settings_scheme,
                                ColorScheme::System,
                                scheme_system.as_str(),
                            );
                            ui.selectable_value(
                                &mut self.settings_scheme,
                                ColorScheme::Dark,
                                scheme_dark.as_str(),
                            );
                            ui.selectable_value(
                                &mut self.settings_scheme,
                                ColorScheme::Light,
                                scheme_light.as_str(),
                            );
                        });
                });
                ui.horizontal(|ui| {
                    ui.label(style_label);
                    egui::ComboBox::from_id_salt("qstyle")
                        .selected_text(self.settings_style.clone())
                        .show_ui(ui, |ui| {
                            for name in crate::theme::style::all_names() {
                                ui.selectable_value(
                                    &mut self.settings_style,
                                    (*name).to_string(),
                                    *name,
                                );
                            }
                        });
                });
                ui.horizontal(|ui| {
                    if ui.button(save_label).clicked() {
                        save = true;
                    }
                    if ui.button(cancel_label).clicked() {
                        cancel = true;
                    }
                });
            });
        if cancel {
            open = false;
        }
        self.show_settings = open;
        if save {
            self.save_settings(ctx);
            self.show_settings = false;
        }
    }

    pub(crate) fn save_settings(&mut self, ctx: &egui::Context) {
        self.config
            .set("onOpenLastFile", json!(self.settings_open_last));
        self.config
            .set("onPosLastFile", json!(self.settings_pos_last));
        self.config.set("SVP", json!(self.settings_svp));
        self.config
            .set("volumePlus", json!(self.settings_volume_plus));
        self.config
            .set("fixedControlPanel", json!(self.settings_fixed_panel));
        self.config
            .set("askPresetOnOpen", json!(self.settings_ask_preset));
        self.settings_sound_offset = self.settings_sound_offset.clamp(-100.0, 100.0);
        self.config
            .set("soundOffset", json!(self.settings_sound_offset));
        self.config
            .set("palette", json!(self.settings_palette.name()));
        self.config.set("style", json!(self.settings_style.clone()));
        self.config
            .set("colorScheme", json!(self.settings_scheme.name()));
        let lang_value = self.settings_lang.config_value().to_string();
        if get_str(&self.config, "language").as_deref() != Some(lang_value.as_str()) {
            self.config.set("language", json!(lang_value));
            self.lang = if self.settings_lang == Lang::Auto {
                i18n::detect_system_lang()
            } else {
                self.settings_lang
            };
        }
        self.player.volume_plus = self.settings_volume_plus;
        if !self.settings_volume_plus && self.player.volume > 100 {
            self.player.set_volume(&*self.mpv, 100);
        }
        self.mpv.set_audio_delay(self.settings_sound_offset);
        self.mpv.set_svp(self.settings_svp);
        self.apply_style(ctx);
        self.apply_theme(ctx);
        self.flush_config();
    }
}
