//! eframe UI: main window, all dialogs and startup logic.
//!
//! Layout: top menu bar, central video surface,
//! bottom transport controls, toggleable right playlist panel, and one
//! `egui::Window` per dialog.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::anime4k;
use crate::config::ConfigManager;
use crate::formats::{AUDIO_FORMATS, SUBTITLES_FORMATS, VIDEO_FORMATS};
use crate::i18n::{self, Lang};
use crate::mpv::{MpvBackend, Track, TrackKind};
use crate::player::{MIN_RESTORE_POSITION, PlayerState};
use crate::theme::{self, ColorScheme, Palette};
#[cfg(feature = "video")]
use crate::video::VideoRenderer;

pub const APP_NAME: &str = "Anime4K Player";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Slider granularity for the seek bar (0.1 s steps).
pub const TIME_SCALE: f64 = 10.0;

/// Idle delay before the cursor auto-hides in fullscreen.
const FULLSCREEN_CURSOR_IDLE: Duration = Duration::from_millis(2000);
/// Bottom strip (px) that reveals the control panel in fullscreen.
const FULLSCREEN_BOTTOM_ZONE: f32 = 140.0;
/// Right strip (px) that reveals the playlist in fullscreen.
const FULLSCREEN_RIGHT_ZONE: f32 = 120.0;

/// How long a transient `status` line (screenshot name, errors, "Success")
/// stays in the video corner before auto-clearing.
const STATUS_TTL: Duration = Duration::from_secs(4);

/// egui pairs clicks into a double click within 300 ms
/// (`max_double_click_delay`): a single-click play/pause newer than this
/// is reverted when the double click lands (see `consume_click_revert`),
/// so entering fullscreen never disturbs playback.
const DOUBLE_CLICK_REVERT_WINDOW: Duration = Duration::from_millis(400);

const SPEEDS: &[f64] = &[
    0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 2.25, 2.5, 2.75, 3.0,
];

/// Donation card from the README (spaced for display, plain for copying).
const DONATE_CARD_SPACED: &str = "2202 2050 1464 4675";
const DONATE_CARD_PLAIN: &str = "2202205014644675";

/// `MM:SS`, or `HH:MM:SS` for videos >= 1h.
fn fmt_time(secs: f64) -> String {
    let total = secs.max(0.0) as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h:02}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

fn shaders_dir() -> PathBuf {
    // Install/dist layout: shaders next to the exe.
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf));
    if let Some(exe_dir) = exe_dir {
        let direct = exe_dir.join("shaders");
        if direct.is_dir() {
            return direct;
        }
        // Dev layout (`target/release/anime4k-player`, repo launched from
        // anywhere): walk up from the exe dir looking for assets/shaders.
        let mut dir = exe_dir;
        for _ in 0..4 {
            let cand = dir.join("assets").join("shaders");
            if cand.is_dir() {
                return cand;
            }
            if !dir.pop() {
                break;
            }
        }
        // Install layout, even if (currently) missing — callers report it.
        return direct;
    }
    // Repo-root launch (CWD-dependent last resort).
    PathBuf::from("assets/shaders")
}

/// Names from `preset` missing under `dir`. A preset pointing at absent
/// files must never reach mpv silently: `glsl-shaders` with garbage paths
/// just doesn't upscale, looking "worse than mpv.net with the same preset".
fn missing_shader_files(dir: &Path, preset: &[String]) -> Vec<String> {
    preset
        .iter()
        .filter(|s| !dir.join(s).is_file())
        .cloned()
        .collect()
}

/// Parses the safe `name=value` subset of launch parameters
/// (arbitrary code is not supported).
pub fn parse_launch_params(text: &str) -> Vec<(String, String)> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let (name, value) = l.split_once('=')?;
            let name = name.trim().trim_start_matches('-').replace('_', "-");
            let value = value.trim().to_string();
            if name.is_empty() || value.is_empty() {
                None
            } else {
                Some((name, value))
            }
        })
        .collect()
}

fn get_str(cfg: &ConfigManager, key: &str) -> Option<String> {
    cfg.get(key).and_then(Value::as_str).map(str::to_string)
}

fn get_bool(cfg: &ConfigManager, key: &str, default: bool) -> bool {
    cfg.get(key).and_then(Value::as_bool).unwrap_or(default)
}

/// Reads the system clipboard (for the `Paste` buttons). Returns empty string on failure.
fn read_clipboard_text() -> String {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.get_text())
        .unwrap_or_default()
}

fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

fn is_url_like(s: &str) -> bool {
    s.contains("://")
}

/// Display name for the window title / playlist
/// (`basename` for files, part after `/` for URLs).
fn display_name_of(path_str: &str) -> String {
    if is_url_like(path_str) {
        path_str.rsplit('/').next().unwrap_or(path_str).to_string()
    } else {
        Path::new(path_str)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path_str)
            .to_string()
    }
}

/// Native "open media file" dialog with video/audio filters.
fn pick_media_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Video", VIDEO_FORMATS)
        .add_filter("Audio", AUDIO_FORMATS)
        .pick_file()
}

/// Native "pick subtitle/audio sidecar file" dialog.
fn pick_sidecar_file(kind: &TrackKind) -> Option<PathBuf> {
    let mut dlg = rfd::FileDialog::new();
    match kind {
        TrackKind::Sub => {
            dlg = dlg.add_filter("Subtitles", SUBTITLES_FORMATS);
        }
        TrackKind::Audio => {
            dlg = dlg
                .add_filter("Audio", AUDIO_FORMATS)
                .add_filter("Video", VIDEO_FORMATS);
        }
    }
    dlg.pick_file()
}

pub struct Anime4KPlayerApp {
    config: ConfigManager,
    lang: Lang,
    mpv: Arc<dyn MpvBackend>,
    #[cfg(feature = "video")]
    video: Option<VideoRenderer>,
    player: PlayerState,
    playlist_visible: bool,
    status: String,
    /// When the current `status` line expires (see `set_status`).
    /// `None` = no transient message to clear.
    status_until: Option<Instant>,

    // Dialog visibility + drafts.
    show_settings: bool,
    show_about: bool,
    show_open_url: bool,
    show_open_folder: bool,
    show_screenshot: bool,
    show_launch_params: bool,
    show_android: bool,
    show_reference: bool,
    show_audio_menu: bool,
    show_sub_menu: bool,
    show_preset_offer: bool,

    url_text: String,
    folder_text: String,
    add_audio_path: String,
    add_sub_path: String,
    screenshot_dir_text: String,
    launch_params_text: String,
    android_path: String,
    android_index: usize,

    settings_lang: Lang,
    settings_open_last: bool,
    settings_pos_last: bool,
    settings_volume_plus: bool,
    settings_svp: bool,
    settings_fixed_panel: bool,
    settings_ask_preset: bool,
    settings_sound_offset: f64,
    settings_palette: Palette,
    settings_style: String,
    settings_scheme: ColorScheme,

    // Anime4K preset offer dialog drafts (quality/mode to apply on choice).
    preset_offer_quality: String,
    preset_offer_mode: String,
    preset_offer_dont_ask: bool,

    // Fullscreen mouse tracking.
    last_mouse_pos: Option<egui::Pos2>,
    last_mouse_time: Instant,

    /// When a single click last toggled play/pause. A double click starts
    /// as a single click, so a still-fresh toggle is reverted when the
    /// double click switches fullscreen (see `consume_click_revert`).
    click_play_armed_at: Option<Instant>,

    /// Last disk flush of the window geometry (see `track_window_geometry`).
    last_window_save: Option<Instant>,

    /// Last `mpv.log` size-cap check (see `crate::mpv::cap_mpv_log`).
    #[cfg(feature = "video")]
    last_log_cap: Option<Instant>,

    /// Set once the current EOF episode has been handled. `eof-reached`
    /// stays true across frames (and can briefly report a stale value right
    /// after `loadfile`), so without edge detection a single EOF would
    /// restart/skip playback on every 100 ms tick.
    eof_handled: bool,

    /// Last polled mpv state (see `poll_mpv`).
    snap: MpvSnapshot,
    /// Last snapshot poll time; polling is time-gated (see `poll_mpv`).
    last_poll: Option<Instant>,
    /// Preset waiting to be applied once playback metadata is in
    /// (see `flush_pending_preset`).
    pending_preset: Option<PresetChoice>,

    /// Perf diagnostics shown in the media-info row while playing:
    /// whole-`ui()` time and the video-panel slice of it (mpv
    /// update()+render()+FBO upkeep; the deferred GL blit excluded).
    /// Distinguishes render-bound (V≈UI≫16ms) from UI-bound (UI≫V)
    /// from repaint-starved (both small, yet perceived fps low) stalls.
    ui_ms_ema: f64,
    video_ms_ema: f64,

    /// When the SBER card number was copied from the About dialog.
    /// Shows a short "copied" confirmation next to the button.
    about_copied_at: Option<Instant>,

    /// Pending screenshot-file watch, armed when a screenshot is taken.
    /// mpv writes the file asynchronously with a name it chooses itself,
    /// so completion is detected by a new file appearing in the folder
    /// (see `poll_screenshot`) instead of trusting silent dispatch.
    screenshot_pending: Option<ScreenshotWatch>,
}

/// A screenshot request waiting for its file to land on disk.
struct ScreenshotWatch {
    dir: PathBuf,
    /// File names present at dispatch; the screenshot is the newcomer.
    before: Vec<String>,
    /// Give up (and report where to look) after this instant.
    deadline: Instant,
}

