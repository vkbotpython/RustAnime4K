//! mpv control layer.
//!
//! [`MpvBackend`] is the seam between [`crate::player::PlayerState`] / the UI
//! and the actual player. [`MockMpv`] records calls for tests and powers
//! builds without the `video` feature. With `--features video` the
//! [`LibMpv`] backend drives real `libmpv` (needs `libmpv` at build time and
//! `libmpv.so` / `mpv-2.dll` at runtime).

use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackKind {
    Audio,
    Sub,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub id: i64,
    pub kind: TrackKind,
    pub codec: String,
    pub lang: Option<String>,
    pub title: Option<String>,
    pub channels: Option<String>,
    pub samplerate: Option<String>,
}

impl Track {
    /// Display string for audio/subtitle track menus.
    pub fn display(&self) -> String {
        match self.kind {
            TrackKind::Sub => {
                let base = self.title.clone().unwrap_or_else(|| self.codec.clone());
                match &self.lang {
                    Some(lang) if self.title.is_some() => format!("{lang} - {base}"),
                    Some(lang) => format!("{lang} - {}", self.codec),
                    None => base,
                }
            }
            TrackKind::Audio => {
                let base = format!(
                    "{} {}ch {} Hz",
                    self.codec,
                    self.channels.as_deref().unwrap_or("?"),
                    self.samplerate.as_deref().unwrap_or("?")
                );
                let base = match &self.title {
                    Some(title) => format!("{title} ({base})"),
                    None => base,
                };
                match &self.lang {
                    Some(lang) => format!("{lang} - {base}"),
                    None => base,
                }
            }
        }
    }
}

pub trait MpvBackend: Send + Sync {
    fn play(&self, path: &str);
    fn stop(&self);
    fn set_pause(&self, pause: bool);
    fn pause(&self) -> bool;
    fn set_volume(&self, volume: i32);
    fn volume(&self) -> i32;
    fn time_pos(&self) -> Option<f64>;
    fn set_time_pos(&self, pos: f64);
    fn duration(&self) -> Option<f64>;
    fn seek(&self, seconds: f64);
    fn set_speed(&self, speed: f64);
    fn video_zoom(&self) -> f64;
    fn set_video_zoom(&self, zoom: f64);
    fn set_glsl_shaders(&self, value: &str);
    /// Takes a screenshot into the directory set by
    /// [`MpvBackend::set_screenshot_directory`]. Returns the dispatch error
    /// instead of swallowing it, so the UI can report it.
    fn screenshot(&self) -> anyhow::Result<()>;
    /// Points mpv screenshots at `dir`, creating it first (mpv itself never
    /// creates `screenshot-directory` and fails silently without it).
    fn set_screenshot_directory(&self, dir: &str) -> anyhow::Result<()>;
    fn set_audio_delay(&self, delay: f64);
    fn set_svp(&self, on: bool);
    /// Applies one `name=value` launch parameter (safe subset of stored params).
    fn set_option_str(&self, name: &str, value: &str);
    fn video_format(&self) -> Option<String>;
    fn audio_codec(&self) -> Option<String>;
    fn width(&self) -> Option<i64>;
    fn height(&self) -> Option<i64>;
    fn fps(&self) -> Option<f64>;
    fn frame_drop_count(&self) -> Option<i64>;
    fn tracks(&self) -> Vec<Track>;
    fn set_audio_track(&self, id: i64);
    fn set_sub_track(&self, id: i64);
    fn add_sub_file(&self, path: &str);
    fn add_audio_file(&self, path: &str);
    fn mpv_version(&self) -> String;
    /// True when nothing is loaded (mirrors `idle-active`).
    /// Used to tell "still loading, don't touch it" apart from
    /// "dead load, safe to (re)start".
    fn is_idle(&self) -> bool {
        false
    }
    /// True while mpv auto-paused for cache refill (demuxer starves: both
    /// audio and video stutter in lockstep while `pause` stays false, so
    /// without this the UI looks "playing" while the core is stalled).
    fn paused_for_cache(&self) -> bool {
        false
    }
    /// Active hardware decoder (`hwdec-current`), if any. Empty/software
    /// decode reads back as `None`/empty; a stuck exotic decoder is the
    /// prime suspect when frames stop with a fast/empty renderer.
    fn hwdec_current(&self) -> Option<String> {
        None
    }
    /// True when playback reached end-of-file (for auto-advance).
    fn eof_reached(&self) -> bool {
        false
    }
    /// Raw libmpv handle for the GPU render context (`video` feature).
    /// Returns `None` for backends without libmpv (mock).
    #[cfg(feature = "video")]
    fn mpv_handle(&self) -> Option<std::sync::Arc<libmpv2::Mpv>> {
        None
    }
}

