//! Настоящий бэкенд libmpv (только с фичей `video`).

use super::{MpvBackend, Track, TrackKind};

/// Real libmpv backend, available with `--features video`.
///
/// Property names follow libmpv (`glsl-shaders`, `time-pos`, ...).
/// Only needs `LC_NUMERIC=C`; libmpv2 handles the rest.
#[cfg(feature = "video")]
pub struct LibMpv {
    inner: std::sync::Arc<libmpv2::Mpv>,
}

#[cfg(feature = "video")]
impl LibMpv {
    pub fn new() -> anyhow::Result<Self> {
        // libmpv requires LC_NUMERIC=C.
        #[cfg(unix)]
        unsafe {
            libc_set_numeric_c();
        }
        let mpv = libmpv2::Mpv::with_initializer(|init| {
            init.set_option("audio-display", "no")?;
            init.set_option("keep-open", "yes")?;
            init.set_option("profile", "gpu-hq")?;
            // No terminal UI: this is a GUI app without a console window,
            // so mpv must not print status lines / logs to stdout.
            init.set_option("terminal", "no")?;
            // Playback diagnostics: mpv's own log (VO/hwdec choice, shader
            // errors, AO underruns, dropped frames) next to the exe.
            // The GUI build has no console (windows_subsystem), so without
            // this file stutter causes are invisible. Fresh file every
            // start + capped while running (see `cap_mpv_log`): mpv only
            // appends, so otherwise the log grows without bound. A failed
            // write must not fail init.
            if let Some(path) = mpv_log_path() {
                let _ = std::fs::write(&path, "");
                let log_value = path.to_string_lossy().into_owned();
                let _ = init.set_option("log-file", log_value.as_str());
            }
            // Quieter log: warnings/errors plus *what* is playing and with
            // which VO (`cplayer`); routine per-chunk info no longer
            // inflates the file.
            let _ = init.set_option("msg-level", "all=warn,cplayer=info");
            // Non-blocking render(): with the default timing offset render()
            // sleeps until the target display time, and since it runs on the
            // UI thread the whole window would tick at video rate (~41 ms
            // per frame at 24 fps — laggy menus, slider and clicks). With
            // offset 0 render() returns the latest frame immediately; pacing
            // comes from the update-callback repaints (video rate) plus
            // eframe vsync (display rate). A/V sync is unaffected: the audio
            // clock drives decoding, `report_swap()` reports presents.
            // (An earlier revision blamed this option for frame-by-frame
            // playback — the actual cause was a seek storm from the timeline
            // slider's `step_by` + `changed()`; seeks are gesture-gated now.)
            init.set_option("video-timing-offset", "0")?;
            // URL playback; ignore the error
            // on builds without youtube-dl support so init still succeeds.
            let _ = init.set_option("ytdl", "yes");
            // Render API mode: frames go to our egui surface (see video.rs).
            init.set_option("vo", "libmpv")?;
            // Hardware decoding with software fallback.
            init.set_option("hwdec", "auto-safe")?;
            Ok(())
        })?;
        Ok(Self {
            inner: std::sync::Arc::new(mpv),
        })
    }

    fn get_opt<T: libmpv2::GetData>(&self, name: &str) -> Option<T> {
        self.inner.get_property(name).ok()
    }
}

#[cfg(feature = "video")]
#[cfg(unix)]
unsafe fn libc_set_numeric_c() {
    unsafe {
        // setlocale(LC_NUMERIC=4, "C")
        unsafe extern "C" {
            fn setlocale(
                category: i32,
                locale: *const std::os::raw::c_char,
            ) -> *mut std::os::raw::c_char;
        }
        setlocale(4, c"C".as_ptr());
    }
}