/// Cached mpv state. During playback egui repaints on every video frame
/// (24-60/s via the render-context update callback); doing a dozen blocking
/// `get_property` round-trips per frame saturates the UI thread and starves
/// the player, so all per-frame UI reads go through this snapshot instead.
#[derive(Debug, Clone, Default)]
struct MpvSnapshot {
    pause: bool,
    duration: Option<f64>,
    time_pos: Option<f64>,
    eof: bool,
    volume: i32,
    video_format: Option<String>,
    audio_codec: Option<String>,
    width: Option<i64>,
    height: Option<i64>,
    fps: Option<f64>,
    frame_drops: Option<i64>,
    /// Core stall flags (see `MpvBackend::paused_for_cache/hwdec_current`):
    /// shown in media-info while playing; they distinguish demux/cache
    /// starvation and a stuck hw decoder from a slow renderer.
    cache_pause: bool,
    hwdec: Option<String>,
    /// Cached only while an audio/sub menu is open (`tracks()` fans out
    /// into hundreds of property reads — never per frame).
    tracks: Vec<Track>,
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
            preset_offer_quality: "M".to_string(),
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
            pending_preset: None,
            ui_ms_ema: 0.0,
            video_ms_ema: 0.0,
            about_copied_at: None,
            screenshot_pending: None,
        };
        app.apply_style(&cc.egui_ctx);
        app.apply_theme(&cc.egui_ctx);
        app.restore_startup(cli_arg);
        app.refresh_title(&cc.egui_ctx);
        app
    }

    fn t(&self, key: &str) -> String {
        i18n::t(self.lang, key)
    }

    /// Transient video-corner message: shown for `STATUS_TTL`, then
    /// auto-cleared by `expire_status` (called every frame in `ui()`).
    fn set_status(&mut self, msg: String) {
        if msg.is_empty() {
            self.status.clear();
            self.status_until = None;
        } else {
            self.status = msg;
            self.status_until = Some(Instant::now() + STATUS_TTL);
        }
    }

    /// Drops an expired transient message (see `set_status`).
    fn expire_status(&mut self) {
        if matches!(self.status_until, Some(t) if Instant::now() >= t) {
            self.status.clear();
            self.status_until = None;
        }
    }

    /// Flushes in-memory config to disk right away (explicit user action —
    /// a crash/exit must not lose it). High-frequency updates (volume drag,
    /// playback position) still rely on the exit save instead.
    fn flush_config(&self) {
        if let Err(err) = self.config.save() {
            eprintln!("cannot save config: {err:#}");
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        // Palette base name + OS light/dark scheme.
        theme::apply(ctx, self.settings_palette, self.settings_scheme.is_dark());
    }

    fn apply_style(&self, ctx: &egui::Context) {
        crate::theme::style::apply(ctx, &self.settings_style);
    }

    /// Saved `fixedControlPanel` value (runtime behavior follows the saved
    /// config, not the unsaved settings-dialog draft).
    fn fixed_panel(&self) -> bool {
        get_bool(&self.config, "fixedControlPanel", false)
    }

    /// Window title, `"{name} - Anime4K Player"` when a file is open.
    fn refresh_title(&self, ctx: &egui::Context) {
        let title = match self.player.current_file() {
            Some(path) => {
                let s = path.to_string_lossy();
                format!("{} - {APP_NAME}", display_name_of(&s))
            }
            None => APP_NAME.to_string(),
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }

    // -- shared actions (menu bar + context menu + shortcuts) -------------

    fn do_open_file(&mut self, ctx: &egui::Context) {
        if let Some(path) = pick_media_file() {
            self.player.open_file(&*self.mpv, &path);
            self.set_opened("file", &path.to_string_lossy());
            self.after_file_changed(ctx);
        }
    }

    fn do_open_url(&mut self, ctx: &egui::Context, link: &str) {
        self.player.open_url(&*self.mpv, link);
        self.set_opened("url", link);
        self.after_file_changed(ctx);
    }

    fn do_open_folder(&mut self, ctx: &egui::Context, folder: &Path) {
        self.playlist_visible = true;
        self.player.open_folder(&*self.mpv, folder, None);
        self.set_opened("folder", &folder.to_string_lossy());
        self.after_file_changed(ctx);
    }

    fn do_close(&mut self, ctx: &egui::Context) {
        self.player.close(&*self.mpv);
        self.config.set("opened", json!(["", ""]));
        self.show_preset_offer = false;
        self.pending_preset = None;
        self.eof_handled = false;
        self.snap.duration = None;
        self.snap.time_pos = None;
        self.snap.pause = false;
        self.refresh_title(ctx);
    }

    fn do_next(&mut self, ctx: &egui::Context) {
        if self.player.next(&*self.mpv) {
            self.after_file_changed(ctx);
        }
    }

    fn do_prev(&mut self, ctx: &egui::Context) {
        if self.player.prev(&*self.mpv) {
            self.after_file_changed(ctx);
        }
    }

    fn do_play_index(&mut self, ctx: &egui::Context, index: usize) {
        if self.player.play_index(&*self.mpv, index) {
            self.after_file_changed(ctx);
        }
    }

    /// Copies saved config into the settings drafts and opens the dialog.
    fn open_settings(&mut self) {
        self.settings_lang = Lang::from_config_value(get_str(&self.config, "language").as_deref());
        self.settings_open_last = get_bool(&self.config, "onOpenLastFile", true);
        self.settings_pos_last = get_bool(&self.config, "onPosLastFile", true);
        self.settings_volume_plus = get_bool(&self.config, "volumePlus", false);
        self.settings_svp = get_bool(&self.config, "SVP", false);
        self.settings_fixed_panel = get_bool(&self.config, "fixedControlPanel", false);
        self.settings_ask_preset = get_bool(&self.config, "askPresetOnOpen", true);
        self.settings_sound_offset = self
            .config
            .get("soundOffset")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        self.settings_palette = Palette::from_name(
            get_str(&self.config, "palette")
                .as_deref()
                .unwrap_or("System"),
        );
        self.settings_style = crate::theme::style::normalize(
            get_str(&self.config, "style")
                .as_deref()
                .unwrap_or("default"),
        )
        .to_string();
        self.settings_scheme = ColorScheme::from_name(
            get_str(&self.config, "colorScheme")
                .as_deref()
                .unwrap_or("System"),
        );
        self.show_settings = true;
    }

    /// Copies saved launch parameters into the draft and opens the dialog.
    fn open_launch(&mut self) {
        self.launch_params_text = get_str(&self.config, "launchParameters").unwrap_or_default();
        self.show_launch_params = true;
    }

    // -- per-frame helpers -------------------------------------------------

    fn track_mouse(&mut self, ctx: &egui::Context) {
        let pos = ctx.input(|i| i.pointer.latest_pos());
        if pos != self.last_mouse_pos {
            self.last_mouse_pos = pos;
            self.last_mouse_time = Instant::now();
        }
    }

    fn mouse_idle(&self) -> bool {
        self.last_mouse_time.elapsed() > FULLSCREEN_CURSOR_IDLE
    }

    fn mouse_in_bottom_zone(&self, ctx: &egui::Context) -> bool {
        let rect = ctx.viewport_rect();
        match self.last_mouse_pos {
            Some(p) => p.y > rect.bottom() - FULLSCREEN_BOTTOM_ZONE,
            None => false,
        }
    }

    fn mouse_in_right_zone(&self, ctx: &egui::Context) -> bool {
        let rect = ctx.viewport_rect();
        match self.last_mouse_pos {
            Some(p) => p.x > rect.right() - FULLSCREEN_RIGHT_ZONE,
            None => false,
        }
    }

    /// Global shortcuts (Ctrl+O/Ctrl+U/F1/F2/F4/F5/F6/F11/F12/Space/
    /// D/0/1/2/3/arrows/+/-). Skipped for single-key shortcuts while typing
    /// in a text field so text entry keeps working.
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
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

    /// Auto-advance the playlist at end-of-file (polls `eof-reached`).
    /// Fires exactly once per EOF episode (see `eof_handled`).
    /// Reads the time-gated snapshot: pausing anywhere else (e.g. mid-VO
    /// reconfig, when live reads can transiently fail) must not skip files.
    fn check_auto_next(&mut self, ctx: &egui::Context) {
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

    fn restore_startup(&mut self, cli_arg: Option<String>) {
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

    fn apply_runtime_settings(&mut self) {
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

    fn save_on_exit(&mut self) {
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

    fn set_opened(&mut self, kind: &str, target: &str) {
        self.config.set("opened", json!([kind, target]));
        self.flush_config();
    }

    /// Storage key for the current file (full path or URL), if any.
    fn current_file_key(&self) -> Option<String> {
        self.player
            .current_file()
            .map(|p| p.to_string_lossy().into_owned())
    }

    fn read_file_preset(&self, key: &str) -> Option<PresetChoice> {
        self.config
            .get("filePresets")?
            .get(key)
            .and_then(PresetChoice::from_json)
    }

    fn write_file_preset(&mut self, key: &str, choice: &PresetChoice) {
        let mut map = self
            .config
            .get("filePresets")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        map.insert(key.to_string(), choice.to_json());
        self.config.set("filePresets", Value::Object(map));
        self.flush_config();
    }

    /// Applies a stored choice without re-saving it.
    /// Returns false when it can't be built (stale data) or its shader
    /// files are missing (reported in `status`, never applied silently).
    fn apply_choice(&mut self, choice: &PresetChoice) -> bool {
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
            PresetChoice::Hq { name } => match anime4k::ultra_hq_presets().get(name.as_str()) {
                Some(preset) => match self.glsl_value(preset) {
                    Some(value) => {
                        self.player
                            .apply_preset(&*self.mpv, &format!("{name} (HQ)"), &value);
                        true
                    }
                    None => false,
                },
                None => false,
            },
        }
    }

    /// Builds the `glsl-shaders` value for `preset`, or reports missing
    /// shader files in `status` (visible in the video corner) and returns
    /// `None` so the caller skips the (ineffective) property write.
    fn glsl_value(&mut self, preset: &[String]) -> Option<String> {
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
    fn note_file_changed(&mut self) {
        // Fresh file: drop stale state so keep-last-good can't leak the
        // previous file's metadata into the new load.
        self.pending_preset = None;
        self.eof_handled = false;
        self.snap.duration = None;
        self.snap.time_pos = None;
        self.snap.pause = self.mpv.pause();
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

    /// Refreshes the mpv snapshot, time-gated to ~10 Hz. Between polls the
    /// UI reuses cached values, so 24-60 fps repaint storms during playback
    /// cost zero mpv round-trips. Transient read failures keep the last good
    /// values instead of flickering the UI.
    fn poll_mpv(&mut self) {
        let now = Instant::now();
        if self
            .last_poll
            .is_some_and(|last| now.duration_since(last) < Duration::from_millis(100))
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
        // `duration` proves the core answered: only then is the `pause`
        // read trustworthy enough to overwrite the cached value.
        if let Some(d) = self.mpv.duration() {
            self.snap.duration = Some(d);
            self.snap.pause = self.mpv.pause();
        }
        if let Some(t) = self.mpv.time_pos() {
            self.snap.time_pos = Some(t);
        }
        self.snap.eof = self.mpv.eof_reached();
        self.snap.volume = self.mpv.volume();
        self.snap.video_format = self.mpv.video_format();
        self.snap.audio_codec = self.mpv.audio_codec();
        self.snap.width = self.mpv.width();
        self.snap.height = self.mpv.height();
        self.snap.fps = self.mpv.fps();
        self.snap.frame_drops = self.mpv.frame_drop_count();
        self.snap.cache_pause = self.mpv.paused_for_cache();
        self.snap.hwdec = self.mpv.hwdec_current();
        if self.show_audio_menu || self.show_sub_menu {
            self.snap.tracks = self.mpv.tracks();
        } else {
            self.snap.tracks.clear();
        }
    }

    /// Applies a queued preset once playback metadata is available, i.e.
    /// the initial VO setup is done and a shader write can't race it.
    fn flush_pending_preset(&mut self) {
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

    fn after_file_changed(&mut self, ctx: &egui::Context) {
        self.refresh_title(ctx);
        self.note_file_changed();
    }
}

/// Per-file Anime4K choice stored under config `filePresets`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PresetChoice {
    Standard { quality: String, mode: String },
    Hq { name: String },
    Disabled,
}

impl PresetChoice {
    fn to_json(&self) -> Value {
        match self {
            Self::Standard { quality, mode } => json!({"quality": quality, "mode": mode}),
            Self::Hq { name } => json!({"hq": name}),
            Self::Disabled => json!({"disabled": true}),
        }
    }

    fn from_json(v: &Value) -> Option<Self> {
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

impl eframe::App for Anime4KPlayerApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        #[cfg(feature = "video")]
        if let Some(renderer) = self.video.as_mut() {
            renderer.destroy_gl_resources();
        }
        self.save_on_exit();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let frame_start = Instant::now();
        // Time-gated snapshot first: everything below reads cached state,
        // so repaint storms (every video frame) cost zero mpv round-trips.
        self.poll_mpv();
        self.flush_pending_preset();
        self.expire_status();
        if matches!(
            self.click_play_armed_at,
            Some(t) if t.elapsed() >= DOUBLE_CLICK_REVERT_WINDOW
        ) {
            self.click_play_armed_at = None;
        }
        self.track_window_geometry(&ctx);
        // mpv only appends to `mpv.log`: re-cap it once a minute so a spammy
        // session can never eat much disk (startup starts fresh, see `mpv.rs`).
        #[cfg(feature = "video")]
        {
            let now = Instant::now();
            if self
                .last_log_cap
                .is_none_or(|t| now.duration_since(t) >= Duration::from_secs(60))
            {
                self.last_log_cap = Some(now);
                crate::mpv::cap_mpv_log();
            }
        }
        // Per-frame repaints come from the render-context update callback
        // (see VideoRenderer::new): it fires exactly when mpv has a new
        // frame, so playback runs at video rate without a busy loop here.
        // render() itself never blocks (video-timing-offset=0, see mpv.rs):
        // keeping ui() to a few ms is what lets the window stay at display
        // rate while video advances at its own rate. This 100 ms fallback
        // only drives the slider/clock UI when no video frames arrive.
        ctx.request_repaint_after(Duration::from_millis(100));
        self.handle_dropped_files(&ctx);
        self.handle_shortcuts(&ctx);
        self.check_auto_next(&ctx);
        self.track_mouse(&ctx);

        let hide_chrome = self.player.fullscreen && !self.fixed_panel();
        if hide_chrome && self.mouse_idle() && !self.mouse_in_bottom_zone(&ctx) {
            ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::None);
        }

        if !hide_chrome {
            self.menu_bar(ui);
        }
        // egui order: top, side, bottom, central. The playlist docks
        // full-height on the right on purpose: sharing only a clean
        // T-junction with the bottom bar makes any overlap between them
        // structurally impossible (a side panel stacked *above* the bottom
        // bar kept bleeding its separator into it).
        let show_playlist =
            self.playlist_visible && (!hide_chrome || self.mouse_in_right_zone(&ctx));
        if show_playlist {
            self.playlist_panel(ui);
        }
        let show_controls = !hide_chrome || self.mouse_in_bottom_zone(&ctx);
        if show_controls {
            self.controls_panel(ui);
        }
        let video_start = Instant::now();
        self.video_panel(ui);
        let video_dt = video_start.elapsed().as_secs_f64() * 1000.0;
        self.video_ms_ema += (video_dt - self.video_ms_ema) * 0.1;
        self.dialogs(&ctx);
        let frame_dt = frame_start.elapsed().as_secs_f64() * 1000.0;
        self.ui_ms_ema += (frame_dt - self.ui_ms_ema) * 0.05;
    }
}

// ---------------------------------------------------------------------------
// Panels
// ---------------------------------------------------------------------------

impl Anime4KPlayerApp {
    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
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

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button(self.t("File"), |ui| {
                    if ui.button(self.t("Open file")).clicked() {
                        let ctx = ui.ctx().clone();
                        self.do_open_file(&ctx);
                    }
                    if ui.button(self.t("Open URL")).clicked() {
                        self.url_text.clear();
                        self.show_open_url = true;
                    }
                    if ui.button(self.t("Open folder")).clicked() {
                        self.folder_text.clear();
                        self.show_open_folder = true;
                    }
                    if ui.button(self.t("Close")).clicked() {
                        let ctx = ui.ctx().clone();
                        self.do_close(&ctx);
                    }
                    if ui.button(self.t("Exit")).clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button(self.t("Playback"), |ui| {
                    if ui.button(self.t("Play | Pause")).clicked() {
                        self.toggle_play();
                    }
                    if ui.button(self.t("Fullscreen")).clicked() {
                        let ctx = ui.ctx().clone();
                        self.toggle_fullscreen(&ctx);
                    }
                    ui.menu_button(self.t("Playback speed"), |ui| {
                        for speed in SPEEDS {
                            if ui.button(format!("x{speed:.2}")).clicked() {
                                self.mpv.set_speed(*speed);
                            }
                        }
                    });
                    if ui.button(self.t("Volume +10")).clicked() {
                        self.player.volume_up(&*self.mpv);
                    }
                    if ui.button(self.t("Volume -10")).clicked() {
                        self.player.volume_down(&*self.mpv);
                    }
                    if ui.button(self.t("Rewind +5 sec")).clicked() {
                        self.mpv.seek(5.0);
                    }
                    if ui.button(self.t("Rewind -5 sec")).clicked() {
                        self.mpv.seek(-5.0);
                    }
                    if ui.button(self.t("Zoom in")).clicked() {
                        self.mpv.set_video_zoom(self.mpv.video_zoom() + 0.01);
                    }
                    if ui.button(self.t("Zoom out")).clicked() {
                        self.mpv.set_video_zoom(self.mpv.video_zoom() - 0.01);
                    }
                });
                ui.menu_button(self.t("Increasing image quality"), |ui| {
                    if ui.button(self.t("Disable")).clicked() {
                        self.disable_anime4k();
                    }
                    for quality in anime4k::QUALITIES {
                        ui.menu_button(format!("{} {quality}", self.t("Quality")), |ui| {
                            for mode in anime4k::MODES {
                                if ui.button(format!("{} {mode}", self.t("Mode"))).clicked() {
                                    self.apply_anime4k(quality, mode);
                                }
                            }
                        });
                    }
                    ui.menu_button(format!("{} HQ", self.t("Quality")), |ui| {
                        let mut names: Vec<String> = anime4k::ultra_hq_presets()
                            .keys()
                            .map(|s| s.to_string())
                            .collect();
                        names.sort();
                        for name in names {
                            if ui.button(format!("{} {name}", self.t("Mode"))).clicked() {
                                self.apply_anime4k_hq(&name);
                            }
                        }
                    });
                });
                ui.menu_button(self.t("Other"), |ui| {
                    if ui.button(self.t("Playlist")).clicked() {
                        self.playlist_visible = !self.playlist_visible;
                    }
                    if ui.button(self.t("Take a screenshot")).clicked() {
                        self.show_screenshot = true;
                    }
                    if ui.button(self.t("Settings")).clicked() {
                        self.open_settings();
                    }
                    if ui.button(self.t("Launch parameters")).clicked() {
                        self.open_launch();
                    }
                    if ui.button(self.t("Create config for Android")).clicked() {
                        self.show_android = true;
                    }
                    if ui.button(self.t("Reference")).clicked() {
                        self.show_reference = true;
                    }
                    if ui.button(self.t("About")).clicked() {
                        self.show_about = true;
                    }
                });
            });
        });
    }

    /// Right-click menu.
    fn context_menu(&mut self, ui: &mut egui::Ui) {
        if ui.button(self.t("Open file")).clicked() {
            let ctx = ui.ctx().clone();
            self.do_open_file(&ctx);
            ui.close();
        }
        if ui.button(self.t("Open folder")).clicked() {
            self.folder_text.clear();
            self.show_open_folder = true;
            ui.close();
        }
        if ui.button(self.t("Open URL")).clicked() {
            self.url_text.clear();
            self.show_open_url = true;
            ui.close();
        }
        if ui.button(self.t("Close")).clicked() {
            let ctx = ui.ctx().clone();
            self.do_close(&ctx);
            ui.close();
        }
        ui.separator();
        ui.menu_button(self.t("Increasing image quality"), |ui| {
            if ui.button(self.t("Disable")).clicked() {
                self.disable_anime4k();
                ui.close();
            }
            for quality in anime4k::QUALITIES {
                ui.menu_button(format!("{} {quality}", self.t("Quality")), |ui| {
                    for mode in anime4k::MODES {
                        if ui.button(format!("{} {mode}", self.t("Mode"))).clicked() {
                            self.apply_anime4k(quality, mode);
                            ui.close();
                        }
                    }
                });
            }
            ui.menu_button(format!("{} HQ", self.t("Quality")), |ui| {
                let mut names: Vec<String> = anime4k::ultra_hq_presets()
                    .keys()
                    .map(|s| s.to_string())
                    .collect();
                names.sort();
                for name in names {
                    if ui.button(format!("{} {name}", self.t("Mode"))).clicked() {
                        self.apply_anime4k_hq(&name);
                        ui.close();
                    }
                }
            });
        });
        ui.separator();
        if ui.button(self.t("Play | Pause")).clicked() {
            self.toggle_play();
            ui.close();
        }
        ui.menu_button(self.t("Playback speed"), |ui| {
            for speed in SPEEDS {
                if ui.button(format!("x{speed:.2}")).clicked() {
                    self.mpv.set_speed(*speed);
                    ui.close();
                }
            }
        });
        if ui.button(self.t("Playlist")).clicked() {
            self.playlist_visible = !self.playlist_visible;
            ui.close();
        }
        if ui.button(self.t("Zoom in")).clicked() {
            self.mpv.set_video_zoom(self.mpv.video_zoom() + 0.01);
            ui.close();
        }
        if ui.button(self.t("Zoom out")).clicked() {
            self.mpv.set_video_zoom(self.mpv.video_zoom() - 0.01);
            ui.close();
        }
        if ui.button(self.t("Take a screenshot")).clicked() {
            self.show_screenshot = true;
            ui.close();
        }
        if ui.button(self.t("Fullscreen")).clicked() {
            let ctx = ui.ctx().clone();
            self.toggle_fullscreen(&ctx);
            ui.close();
        }
        ui.separator();
        if ui.button(self.t("Settings")).clicked() {
            self.open_settings();
            ui.close();
        }
        if ui.button(self.t("About")).clicked() {
            self.show_about = true;
            ui.close();
        }
        ui.separator();
        if ui.button(self.t("Exit")).clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            ui.close();
        }
    }

    /// Central panel for the video surface. Fullscreen is pure cinema:
    /// the default 8px panel margin (`Frame::central_panel`) is dropped so
    /// the picture reaches the screen edges instead of sitting in a black
    /// frame. Windowed mode keeps the margin.
    fn video_central_panel(fullscreen: bool) -> egui::CentralPanel {
        let mut panel = egui::CentralPanel::default();
        if fullscreen {
            panel = panel.frame(egui::Frame::NONE);
        }
        panel
    }

    fn video_panel(&mut self, ui: &mut egui::Ui) {
        #[cfg(feature = "video")]
        if self.video.is_some() {
            let status = self.status.clone();
            // Render errors are invisible without a console, so they take
            // over the status corner until rendering recovers.
            let render_error = self
                .video
                .as_ref()
                .and_then(|r| r.last_error().map(str::to_string));
            let resp = Self::video_central_panel(self.player.fullscreen)
                .show(ui, |ui| {
                    // A wide playlist can starve the center of space: clamp
                    // the allocation so a negative size can't invert the rect
                    // and splat paint outside the panel.
                    let available = ui.available_size();
                    let available = egui::vec2(available.x.max(0.0), available.y.max(0.0));
                    let (rect, resp) = ui.allocate_exact_size(available, egui::Sense::click());
                    if rect.width() <= 0.0 || rect.height() <= 0.0 {
                        return resp;
                    }
                    ui.painter().rect_filled(rect, 0.0, egui::Color32::BLACK);
                    if let Some(renderer) = self.video.as_mut() {
                        renderer.show(ui, rect);
                    }
                    if let Some(err) = render_error {
                        ui.painter().text(
                            rect.left_bottom() + egui::vec2(8.0, -8.0),
                            egui::Align2::LEFT_BOTTOM,
                            err,
                            egui::FontId::proportional(13.0),
                            egui::Color32::LIGHT_RED,
                        );
                    } else if !status.is_empty() {
                        ui.painter().text(
                            rect.left_bottom() + egui::vec2(8.0, -8.0),
                            egui::Align2::LEFT_BOTTOM,
                            status,
                            egui::FontId::proportional(13.0),
                            egui::Color32::LIGHT_GRAY,
                        );
                    }
                    resp
                })
                .inner;
            // Single click = play/pause, double click = fullscreen. A double
            // click physically starts with a single click that already
            // toggled playback: revert that toggle instead of delaying
            // single clicks (see `consume_click_revert`).
            use egui::PointerButton;
            let fullscreen_click = resp.double_clicked_by(PointerButton::Primary);
            let play_click = !fullscreen_click && resp.clicked_by(PointerButton::Primary);
            resp.context_menu(|ui| {
                self.context_menu(ui);
            });
            if fullscreen_click {
                if self.consume_click_revert() {
                    self.toggle_play();
                }
                let ctx = ui.ctx().clone();
                self.toggle_fullscreen(&ctx);
            } else if play_click {
                self.toggle_play();
                self.click_play_armed_at = Some(Instant::now());
            }
            return;
        }
        let mut clicked_play = false;
        let mut clicked_fullscreen = false;
        Self::video_central_panel(self.player.fullscreen).show(ui, |ui| {
            // Same degenerate-rect guard as the video branch above.
            let available = ui.available_size();
            let available = egui::vec2(available.x.max(0.0), available.y.max(0.0));
            let (rect, resp) = ui.allocate_exact_size(available, egui::Sense::click());
            if rect.width() <= 0.0 || rect.height() <= 0.0 {
                return;
            }
            ui.painter().rect_filled(rect, 0.0, egui::Color32::BLACK);
            if self.player.current.is_none() {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "▶",
                    egui::FontId::proportional(64.0),
                    egui::Color32::GRAY,
                );
            }
            if !self.status.is_empty() {
                let status = self.status.clone();
                ui.painter().text(
                    rect.left_bottom() + egui::vec2(8.0, -8.0),
                    egui::Align2::LEFT_BOTTOM,
                    status,
                    egui::FontId::proportional(13.0),
                    egui::Color32::LIGHT_GRAY,
                );
            }
            use egui::PointerButton;
            if resp.double_clicked_by(PointerButton::Primary) {
                clicked_fullscreen = true;
            } else if resp.clicked_by(PointerButton::Primary) {
                clicked_play = true;
            }
            resp.context_menu(|ui| {
                self.context_menu(ui);
            });
        });
        if clicked_fullscreen {
            // Same revert as the video branch above: the double click's
            // first tap already toggled playback.
            if self.consume_click_revert() {
                self.toggle_play();
            }
            let ctx = ui.ctx().clone();
            self.toggle_fullscreen(&ctx);
        } else if clicked_play {
            self.toggle_play();
            self.click_play_armed_at = Some(Instant::now());
        }
    }

    fn media_info(&self) -> String {
        let codec = self
            .snap
            .video_format
            .clone()
            .or_else(|| self.snap.audio_codec.clone())
            .unwrap_or_default()
            .to_uppercase();
        let resolution = match (self.snap.width, self.snap.height) {
            (Some(w), Some(h)) => format!("{w}x{h}"),
            _ => String::new(),
        };
        let fps = match self.snap.fps {
            Some(f) if !resolution.is_empty() => format!("{f:.1} FPS"),
            _ => String::new(),
        };
        let drops = match self.snap.frame_drops {
            Some(n) => format!("{}: {n}", self.t("Frames lost")),
            None => String::new(),
        };
        let mut parts = [
            self.player.preset_name.clone(),
            codec,
            resolution,
            fps,
            drops,
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
        // Live perf readout while playing (see ui_ms_ema/video_ms_ema).
        if self.player.current.is_some() {
            parts.push(format!("UI {:.0}ms", self.ui_ms_ema));
            #[cfg(feature = "video")]
            parts.push(format!("V {:.0}ms", self.video_ms_ema));
            if self.snap.cache_pause {
                parts.push("CACHE-WAIT".to_string());
            }
            if let Some(hw) = &self.snap.hwdec {
                parts.push(format!("HW {hw}"));
            }
        }
        parts.join(" | ")
    }

    fn controls_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("controls").show(ui, |ui| {
            let duration = self.snap.duration.unwrap_or(0.0);
            let pos = self.snap.time_pos.unwrap_or(0.0);
            let max = (duration * TIME_SCALE).max(1.0);
            let mut slider_val = pos * TIME_SCALE;

            // Row 1: full-width seek bar, times pinned to the window edges.
            ui.horizontal(|ui| {
                ui.monospace(fmt_time(pos));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.monospace(fmt_time(duration));
                    ui.spacing_mut().slider_width = ui.available_width();
                    let resp = ui
                        .add(
                            egui::Slider::new(&mut slider_val, 0.0..=max)
                                .show_value(false)
                                .trailing_fill(true)
                                .step_by(0.5),
                        )
                        .on_hover_text(format!("{} / {}", fmt_time(pos), fmt_time(duration)));
                    // Seek on real gestures only — NEVER on bare `changed()`.
                    // With `step_by` the slider rounds the live `time-pos`
                    // back into `slider_val` every frame (`slider.rs`:
                    // `set_value` rounds to the step grid, `add_contents`
                    // then marks `changed`), so `changed()` is true on
                    // every frame without any user input. Seeking there
                    // resets the demuxer/decoder continuously: video crawls
                    // frame-by-frame, audio chops into seek-bounded pieces
                    // and playback eventually wedges. `dragged` covers
                    // scrubbing, `clicked` covers click-to-jump; rounding
                    // alone triggers neither.
                    if resp.dragged() || resp.clicked() {
                        let new_pos = slider_val / TIME_SCALE;
                        self.mpv.set_time_pos(new_pos);
                        // Optimistic: don't wait for the next poll tick.
                        self.snap.time_pos = Some(new_pos);
                    }
                });
            });
            ui.horizontal(|ui| {
                if ui
                    .button("⏮")
                    .on_hover_text(self.t("Previous file"))
                    .clicked()
                {
                    let ctx = ui.ctx().clone();
                    self.do_prev(&ctx);
                }
                let playing = !self.snap.pause && self.snap.duration.is_some();
                let play_tip = if playing {
                    self.t("Pause")
                } else {
                    self.t("Play")
                };
                if ui
                    .button(if playing { "⏸" } else { "▶" })
                    .on_hover_text(play_tip)
                    .clicked()
                {
                    self.toggle_play();
                }
                if ui.button("⏭").on_hover_text(self.t("Next file")).clicked() {
                    let ctx = ui.ctx().clone();
                    self.do_next(&ctx);
                }
                if ui.button("⛶").on_hover_text(self.t("Fullscreen")).clicked() {
                    let ctx = ui.ctx().clone();
                    self.toggle_fullscreen(&ctx);
                }
                if ui
                    .button("🎵")
                    .on_hover_text(self.t("Soundtrack"))
                    .clicked()
                {
                    self.show_audio_menu = !self.show_audio_menu;
                    self.show_sub_menu = false;
                }
                if ui.button("💬").on_hover_text(self.t("Subtitles")).clicked() {
                    self.show_sub_menu = !self.show_sub_menu;
                    self.show_audio_menu = false;
                }
                if ui.button("☰").on_hover_text(self.t("Menu")).clicked() {
                    self.playlist_visible = !self.playlist_visible;
                }

                let mut vol = self.snap.volume;
                let vmax = self.player.max_volume_self();
                let vol_resp = ui
                    .add(egui::Slider::new(&mut vol, 0..=vmax).show_value(false))
                    .on_hover_text(format!("{}: {}%", self.t("Volume level"), vol));
                if vol_resp.changed() {
                    self.player.set_volume(&*self.mpv, vol);
                    // Optimistic: don't wait for the next poll tick.
                    self.snap.volume = self.player.volume;
                    self.config.set("volume", json!(self.player.volume));
                }
                // Slider drags fire `changed()` every frame: flush once on
                // release instead of writing config.json continuously.
                if vol_resp.drag_stopped() {
                    self.flush_config();
                }
                ui.monospace(format!("{}%", self.player.volume))
                    .on_hover_text(self.t("Volume level"));
            });
            ui.horizontal(|ui| {
                ui.monospace(self.media_info());
            });
            if self.show_audio_menu {
                self.track_menu(ui, TrackKind::Audio);
            }
            if self.show_sub_menu {
                self.track_menu(ui, TrackKind::Sub);
            }
        });
    }

    fn track_menu(&mut self, ui: &mut egui::Ui, kind: TrackKind) {
        let title = match kind {
            TrackKind::Audio => self.t("Soundtrack"),
            TrackKind::Sub => self.t("Subtitles"),
        };
        let no_tracks = match kind {
            TrackKind::Audio => self.t("No audio tracks"),
            TrackKind::Sub => self.t("No subtitles"),
        };
        let disable = self.t("Disable");
        let paste_label = self.t("Paste");
        let (add_label, select) = match kind {
            TrackKind::Audio => (self.t("Add audio track"), self.t("Select")),
            TrackKind::Sub => (self.t("Add subtitles"), self.t("Select")),
        };
        ui.group(|ui| {
            ui.label(title);
            let tracks: Vec<_> = self
                .snap
                .tracks
                .iter()
                .filter(|t| t.kind == kind)
                .cloned()
                .collect();
            if tracks.is_empty() {
                ui.label(no_tracks);
            } else {
                if ui.button(disable).clicked() {
                    match kind {
                        TrackKind::Audio => self.mpv.set_audio_track(0),
                        TrackKind::Sub => self.mpv.set_sub_track(0),
                    }
                }
                for track in tracks {
                    if ui
                        .button(format!("{}) {}", track.id, track.display()))
                        .clicked()
                    {
                        match kind {
                            TrackKind::Audio => self.mpv.set_audio_track(track.id),
                            TrackKind::Sub => self.mpv.set_sub_track(track.id),
                        }
                    }
                }
            }
            ui.horizontal(|ui| {
                ui.label(add_label);
                let choose_file = self.t("Choose file");
                let path_field = match kind {
                    TrackKind::Audio => &mut self.add_audio_path,
                    TrackKind::Sub => &mut self.add_sub_path,
                };
                ui.text_edit_singleline(path_field);
                if ui.button(paste_label).clicked() {
                    let text = read_clipboard_text();
                    if !text.trim().is_empty() {
                        *path_field = text.trim().to_string();
                    }
                }
                let picked = ui
                    .button(choose_file)
                    .clicked()
                    .then(|| pick_sidecar_file(&kind))
                    .flatten();
                if let Some(picked) = picked {
                    *path_field = picked.to_string_lossy().into_owned();
                }
                let path = path_field.trim().to_string();
                if ui.button(select).clicked() && !path.is_empty() {
                    match kind {
                        TrackKind::Audio => self.mpv.add_audio_file(&path),
                        TrackKind::Sub => self.mpv.add_sub_file(&path),
                    }
                    match kind {
                        TrackKind::Audio => self.add_audio_path.clear(),
                        TrackKind::Sub => self.add_sub_path.clear(),
                    }
                }
            });
        });
    }

    fn playlist_panel(&mut self, ui: &mut egui::Ui) {
        // Drag width persists across frames: without a cap the panel can be
        // pulled wider than the window allows (or outlive a window shrink
        // while wide), starving the video of space. Clamp to a viewport
        // fraction. Content is separately forced to fill the full width
        // below so the frame never leaves an unpainted strip.
        const MIN_WIDTH: f32 = 200.0;
        let max_width = (ui.ctx().viewport_rect().width() * 0.45).max(MIN_WIDTH);
        egui::Panel::right("playlist")
            .size_range(MIN_WIDTH..=max_width)
            .show(ui, |ui| {
                ui.heading(self.t("Playlist"));
                let names = self.player.filenames_only();
                let current = self.player.current;
                let mut chosen: Option<usize> = None;
                // NB: a vertical ScrollArea shrinks horizontally to fit its
                // content by default. The panel frame then covers only the
                // text width and anything dragged wider stays unpainted
                // (black strip). Forcing full width keeps the frame over
                // the whole allocation.
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Stretch rows across the full width so the highlight
                        // and the frame cover the whole panel, not just text.
                        ui.set_width(ui.available_width());
                        for (i, name) in names.iter().enumerate() {
                            let selected = current == Some(i);
                            if ui.selectable_label(selected, name).clicked() {
                                chosen = Some(i);
                            }
                        }
                    });
                if let Some(i) = chosen {
                    let ctx = ui.ctx().clone();
                    self.do_play_index(&ctx, i);
                }
                // List above, source info pinned to the bottom: the scroll
                // area fills only the *remaining* height, so the footer can
                // never overflow the panel into the controls bar.
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    // Source info: folder path, or "URL" for links.
                    // Truncated: with the width capped, a long path must not
                    // paint past the panel frame.
                    if let Some(cur) = self.player.current_file() {
                        let s = cur.to_string_lossy();
                        if is_url_like(&s) {
                            ui.monospace("URL");
                        } else if let Some(parent) = cur.parent() {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(parent.to_string_lossy()).monospace(),
                                )
                                .truncate(),
                            );
                        }
                    }
                    // NB: a vertical ScrollArea shrinks horizontally to fit
                    // its content by default. The panel frame then covers
                    // only the text width and anything dragged wider stays
                    // unpainted (black strip). Forcing full width keeps the
                    // frame over the whole allocation.
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            // Stretch rows across the full width so the
                            // highlight and the frame cover the whole panel,
                            // not just text.
                            ui.set_width(ui.available_width());
                            for (i, name) in names.iter().enumerate() {
                                let selected = current == Some(i);
                                if ui.selectable_label(selected, name).clicked() {
                                    chosen = Some(i);
                                }
                            }
                        });
                });
            });
    }

    fn toggle_play(&mut self) {
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

    fn toggle_fullscreen(&mut self, ctx: &egui::Context) {
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
    fn consume_click_revert(&mut self) -> bool {
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
    fn track_window_geometry(&mut self, ctx: &egui::Context) {
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

    /// Explicit user choice: remember a standard preset for the current
    /// file; applied once playback is stable (see `flush_pending_preset`).
    fn apply_anime4k(&mut self, quality: &str, mode: &str) {
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
    fn apply_anime4k_hq(&mut self, name: &str) {
        if anime4k::ultra_hq_presets().contains_key(name) {
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
    fn disable_anime4k(&mut self) {
        if let Some(key) = self.current_file_key() {
            self.write_file_preset(&key, &PresetChoice::Disabled);
        }
        self.pending_preset = Some(PresetChoice::Disabled);
    }
}

// ---------------------------------------------------------------------------
// Dialogs (one `egui::Window` per dialog)
// ---------------------------------------------------------------------------

impl Anime4KPlayerApp {
    fn dialogs(&mut self, ctx: &egui::Context) {
        self.poll_screenshot();
        self.settings_dialog(ctx);
        self.about_dialog(ctx);
        self.preset_offer_dialog(ctx);
        self.open_url_dialog(ctx);
        self.open_folder_dialog(ctx);
        self.screenshot_dialog(ctx);
        self.launch_params_dialog(ctx);
        self.android_dialog(ctx);
        self.reference_dialog(ctx);
    }

    fn settings_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_settings {
            return;
        }
        let title = self.t("Settings");
        let language_label = self.t("Language selection");
        let open_last_label = self.t("On startup open the last opened file");
        let pos_last_label = self.t("Set the position of the last opened file");
        let volume_plus_label = self.t("Increase maximum volume up to 150%");
        let svp_label = self.t("Activate SVP");
        let fixed_label = self.t("Fixed multimedia control panel");
        let ask_preset_label = self.t("Offer to choose an Anime4K preset when opening a file");
        let offset_label = self.t("Sound offset");
        let palette_label = self.t("Palette");
        let style_label = self.t("Theme style");
        let scheme_label = self.t("Color scheme");
        let save_label = self.t("Save");
        let cancel_label = self.t("Cancel");
        let scheme_system = self.t("System");
        let scheme_dark = self.t("Dark");
        let scheme_light = self.t("Light");
        let mut open = self.show_settings;
        let mut save = false;
        let mut cancel = false;
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .min_width(300.0)
            .show(ctx, |ui| {
                ui.label(language_label);
                egui::ComboBox::from_id_salt("lang")
                    .selected_text(self.settings_lang.config_value())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.settings_lang,
                            Lang::Auto,
                            Lang::Auto.config_value(),
                        );
                        for lang in Lang::all() {
                            ui.selectable_value(
                                &mut self.settings_lang,
                                *lang,
                                lang.config_value(),
                            );
                        }
                    });
                ui.checkbox(&mut self.settings_open_last, open_last_label);
                ui.checkbox(&mut self.settings_pos_last, pos_last_label);
                ui.checkbox(&mut self.settings_volume_plus, volume_plus_label);
                ui.checkbox(&mut self.settings_svp, svp_label);
                ui.checkbox(&mut self.settings_fixed_panel, fixed_label);
                ui.checkbox(&mut self.settings_ask_preset, ask_preset_label);
                ui.horizontal(|ui| {
                    ui.label(offset_label);
                    // Sound offset: -100..100 s, step 0.01.
                    ui.add(
                        egui::DragValue::new(&mut self.settings_sound_offset)
                            .speed(0.01)
                            .range(-100.0..=100.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(palette_label);
                    egui::ComboBox::from_id_salt("palette")
                        .selected_text(self.settings_palette.name())
                        .show_ui(ui, |ui| {
                            for name in Palette::all_names() {
                                ui.selectable_value(
                                    &mut self.settings_palette,
                                    Palette::from_name(name),
                                    *name,
                                );
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label(scheme_label);
                    egui::ComboBox::from_id_salt("scheme")
                        .selected_text(self.settings_scheme.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.settings_scheme,
                                ColorScheme::System,
                                scheme_system.as_str(),
                            );
                            ui.selectable_value(
                                &mut self.settings_scheme,
                                ColorScheme::Dark,
                                scheme_dark.as_str(),
                            );
                            ui.selectable_value(
                                &mut self.settings_scheme,
                                ColorScheme::Light,
                                scheme_light.as_str(),
                            );
                        });
                });
                ui.horizontal(|ui| {
                    ui.label(style_label);
                    egui::ComboBox::from_id_salt("qstyle")
                        .selected_text(self.settings_style.clone())
                        .show_ui(ui, |ui| {
                            for name in crate::theme::style::all_names() {
                                ui.selectable_value(
                                    &mut self.settings_style,
                                    (*name).to_string(),
                                    *name,
                                );
                            }
                        });
                });
                ui.horizontal(|ui| {
                    if ui.button(save_label).clicked() {
                        save = true;
                    }
                    if ui.button(cancel_label).clicked() {
                        cancel = true;
                    }
                });
            });
        if cancel {
            open = false;
        }
        self.show_settings = open;
        if save {
            self.save_settings(ctx);
            self.show_settings = false;
        }
    }

    fn save_settings(&mut self, ctx: &egui::Context) {
        self.config
            .set("onOpenLastFile", json!(self.settings_open_last));
        self.config
            .set("onPosLastFile", json!(self.settings_pos_last));
        self.config.set("SVP", json!(self.settings_svp));
        self.config
            .set("volumePlus", json!(self.settings_volume_plus));
        self.config
            .set("fixedControlPanel", json!(self.settings_fixed_panel));
        self.config
            .set("askPresetOnOpen", json!(self.settings_ask_preset));
        self.settings_sound_offset = self.settings_sound_offset.clamp(-100.0, 100.0);
        self.config
            .set("soundOffset", json!(self.settings_sound_offset));
        self.config
            .set("palette", json!(self.settings_palette.name()));
        self.config.set("style", json!(self.settings_style.clone()));
        self.config
            .set("colorScheme", json!(self.settings_scheme.name()));
        let lang_value = self.settings_lang.config_value().to_string();
        if get_str(&self.config, "language").as_deref() != Some(lang_value.as_str()) {
            self.config.set("language", json!(lang_value));
            self.lang = if self.settings_lang == Lang::Auto {
                i18n::detect_system_lang()
            } else {
                self.settings_lang
            };
        }
        self.player.volume_plus = self.settings_volume_plus;
        if !self.settings_volume_plus && self.player.volume > 100 {
            self.player.set_volume(&*self.mpv, 100);
        }
        self.mpv.set_audio_delay(self.settings_sound_offset);
        self.mpv.set_svp(self.settings_svp);
        self.apply_style(ctx);
        self.apply_theme(ctx);
        self.flush_config();
    }

    fn about_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_about {
            return;
        }
        let title = self.t("About");
        let about_text = self.t("About program");
        let copy_label = self.t("Copy");
        let mpv_version = self.mpv.mpv_version();
        let russian = self.lang == Lang::Russian;
        let copied_label = if russian {
            "Скопировано"
        } else {
            "Copied"
        };
        let copied =
            matches!(self.about_copied_at, Some(t) if t.elapsed() < Duration::from_secs(2));
        let mut open = self.show_about;
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .min_width(460.0)
            .show(ctx, |ui| {
                // --- шапка: иконка, название, версия, слоган ---
                ui.vertical_centered(|ui| {
                    ui.add(
                        egui::Image::from_bytes(
                            "bytes://anime4k-player-icon",
                            include_bytes!("../assets/images/anime4k-player-icon.png"),
                        )
                        .max_width(96.0),
                    );
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(APP_NAME).size(22.0).strong());
                    ui.label(
                        egui::RichText::new(format!(" v{VERSION} "))
                            .size(13.0)
                            .strong()
                            .color(egui::Color32::WHITE)
                            .background_color(ui.visuals().hyperlink_color),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new("Rust · egui · libmpv · Anime4K")
                            .size(12.5)
                            .italics()
                            .color(ui.visuals().weak_text_color()),
                    );
                });
                ui.add_space(4.0);
                // --- стек технологий ---
                ui.vertical_centered(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        for (name, color) in [
                            ("Rust", egui::Color32::from_rgb(206, 92, 0)),
                            ("egui", egui::Color32::from_rgb(25, 118, 210)),
                            ("libmpv", egui::Color32::from_rgb(123, 31, 162)),
                            ("Anime4K", egui::Color32::from_rgb(211, 47, 47)),
                        ] {
                            ui.label(
                                egui::RichText::new(format!(" {name} "))
                                    .size(12.5)
                                    .strong()
                                    .color(egui::Color32::WHITE)
                                    .background_color(color),
                            );
                        }
                    });
                });
                ui.separator();
                // --- описание ---
                ui.add(egui::Label::new(egui::RichText::new(about_text).size(13.5)).wrap());
                ui.add_space(4.0);
                // --- движок и лицензия ---
                egui::Grid::new("about_info")
                    .num_columns(2)
                    .spacing([12.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(if russian { "Движок" } else { "Engine" })
                                .strong()
                                .size(12.5),
                        );
                        ui.monospace(&mpv_version);
                        ui.end_row();
                        ui.label(
                            egui::RichText::new(if russian {
                                "Лицензия"
                            } else {
                                "License"
                            })
                            .strong()
                            .size(12.5),
                        );
                        ui.monospace("GPL-3.0-or-later");
                        ui.end_row();
                    });
                ui.add_space(6.0);
                // --- связь и исходный код ---
                egui::Frame::new()
                    .fill(ui.visuals().faint_bg_color)
                    .inner_margin(10.0)
                    .corner_radius(8.0)
                    .stroke(egui::Stroke::new(
                        1.0,
                        ui.visuals().widgets.noninteractive.bg_stroke.color,
                    ))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(if russian {
                                "Связь и исходный код"
                            } else {
                                "Contact & source"
                            })
                            .strong()
                            .size(13.0),
                        );
                        ui.horizontal(|ui| {
                            ui.label("YouTube:");
                            ui.hyperlink("https://www.youtube.com/@avencores/");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Telegram:");
                            ui.hyperlink("https://t.me/avencoresyt");
                        });
                        ui.horizontal(|ui| {
                            ui.label("VK:");
                            ui.hyperlink("https://vk.ru/avencoresreuploads");
                        });
                        ui.horizontal(|ui| {
                            ui.label(if russian { "Дзен:" } else { "Dzen:" });
                            ui.hyperlink("https://dzen.ru/avencores");
                        });
                        ui.horizontal(|ui| {
                            ui.label("GitHub:");
                            ui.hyperlink("https://github.com/vkbotpython/RustAnime4K");
                        });
                    });
                ui.add_space(6.0);
                // --- донат ---
                egui::Frame::new()
                    .fill(ui.visuals().faint_bg_color)
                    .inner_margin(10.0)
                    .corner_radius(8.0)
                    .stroke(egui::Stroke::new(
                        1.0,
                        ui.visuals().widgets.noninteractive.bg_stroke.color,
                    ))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(if russian {
                                "Поддержать автора"
                            } else {
                                "Support the author"
                            })
                            .strong()
                            .size(13.0),
                        );
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(" SBER ")
                                    .strong()
                                    .size(12.5)
                                    .color(egui::Color32::WHITE)
                                    .background_color(egui::Color32::from_rgb(46, 125, 50)),
                            );
                            ui.monospace(DONATE_CARD_SPACED);
                            if ui.button(copy_label).clicked() {
                                // Bank apps paste card numbers without spaces.
                                ui.ctx().copy_text(DONATE_CARD_PLAIN.to_string());
                                self.about_copied_at = Some(Instant::now());
                            }
                            if copied {
                                ui.label(
                                    egui::RichText::new(copied_label)
                                        .strong()
                                        .size(12.5)
                                        .color(egui::Color32::from_rgb(46, 125, 50)),
                                );
                            }
                        });
                    });
            });
        self.show_about = open;
    }

    /// Offered when a video starts and the file has no saved preset:
    /// choose an Anime4K preset or disable it entirely. The choice is
    /// remembered per file (`filePresets`); closing without choosing
    /// asks again on the next file.
    fn preset_offer_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_preset_offer {
            return;
        }
        let Some(current) = self.player.current_file().cloned() else {
            self.show_preset_offer = false;
            return;
        };
        let title = self.t("Choose an Anime4K preset");
        let quality_label = self.t("Quality");
        let mode_label = self.t("Mode");
        let apply_label = self.t("Apply");
        let disable_label = self.t("Disable");
        let dont_ask_label = self.t("Don't ask again");
        let name = display_name_of(&current.to_string_lossy());

        // Modes depend on the selected quality (standard 6 vs HQ list).
        let hq_names: Vec<String> = anime4k::ultra_hq_presets()
            .keys()
            .map(|s| s.to_string())
            .collect();
        let mut hq_names = hq_names;
        hq_names.sort();
        let is_hq = self.preset_offer_quality == "HQ";
        if is_hq {
            if !hq_names.contains(&self.preset_offer_mode) {
                self.preset_offer_mode = hq_names.first().cloned().unwrap_or_default();
            }
        } else {
            if !anime4k::MODES.contains(&self.preset_offer_mode.as_str()) {
                self.preset_offer_mode = "A".to_string();
            }
            if !anime4k::QUALITIES.contains(&self.preset_offer_quality.as_str()) {
                self.preset_offer_quality = "M".to_string();
            }
        }

        let mut open = self.show_preset_offer;
        let mut applied = false;
        let mut disabled = false;
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .min_width(380.0)
            .show(ctx, |ui| {
                ui.monospace(&name);
                ui.horizontal(|ui| {
                    ui.label(quality_label);
                    egui::ComboBox::from_id_salt("offer-quality")
                        .selected_text(self.preset_offer_quality.clone())
                        .show_ui(ui, |ui| {
                            for quality in anime4k::QUALITIES
                                .iter()
                                .map(|s| s.to_string())
                                .chain(std::iter::once("HQ".to_string()))
                            {
                                ui.selectable_value(
                                    &mut self.preset_offer_quality,
                                    quality.clone(),
                                    quality,
                                );
                            }
                        });
                    ui.label(mode_label);
                    egui::ComboBox::from_id_salt("offer-mode")
                        .selected_text(self.preset_offer_mode.clone())
                        .show_ui(ui, |ui| {
                            if is_hq {
                                for mode in &hq_names {
                                    ui.selectable_value(
                                        &mut self.preset_offer_mode,
                                        mode.clone(),
                                        mode,
                                    );
                                }
                            } else {
                                for mode in anime4k::MODES {
                                    ui.selectable_value(
                                        &mut self.preset_offer_mode,
                                        (*mode).to_string(),
                                        *mode,
                                    );
                                }
                            }
                        });
                });
                ui.horizontal(|ui| {
                    if ui.button(apply_label).clicked() {
                        applied = true;
                    }
                    if ui.button(disable_label).clicked() {
                        disabled = true;
                    }
                });
                ui.checkbox(&mut self.preset_offer_dont_ask, dont_ask_label);
            });
        if applied {
            if is_hq {
                self.apply_anime4k_hq(&self.preset_offer_mode.clone());
            } else {
                let quality = self.preset_offer_quality.clone();
                let mode = self.preset_offer_mode.clone();
                self.apply_anime4k(&quality, &mode);
            }
            self.finish_preset_offer();
            self.show_preset_offer = false;
        } else if disabled {
            self.disable_anime4k();
            self.finish_preset_offer();
            self.show_preset_offer = false;
        } else {
            self.show_preset_offer = open;
        }
    }

    /// Persists the "don't ask again" opt-out after an explicit choice.
    fn finish_preset_offer(&mut self) {
        if self.preset_offer_dont_ask {
            self.config.set("askPresetOnOpen", json!(false));
            self.flush_config();
        }
    }

    fn open_url_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_open_url {
            return;
        }
        let title = self.t("Opening a link");
        let prompt = self.t("Enter the URL");
        let clear = self.t("Clear");
        let paste = self.t("Paste");
        let go = self.t("Select");
        let cancel = self.t("Cancel");
        let mut open = self.show_open_url;
        let mut chosen: Option<String> = None;
        let mut cancelled = false;
        let history = self
            .config
            .get("linksHistory")
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .min_width(480.0)
            .show(ctx, |ui| {
                ui.label(prompt);
                ui.text_edit_singleline(&mut self.url_text);
                ui.horizontal(|ui| {
                    if ui.button(paste).clicked() {
                        let text = read_clipboard_text();
                        if !text.trim().is_empty() {
                            self.url_text = text.trim().to_string();
                        }
                    }
                    if ui.button(clear).clicked() {
                        self.config.set("linksHistory", json!(Vec::<String>::new()));
                        self.flush_config();
                    }
                });
                for link in history.iter().filter_map(|v| v.as_str()) {
                    if ui.link(link).clicked() {
                        self.url_text = link.to_string();
                    }
                }
                ui.horizontal(|ui| {
                    if ui.button(go).clicked() {
                        let link = self.url_text.trim().to_string();
                        if !link.is_empty() {
                            let mut updated = history.clone();
                            updated.retain(|v| v.as_str() != Some(link.as_str()));
                            updated.insert(0, Value::String(link.clone()));
                            self.config.set("linksHistory", Value::Array(updated));
                            self.flush_config();
                            chosen = Some(link);
                        }
                    }
                    if ui.button(cancel).clicked() {
                        cancelled = true;
                    }
                });
            });
        if cancelled {
            open = false;
        }
        self.show_open_url = open;
        if let Some(link) = chosen {
            self.do_open_url(ctx, &link);
            self.show_open_url = false;
        }
    }

    fn open_folder_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_open_folder {
            return;
        }
        let title = self.t("Opening a folder");
        let prompt = self.t("Select a folder");
        let clear = self.t("Clear");
        let paste = self.t("Paste");
        let go = self.t("Select");
        let cancel = self.t("Cancel");
        let err = self.t("Error");
        let mut open = self.show_open_folder;
        let mut chosen: Option<String> = None;
        let mut cancelled = false;
        let history = self
            .config
            .get("foldersHistory")
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .min_width(480.0)
            .show(ctx, |ui| {
                ui.label(prompt);
                ui.text_edit_singleline(&mut self.folder_text);
                ui.horizontal(|ui| {
                    let dir = ui
                        .button(self.t("Select"))
                        .clicked()
                        .then(|| rfd::FileDialog::new().pick_folder())
                        .flatten();
                    if let Some(dir) = dir {
                        self.folder_text = dir.to_string_lossy().into_owned();
                    }
                    if ui.button(paste).clicked() {
                        let text = read_clipboard_text();
                        if !text.trim().is_empty() {
                            self.folder_text = text.trim().to_string();
                        }
                    }
                    if ui.button(clear).clicked() {
                        self.config
                            .set("foldersHistory", json!(Vec::<String>::new()));
                        self.flush_config();
                    }
                });
                for folder in history.iter().filter_map(|v| v.as_str()) {
                    if ui.link(folder).clicked() {
                        self.folder_text = folder.to_string();
                    }
                }
                ui.horizontal(|ui| {
                    if ui.button(go).clicked() {
                        let folder = self.folder_text.trim().to_string();
                        if !folder.is_empty() {
                            let mut updated = history.clone();
                            updated.retain(|v| v.as_str() != Some(folder.as_str()));
                            updated.insert(0, Value::String(folder.clone()));
                            self.config.set("foldersHistory", Value::Array(updated));
                            self.flush_config();
                            chosen = Some(folder);
                        }
                    }
                    if ui.button(cancel).clicked() {
                        cancelled = true;
                    }
                });
            });
        if cancelled {
            open = false;
        }
        self.show_open_folder = open;
        if let Some(folder) = chosen {
            let path = Path::new(&folder);
            if path.is_dir() {
                self.do_open_folder(ctx, path);
            } else {
                self.set_status(format!("{err}: {folder}"));
            }
            self.show_open_folder = false;
        }
    }

    /// Non-blocking completion check for an armed screenshot watch.
    /// Called every frame: reports the landed file by name, or — past the
    /// deadline — points at `mpv.log` instead of pretending it worked.
    fn poll_screenshot(&mut self) {
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

    fn screenshot_dialog(&mut self, ctx: &egui::Context) {
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

    fn launch_params_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_launch_params {
            return;
        }
        let title = self.t("Launch parameters");
        let prompt = self.t("Manual launch parameters");
        let save_label = self.t("Save");
        let apply_label = self.t("Apply");
        let cancel_label = self.t("Cancel");
        let success = self.t("Success");
        let err = self.t("Error");
        let mut open = self.show_launch_params;
        let mut saved = false;
        let mut cancelled = false;
        let mut applied_ok: Option<bool> = None;
        egui::Window::new(title)
            .open(&mut open)
            .min_width(420.0)
            .show(ctx, |ui| {
                ui.label(prompt);
                ui.label("name=value");
                // Buttons pinned to the bottom, editor fills the rest with
                // an explicit size: a fixed-rows TextEdit lets the window
                // snap back to content height, making vertical resize
                // impossible. Filling the allocation keeps dragged height.
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(save_label).clicked() {
                            saved = true;
                        }
                        if ui.button(apply_label).clicked() {
                            let mut ok = true;
                            for (name, value) in
                                parse_launch_params(&self.launch_params_text.clone())
                            {
                                if name == "volume" {
                                    if let Ok(v) = value.parse::<i32>() {
                                        self.player.set_volume(&*self.mpv, v);
                                    } else {
                                        ok = false;
                                    }
                                } else {
                                    self.mpv.set_option_str(&name, &value);
                                }
                            }
                            saved = true;
                            applied_ok = Some(ok);
                        }
                        if ui.button(cancel_label).clicked() {
                            // Discard the draft on Cancel.
                            cancelled = true;
                        }
                    });
                    let avail = ui.available_size();
                    ui.add_sized(
                        egui::vec2(avail.x, avail.y.max(120.0)),
                        egui::TextEdit::multiline(&mut self.launch_params_text),
                    );
                });
            });
        if cancelled {
            self.launch_params_text = get_str(&self.config, "launchParameters").unwrap_or_default();
            open = false;
        }
        if saved {
            self.config
                .set("launchParameters", json!(self.launch_params_text.clone()));
            self.flush_config();
        }
        match applied_ok {
            Some(true) => self.set_status(success),
            Some(false) => self.set_status(err),
            None => {}
        }
        self.show_launch_params = open;
    }

    fn android_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_android {
            return;
        }
        let title = self.t("Create config for Android");
        let intro = self.t(
            "You can use this config to use the Anime4K algorithm in the mpv video player on android",
        );
        let path_label = self.t("Enter the path to the shaders");
        let preset_label = self.t("Select the algorithm configuration");
        let selected_label = self.t("Selected");
        let all_label = self.t("All");
        let mut combos: Vec<(String, String, String)> = Vec::new();
        for quality in anime4k::QUALITIES {
            for mode in anime4k::MODES {
                combos.push((
                    format!("{} {mode} ({quality})", self.t("Mode")),
                    quality.to_string(),
                    mode.to_string(),
                ));
            }
        }
        if self.android_index >= combos.len() {
            self.android_index = 0;
        }
        let mut open = self.show_android;
        egui::Window::new(title).open(&mut open).show(ctx, |ui| {
            ui.label(intro);
            ui.label(path_label);
            ui.text_edit_singleline(&mut self.android_path);
            ui.label(preset_label);
            egui::ComboBox::from_id_salt("android-preset")
                .selected_text(&combos[self.android_index].0)
                .show_ui(ui, |ui| {
                    for (i, (label, _, _)) in combos.iter().enumerate() {
                        ui.selectable_value(&mut self.android_index, i, label);
                    }
                });
            let selected_text = combos[self.android_index].0.clone();
            let quality = combos[self.android_index].1.clone();
            let mode = combos[self.android_index].2.clone();
            let single = anime4k::create_preset(&quality, &mode)
                .map(|p| anime4k::android_config_line(&p, &self.android_path.clone()))
                .unwrap_or_default();
            ui.horizontal(|ui| {
                if ui.button(selected_label).clicked() {
                    ui.ctx().copy_text(format!("# {selected_text}\n{single}"));
                }
                if ui.button(all_label).clicked() {
                    let mut blocks = Vec::new();
                    for (label, q, m) in &combos {
                        if let Ok(p) = anime4k::create_preset(q, m) {
                            blocks.push(format!(
                                "# {label}\n# {}",
                                anime4k::android_config_line(&p, &self.android_path)
                            ));
                        }
                    }
                    ui.ctx().copy_text(blocks.join("\n\n"));
                }
            });
            // No auto-shrink: otherwise the window snaps back to content
            // height on release and vertical resize is impossible.
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.monospace(format!("# {selected_text}\n{single}"));
                });
        });
        // The path field has no Save button (actions only copy to the
        // clipboard): remember it on close so it survives a restart.
        if self.show_android && !open {
            self.config
                .set("androidPath", json!(self.android_path.clone()));
            self.flush_config();
        }
        self.show_android = open;
    }

    fn reference_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_reference {
            return;
        }
        let title = self.t("Reference");
        let russian = self.lang == Lang::Russian;
        let mut open = self.show_reference;
        egui::Window::new(title)
            .open(&mut open)
            .default_size([680.0, 560.0])
            .min_width(540.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        crate::reference::show(ui, russian);
                    });
            });
        self.show_reference = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_params_parse() {
        let params = parse_launch_params(
            "# comment\nvolume=80\n--hwdec=auto\nbad line\nempty=\n audio-delay = 0.2 ",
        );
        assert_eq!(
            params,
            vec![
                ("volume".to_string(), "80".to_string()),
                ("hwdec".to_string(), "auto".to_string()),
                ("audio-delay".to_string(), "0.2".to_string()),
            ]
        );
    }

    #[test]
    fn time_format() {
        assert_eq!(fmt_time(0.0), "00:00");
        assert_eq!(fmt_time(65.0), "01:05");
        assert_eq!(fmt_time(-3.0), "00:00");
        assert_eq!(fmt_time(3600.0), "01:00:00");
        assert_eq!(fmt_time(3723.0), "01:02:03");
    }

    #[test]
    fn display_name_of_url_and_file() {
        assert_eq!(
            display_name_of("https://example.com/video.mp4"),
            "video.mp4"
        );
        assert_eq!(display_name_of("/media/a.mkv"), "a.mkv");
    }

    #[test]
    fn missing_shader_files_detects_absent() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.glsl"), "// x").unwrap();
        let preset = vec!["a.glsl".to_string(), "b.glsl".to_string()];
        assert_eq!(
            missing_shader_files(dir.path(), &preset),
            vec!["b.glsl".to_string()]
        );
        assert!(missing_shader_files(dir.path(), &["a.glsl".to_string()]).is_empty());
    }

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
