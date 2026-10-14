//! Диалог скриншотов.

use crate::app::helpers::read_clipboard_text;
use crate::app::state::Anime4KPlayerApp;
use crate::app::state::ScreenshotWatch;
use crate::i18n::Lang;
use egui::RichText;
use serde_json::json;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::ui_kit::{card, field_title, hint, text_field};

impl Anime4KPlayerApp {
    /// Non-blocking completion check for an armed screenshot watch.
    /// Throttled to `SCREENSHOT_POLL_INTERVAL`: called every frame, but
    /// `read_dir` runs at most 4x/sec instead of 60x/sec.
    pub(crate) fn poll_screenshot(&mut self) {
        use super::super::state::SCREENSHOT_POLL_INTERVAL;

        let Some(mut watch) = self.screenshot_pending.take() else {
            return;
        };
        let now = Instant::now();
        // Deadline first: no more polling past it.
        if now >= watch.deadline {
            self.set_status(format!("{} (mpv.log)", self.t("Error")));
            return;
        }
        // Throttle filesystem polls; first check runs immediately.
        if watch
            .last_check
            .is_some_and(|last| now.duration_since(last) < SCREENSHOT_POLL_INTERVAL)
        {
            self.screenshot_pending = Some(watch);
            return;
        }
        watch.last_check = Some(now);
        if let Some(name) = crate::mpv::find_new_file_set(&watch.before, &watch.dir) {
            self.set_status(format!("{}: {name}", self.t("Screenshot")));
        } else {
            self.screenshot_pending = Some(watch);
        }
    }

    pub(crate) fn screenshot_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_screenshot {
            return;
        }
        let title = self.t("Screenshot");
        let prompt = self.t("Enter folder path for screenshots");
        let save = self.t("Save");
        let paste = self.t("Paste");
        let browse = self.t("Select");
        let take_shot = self.t("Take a screenshot");
        let err = self.t("Error");
        let russian = self.lang == Lang::Russian;
        let mut open = self.show_screenshot;
        let mut done = false;
        let mut failed = false;
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .min_width(480.0)
            .show(ctx, |ui| {
                card(ui, |ui| {
                    field_title(ui, prompt);
                    text_field(
                        ui,
                        &mut self.screenshot_dir_text,
                        if russian {
                            "Папка для снимков…"
                        } else {
                            "Screenshots folder…"
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
                            self.screenshot_dir_text = dir.to_string_lossy().into_owned();
                        }
                        if ui.button(paste).clicked() {
                            let text = read_clipboard_text();
                            if !text.trim().is_empty() {
                                self.screenshot_dir_text = text.trim().to_string();
                            }
                        }
                        // "Save" only remembers the folder (no capture here):
                        // the shot itself is taken by the dedicated button below.
                        if ui.button(save).clicked() {
                            let dir = self.screenshot_dir_text.trim().to_string();
                            if dir.is_empty() {
                                failed = true;
                            } else if let Err(e) = self.mpv.set_screenshot_directory(&dir) {
                                eprintln!("screenshot directory failed: {e:#}");
                                failed = true;
                            } else {
                                self.config.set("screenshotDirectory", json!(dir));
                                self.flush_config();
                                done = true;
                            }
                        }
                    });
                    hint(
                        ui,
                        if russian {
                            "«Сохранить» только запоминает папку без съёмки кадра."
                        } else {
                            "“Save” only remembers the folder without capturing."
                        },
                    );
                });
                card(ui, |ui| {
                    field_title(
                        ui,
                        if russian {
                            "Съёмка кадра"
                        } else {
                            "Capture"
                        },
                    );
                    // Explicit capture: snapshot the folder contents BEFORE
                    // dispatching, otherwise the new file is already in `before`
                    // and `poll_screenshot` can never spot it.
                    if ui
                        .add_sized(
                            egui::vec2(ui.available_width(), 30.0),
                            egui::Button::new(RichText::new(take_shot).strong()),
                        )
                        .clicked()
                    {
                        let dir = self.screenshot_dir_text.trim().to_string();
                        if dir.is_empty() {
                            failed = true;
                        } else {
                            let dir_path = PathBuf::from(&dir);
                            let before =
                                crate::mpv::dir_file_names(&dir_path).into_iter().collect();
                            let dispatched = self
                                .mpv
                                .set_screenshot_directory(&dir)
                                .and_then(|()| self.mpv.screenshot());
                            match dispatched {
                                Ok(()) => {
                                    self.config.set("screenshotDirectory", json!(dir));
                                    self.flush_config();
                                    self.screenshot_pending = Some(ScreenshotWatch {
                                        dir: dir_path,
                                        before,
                                        deadline: Instant::now() + Duration::from_secs(5),
                                        last_check: None,
                                    });
                                }
                                Err(e) => {
                                    eprintln!("screenshot failed: {e:#}");
                                    failed = true;
                                }
                            }
                        }
                    }
                    hint(
                        ui,
                        if russian {
                            "Файл появится в указанной папке, имя задаёт mpv."
                        } else {
                            "The file lands in that folder with an mpv-chosen name."
                        },
                    );
                });
            });
        if failed {
            self.set_status(err.to_owned());
        }
        self.show_screenshot = open && !done;
    }
}
