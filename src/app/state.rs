//! Состояние приложения: структура, снапшот mpv, выбор пресета.

use crate::config::ConfigManager;
use crate::i18n::{self, Lang};
use crate::mpv::{MpvBackend, Track};
use crate::player::PlayerState;
use crate::theme::{self, ColorScheme, Palette};
#[cfg(feature = "video")]
use crate::video::VideoRenderer;
use serde_json::{Value, json};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::helpers::{display_name_of, get_bool, get_str};

pub const APP_NAME: &str = "Anime4K Player";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Slider granularity for the seek bar (0.1 s steps).
pub const TIME_SCALE: f64 = 10.0;

/// Idle delay before the cursor auto-hides in fullscreen.
pub(crate) const FULLSCREEN_CURSOR_IDLE: Duration = Duration::from_millis(2000);
/// Bottom strip (px) that reveals the control panel in fullscreen.
pub(crate) const FULLSCREEN_BOTTOM_ZONE: f32 = 140.0;
/// Right strip (px) that reveals the playlist in fullscreen.
pub(crate) const FULLSCREEN_RIGHT_ZONE: f32 = 120.0;

/// How long a transient `status` line (screenshot name, errors, "Success")
/// stays in the video corner before auto-clearing.
pub(crate) const STATUS_TTL: Duration = Duration::from_secs(4);

/// egui pairs clicks into a double click within 300 ms
/// (`max_double_click_delay`): a single-click play/pause newer than this
/// is reverted when the double click lands (see `consume_click_revert`),
/// so entering fullscreen never disturbs playback.
pub(crate) const DOUBLE_CLICK_REVERT_WINDOW: Duration = Duration::from_millis(400);

pub(crate) const SPEEDS: &[f64] = &[
    0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 2.25, 2.5, 2.75, 3.0,
];

/// Donation card from the README (spaced for display, plain for copying).
pub(crate) const DONATE_CARD_SPACED: &str = "2202 2050 1464 4675";
pub(crate) const DONATE_CARD_PLAIN: &str = "2202205014644675";

pub struct Anime4KPlayerApp {
    pub(crate) config: ConfigManager,
    pub(crate) lang: Lang,
    pub(crate) mpv: Arc<dyn MpvBackend>,
    #[cfg(feature = "video")]
    pub(crate) video: Option<VideoRenderer>,
    pub(crate) player: PlayerState,
    pub(crate) playlist_visible: bool,
    pub(crate) status: String,
    /// When the current `status` line expires (see `set_status`).
    /// `None` = no transient message to clear.
    pub(crate) status_until: Option<Instant>,

    // Dialog visibility + drafts.
    pub(crate) show_settings: bool,
    pub(crate) show_about: bool,
    pub(crate) show_open_url: bool,
    pub(crate) show_open_folder: bool,
    pub(crate) show_screenshot: bool,
    pub(crate) show_launch_params: bool,
    pub(crate) show_android: bool,
    pub(crate) show_reference: bool,
    pub(crate) show_audio_menu: bool,
    pub(crate) show_sub_menu: bool,
    pub(crate) show_preset_offer: bool,

    pub(crate) url_text: String,
    pub(crate) folder_text: String,
    pub(crate) add_audio_path: String,
    pub(crate) add_sub_path: String,
    pub(crate) screenshot_dir_text: String,
    pub(crate) launch_params_text: String,
    pub(crate) android_path: String,
    pub(crate) android_index: usize,

    pub(crate) settings_lang: Lang,
    pub(crate) settings_open_last: bool,
    pub(crate) settings_pos_last: bool,
    pub(crate) settings_volume_plus: bool,
    pub(crate) settings_svp: bool,
    pub(crate) settings_fixed_panel: bool,
    pub(crate) settings_ask_preset: bool,
    pub(crate) settings_sound_offset: f64,
    pub(crate) settings_palette: Palette,
    pub(crate) settings_style: String,
    pub(crate) settings_scheme: ColorScheme,

    // Anime4K preset offer dialog drafts (quality/mode to apply on choice).
    pub(crate) preset_offer_quality: String,
    pub(crate) preset_offer_mode: String,
    pub(crate) preset_offer_dont_ask: bool,

    // Fullscreen mouse tracking.
    pub(crate) last_mouse_pos: Option<egui::Pos2>,
    pub(crate) last_mouse_time: Instant,

