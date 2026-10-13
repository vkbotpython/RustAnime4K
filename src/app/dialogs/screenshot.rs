//! Диалог скриншотов.

use crate::app::helpers::read_clipboard_text;
use crate::app::state::Anime4KPlayerApp;
use crate::app::state::ScreenshotWatch;
use serde_json::json;
use std::path::PathBuf;
use std::time::{Duration, Instant};

impl Anime4KPlayerApp {
    /// Non-blocking completion check for an armed screenshot watch.
    /// Called every frame: reports the landed file by name, or — past the
    /// deadline — points at `mpv.log` instead of pretending it worked.
    pub(crate) fn poll_screenshot(&mut self) {
        let Some(watch) = self.screenshot_pending.take() else {
            return;
        };
        if let Some(name) = crate::mpv::find_new_file(&watch.before, &watch.dir) {
            self.set_status(format!("{}: {name}", self.t("Screenshot")));
        } else if Instant::now() >= watch.deadline {
            self.set_status(format!("{} (mpv.log)", self.t("Error")));
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
        let take_shot = self.t("Take a screenshot");
        let err = self.t("Error");
        let mut open = self.show_screenshot;
        let mut done = false;
        let mut failed = false;
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .min_width(460.0)
            .show(ctx, |ui| {
                ui.label(prompt);
                ui.text_edit_singleline(&mut self.screenshot_dir_text);
                ui.horizontal(|ui| {
                    let dir = ui
                        .button(self.t("Select"))
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
                // Explicit capture: snapshot the folder contents BEFORE
                // dispatching, otherwise the new file is already in `before`
                // and `poll_screenshot` can never spot it.
                if ui.button(take_shot).clicked() {
                    let dir = self.screenshot_dir_text.trim().to_string();
                    if dir.is_empty() {
                        failed = true;
                    } else {
                        let dir_path = PathBuf::from(&dir);
                        let before = crate::mpv::dir_file_names(&dir_path);
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
                                });
                            }
                            Err(e) => {
                                eprintln!("screenshot failed: {e:#}");
                                failed = true;
                            }
                        }
                    }
                }
            });
        if failed {
            self.set_status(err);
        }
        self.show_screenshot = open && !done;
    }
}
