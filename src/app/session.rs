//! Сессия: восстановление при старте и сохранение при выходе.

use super::helpers::{get_bool, get_str, parse_launch_params};
use super::state::Anime4KPlayerApp;
use crate::player::MIN_RESTORE_POSITION;
use serde_json::{Value, json};
use std::path::Path;

impl Anime4KPlayerApp {
    pub(crate) fn restore_startup(&mut self, cli_arg: Option<String>) {
        if let Some(arg) = cli_arg {
            // CLI arg wins, no last-file restore.
            let path = Path::new(&arg);
            if path.is_file() {
                self.player.open_file(&*self.mpv, path);
            } else if path.is_dir() {
                self.playlist_visible = true;
                self.player.open_folder(&*self.mpv, path, None);
            } else {
                self.player.open_url(&*self.mpv, &arg);
            }
            self.apply_runtime_settings();
            self.note_file_changed();
            return;
        }
        let open_last = get_bool(&self.config, "onOpenLastFile", true);
        let pos_last = get_bool(&self.config, "onPosLastFile", true);
        let mut position = self
            .config
            .get("position")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        if !pos_last {
            position = 0.0;
        }
        let opened = open_last
            .then(|| self.config.get("opened").and_then(Value::as_array))
            .flatten();
        if let Some(opened) = opened {
            let kind = opened.first().and_then(Value::as_str).unwrap_or("");
            let target = opened.get(1).and_then(Value::as_str).unwrap_or("");
            match kind {
                "file" if Path::new(target).exists() => {
                    self.player.open_file(&*self.mpv, Path::new(target));
                    self.mpv.set_pause(true);
                }
                "url" if !target.is_empty() => {
                    self.player.open_url(&*self.mpv, target);
                    self.mpv.set_pause(true);
                }
                "folder" if Path::new(target).is_dir() => {
                    let file = get_str(&self.config, "file").unwrap_or_default();
                    let file = (!file.is_empty())
                        .then(|| Path::new(&file))
                        .filter(|p| p.exists());
                    self.playlist_visible = true;
                    self.player.open_folder(&*self.mpv, Path::new(target), file);
                    self.mpv.set_pause(true);
                }
                _ => {}
            }
            if position >= MIN_RESTORE_POSITION {
                self.mpv.set_time_pos(position);
            }
        }
        self.apply_runtime_settings();
        self.note_file_changed();
    }

    pub(crate) fn apply_runtime_settings(&mut self) {
        self.mpv.set_volume(self.player.volume);
        self.mpv.set_svp(get_bool(&self.config, "SVP", false));
        let offset = self
            .config
            .get("soundOffset")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        if offset != 0.0 {
            self.mpv.set_audio_delay(offset);
        }
        let params = get_str(&self.config, "launchParameters").unwrap_or_default();
        for (name, value) in parse_launch_params(&params) {
            self.mpv.set_option_str(&name, &value);
        }
        // Remembered screenshot folder (see `screenshot_dialog`): re-point
        // mpv so F12 shots land there without reopening the dialog.
        let shot_dir = get_str(&self.config, "screenshotDirectory").unwrap_or_default();
        if !shot_dir.trim().is_empty() {
            let _ = self.mpv.set_screenshot_directory(shot_dir.trim());
        }
    }

    pub(crate) fn save_on_exit(&mut self) {
        // Persist current file + position for session restore.
        if let Some(file) = self.player.current_file() {
            self.config
                .set("file", Value::String(file.to_string_lossy().into_owned()));
        } else {
            self.config.set("file", Value::Null);
        }
        match self.mpv.time_pos() {
            Some(pos) if pos >= MIN_RESTORE_POSITION => {
                self.config.set("position", json!(pos));
            }
            _ => self.config.set("position", Value::Null),
        }
        // Dialog drafts that have no explicit Save (or were typed but never
        // saved) must survive a restart too: sync them before writing.
        self.config.set(
            "screenshotDirectory",
            json!(self.screenshot_dir_text.trim()),
        );
        self.config
            .set("androidPath", json!(self.android_path.clone()));
        self.config
            .set("launchParameters", json!(self.launch_params_text.clone()));
        // Closed while in borderless fullscreen (`track_window_geometry`
        // skips it): reopen as a plain maximized window, never fullscreen.
        if self.player.fullscreen {
            self.config.set("windowMaximized", json!(true));
        }
        if let Err(err) = self.config.save() {
            eprintln!("cannot save config: {err:#}");
        }
    }

    pub(crate) fn set_opened(&mut self, kind: &str, target: &str) {
        self.config.set("opened", json!([kind, target]));
        self.flush_config();
    }
}
