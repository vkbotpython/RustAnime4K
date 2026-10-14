//! Воспроизведение: открытие, пресеты Anime4K, опрос mpv.

use super::helpers::{get_bool, missing_shader_files, pick_media_file, shaders_dir};
use super::state::Anime4KPlayerApp;
use super::state::PresetChoice;
use super::state::{FAST_POLL_INTERVAL, MAX_FILE_PRESETS, SLOW_POLL_INTERVAL};
use crate::anime4k;
use crate::mpv::TrackKind;
use serde_json::json;
use std::path::Path;
use std::time::Instant;

impl Anime4KPlayerApp {
    pub(crate) fn do_open_file(&mut self, ctx: &egui::Context) {
        if let Some(path) = pick_media_file() {
            self.player.open_file(&*self.mpv, &path);
            self.set_opened("file", &path.to_string_lossy());
            self.after_file_changed(ctx);
        }
    }

    pub(crate) fn do_open_url(&mut self, ctx: &egui::Context, link: &str) {
        self.player.open_url(&*self.mpv, link);
        self.set_opened("url", link);
        self.after_file_changed(ctx);
    }

    pub(crate) fn do_open_folder(&mut self, ctx: &egui::Context, folder: &Path) {
        self.playlist_visible = true;
        self.player.open_folder(&*self.mpv, folder, None);
        self.set_opened("folder", &folder.to_string_lossy());
        self.after_file_changed(ctx);
    }

    pub(crate) fn do_close(&mut self, ctx: &egui::Context) {
        self.player.close(&*self.mpv);
        self.config.set("opened", json!(["", ""]));
        self.show_preset_offer = false;
        self.pending_preset = None;
        self.eof_handled = false;
        self.snap.duration = None;
        self.snap.time_pos = None;
        self.snap.pause = false;
        self.snap_static_key = None;
        self.tracks_count_cache = None;
        self.tracks_file_key = None;
        self.snap.tracks.clear();
        self.snap.audio_rows.clear();
        self.snap.sub_rows.clear();
        self.refresh_title(ctx);
    }

    pub(crate) fn do_next(&mut self, ctx: &egui::Context) {
        if self.player.next(&*self.mpv) {
            self.after_file_changed(ctx);
        }
    }

    pub(crate) fn do_prev(&mut self, ctx: &egui::Context) {
        if self.player.prev(&*self.mpv) {
            self.after_file_changed(ctx);
        }
    }

    pub(crate) fn do_play_index(&mut self, ctx: &egui::Context, index: usize) {
        if self.player.play_index(&*self.mpv, index) {
            self.after_file_changed(ctx);
        }
    }

    /// Auto-advance the playlist at end-of-file (polls `eof-reached`).
    /// Fires exactly once per EOF episode (see `eof_handled`).
    /// Reads the time-gated snapshot: pausing anywhere else (e.g. mid-VO
    /// reconfig, when live reads can transiently fail) must not skip files.
    pub(crate) fn check_auto_next(&mut self, ctx: &egui::Context) {
        let at_eof = self.player.current.is_some()
            && self.snap.duration.is_some()
            && !self.snap.pause
            && self.snap.eof;
        if !at_eof {
            // Re-arm once EOF clears (a new file is actually playing).
            if !self.snap.eof {
                self.eof_handled = false;
            }
            return;
        }
        if self.eof_handled {
            return;
        }
        self.eof_handled = true;
        if self.player.next(&*self.mpv) {
            self.after_file_changed(ctx);
        }
    }

    /// Storage key for the current file (full path or URL), if any.
    pub(crate) fn current_file_key(&self) -> Option<String> {
        self.player
            .current_file()
            .map(|p| p.to_string_lossy().into_owned())
    }

    /// Compares the current file key without allocating: `to_string_lossy`
    /// borrows for valid UTF-8 paths, so the common "same file" poll tick
    /// costs zero heap allocs (was: one full-path `String` every 100ms).
    pub(crate) fn current_file_key_matches(&self, other: &Option<String>) -> bool {
        match (self.player.current_file(), other) {
            (None, None) => true,
            (Some(p), Some(o)) => p.to_string_lossy().as_ref() == o.as_str(),
            _ => false,
        }
    }

    pub(crate) fn read_file_preset(&self, key: &str) -> Option<PresetChoice> {
        self.config
            .get("filePresets")?
            .get(key)
            .and_then(PresetChoice::from_json)
    }

    pub(crate) fn write_file_preset(&mut self, key: &str, choice: &PresetChoice) {
        // In-place insert, no whole-table clone; bounded so the table
        // cannot grow without limit (one key per opened file).
        self.config
            .set_nested_capped("filePresets", key, choice.to_json(), MAX_FILE_PRESETS);
        self.flush_config();
    }