#[cfg(feature = "video")]
impl MpvBackend for LibMpv {
    fn play(&self, path: &str) {
        let _ = self.inner.command("loadfile", &[path]);
    }
    fn stop(&self) {
        let _ = self.inner.command("stop", &[]);
    }
    fn set_pause(&self, pause: bool) {
        let _ = self.inner.set_property("pause", pause);
    }
    fn pause(&self) -> bool {
        self.get_opt("pause").unwrap_or(false)
    }
    fn set_volume(&self, volume: i32) {
        let _ = self.inner.set_property("volume", volume as i64);
    }
    fn volume(&self) -> i32 {
        self.get_opt::<i64>("volume").unwrap_or(80) as i32
    }
    fn time_pos(&self) -> Option<f64> {
        self.get_opt("time-pos")
    }
    fn set_time_pos(&self, pos: f64) {
        let _ = self.inner.set_property("time-pos", pos);
    }
    fn duration(&self) -> Option<f64> {
        self.get_opt("duration")
    }
    fn seek(&self, seconds: f64) {
        let _ = self.inner.command("seek", &[&seconds.to_string()]);
    }
    fn set_speed(&self, speed: f64) {
        let _ = self.inner.set_property("speed", speed);
    }
    fn video_zoom(&self) -> f64 {
        self.get_opt("video-zoom").unwrap_or(0.0)
    }
    fn set_video_zoom(&self, zoom: f64) {
        let _ = self.inner.set_property("video-zoom", zoom);
    }
    fn set_glsl_shaders(&self, value: &str) {
        let _ = self.inner.set_property("glsl-shaders", value.to_string());
    }
    fn screenshot(&self) -> anyhow::Result<()> {
        self.inner
            .command("screenshot", &[])
            .map_err(|e| anyhow::anyhow!("mpv screenshot failed: {e}"))?;
        Ok(())
    }
    fn set_screenshot_directory(&self, dir: &str) -> anyhow::Result<()> {
        // mpv never creates this directory itself: a missing one makes
        // every screenshot fail without a peep.
        std::fs::create_dir_all(dir)
            .map_err(|e| anyhow::anyhow!("cannot create screenshot dir {dir}: {e}"))?;
        self.inner
            .set_property("screenshot-directory", dir.to_string())
            .map_err(|e| anyhow::anyhow!("mpv screenshot-directory failed: {e}"))?;
        let _ = self.inner.set_property("screenshot-jpeg-quality", 100_i64);
        Ok(())
    }
    fn set_audio_delay(&self, delay: f64) {
        let _ = self.inner.set_property("audio-delay", delay);
    }
    fn set_svp(&self, on: bool) {
        if on {
            let _ = self
                .inner
                .set_property("input-ipc-server", "mpvpipe".to_string());
            let _ = self.inner.set_property("hwdec", "auto-copy".to_string());
            let _ = self.inner.set_property("hwdec-codecs", "all".to_string());
            let _ = self.inner.set_property("hr-seek-framedrop", false);
        } else {
            let _ = self.inner.set_property("input-ipc-server", String::new());
            let _ = self.inner.set_property("hwdec", "auto".to_string());
            let _ = self.inner.set_property("hr-seek-framedrop", true);
        }
    }
    fn set_option_str(&self, name: &str, value: &str) {
        let _ = self.inner.set_property(name, value.to_string());
    }
    fn video_format(&self) -> Option<String> {
        self.get_opt("video-format")
    }
    fn audio_codec(&self) -> Option<String> {
        self.get_opt("audio-codec-name")
    }
    fn width(&self) -> Option<i64> {
        self.get_opt("width")
    }
    fn height(&self) -> Option<i64> {
        self.get_opt("height")
    }
    fn fps(&self) -> Option<f64> {
        self.get_opt("estimated-vf-fps")
    }
    fn frame_drop_count(&self) -> Option<i64> {
        self.get_opt("frame-drop-count")
    }
    fn tracks(&self) -> Vec<Track> {
        // `track-list` is a node property; read it back through the indexed
        // scalar aliases (`track-list/0/type`, ...) instead.
        let count: i64 = self.get_opt("track-list/count").unwrap_or(0);
        let mut out = Vec::new();
        for i in 0..count.min(64) {
            let base = format!("track-list/{i}");
            let ty: String = match self.get_opt(&format!("{base}/type")) {
                Some(ty) => ty,
                None => continue,
            };
            let kind = match ty.as_str() {
                "audio" => TrackKind::Audio,
                "sub" => TrackKind::Sub,
                _ => continue,
            };
            let id: i64 = match self.get_opt(&format!("{base}/id")) {
                Some(id) => id,
                None => continue,
            };
            let codec: String = self.get_opt(&format!("{base}/codec")).unwrap_or_default();
            let lang: Option<String> = self.get_opt(&format!("{base}/lang"));
            let title: Option<String> = self.get_opt(&format!("{base}/title"));
            let (channels, samplerate) = if kind == TrackKind::Audio {
                let channels: Option<String> = self
                    .get_opt::<i64>(&format!("{base}/audio-channels"))
                    .map(|c| c.to_string());
                let samplerate: Option<String> = self
                    .get_opt::<f64>(&format!("{base}/demux-samplerate"))
                    .map(|s| format!("{s:.0}"));
                (channels, samplerate)
            } else {
                (None, None)
            };
            out.push(Track {
                id,
                kind,
                codec,
                lang,
                title,
                channels,
                samplerate,
            });
        }
        out
    }
    fn set_audio_track(&self, id: i64) {
        let _ = self.inner.set_property("aid", id);
    }
    fn set_sub_track(&self, id: i64) {
        let _ = self.inner.set_property("sid", id);
    }
    fn add_sub_file(&self, path: &str) {
        let _ = self.inner.command("sub-add", &[path]);
    }
    fn add_audio_file(&self, path: &str) {
        let _ = self.inner.command("audio-add", &[path]);
    }
    fn mpv_version(&self) -> String {
        self.get_opt::<String>("mpv-version")
            .unwrap_or_else(|| "mpv".to_string())
    }
    fn eof_reached(&self) -> bool {
        self.get_opt::<bool>("eof-reached").unwrap_or(false)
    }
    fn is_idle(&self) -> bool {
        self.get_opt::<bool>("idle-active")
            .or_else(|| self.get_opt::<bool>("core-idle"))
            .unwrap_or(false)
    }
    fn paused_for_cache(&self) -> bool {
        self.get_opt::<bool>("paused-for-cache").unwrap_or(false)
    }
    fn hwdec_current(&self) -> Option<String> {
        let cur: Option<String> = self.get_opt("hwdec-current");
        match cur {
            Some(s) if !s.is_empty() => Some(s),
            _ => None,
        }
    }

