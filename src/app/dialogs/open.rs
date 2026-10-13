//! Диалоги открытия URL и папки.

use crate::app::helpers::read_clipboard_text;
use crate::app::state::Anime4KPlayerApp;
use serde_json::{Value, json};
use std::path::Path;

impl Anime4KPlayerApp {
    pub(crate) fn open_url_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_open_url {
            return;
        }
        let title = self.t("Opening a link");
        let prompt = self.t("Enter the URL");
        let clear = self.t("Clear");
        let paste = self.t("Paste");
        let go = self.t("Select");
        let cancel = self.t("Cancel");
        let mut open = self.show_open_url;
        let mut chosen: Option<String> = None;
        let mut cancelled = false;
        let history = self
            .config
            .get("linksHistory")
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .min_width(480.0)
            .show(ctx, |ui| {
                ui.label(prompt);
                ui.text_edit_singleline(&mut self.url_text);
                ui.horizontal(|ui| {
                    if ui.button(paste).clicked() {
                        let text = read_clipboard_text();
                        if !text.trim().is_empty() {
                            self.url_text = text.trim().to_string();
                        }
                    }
                    if ui.button(clear).clicked() {
                        self.config.set("linksHistory", json!(Vec::<String>::new()));
                        self.flush_config();
                    }
                });
                for link in history.iter().filter_map(|v| v.as_str()) {
                    if ui.link(link).clicked() {
                        self.url_text = link.to_string();
                    }
                }
                ui.horizontal(|ui| {
                    if ui.button(go).clicked() {
                        let link = self.url_text.trim().to_string();
                        if !link.is_empty() {
                            let mut updated = history.clone();
                            updated.retain(|v| v.as_str() != Some(link.as_str()));
                            updated.insert(0, Value::String(link.clone()));
                            self.config.set("linksHistory", Value::Array(updated));
                            self.flush_config();
                            chosen = Some(link);
                        }
                    }
                    if ui.button(cancel).clicked() {
                        cancelled = true;
                    }
                });
            });
        if cancelled {
            open = false;
        }
        self.show_open_url = open;
        if let Some(link) = chosen {
            self.do_open_url(ctx, &link);
            self.show_open_url = false;
        }
    }

    pub(crate) fn open_folder_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_open_folder {
            return;
        }
        let title = self.t("Opening a folder");
        let prompt = self.t("Select a folder");
        let clear = self.t("Clear");
        let paste = self.t("Paste");
        let go = self.t("Select");
        let cancel = self.t("Cancel");
        let err = self.t("Error");
        let mut open = self.show_open_folder;
        let mut chosen: Option<String> = None;
        let mut cancelled = false;
        let history = self
            .config
            .get("foldersHistory")
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .min_width(480.0)
            .show(ctx, |ui| {
                ui.label(prompt);
                ui.text_edit_singleline(&mut self.folder_text);
                ui.horizontal(|ui| {
                    let dir = ui
                        .button(self.t("Select"))
                        .clicked()
                        .then(|| rfd::FileDialog::new().pick_folder())
                        .flatten();
                    if let Some(dir) = dir {
                        self.folder_text = dir.to_string_lossy().into_owned();
                    }
                    if ui.button(paste).clicked() {
                        let text = read_clipboard_text();
                        if !text.trim().is_empty() {
                            self.folder_text = text.trim().to_string();
                        }
                    }
                    if ui.button(clear).clicked() {
                        self.config
                            .set("foldersHistory", json!(Vec::<String>::new()));
                        self.flush_config();
                    }
                });
                for folder in history.iter().filter_map(|v| v.as_str()) {
                    if ui.link(folder).clicked() {
                        self.folder_text = folder.to_string();
                    }
                }
                ui.horizontal(|ui| {
                    if ui.button(go).clicked() {
                        let folder = self.folder_text.trim().to_string();
                        if !folder.is_empty() {
                            let mut updated = history.clone();
                            updated.retain(|v| v.as_str() != Some(folder.as_str()));
                            updated.insert(0, Value::String(folder.clone()));
                            self.config.set("foldersHistory", Value::Array(updated));
                            self.flush_config();
                            chosen = Some(folder);
                        }
                    }
                    if ui.button(cancel).clicked() {
                        cancelled = true;
                    }
                });
            });
        if cancelled {
            open = false;
        }
        self.show_open_folder = open;
        if let Some(folder) = chosen {
            let path = Path::new(&folder);
            if path.is_dir() {
                self.do_open_folder(ctx, path);
            } else {
                self.set_status(format!("{err}: {folder}"));
            }
            self.show_open_folder = false;
        }
    }
}