    /// When a single click last toggled play/pause. A double click starts
    /// as a single click, so a still-fresh toggle is reverted when the
    /// double click switches fullscreen (see `consume_click_revert`).
    pub(crate) click_play_armed_at: Option<Instant>,

    /// Last disk flush of the window geometry (see `track_window_geometry`).
    pub(crate) last_window_save: Option<Instant>,

    /// Last `mpv.log` size-cap check (see `crate::mpv::cap_mpv_log`).
    #[cfg(feature = "video")]
    pub(crate) last_log_cap: Option<Instant>,

    /// Set once the current EOF episode has been handled. `eof-reached`
    /// stays true across frames (and can briefly report a stale value right
    /// after `loadfile`), so without edge detection a single EOF would
    /// restart/skip playback on every 100 ms tick.
    pub(crate) eof_handled: bool,

    /// Last polled mpv state (see `poll_mpv`).
    pub(crate) snap: MpvSnapshot,
    /// Last snapshot poll time; polling is time-gated (see `poll_mpv`).
    pub(crate) last_poll: Option<Instant>,
    /// Last slow-tier poll time (`volume/drops/cache`, see `poll_mpv`).
    pub(crate) last_slow_poll: Option<Instant>,
    /// File key the static snapshot fields were fetched for. `None` forces
    /// a refetch (file changed or metadata not ready yet).
    pub(crate) snap_static_key: Option<String>,
    /// `track-list/count` at the last `tracks()` fetch; a change forces a
    /// refetch while menus are open instead of polling every tick.
    pub(crate) tracks_count_cache: Option<i64>,
    /// File key the cached track list belongs to.
    pub(crate) tracks_file_key: Option<String>,
    /// Last debounced config flush (see `flush_config_debounced`).
    pub(crate) last_config_save: Option<Instant>,
    /// Config changed since the last debounced flush and still needs a write.
    pub(crate) config_dirty: bool,
    /// Preset waiting to be applied once playback metadata is in
    /// (see `flush_pending_preset`).
    pub(crate) pending_preset: Option<PresetChoice>,

    /// Perf diagnostics shown in the media-info row while playing:
    /// whole-`ui()` time and the video-panel slice of it (mpv
    /// update()+render()+FBO upkeep; the deferred GL blit excluded).
    /// Distinguishes render-bound (V≈UI≫16ms) from UI-bound (UI≫V)
    /// from repaint-starved (both small, yet perceived fps low) stalls.
    pub(crate) ui_ms_ema: f64,
    pub(crate) video_ms_ema: f64,

    /// When the SBER card number was copied from the About dialog.
    /// Shows a short "copied" confirmation next to the button.
    pub(crate) about_copied_at: Option<Instant>,

    /// Pending screenshot-file watch, armed when a screenshot is taken.
    /// mpv writes the file asynchronously with a name it chooses itself,
    /// so completion is detected by a new file appearing in the folder
    /// (see `poll_screenshot`) instead of trusting silent dispatch.
    pub(crate) screenshot_pending: Option<ScreenshotWatch>,

    /// Cached media-info row. `media_info()` allocated ~8 `String` every
    /// frame; the cache rebuilds only when snapshot inputs change.
    pub(crate) media_info_cache: String,
    pub(crate) media_info_sig: Option<MediaInfoSig>,
    /// Cached `MM:SS` labels for the seek bar. `fmt_time` allocates 2
    /// `String` per frame at 60fps; the cache rebuilds only when the
    /// displayed second changes (position is quantized to 0.1s).
    pub(crate) time_pos_text: String,
    pub(crate) time_dur_text: String,
    pub(crate) time_pos_tick: i64,
    pub(crate) time_dur_tick: i64,
    /// Cached `"{vol}%"` label: rebuilt only when the volume changes.
    pub(crate) volume_text: String,
    pub(crate) volume_text_vol: i32,
}

/// Fingerprint of `media_info()` inputs. Any change means the row must be
/// rebuilt; equality means the cached string can be reused as-is.
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct MediaInfoSig {
    pub(crate) preset: String,
    pub(crate) video_format: Option<String>,
    pub(crate) audio_codec: Option<String>,
    pub(crate) width: Option<i64>,
    pub(crate) height: Option<i64>,
    pub(crate) fps_bits: Option<u64>,
    pub(crate) frame_drops: Option<i64>,
    pub(crate) hwdec: Option<String>,
    pub(crate) cache_pause: bool,
    pub(crate) has_current: bool,
    pub(crate) ui_ms: i64,
    pub(crate) video_ms: i64,
    pub(crate) lang: Lang,
}

