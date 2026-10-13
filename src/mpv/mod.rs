//! mpv control layer.
//!
//! [`MpvBackend`] is the seam between [`crate::player::PlayerState`] / the UI
//! and the actual player. [`MockMpv`] records calls for tests and powers
//! builds without the `video` feature. With `--features video` the
//! [`LibMpv`] backend drives real `libmpv` (needs `libmpv` at build time and
//! `libmpv.so` / `mpv-2.dll` at runtime).

mod files;
#[cfg(feature = "video")]
mod libmpv;
mod mock;

pub(crate) use self::files::{dir_file_names, find_new_file};
#[cfg(feature = "video")]
pub use self::libmpv::{LibMpv, cap_mpv_log};
pub use self::mock::MockMpv;

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
