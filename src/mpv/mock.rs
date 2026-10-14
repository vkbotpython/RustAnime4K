//! In-memory бэкенд для тестов и сборок без фичи `video`.

use super::{MpvBackend, Track};
use std::sync::Mutex;

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
    use crate::mpv::{TrackKind, dir_file_names, find_new_file, find_new_file_set};

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

    #[test]
    fn find_new_file_set_matches_slice_version() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.bmp"), b"x").unwrap();
        let before: Vec<String> = dir_file_names(dir.path());
        let before_set: std::collections::HashSet<String> = before.iter().cloned().collect();
        assert!(find_new_file(&before, dir.path()).is_none());
        assert!(find_new_file_set(&before_set, dir.path()).is_none());
        std::fs::write(dir.path().join("b.bmp"), b"y").unwrap();
        assert_eq!(find_new_file(&before, dir.path()).as_deref(), Some("b.bmp"));
        assert_eq!(
            find_new_file_set(&before_set, dir.path()).as_deref(),
            Some("b.bmp")
        );
    }
}
