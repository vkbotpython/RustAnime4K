//! Диалоги открытия URL и папки.

use crate::app::helpers::read_clipboard_text;
use crate::app::state::Anime4KPlayerApp;
use crate::i18n::Lang;
use serde_json::json;
use std::path::Path;

use super::ui_kit::{bottom_bar, card, field_title, hint, text_field};

/// Upper bound for URL/folder MRU histories. `filePresets` already has
/// `MAX_FILE_PRESETS=500`; these two arrays had no cap and were cloned
/// whole every frame while the dialog was open, so they grew without bound.
pub(crate) const MAX_HISTORY_ENTRIES: usize = 100;

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
        let russian = self.lang == Lang::Russian;
        let mut open = self.show_open_url;
        let mut chosen: Option<String> = None;
        let mut cancelled = false;
        // Bounded clone: storage is capped at `MAX_HISTORY_ENTRIES` on every
        // write (see `push_history_capped`), so this per-frame copy stays
        // ~10KB instead of growing without limit.
        let history = self
            .config
            .get("linksHistory")
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .min_width(500.0)
            .show(ctx, |ui| {
                card(ui, |ui| {
                    field_title(ui, prompt);
                    text_field(ui, &mut self.url_text, "https://…");
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
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
                    hint(
                        ui,
                        if russian {
                            "Вставьте ссылку или выберите из истории ниже."
                        } else {
                            "Paste a link or pick one from the history below."
                        },
                    );
                });
                card(ui, |ui| {
                    field_title(
                        ui,
                        if russian {
                            "Недавние ссылки"
                        } else {
                            "Recent links"
                        },
                    );
                    Self::history_list(
                        ui,
                        history.iter().filter_map(|v| v.as_str()),
                        if russian {
                            "Пока пусто"
                        } else {
                            "Empty so far"
                        },
                        &mut self.url_text,
                    );
                });
                bottom_bar(ui, |ui| {
                    if ui.button(go).clicked() {
                        let link = self.url_text.trim().to_string();
                        if !link.is_empty() {
                            // In-place MRU update, capped — no whole-array
                            // clone/retain/insert without `truncate`.
                            self.config.push_history_capped(
                                "linksHistory",
                                &link,
                                MAX_HISTORY_ENTRIES,
                            );
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
        let browse = self.t("Select");
        let err = self.t("Error");
        let russian = self.lang == Lang::Russian;
        let mut open = self.show_open_folder;
        let mut chosen: Option<String> = None;
        let mut cancelled = false;
        // Bounded clone, same as the URL dialog above.
        let history = self
            .config
            .get("foldersHistory")
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .min_width(500.0)
            .show(ctx, |ui| {
                card(ui, |ui| {
                    field_title(ui, prompt);
                    text_field(
                        ui,
                        &mut self.folder_text,
                        if russian {
                            "Путь к папке с видео…"
                        } else {
                            "Path to a video folder…"
                        },
                    );
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        let dir = ui
                            .button(browse)
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
                    hint(
                        ui,
                        if russian {
                            "Укажите путь вручную, вставьте из буфера или выберите обзором."
                        } else {
                            "Type a path, paste it, or pick with the file browser."
                        },
                    );
                });
                card(ui, |ui| {
                    field_title(
                        ui,
                        if russian {
                            "Недавние папки"
                        } else {
                            "Recent folders"
                        },
                    );
                    Self::history_list(
                        ui,
                        history.iter().filter_map(|v| v.as_str()),
                        if russian {
                            "Пока пусто"
                        } else {
                            "Empty so far"
                        },
                        &mut self.folder_text,
                    );
                });
                bottom_bar(ui, |ui| {
                    if ui.button(go).clicked() {
                        let folder = self.folder_text.trim().to_string();
                        if !folder.is_empty() {
                            self.config.push_history_capped(
                                "foldersHistory",
                                &folder,
                                MAX_HISTORY_ENTRIES,
                            );
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

    /// История открытий: пустое состояние-подсказка либо прокручиваемый
    /// список ссылок; клик подставляет значение в поле ввода.
    fn history_list<'a>(
        ui: &mut egui::Ui,
        items: impl Iterator<Item = &'a str>,
        empty_hint: &str,
        target: &mut String,
    ) {
        // No `Vec<&str>` collect: the old version allocated O(N) pointers
        // every frame while the dialog was open. `Peekable` checks emptiness
        // without any allocation.
        let mut items = items.peekable();
        if items.peek().is_none() {
            hint(ui, empty_hint);
            return;
        }
        egui::ScrollArea::vertical()
            .max_height(150.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                for item in items {
                    if ui.link(item).clicked() {
                        *target = item.to_string();
                    }
                }
            });
    }
}
