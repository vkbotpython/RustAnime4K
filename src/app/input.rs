//! Ввод: горячие клавиши, мышь, fullscreen, drag & drop.

use super::helpers::is_symlink;
use super::state::Anime4KPlayerApp;
use super::state::{
    DOUBLE_CLICK_REVERT_WINDOW, FULLSCREEN_BOTTOM_ZONE, FULLSCREEN_CURSOR_IDLE,
    FULLSCREEN_RIGHT_ZONE,
};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::time::{Duration, Instant};

impl Anime4KPlayerApp {
    pub(crate) fn track_mouse(&mut self, ctx: &egui::Context) {
        let pos = ctx.input(|i| i.pointer.latest_pos());
        if pos != self.last_mouse_pos {
            self.last_mouse_pos = pos;
            self.last_mouse_time = Instant::now();
        }
    }

    pub(crate) fn mouse_idle(&self) -> bool {
        self.last_mouse_time.elapsed() > FULLSCREEN_CURSOR_IDLE
    }

    pub(crate) fn mouse_in_bottom_zone(&self, ctx: &egui::Context) -> bool {
        let rect = ctx.viewport_rect();
        match self.last_mouse_pos {
            Some(p) => p.y > rect.bottom() - FULLSCREEN_BOTTOM_ZONE,
            None => false,
        }
    }

    pub(crate) fn mouse_in_right_zone(&self, ctx: &egui::Context) -> bool {
        let rect = ctx.viewport_rect();
        match self.last_mouse_pos {
            Some(p) => p.x > rect.right() - FULLSCREEN_RIGHT_ZONE,
            None => false,
        }
    }

    /// Global shortcuts (Ctrl+O/Ctrl+U/F1/F2/F4/F5/F6/F11/F12/Space/
    /// D/0/1/2/3/arrows/+/-). Skipped for single-key shortcuts while typing
    /// in a text field so text entry keeps working.
    pub(crate) fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        use egui::{Key, Modifiers};
        let typing = ctx.egui_wants_keyboard_input();
        let pressed = |key: Key, mods: Modifiers| {
            ctx.input(|i| i.key_pressed(key) && i.modifiers.matches_logically(mods))
        };