#[derive(Debug, Default)]
struct MockState {
    pub last_played: Option<String>,
    pub stopped: u32,
    pub pause: bool,
    pub volume: i32,
    pub time_pos: Option<f64>,
    pub duration: Option<f64>,
    pub speed: f64,
    pub video_zoom: f64,
    pub glsl_shaders: String,
    pub screenshots: u32,
    pub screenshot_dir: Option<String>,
    pub eof_reached: bool,
    pub options: Vec<(String, String)>,
}

/// In-memory backend for tests and `--no-default-features` builds.
#[derive(Debug, Default)]
pub struct MockMpv {
    state: Mutex<MockState>,
}

impl MockMpv {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(MockState {
                speed: 1.0,
                volume: 80,
                ..Default::default()
            }),
        }
    }

    /// Test helper: last path passed to [`MpvBackend::play`].
    #[cfg(test)]
    pub fn last_played(&self) -> Option<String> {
        self.state.lock().unwrap().last_played.clone()
    }

    /// Test helper: last `glsl-shaders` value.
    #[cfg(test)]
    pub fn glsl_shaders(&self) -> String {
        self.state.lock().unwrap().glsl_shaders.clone()
    }

    /// Test helper: force `eof_reached` for auto-advance tests.
    #[cfg(test)]
    pub fn set_eof(&self, eof: bool) {
        self.state.lock().unwrap().eof_reached = eof;
    }
}