    /// Applies a stored choice without re-saving it.
    /// Returns false when it can't be built (stale data) or its shader
    /// files are missing (reported in `status`, never applied silently).
    pub(crate) fn apply_choice(&mut self, choice: &PresetChoice) -> bool {
        match choice {
            PresetChoice::Disabled => {
                self.player.disable_anime4k(&*self.mpv);
                true
            }
            PresetChoice::Standard { quality, mode } => {
                match anime4k::create_preset(quality, mode) {
                    Ok(preset) => match self.glsl_value(&preset) {
                        Some(value) => {
                            self.player.apply_preset(
                                &*self.mpv,
                                &format!("{mode} ({quality})"),
                                &value,
                            );
                            true
                        }
                        None => false,
                    },
                    Err(_) => false,
                }
            }
            PresetChoice::Hq { name } => {
                match anime4k::ultra_hq_presets_cached().get(name.as_str()) {
                    Some(preset) => match self.glsl_value(preset) {
                        Some(value) => {
                            self.player
                                .apply_preset(&*self.mpv, &format!("{name} (HQ)"), &value);
                            true
                        }
                        None => false,
                    },
                    None => false,
                }
            }
        }
    }

    /// Builds the `glsl-shaders` value for `preset`, or reports missing
    /// shader files in `status` (visible in the video corner) and returns
    /// `None` so the caller skips the (ineffective) property write.
    pub(crate) fn glsl_value(&mut self, preset: &[String]) -> Option<String> {
        let dir = shaders_dir();
        let missing = missing_shader_files(&dir, preset);
        if missing.is_empty() {
            return Some(anime4k::preset_to_glsl_shaders(&dir, preset));
        }
        self.set_status(format!(
            "{}: Anime4K shaders not found in {} (e.g. {})",
            self.t("Error"),
            dir.display(),
            missing.first().map(String::as_str).unwrap_or("*.glsl"),
        ));
        None
    }

    /// Called whenever the current file changes (or playback starts from
    /// scratch): queues the saved per-file preset for application once
    /// stable, otherwise offers to choose one (unless the user opted out).
    pub(crate) fn note_file_changed(&mut self) {
        // Fresh file: drop stale state so keep-last-good can't leak the
        // previous file's metadata into the new load.
        self.pending_preset = None;
        self.eof_handled = false;
        self.snap.duration = None;
        self.snap.time_pos = None;
        self.snap.pause = self.mpv.pause();
        // Static/track caches belong to the previous file.
        self.snap_static_key = None;
        self.tracks_count_cache = None;
        self.tracks_file_key = None;
        self.snap.tracks.clear();
        self.snap.audio_rows.clear();
        self.snap.sub_rows.clear();
        let Some(key) = self.current_file_key() else {
            self.show_preset_offer = false;
            return;
        };
        let ask = get_bool(&self.config, "askPresetOnOpen", true);
        match self.read_file_preset(&key) {
            Some(choice) => {
                // Applied by flush_pending_preset once metadata is in:
                // writing glsl-shaders mid-load races VO init and wedges
                // playback (frozen video + stalled UI thread).
                self.pending_preset = Some(choice);
                self.show_preset_offer = false;
            }
            None => {
                self.show_preset_offer = ask;
            }
        }
        if self.show_preset_offer {
            self.preset_offer_dont_ask = false;
        }
    }

    /// Refreshes the mpv snapshot in tiers. Between polls the UI reuses
    /// cached values, so 24-60 fps repaint storms during playback cost few
    /// mpv round-trips. Transient read failures keep the last good values
    /// instead of flickering the UI.
    ///
    /// - fast (~10 Hz): `duration` (until known), `time-pos`, `pause`, `eof`.
    /// - static (once per file, retried until metadata answers):
    ///   `video-format`, `audio-codec`, `width`, `height`, `fps`, `hwdec`.
    /// - slow (~0.7 Hz): `volume`, `frame-drop-count`, `paused-for-cache`.
    /// - tracks (menus open only): refetched only when `track-list/count`
    ///   changed or the file changed — never blindly every tick.
    pub(crate) fn poll_mpv(&mut self) {
        let now = Instant::now();
        if self
            .last_poll
            .is_some_and(|last| now.duration_since(last) < FAST_POLL_INTERVAL)
        {
            return;
        }
        self.last_poll = Some(now);
        if self.player.current.is_none() {
            self.snap.duration = None;
            self.snap.time_pos = None;
            self.snap.pause = false;
            self.snap.eof = false;
            return;
        }
        // Fast tier: `duration` proves the core answered — only then is the
        // `pause` read trustworthy enough to overwrite the cached value.
        if let Some(d) = self.mpv.duration() {
            self.snap.duration = Some(d);
            self.snap.pause = self.mpv.pause();
        }
        if let Some(t) = self.mpv.time_pos() {
            self.snap.time_pos = Some(t);
        }
        self.snap.eof = self.mpv.eof_reached();
        // Static tier: properties that never change within one file.
        // Right after `loadfile` they read back as `None`: keep the key
        // unset so the next tick retries instead of caching `None`s.
        // Allocation-free check: the key is cloned only when it changed.
        if !self.current_file_key_matches(&self.snap_static_key) {
            self.snap.video_format = self.mpv.video_format();
            self.snap.audio_codec = self.mpv.audio_codec();
            self.snap.width = self.mpv.width();
            self.snap.height = self.mpv.height();
            self.snap.fps = self.mpv.fps();
            self.snap.hwdec = self.mpv.hwdec_current();
            if self.snap.duration.is_some() {
                self.snap_static_key = self.current_file_key();
            }
        }
        // Slow tier: counters/flags nobody needs at 10 Hz.
        if self
            .last_slow_poll
            .is_none_or(|last| now.duration_since(last) >= SLOW_POLL_INTERVAL)
        {
            self.last_slow_poll = Some(now);
            self.snap.volume = self.mpv.volume();
            self.snap.frame_drops = self.mpv.frame_drop_count();
            self.snap.cache_pause = self.mpv.paused_for_cache();
        }
        // Tracks: menus open only, refetch on count/file change.
        // The last list is kept otherwise (no per-tick clear).
        if self.show_audio_menu || self.show_sub_menu {
            let count = self.mpv.track_count();
            if !self.current_file_key_matches(&self.tracks_file_key)
                || self.tracks_count_cache != Some(count)
            {
                let tracks = self.mpv.tracks();
                // Full labels are prebuilt once per refetch: the menu only
                // iterates `&str` afterwards (zero per-frame `format!`).
                self.snap.audio_rows = tracks
                    .iter()
                    .filter(|t| t.kind == TrackKind::Audio)
                    .map(|t| {
                        let text = t.display();
                        (t.id, format!("{}) {text}", t.id))
                    })
                    .collect();
                self.snap.sub_rows = tracks
                    .iter()
                    .filter(|t| t.kind == TrackKind::Sub)
                    .map(|t| {
                        let text = t.display();
                        (t.id, format!("{}) {text}", t.id))
                    })
                    .collect();
                self.snap.tracks = tracks;
                self.tracks_count_cache = Some(count);
                self.tracks_file_key = self.current_file_key();
            }
        }
    }