        // Modifier/function shortcuts work even while typing.
        if pressed(Key::O, Modifiers::CTRL) {
            self.do_open_file(ctx);
            return;
        }
        if pressed(Key::U, Modifiers::CTRL) {
            self.url_text.clear();
            self.show_open_url = true;
            return;
        }
        if pressed(Key::F2, Modifiers::NONE) {
            self.folder_text.clear();
            self.show_open_folder = true;
            return;
        }
        if pressed(Key::F4, Modifiers::ALT) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if pressed(Key::F4, Modifiers::NONE) {
            self.do_close(ctx);
            return;
        }
        if pressed(Key::F5, Modifiers::NONE) {
            self.open_settings();
            return;
        }
        if pressed(Key::F11, Modifiers::NONE) {
            self.toggle_fullscreen(ctx);
            return;
        }
        if pressed(Key::F12, Modifiers::NONE) {
            self.show_screenshot = true;
            return;
        }
        if pressed(Key::F1, Modifiers::NONE) {
            self.show_about = true;
            return;
        }
        if pressed(Key::F6, Modifiers::NONE) {
            self.playlist_visible = !self.playlist_visible;
            return;
        }
        if pressed(Key::Escape, Modifiers::NONE) {
            if self.player.fullscreen {
                self.toggle_fullscreen(ctx);
            } else {
                self.show_settings = false;
                self.show_about = false;
                self.show_open_url = false;
                self.show_open_folder = false;
                self.show_screenshot = false;
                self.show_launch_params = false;
                self.show_android = false;
                self.show_reference = false;
            }
            return;
        }
        if typing {
            return;
        }
        if pressed(Key::Space, Modifiers::NONE) {
            self.toggle_play();
            return;
        }
        if pressed(Key::D, Modifiers::NONE) {
            self.disable_anime4k();
            return;
        }
        if pressed(Key::Num0, Modifiers::NONE) {
            self.mpv.set_speed(0.25);
            return;
        }
        if pressed(Key::Num1, Modifiers::NONE) {
            self.mpv.set_speed(1.0);
            return;
        }
        if pressed(Key::Num2, Modifiers::NONE) {
            self.mpv.set_speed(2.0);
            return;
        }
        if pressed(Key::Num3, Modifiers::NONE) {
            self.mpv.set_speed(3.0);
            return;
        }
        if pressed(Key::ArrowRight, Modifiers::NONE) {
            self.mpv.seek(5.0);
            return;
        }
        if pressed(Key::ArrowLeft, Modifiers::NONE) {
            self.mpv.seek(-5.0);
            return;
        }
        if pressed(Key::ArrowUp, Modifiers::NONE) {
            self.player.volume_up(&*self.mpv);
            self.config.set("volume", json!(self.player.volume));
            self.flush_config();
            return;
        }
        if pressed(Key::ArrowDown, Modifiers::NONE) {
            self.player.volume_down(&*self.mpv);
            self.config.set("volume", json!(self.player.volume));
            self.flush_config();
            return;
        }
        if pressed(Key::Plus, Modifiers::NONE)
            || pressed(Key::Equals, Modifiers::NONE)
            || pressed(Key::Plus, Modifiers::SHIFT)
        {
            self.mpv.set_video_zoom(self.mpv.video_zoom() + 0.01);
            return;
        }
        if pressed(Key::Minus, Modifiers::NONE) {
            self.mpv.set_video_zoom(self.mpv.video_zoom() - 0.01);
        }
    }

    pub(crate) fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        let mut first_new = true;
        for path in dropped {
            // Symlinks open as URLs.
            if is_symlink(&path) {
                let link = path.to_string_lossy().into_owned();
                self.do_open_url(ctx, &link);
                break;
            } else if crate::formats::is_subtitles_file(&path.to_string_lossy()) && path.is_file() {
                self.mpv.add_sub_file(&path.to_string_lossy());
            } else if path.is_file() {
                if first_new {
                    self.player.open_file(&*self.mpv, &path);
                    self.set_opened("file", &path.to_string_lossy());
                    self.after_file_changed(ctx);
                    first_new = false;
                } else {
                    self.player.add_file(path);
                }
            } else if path.is_dir() {
                self.do_open_folder(ctx, &path);
                break;
            }
        }
    }

    pub(crate) fn toggle_fullscreen(&mut self, ctx: &egui::Context) {
        self.player.fullscreen = !self.player.fullscreen;
        // Borderless fullscreen covers the monitor with no OS frame by
        // itself. Do NOT combine with `Decorations(false)`: restyling the
        // window while it is fullscreen confuses DWM and leaves a broken
        // black border/shadow around the window.
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.player.fullscreen));
    }

    /// A double click begins life as a single click that already flipped
    /// play/pause (see `video_panel`): while that flip is still fresh,
    /// report true so the caller flips it back — fullscreen then never
    /// disturbs playback, and single clicks stay instant (no hold delay).
    pub(crate) fn consume_click_revert(&mut self) -> bool {
        let fresh = matches!(
            self.click_play_armed_at,
            Some(t) if t.elapsed() < DOUBLE_CLICK_REVERT_WINDOW
        );
        self.click_play_armed_at = None;
        fresh
    }

    /// Remembers window position/size (windowed mode only) so the next
    /// launch restores them. Recorded in memory every frame, flushed to
    /// disk at most every few seconds (plus once more on exit).
    pub(crate) fn track_window_geometry(&mut self, ctx: &egui::Context) {
        if self.player.fullscreen {
            return;
        }
        let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        if maximized {
            // Keep the last windowed rect untouched, remember only the
            // maximized state itself.
            let already = self
                .config
                .get("windowMaximized")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if !already {
                self.config.set("windowMaximized", json!(true));
                self.flush_config();
            }
            return;
        }
        let (pos, size) = ctx.input(|i| (i.viewport().outer_rect, i.viewport().inner_rect));
        let (Some(outer), Some(inner)) = (pos, size) else {
            return;
        };
        let (x, y) = (outer.min.x as f64, outer.min.y as f64);
        let (w, h) = (inner.width() as f64, inner.height() as f64);
        if !(x.is_finite() && y.is_finite() && w > 0.0 && h > 0.0) {
            return;
        }
        let get = |k: &str| self.config.get(k).and_then(Value::as_f64);
        let close = |a: Option<f64>, b: f64| a.is_some_and(|v| (v - b).abs() < 0.5);
        let maxed = self
            .config
            .get("windowMaximized")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if maxed
            || !close(get("windowX"), x)
            || !close(get("windowY"), y)
            || !close(get("windowW"), w)
            || !close(get("windowH"), h)
        {
            self.config.set("windowX", json!(x));
            self.config.set("windowY", json!(y));
            self.config.set("windowW", json!(w));
            self.config.set("windowH", json!(h));
            self.config.set("windowMaximized", json!(false));
            let now = Instant::now();
            if self
                .last_window_save
                .is_none_or(|t| now.duration_since(t) >= Duration::from_secs(2))
            {
                self.last_window_save = Some(now);
                self.flush_config();
            }
        }
    }
}