/// A screenshot request waiting for its file to land on disk.
pub(crate) struct ScreenshotWatch {
    pub(crate) dir: PathBuf,
    /// File names present at dispatch; the screenshot is the newcomer.
    /// `HashSet` for O(1) lookup instead of O(n*m) scans.
    pub(crate) before: HashSet<String>,
    /// Give up (and report where to look) after this instant.
    pub(crate) deadline: Instant,
    /// Last filesystem poll. `poll_screenshot` runs every frame; without
    /// throttling it issues `read_dir` 60x/sec.
    pub(crate) last_check: Option<Instant>,
}

/// Minimum interval between screenshot-folder polls.
pub(crate) const SCREENSHOT_POLL_INTERVAL: Duration = Duration::from_millis(250);

/// Cached mpv state. During playback egui repaints on every video frame
/// (24-60/s via the render-context update callback); doing a dozen blocking
/// `get_property` round-trips per frame saturates the UI thread and starves
/// the player, so all per-frame UI reads go through this snapshot instead.
#[derive(Debug, Clone, Default)]
pub(crate) struct MpvSnapshot {
    pub(crate) pause: bool,
    pub(crate) duration: Option<f64>,
    pub(crate) time_pos: Option<f64>,
    pub(crate) eof: bool,
    pub(crate) volume: i32,
    pub(crate) video_format: Option<String>,
    pub(crate) audio_codec: Option<String>,
    pub(crate) width: Option<i64>,
    pub(crate) height: Option<i64>,
    pub(crate) fps: Option<f64>,
    pub(crate) frame_drops: Option<i64>,
    /// Core stall flags (see `MpvBackend::paused_for_cache/hwdec_current`):
    /// shown in media-info while playing; they distinguish demux/cache
    /// starvation and a stuck hw decoder from a slow renderer.
    pub(crate) cache_pause: bool,
    pub(crate) hwdec: Option<String>,
    /// Cached only while an audio/sub menu is open (`tracks()` fans out
    /// into hundreds of property reads — never per frame).
    pub(crate) tracks: Vec<Track>,
    /// Prebuilt menu rows `(id, label)` per kind, refreshed together with
    /// `tracks` so `track_menu` iterates `&str` without per-frame `format!`.
    pub(crate) audio_rows: Vec<(i64, String)>,
    pub(crate) sub_rows: Vec<(i64, String)>,
}

/// Fast snapshot tier (position/state), see `poll_mpv`.
pub(crate) const FAST_POLL_INTERVAL: Duration = Duration::from_millis(100);
/// Slow snapshot tier (`volume/drops/cache`), see `poll_mpv`.
pub(crate) const SLOW_POLL_INTERVAL: Duration = Duration::from_millis(1500);
/// Minimum interval between debounced config writes, see
/// `flush_config_debounced`.
pub(crate) const CONFIG_FLUSH_DEBOUNCE: Duration = Duration::from_millis(500);
/// Upper bound for per-file presets (`filePresets` grows unbounded
/// otherwise — one key per opened file).
pub(crate) const MAX_FILE_PRESETS: usize = 500;

/// Per-file Anime4K choice stored under config `filePresets`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PresetChoice {
    Standard { quality: String, mode: String },
    Hq { name: String },
    Disabled,
}

impl PresetChoice {
    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::Standard { quality, mode } => json!({"quality": quality, "mode": mode}),
            Self::Hq { name } => json!({"hq": name}),
            Self::Disabled => json!({"disabled": true}),
        }
    }

    pub(crate) fn from_json(v: &Value) -> Option<Self> {
        if v.get("disabled").and_then(Value::as_bool).unwrap_or(false) {
            return Some(Self::Disabled);
        }
        if let Some(name) = v.get("hq").and_then(Value::as_str) {
            return Some(Self::Hq {
                name: name.to_string(),
            });
        }
        match (
            v.get("quality").and_then(Value::as_str),
            v.get("mode").and_then(Value::as_str),
        ) {
            (Some(quality), Some(mode)) => Some(Self::Standard {
                quality: quality.to_string(),
                mode: mode.to_string(),
            }),
            _ => None,
        }
    }
}