    /// Applies a queued preset once playback metadata is available, i.e.
    /// the initial VO setup is done and a shader write can't race it.
    pub(crate) fn flush_pending_preset(&mut self) {
        if self.player.current.is_none() {
            self.pending_preset = None;
            return;
        }
        if self.pending_preset.is_none() || self.snap.duration.is_none() {
            return;
        }
        if self
            .pending_preset
            .take()
            .is_some_and(|choice| !self.apply_choice(&choice))
        {
            // Stale stored data: fall back to offering a choice.
            if get_bool(&self.config, "askPresetOnOpen", true) {
                self.show_preset_offer = true;
            }
        }
    }

    pub(crate) fn after_file_changed(&mut self, ctx: &egui::Context) {
        self.refresh_title(ctx);
        self.note_file_changed();
    }

    pub(crate) fn toggle_play(&mut self) {
        if self.player.current.is_none() {
            return;
        }
        if self.snap.duration.is_none() {
            // No metadata (yet): the file is either still loading or the
            // load died. Restarting the load on every click/Space while it
            // is still loading resets progress forever (flickering play
            // button, video never starts), so replay only when truly idle
            // and just unpause otherwise.
            let replay = self
                .mpv
                .is_idle()
                .then(|| self.player.current_file().cloned())
                .flatten();
            if let Some(file) = replay {
                self.mpv.play(&file.to_string_lossy());
            }
            self.mpv.set_pause(false);
            self.snap.pause = false;
        } else {
            let paused = !self.snap.pause;
            self.mpv.set_pause(paused);
            self.snap.pause = paused;
        }
    }

    /// Explicit user choice: remember a standard preset for the current
    /// file; applied once playback is stable (see `flush_pending_preset`).
    pub(crate) fn apply_anime4k(&mut self, quality: &str, mode: &str) {
        match anime4k::create_preset(quality, mode) {
            Ok(_) => {
                let choice = PresetChoice::Standard {
                    quality: quality.to_string(),
                    mode: mode.to_string(),
                };
                if let Some(key) = self.current_file_key() {
                    self.write_file_preset(&key, &choice);
                }
                self.pending_preset = Some(choice);
            }
            Err(err) => self.set_status(format!("{}: {err:#}", self.t("Error"))),
        }
    }

    /// Explicit user choice: remember an HQ preset for the current file;
    /// applied once playback is stable (see `flush_pending_preset`).
    pub(crate) fn apply_anime4k_hq(&mut self, name: &str) {
        if anime4k::ultra_hq_presets_cached().contains_key(name) {
            let choice = PresetChoice::Hq {
                name: name.to_string(),
            };
            if let Some(key) = self.current_file_key() {
                self.write_file_preset(&key, &choice);
            }
            self.pending_preset = Some(choice);
        }
    }

    /// Explicit user choice: remember "disabled" for the current file;
    /// applied once playback is stable (see `flush_pending_preset`).
    pub(crate) fn disable_anime4k(&mut self) {
        if let Some(key) = self.current_file_key() {
            self.write_file_preset(&key, &PresetChoice::Disabled);
        }
        self.pending_preset = Some(PresetChoice::Disabled);
    }
}
