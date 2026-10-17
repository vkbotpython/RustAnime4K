//! Диалог параметров запуска.

use crate::app::helpers::{get_str, parse_launch_params};
use crate::app::state::Anime4KPlayerApp;
use serde_json::json;

use super::ui_kit::{card, hint, para};

impl Anime4KPlayerApp {
    /// Copies saved launch parameters into the draft and opens the dialog.
    pub(crate) fn open_launch(&mut self) {
        self.launch_params_text = get_str(&self.config, "launchParameters").unwrap_or_default();
        self.show_launch_params = true;
    }

    pub(crate) fn launch_params_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_launch_params {
            return;
        }
        let title = self.t("Launch parameters");
        let prompt = self.t("Manual launch parameters");
        let save_label = self.t("Save");
        let apply_label = self.t("Apply");
        let cancel_label = self.t("Cancel");
        let success = self.t("Success");
        let err = self.t("Error");
        let mut open = self.show_launch_params;
        let mut saved = false;
        let mut cancelled = false;
        let mut applied_ok: Option<bool> = None;
        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .min_width(440.0)
            .default_size([540.0, 420.0])
            .show(ctx, |ui| {
                card(ui, |ui| {
                    para(ui, prompt);
                    ui.monospace("volume=50");
                    hint(ui, "name=value");
                });
                // Top-down order with the editor LAST, filling the rest of
                // the window: buttons first, then the stretching editor.
                // Anything else feeds back into the auto-sized window: a
                // fill-sized editor with widgets after it (or a `bottom_up`
                // region) makes the content larger than the window every
                // frame until the screen edge, while a fixed editor snaps a
                // manual resize back. Laid out last, the editor is exactly
                // the remaining space at any window size: stable on open,
                // stretchable by dragging.
                ui.horizontal(|ui| {
                    if ui.button(apply_label).clicked() {
                        let mut ok = true;
                        // Borrow, don't clone the whole draft for parsing.
                        for (name, value) in parse_launch_params(&self.launch_params_text) {
                            if name == "volume" {
                                if let Ok(v) = value.parse::<i32>() {
                                    self.player.set_volume(&*self.mpv, v);
                                } else {
                                    ok = false;
                                }
                            } else {
                                self.mpv.set_option_str(&name, &value);
                            }
                        }
                        saved = true;
                        applied_ok = Some(ok);
                    }
                    if ui.button(save_label).clicked() {
                        saved = true;
                    }
                    if ui.button(cancel_label).clicked() {
                        // Discard the draft on Cancel.
                        cancelled = true;
                    }
                });
                ui.add_space(4.0);
                let avail = ui.available_size();
                let resp = ui.add_sized(
                    egui::vec2(avail.x, avail.y.max(120.0)),
                    egui::TextEdit::multiline(&mut self.launch_params_text)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("name=value"),
                );
                crate::theme::focus_underline(ui, &resp);
            });
        if cancelled {
            self.launch_params_text = get_str(&self.config, "launchParameters").unwrap_or_default();
            open = false;
        }
        if saved {
            self.config
                .set("launchParameters", json!(self.launch_params_text.clone()));
            self.flush_config();
        }
        match applied_ok {
            Some(true) => self.set_status(success.to_owned()),
            Some(false) => self.set_status(err.to_owned()),
            None => {}
        }
        self.show_launch_params = open;
    }
}