impl Anime4KPlayerApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        mpv: Arc<dyn MpvBackend>,
        cli_arg: Option<String>,
    ) -> Self {
        // Enables PNG decoding for `egui::Image::from_bytes` (About dialog).
        // Without this, images render as a ⚠ placeholder.
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let config = ConfigManager::new("config.json");
        let lang = i18n::resolve(get_str(&config, "language").as_deref());
        let volume_plus = get_bool(&config, "volumePlus", false);
        let volume = config.get("volume").and_then(Value::as_i64).unwrap_or(80) as i32;
        #[cfg(feature = "video")]
        let video = match mpv.mpv_handle() {
            Some(handle) => match VideoRenderer::new(cc, handle) {
                Ok(renderer) => Some(renderer),
                Err(err) => {
                    eprintln!("video renderer init failed: {err:#}");
                    None
                }
            },
            None => None,
        };
        let mut app = Self {
            lang,
            player: PlayerState::new(volume, volume_plus),
            mpv,
            #[cfg(feature = "video")]
            video,
            playlist_visible: false,
            status: String::new(),
            status_until: None,
            show_settings: false,
            show_about: false,
            show_open_url: false,
            show_open_folder: false,
            show_screenshot: false,
            show_launch_params: false,
            show_android: false,
            show_reference: false,
            show_audio_menu: false,
            show_sub_menu: false,
            show_preset_offer: false,
            url_text: String::new(),
            folder_text: String::new(),
            add_audio_path: String::new(),
            add_sub_path: String::new(),
            screenshot_dir_text: get_str(&config, "screenshotDirectory").unwrap_or_default(),
            launch_params_text: get_str(&config, "launchParameters").unwrap_or_default(),
            android_path: get_str(&config, "androidPath")
                .unwrap_or_else(|| "/storage/emulated/0/mpv/shaders/".to_string()),
            android_index: 0,
            settings_lang: Lang::from_config_value(get_str(&config, "language").as_deref()),
            settings_open_last: get_bool(&config, "onOpenLastFile", true),
            settings_pos_last: get_bool(&config, "onPosLastFile", true),
            settings_volume_plus: volume_plus,
            settings_svp: get_bool(&config, "SVP", false),
            settings_fixed_panel: get_bool(&config, "fixedControlPanel", false),
            settings_ask_preset: get_bool(&config, "askPresetOnOpen", true),
            settings_sound_offset: config
                .get("soundOffset")
                .and_then(Value::as_f64)
                .unwrap_or(0.0),
            settings_palette: Palette::from_name(
                get_str(&config, "palette").as_deref().unwrap_or("System"),
            ),
            settings_style: crate::theme::style::normalize(
                get_str(&config, "style").as_deref().unwrap_or("default"),
            )
            .to_string(),
            settings_scheme: ColorScheme::from_name(
                get_str(&config, "colorScheme")
                    .as_deref()
                    .unwrap_or("System"),
            ),
            preset_offer_quality: "HQ".to_string(),
            preset_offer_mode: "A".to_string(),
            preset_offer_dont_ask: false,
            config,
            last_mouse_pos: None,
            last_mouse_time: Instant::now(),
            click_play_armed_at: None,
            last_window_save: None,
            #[cfg(feature = "video")]
            last_log_cap: None,
            eof_handled: false,
            snap: MpvSnapshot {
                volume,
                ..Default::default()
            },
            last_poll: None,
            last_slow_poll: None,
            snap_static_key: None,
            tracks_count_cache: None,
            tracks_file_key: None,
            last_config_save: None,
            config_dirty: false,
            pending_preset: None,
            ui_ms_ema: 0.0,
            video_ms_ema: 0.0,
            about_copied_at: None,
            screenshot_pending: None,
            media_info_cache: String::new(),
            media_info_sig: None,
            time_pos_text: String::from("00:00"),
            time_dur_text: String::from("00:00"),
            time_pos_tick: i64::MIN,
            time_dur_tick: i64::MIN,
            volume_text: String::new(),
            volume_text_vol: i32::MIN,
        };
        app.apply_style(&cc.egui_ctx);
        app.apply_theme(&cc.egui_ctx);
        app.restore_startup(cli_arg);
        app.refresh_title(&cc.egui_ctx);
        app
    }

    pub(crate) fn t(&self, key: &'static str) -> &'static str {
        i18n::t_static(self.lang, key)
    }

    /// Transient video-corner message: shown for `STATUS_TTL`, then
    /// auto-cleared by `expire_status` (called every frame in `ui()`).
    pub(crate) fn set_status(&mut self, msg: String) {
        if msg.is_empty() {
            self.status.clear();
            self.status_until = None;
        } else {
            self.status = msg;
            self.status_until = Some(Instant::now() + STATUS_TTL);
        }
    }

    /// Drops an expired transient message (see `set_status`).
    pub(crate) fn expire_status(&mut self) {
        if matches!(self.status_until, Some(t) if Instant::now() >= t) {
            self.status.clear();
            self.status_until = None;
        }
    }

    /// Flushes in-memory config to disk right away (explicit user action —
    /// a crash/exit must not lose it). High-frequency updates (volume drag,
    /// playback position) still rely on the exit save instead.
    pub(crate) fn flush_config(&mut self) {
        self.last_config_save = Some(Instant::now());
        self.config_dirty = false;
        if let Err(err) = self.config.save() {
            eprintln!("cannot save config: {err:#}");
        }
    }

    /// Debounced config write for high-frequency updates (volume
    /// hotkeys/autorepeat): writes at most every `CONFIG_FLUSH_DEBOUNCE`
    /// and remembers the remainder via `config_dirty` instead of issuing
    /// dozens of writes per second. The pending write lands on the next
    /// due call or at the latest on exit (`save_on_exit` always saves).
    pub(crate) fn flush_config_debounced(&mut self) {
        use super::state::CONFIG_FLUSH_DEBOUNCE;

        let now = Instant::now();
        if self
            .last_config_save
            .is_some_and(|last| now.duration_since(last) < CONFIG_FLUSH_DEBOUNCE)
        {
            self.config_dirty = true;
            return;
        }
        self.flush_config();
    }

    /// Writes a debounced-pending config update once it is due. Called
    /// every frame from `ui()`; a no-op unless `flush_config_debounced`
    /// deferred a write earlier.
    pub(crate) fn flush_config_if_due(&mut self) {
        use super::state::CONFIG_FLUSH_DEBOUNCE;

        if !self.config_dirty {
            return;
        }
        let now = Instant::now();
        if self
            .last_config_save
            .is_none_or(|last| now.duration_since(last) >= CONFIG_FLUSH_DEBOUNCE)
        {
            self.flush_config();
        }
    }

    pub(crate) fn apply_theme(&self, ctx: &egui::Context) {
        // Palette base name + OS light/dark scheme.
        theme::apply(ctx, self.settings_palette, self.settings_scheme.is_dark());
    }

    pub(crate) fn apply_style(&self, ctx: &egui::Context) {
        crate::theme::style::apply(ctx, &self.settings_style);
    }

    /// Saved `fixedControlPanel` value (runtime behavior follows the saved
    /// config, not the unsaved settings-dialog draft).
    pub(crate) fn fixed_panel(&self) -> bool {
        get_bool(&self.config, "fixedControlPanel", false)
    }

    /// Window title, `"{name} - Anime4K Player"` when a file is open.
    pub(crate) fn refresh_title(&self, ctx: &egui::Context) {
        let title = match self.player.current_file() {
            Some(path) => {
                let s = path.to_string_lossy();
                format!("{} - {APP_NAME}", display_name_of(&s))
            }
            None => APP_NAME.to_string(),
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_choice_json_roundtrip() {
        let standard = PresetChoice::Standard {
            quality: "M".to_string(),
            mode: "A".to_string(),
        };
        assert_eq!(PresetChoice::from_json(&standard.to_json()), Some(standard));
        let hq = PresetChoice::Hq {
            name: "A (Death)".to_string(),
        };
        assert_eq!(PresetChoice::from_json(&hq.to_json()), Some(hq));
        assert_eq!(
            PresetChoice::from_json(&PresetChoice::Disabled.to_json()),
            Some(PresetChoice::Disabled)
        );
        assert_eq!(PresetChoice::from_json(&json!({})), None);
    }
}