impl MpvBackend for MockMpv {
    fn play(&self, path: &str) {
        self.state.lock().unwrap().last_played = Some(path.to_string());
    }
    fn stop(&self) {
        let mut s = self.state.lock().unwrap();
        s.stopped += 1;
        s.time_pos = None;
        s.duration = None;
    }
    fn set_pause(&self, pause: bool) {
        self.state.lock().unwrap().pause = pause;
    }
    fn pause(&self) -> bool {
        self.state.lock().unwrap().pause
    }
    fn set_volume(&self, volume: i32) {
        self.state.lock().unwrap().volume = volume;
    }
    fn volume(&self) -> i32 {
        self.state.lock().unwrap().volume
    }
    fn time_pos(&self) -> Option<f64> {
        self.state.lock().unwrap().time_pos
    }
    fn set_time_pos(&self, pos: f64) {
        self.state.lock().unwrap().time_pos = Some(pos);
    }
    fn duration(&self) -> Option<f64> {
        self.state.lock().unwrap().duration
    }
    fn seek(&self, seconds: f64) {
        let mut s = self.state.lock().unwrap();
        let cur = s.time_pos.unwrap_or(0.0);
        s.time_pos = Some((cur + seconds).max(0.0));
    }
    fn set_speed(&self, speed: f64) {
        self.state.lock().unwrap().speed = speed;
    }
    fn video_zoom(&self) -> f64 {
        self.state.lock().unwrap().video_zoom
    }
    fn set_video_zoom(&self, zoom: f64) {
        self.state.lock().unwrap().video_zoom = zoom;
    }
    fn set_glsl_shaders(&self, value: &str) {
        self.state.lock().unwrap().glsl_shaders = value.to_string();
    }
    fn screenshot(&self) -> anyhow::Result<()> {
        let mut s = self.state.lock().unwrap();
        s.screenshots += 1;
        let n = s.screenshots;
        let dir = s.screenshot_dir.clone();
        drop(s);
        // Mock mirrors the real backend end to end (directory + a file the
        // UI watch can find), so the dialog flow is testable without libmpv.
        // A 1x1 BMP needs no encoder crates; it is only a stand-in.
        if let Some(dir) = dir {
            std::fs::create_dir_all(&dir)
                .map_err(|e| anyhow::anyhow!("cannot create {dir}: {e}"))?;
            std::fs::write(dir_file(dir, n), mock_bmp_bytes())
                .map_err(|e| anyhow::anyhow!("cannot write mock screenshot: {e}"))?;
        }
        Ok(())
    }
    fn set_screenshot_directory(&self, dir: &str) -> anyhow::Result<()> {
        let mut s = self.state.lock().unwrap();
        s.screenshot_dir = Some(dir.to_string());
        s.options
            .push(("screenshot-directory".to_string(), dir.to_string()));
        Ok(())
    }
    fn set_audio_delay(&self, delay: f64) {
        self.state
            .lock()
            .unwrap()
            .options
            .push(("audio-delay".to_string(), delay.to_string()));
    }
    fn set_svp(&self, on: bool) {
        let mut s = self.state.lock().unwrap();
        if on {
            s.options
                .push(("input-ipc-server".to_string(), "mpvpipe".to_string()));
            s.options
                .push(("hwdec".to_string(), "auto-copy".to_string()));
            s.options
                .push(("hwdec-codecs".to_string(), "all".to_string()));
            s.options
                .push(("hr-seek-framedrop".to_string(), "false".to_string()));
        } else {
            s.options
                .push(("input-ipc-server".to_string(), String::new()));
            s.options.push(("hwdec".to_string(), "auto".to_string()));
            s.options
                .push(("hr-seek-framedrop".to_string(), "true".to_string()));
        }
    }
    fn set_option_str(&self, name: &str, value: &str) {
        self.state
            .lock()
            .unwrap()
            .options
            .push((name.to_string(), value.to_string()));
    }
    fn video_format(&self) -> Option<String> {
        None
    }
    fn audio_codec(&self) -> Option<String> {
        None
    }
    fn width(&self) -> Option<i64> {
        None
    }
    fn height(&self) -> Option<i64> {
        None
    }
    fn fps(&self) -> Option<f64> {
        None
    }
    fn frame_drop_count(&self) -> Option<i64> {
        None
    }
    fn tracks(&self) -> Vec<Track> {
        Vec::new()
    }
    fn set_audio_track(&self, _id: i64) {}
    fn set_sub_track(&self, _id: i64) {}
    fn add_sub_file(&self, path: &str) {
        self.state.lock().unwrap().last_played = Some(format!("sub:{path}"));
    }
    fn add_audio_file(&self, path: &str) {
        self.state.lock().unwrap().last_played = Some(format!("audio:{path}"));
    }
    fn mpv_version(&self) -> String {
        "mpv (mock)".to_string()
    }
    fn eof_reached(&self) -> bool {
        self.state.lock().unwrap().eof_reached
    }
}

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
            // this file stutter causes are invisible. Overwritten every
            // start; a failed write must not fail init.
            let log_dir = std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.to_path_buf()));
            if let Some(dir) = log_dir {
                let log_value = dir.join("mpv.log").to_string_lossy().into_owned();
                let _ = init.set_option("log-file", log_value.as_str());
            }
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