    #[cfg(feature = "video")]
    fn mpv_handle(&self) -> Option<std::sync::Arc<libmpv2::Mpv>> {
        Some(std::sync::Arc::clone(&self.inner))
    }
}

/// Hard cap for `mpv.log`: mpv only appends, so without rotation the file
/// grows without bound (see `cap_mpv_log`).
#[cfg(feature = "video")]
pub const MPV_LOG_LIMIT_BYTES: u64 = 512 * 1024;

/// Path of the mpv log file (next to the exe), if determinable.
#[cfg(feature = "video")]
pub fn mpv_log_path() -> Option<std::path::PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .map(|dir| dir.join("mpv.log"))
}

/// Truncates `path` when it exceeds `MPV_LOG_LIMIT_BYTES`. mpv keeps the
/// file open in append mode, so truncating from our side is safe: the next
/// message lands at the new end. Never fails.
#[cfg(feature = "video")]
fn truncate_log_if_large(path: &std::path::Path) {
    let over = std::fs::metadata(path)
        .map(|m| m.len() > MPV_LOG_LIMIT_BYTES)
        .unwrap_or(false);
    if over {
        let _ = std::fs::write(path, "");
    }
}

/// Caps `mpv.log` at `MPV_LOG_LIMIT_BYTES` (see `truncate_log_if_large`).
/// Called at startup (after a fresh truncate in `LibMpv::new`) and
/// periodically while running, so the log can never take much disk space.
#[cfg(feature = "video")]
pub fn cap_mpv_log() {
    if let Some(path) = mpv_log_path() {
        truncate_log_if_large(&path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_log_is_truncated_small_log_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let big = dir.path().join("big.log");
        let small = dir.path().join("small.log");
        std::fs::write(&big, vec![b'x'; MPV_LOG_LIMIT_BYTES as usize + 1]).unwrap();
        std::fs::write(&small, b"short").unwrap();
        truncate_log_if_large(&big);
        truncate_log_if_large(&small);
        assert_eq!(std::fs::metadata(&big).unwrap().len(), 0);
        assert_eq!(std::fs::read(&small).unwrap(), b"short");
        // Missing file: no panic, no file created.
        truncate_log_if_large(&dir.path().join("absent.log"));
    }
}