/// File names (lossy) currently inside `dir`. Read failures yield an empty
/// list: the screenshot watch treats them as "nothing new yet".
pub(crate) fn dir_file_names(dir: &std::path::Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// First file in `dir` absent from `before` (the mpv screenshot that just
/// landed), if any.
pub(crate) fn find_new_file(before: &[String], dir: &std::path::Path) -> Option<String> {
    dir_file_names(dir)
        .into_iter()
        .find(|name| !before.iter().any(|b| b == name))
}

/// `dir` + per-shot file name for the mock backend's stand-in screenshot.
fn dir_file(dir: String, n: u32) -> std::path::PathBuf {
    std::path::Path::new(&dir).join(format!("mock-shot-{n}.bmp"))
}

/// Bytes of a 1x1 24-bit BMP. Hand-rolled so the mock backend needs no
/// image-encoder crates: 14-byte file header + 40-byte info header +
/// one padded pixel row (58 bytes total).
fn mock_bmp_bytes() -> Vec<u8> {
    let mut out = Vec::with_capacity(58);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&58u32.to_le_bytes()); // file size
    out.extend_from_slice(&[0u8; 4]); // reserved
    out.extend_from_slice(&54u32.to_le_bytes()); // pixel data offset
    out.extend_from_slice(&40u32.to_le_bytes()); // info header size
    out.extend_from_slice(&1i32.to_le_bytes()); // width
    out.extend_from_slice(&1i32.to_le_bytes()); // height
    out.extend_from_slice(&1u16.to_le_bytes()); // planes
    out.extend_from_slice(&24u16.to_le_bytes()); // bits per pixel
    out.extend_from_slice(&0u32.to_le_bytes()); // compression (none)
    out.extend_from_slice(&4u32.to_le_bytes()); // image size
    out.extend_from_slice(&[0u8; 16]); // resolution + palette fields
    out.extend_from_slice(&[0x20, 0x20, 0x20, 0x00]); // one gray pixel + pad
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_display_variants() {
        let t = Track {
            id: 1,
            kind: TrackKind::Sub,
            codec: "ass".into(),
            lang: Some("eng".into()),
            title: Some("Signs".into()),
            channels: None,
            samplerate: None,
        };
        assert_eq!(t.display(), "eng - Signs");

        let t2 = Track {
            id: 2,
            kind: TrackKind::Sub,
            codec: "subrip".into(),
            lang: None,
            title: None,
            channels: None,
            samplerate: None,
        };
        assert_eq!(t2.display(), "subrip");
    }

    #[test]
    fn audio_display() {
        let t = Track {
            id: 1,
            kind: TrackKind::Audio,
            codec: "aac".into(),
            lang: Some("jpn".into()),
            title: None,
            channels: Some("2".into()),
            samplerate: Some("48000".into()),
        };
        assert_eq!(t.display(), "jpn - aac 2ch 48000 Hz");
    }

    #[test]
    fn mock_seek_and_volume() {
        let mpv = MockMpv::new();
        mpv.set_time_pos(10.0);
        mpv.seek(5.0);
        assert_eq!(mpv.time_pos(), Some(15.0));
        mpv.set_glsl_shaders("a.glsl:b.glsl");
        assert_eq!(mpv.glsl_shaders(), "a.glsl:b.glsl");
        mpv.set_svp(true);
        mpv.screenshot().unwrap();
    }

    #[test]
    fn mock_eof_flag() {
        let mpv = MockMpv::new();
        assert!(!mpv.eof_reached());
        mpv.set_eof(true);
        assert!(mpv.eof_reached());
    }

    #[test]
    fn mock_screenshot_writes_findable_file() {
        let dir = tempfile::tempdir().unwrap();
        let dir_str = dir.path().to_string_lossy().into_owned();
        let mpv = MockMpv::new();
        // Screenshot without a directory: counted, no file, still Ok.
        mpv.screenshot().unwrap();
        assert!(dir_file_names(dir.path()).is_empty());
        // With a directory: one new BMP per shot, discoverable by the watch.
        mpv.set_screenshot_directory(&dir_str).unwrap();
        let before = dir_file_names(dir.path());
        mpv.screenshot().unwrap();
        let found = find_new_file(&before, dir.path()).expect("new shot file");
        assert!(found.starts_with("mock-shot-") && found.ends_with(".bmp"));
        let bytes = std::fs::read(dir.path().join(&found)).unwrap();
        assert_eq!(&bytes[0..2], b"BM");
        assert_eq!(bytes.len(), 58);
        assert_eq!(i32::from_le_bytes(bytes[18..22].try_into().unwrap()), 1);
        assert_eq!(i32::from_le_bytes(bytes[22..26].try_into().unwrap()), 1);
        // Second shot: a different file, the first one is "old" now.
        let before = dir_file_names(dir.path());
        mpv.screenshot().unwrap();
        let found2 = find_new_file(&before, dir.path()).expect("second shot file");
        assert_ne!(found, found2);
        assert!(find_new_file(&dir_file_names(dir.path()), dir.path()).is_none());
    }

    #[test]
    fn screenshot_creates_missing_directory() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a").join("b");
        assert!(!nested.exists());
        let mpv = MockMpv::new();
        mpv.set_screenshot_directory(&nested.to_string_lossy())
            .unwrap();
        mpv.screenshot().unwrap();
        assert!(nested.is_dir());
        assert_eq!(dir_file_names(&nested).len(), 1);
    }
}
