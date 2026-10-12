//! eframe UI (port of `main.py`: `MainWindow` + all dialogs + startup logic).
//!
//! Layout mirrors the Qt original: top menu bar, central video surface,
//! bottom transport controls, toggleable right playlist panel, and one
//! `egui::Window` per former `QDialog`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};

use crate::anime4k;
use crate::config::ConfigManager;
use crate::formats::{AUDIO_FORMATS, SUBTITLES_FORMATS, VIDEO_FORMATS};
use crate::i18n::{self, Lang};
use crate::mpv::{MpvBackend, TrackKind};
use crate::player::{PlayerState, MIN_RESTORE_POSITION};
use crate::theme::{self, Palette};
#[cfg(feature = "video")]
use crate::video::VideoRenderer;

pub const APP_NAME: &str = "Anime Player";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Slider granularity, mirrors `TIME_SCALE = 10`.
pub const TIME_SCALE: f64 = 10.0;

const SPEEDS: &[f64] = &[
    0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 2.25, 2.5, 2.75, 3.0,
];

fn mmss(secs: f64) -> String {
    let total = secs.max(0.0) as u64;
    format!("{:02}:{:02}", total / 60, total % 60)
}

fn shaders_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));
    if let Some(dir) = exe_dir.map(|d| d.join("shaders")) {
        if dir.is_dir() {
            return dir;
        }
    }
    PathBuf::from("assets/shaders")
}

/// Parses the safe `name=value` subset of launch parameters
/// (replaces the Python `exec(...)`; arbitrary code is not supported).
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

pub struct AnimePlayerApp {
    config: ConfigManager,
    lang: Lang,
    mpv: Arc<dyn MpvBackend>,
    #[cfg(feature = "video")]
    video: Option<VideoRenderer>,
    player: PlayerState,
    playlist_visible: bool,
    status: String,

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
    settings_sound_offset: f64,
    settings_palette: Palette,
}

impl AnimePlayerApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        mpv: Arc<dyn MpvBackend>,
        cli_arg: Option<String>,
    ) -> Self {
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
            url_text: String::new(),
            folder_text: String::new(),
            add_audio_path: String::new(),
            add_sub_path: String::new(),
            screenshot_dir_text: String::new(),
            launch_params_text: get_str(&config, "launchParameters").unwrap_or_default(),
            android_path: "/storage/emulated/0/mpv/shaders/".to_string(),
            android_index: 0,
            settings_lang: Lang::from_config_value(get_str(&config, "language").as_deref()),
            settings_open_last: get_bool(&config, "onOpenLastFile", true),
            settings_pos_last: get_bool(&config, "onPosLastFile", true),
            settings_volume_plus: volume_plus,
            settings_svp: get_bool(&config, "SVP", false),
            settings_fixed_panel: get_bool(&config, "fixedControlPanel", false),
            settings_sound_offset: config
                .get("soundOffset")
                .and_then(Value::as_f64)
                .unwrap_or(0.0),
            settings_palette: Palette::from_name(
                get_str(&config, "palette").as_deref().unwrap_or("System"),
            ),
            config,
        };
        app.apply_theme(&cc.egui_ctx);
        app.restore_startup(cli_arg);
        app
    }

    fn t(&self, key: &str) -> String {
        i18n::t(self.lang, key)
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        // Mirrors the Qt logic: palette name + OS light/dark scheme.
        // TODO: read the real OS scheme from the viewport once eframe exposes it.
        theme::apply(ctx, self.settings_palette, true);
    }

    fn restore_startup(&mut self, cli_arg: Option<String>) {
        if let Some(arg) = cli_arg {
            // Mirrors start_player(args): CLI arg wins, no last-file restore.
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
        if open_last {
            if let Some(opened) = self.config.get("opened").and_then(Value::as_array) {
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
        }
        self.apply_runtime_settings();
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
    }

    fn save_on_exit(&mut self) {
        // Mirrors Player.save_parameters.
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
        if let Err(err) = self.config.save() {
            eprintln!("cannot save config: {err:#}");
        }
    }

    fn set_opened(&mut self, kind: &str, target: &str) {
        self.config.set("opened", json!([kind, target]));
    }
}

impl eframe::App for AnimePlayerApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        #[cfg(feature = "video")]
        if let Some(renderer) = self.video.as_mut() {
            renderer.destroy_gl_resources();
        }
        self.save_on_exit();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Mirrors the QTimer(100ms) tick: poll mpv state and repaint.
        ctx.request_repaint_after(Duration::from_millis(100));
        self.handle_dropped_files(&ctx);
        self.menu_bar(ui);
        if self.playlist_visible {
            self.playlist_panel(ui);
        }
        self.controls_panel(ui);
        self.video_panel(ui);
        self.dialogs(&ctx);
    }
}

// ---------------------------------------------------------------------------
// Panels
// ---------------------------------------------------------------------------

impl AnimePlayerApp {
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
            if crate::formats::is_subtitles_file(&path.to_string_lossy()) && path.is_file() {
                self.mpv.add_sub_file(&path.to_string_lossy());
            } else if path.is_file() {
                self.player.open_file(&*self.mpv, &path);
                if first_new {
                    self.set_opened("file", &path.to_string_lossy());
                    first_new = false;
                } else {
                    self.player.add_file(path);
                }
            } else if path.is_dir() {
                self.playlist_visible = true;
                self.player.open_folder(&*self.mpv, &path, None);
                self.set_opened("folder", &path.to_string_lossy());
                break;
            }
        }
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button(self.t("File"), |ui| {
                    if ui.button(self.t("Open file")).clicked() {
                        if let Some(path) = pick_media_file() {
                            self.player.open_file(&*self.mpv, &path);
                            self.set_opened("file", &path.to_string_lossy());
                        }
                    }
                    if ui.button(self.t("Open URL")).clicked() {
                        self.show_open_url = true;
                    }
                    if ui.button(self.t("Open folder")).clicked() {
                        self.show_open_folder = true;
                    }
                    if ui.button(self.t("Close")).clicked() {
                        self.player.close(&*self.mpv);
                        self.config.set("opened", json!(["", ""]));
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
                        self.toggle_fullscreen(ui.ctx());
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
                        self.player.disable_anime4k(&*self.mpv);
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
                        self.show_settings = true;
                    }
                    if ui.button(self.t("Launch parameters")).clicked() {
                        self.show_launch_params = true;
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

    fn video_panel(&mut self, ui: &mut egui::Ui) {
        #[cfg(feature = "video")]
        if let Some(renderer) = self.video.as_mut() {
            let status = self.status.clone();
            egui::CentralPanel::default().show(ui, |ui| {
                let available = ui.available_size();
                let (rect, _resp) = ui.allocate_exact_size(available, egui::Sense::click());
                ui.painter().rect_filled(rect, 0.0, egui::Color32::BLACK);
                renderer.show(ui, rect);
                if !status.is_empty() {
                    ui.painter().text(
                        rect.left_bottom() + egui::vec2(8.0, -8.0),
                        egui::Align2::LEFT_BOTTOM,
                        status,
                        egui::FontId::proportional(13.0),
                        egui::Color32::LIGHT_GRAY,
                    );
                }
            });
            return;
        }
        egui::CentralPanel::default().show(ui, |ui| {
            let available = ui.available_size();
            let (rect, _resp) = ui.allocate_exact_size(available, egui::Sense::click());
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
        });
    }

    fn media_info(&self) -> String {
        let codec = self
            .mpv
            .video_format()
            .or_else(|| self.mpv.audio_codec())
            .unwrap_or_default()
            .to_uppercase();
        let resolution = match (self.mpv.width(), self.mpv.height()) {
            (Some(w), Some(h)) => format!("{w}x{h}"),
            _ => String::new(),
        };
        let fps = match self.mpv.fps() {
            Some(f) if !resolution.is_empty() => format!("{f:.1} FPS"),
            _ => String::new(),
        };
        let drops = match self.mpv.frame_drop_count() {
            Some(n) => format!("{}: {n}", self.t("Frames lost")),
            None => String::new(),
        };
        [
            self.player.preset_name.clone(),
            codec,
            resolution,
            fps,
            drops,
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" | ")
    }

    fn controls_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("controls").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⏮").clicked() {
                    self.player.prev(&*self.mpv);
                }
                let playing = !self.mpv.pause() && self.mpv.duration().is_some();
                if ui.button(if playing { "⏸" } else { "▶" }).clicked() {
                    self.toggle_play();
                }
                if ui.button("⏭").clicked() {
                    self.player.next(&*self.mpv);
                }
                if ui.button("⛶").clicked() {
                    let ctx = ui.ctx().clone();
                    self.toggle_fullscreen(&ctx);
                }
                if ui.button("🎵").clicked() {
                    self.show_audio_menu = !self.show_audio_menu;
                    self.show_sub_menu = false;
                }
                if ui.button("💬").clicked() {
                    self.show_sub_menu = !self.show_sub_menu;
                    self.show_audio_menu = false;
                }
                if ui.button("☰").clicked() {
                    self.playlist_visible = !self.playlist_visible;
                }

                let duration = self.mpv.duration().unwrap_or(0.0);
                let pos = self.mpv.time_pos().unwrap_or(0.0);
                let max = (duration * TIME_SCALE).max(1.0);
                let mut slider_val = pos * TIME_SCALE;
                let resp = ui.add(egui::Slider::new(&mut slider_val, 0.0..=max).show_value(false));
                if resp.changed() {
                    self.mpv.set_time_pos(slider_val / TIME_SCALE);
                }
                ui.monospace(format!("{} / {}", mmss(pos), mmss(duration)));

                let mut vol = self.mpv.volume();
                let vmax = self.player.max_volume_self();
                if ui
                    .add(egui::Slider::new(&mut vol, 0..=vmax).show_value(false))
                    .changed()
                {
                    self.player.set_volume(&*self.mpv, vol);
                    self.config.set("volume", json!(self.player.volume));
                }
                ui.monospace(format!("{}%", self.player.volume));
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
        let (add_label, select) = match kind {
            TrackKind::Audio => (self.t("Add audio track"), self.t("Select")),
            TrackKind::Sub => (self.t("Add subtitles"), self.t("Select")),
        };
        ui.group(|ui| {
            ui.label(title);
            let tracks: Vec<_> = self
                .mpv
                .tracks()
                .into_iter()
                .filter(|t| t.kind == kind)
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
                if ui.button(choose_file).clicked() {
                    if let Some(picked) = pick_sidecar_file(&kind) {
                        *path_field = picked.to_string_lossy().into_owned();
                    }
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
        egui::Panel::right("playlist").show(ui, |ui| {
            ui.heading(self.t("Playlist"));
            let names = self.player.filenames_only();
            let current = self.player.current;
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, name) in names.iter().enumerate() {
                    let selected = current == Some(i);
                    if ui.selectable_label(selected, name).clicked() {
                        self.player.play_index(&*self.mpv, i);
                    }
                }
            });
            if let Some(cur) = self.player.current_file() {
                if let Some(parent) = cur.parent() {
                    ui.monospace(parent.to_string_lossy());
                }
            }
        });
    }

    fn toggle_play(&mut self) {
        if self.player.current.is_none() {
            return;
        }
        if self.mpv.duration().is_none() {
            if let Some(file) = self.player.current_file().cloned() {
                self.mpv.play(&file.to_string_lossy());
                self.mpv.set_pause(false);
            }
        } else {
            self.mpv.set_pause(!self.mpv.pause());
        }
    }

    fn toggle_fullscreen(&mut self, ctx: &egui::Context) {
        self.player.fullscreen = !self.player.fullscreen;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.player.fullscreen));
    }

    fn apply_anime4k(&mut self, quality: &str, mode: &str) {
        match anime4k::create_preset(quality, mode) {
            Ok(preset) => {
                let value = anime4k::preset_to_glsl_shaders(&shaders_dir(), &preset);
                let name = format!("{mode} ({quality})");
                self.player.apply_preset(&*self.mpv, &name, &value);
            }
            Err(err) => self.status = format!("{}: {err:#}", self.t("Error")),
        }
    }

    fn apply_anime4k_hq(&mut self, name: &str) {
        if let Some(preset) = anime4k::ultra_hq_presets().get(name) {
            let value = anime4k::preset_to_glsl_shaders(&shaders_dir(), preset);
            let label = format!("{name} (HQ)");
            self.player.apply_preset(&*self.mpv, &label, &value);
        }
    }
}

// ---------------------------------------------------------------------------
// Dialogs (one Window per former QDialog)
// ---------------------------------------------------------------------------

impl AnimePlayerApp {
    fn dialogs(&mut self, ctx: &egui::Context) {
        self.settings_dialog(ctx);
        self.about_dialog(ctx);
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
        let offset_label = self.t("Sound offset");
        let palette_label = self.t("Palette");
        let save_label = self.t("Save");
        let mut open = self.show_settings;
        let mut save = false;
        egui::Window::new(title).open(&mut open).show(ctx, |ui| {
            ui.label(language_label);
            egui::ComboBox::from_id_salt("lang")
                .selected_text(self.settings_lang.config_value())
                .show_ui(ui, |ui| {
                    for lang in [
                        Lang::Auto,
                        Lang::English,
                        Lang::Russian,
                        Lang::Japanese,
                        Lang::PortugueseBrazil,
                    ] {
                        ui.selectable_value(&mut self.settings_lang, lang, lang.config_value());
                    }
                });
            ui.checkbox(&mut self.settings_open_last, open_last_label);
            ui.checkbox(&mut self.settings_pos_last, pos_last_label);
            ui.checkbox(&mut self.settings_volume_plus, volume_plus_label);
            ui.checkbox(&mut self.settings_svp, svp_label);
            ui.checkbox(&mut self.settings_fixed_panel, fixed_label);
            ui.horizontal(|ui| {
                ui.label(offset_label);
                ui.add(egui::DragValue::new(&mut self.settings_sound_offset).speed(0.1));
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
            if ui.button(save_label).clicked() {
                save = true;
            }
        });
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
            .set("soundOffset", json!(self.settings_sound_offset));
        self.config
            .set("palette", json!(self.settings_palette.name()));
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
        self.apply_theme(ctx);
    }

    fn about_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_about {
            return;
        }
        let title = self.t("About");
        let about_text = self.t("About program");
        let version = self.mpv.mpv_version();
        let mut open = self.show_about;
        egui::Window::new(title).open(&mut open).show(ctx, |ui| {
            ui.heading(format!("{APP_NAME} v{VERSION}"));
            ui.label(version);
            ui.label(about_text);
        });
        self.show_about = open;
    }

    fn open_url_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_open_url {
            return;
        }
        let title = self.t("Opening a link");
        let prompt = self.t("Enter the URL");
        let clear = self.t("Clear");
        let go = self.t("Select");
        let mut open = self.show_open_url;
        let mut chosen: Option<String> = None;
        let history = self
            .config
            .get("linksHistory")
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
        egui::Window::new(title).open(&mut open).show(ctx, |ui| {
            ui.label(prompt);
            ui.text_edit_singleline(&mut self.url_text);
            if ui.button(clear).clicked() {
                self.config.set("linksHistory", json!(Vec::<String>::new()));
            }
            for link in &history {
                if let Some(link) = link.as_str() {
                    if ui.link(link).clicked() {
                        self.url_text = link.to_string();
                    }
                }
            }
            if ui.button(go).clicked() {
                let link = self.url_text.trim().to_string();
                if !link.is_empty() {
                    let mut updated = history.clone();
                    updated.retain(|v| v.as_str() != Some(link.as_str()));
                    updated.insert(0, Value::String(link.clone()));
                    self.config.set("linksHistory", Value::Array(updated));
                    chosen = Some(link);
                }
            }
        });
        self.show_open_url = open;
        if let Some(link) = chosen {
            self.player.open_url(&*self.mpv, &link);
            self.set_opened("url", &link);
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
        let go = self.t("Select");
        let err = self.t("Error");
        let mut open = self.show_open_folder;
        let mut chosen: Option<String> = None;
        let history = self
            .config
            .get("foldersHistory")
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
        egui::Window::new(title).open(&mut open).show(ctx, |ui| {
            ui.label(prompt);
            ui.text_edit_singleline(&mut self.folder_text);
            ui.horizontal(|ui| {
                if ui.button(self.t("Select")).clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        self.folder_text = dir.to_string_lossy().into_owned();
                    }
                }
                if ui.button(clear).clicked() {
                    self.config
                        .set("foldersHistory", json!(Vec::<String>::new()));
                }
            });
            for folder in &history {
                if let Some(folder) = folder.as_str() {
                    if ui.link(folder).clicked() {
                        self.folder_text = folder.to_string();
                    }
                }
            }
            if ui.button(go).clicked() {
                let folder = self.folder_text.trim().to_string();
                if !folder.is_empty() {
                    let mut updated = history.clone();
                    updated.retain(|v| v.as_str() != Some(folder.as_str()));
                    updated.insert(0, Value::String(folder.clone()));
                    self.config.set("foldersHistory", Value::Array(updated));
                    chosen = Some(folder);
                }
            }
        });
        self.show_open_folder = open;
        if let Some(folder) = chosen {
            let path = Path::new(&folder);
            if path.is_dir() {
                self.playlist_visible = true;
                self.player.open_folder(&*self.mpv, path, None);
                self.set_opened("folder", &folder);
            } else {
                self.status = format!("{err}: {folder}");
            }
            self.show_open_folder = false;
        }
    }

    fn screenshot_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_screenshot {
            return;
        }
        let title = self.t("Screenshot");
        let prompt = self.t("Enter folder path for screenshots");
        let save = self.t("Save");
        let err = self.t("Error");
        let mut open = self.show_screenshot;
        let mut done = false;
        let mut failed = false;
        egui::Window::new(title).open(&mut open).show(ctx, |ui| {
            ui.label(prompt);
            ui.text_edit_singleline(&mut self.screenshot_dir_text);
            ui.horizontal(|ui| {
                if ui.button(self.t("Select")).clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        self.screenshot_dir_text = dir.to_string_lossy().into_owned();
                    }
                }
                if ui.button(save).clicked() {
                    let dir = self.screenshot_dir_text.trim().to_string();
                    if !dir.is_empty() {
                        self.mpv.set_screenshot_directory(&dir);
                        self.mpv.screenshot();
                        done = true;
                    } else {
                        failed = true;
                    }
                }
            });
        });
        if failed {
            self.status = err;
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
        let success = self.t("Success");
        let err = self.t("Error");
        let mut open = self.show_launch_params;
        let mut saved = false;
        let mut applied_ok: Option<bool> = None;
        egui::Window::new(title).open(&mut open).show(ctx, |ui| {
            ui.label(prompt);
            ui.label("name=value");
            ui.add(
                egui::TextEdit::multiline(&mut self.launch_params_text)
                    .desired_rows(8)
                    .desired_width(f32::INFINITY),
            );
            ui.horizontal(|ui| {
                if ui.button(save_label).clicked() {
                    saved = true;
                }
                if ui.button(apply_label).clicked() {
                    let mut ok = true;
                    for (name, value) in parse_launch_params(&self.launch_params_text.clone()) {
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
            });
        });
        if saved {
            self.config
                .set("launchParameters", json!(self.launch_params_text.clone()));
        }
        match applied_ok {
            Some(true) => self.status = success,
            Some(false) => self.status = err,
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
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.monospace(format!("# {selected_text}\n{single}"));
            });
        });
        self.show_android = open;
    }

    fn reference_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_reference {
            return;
        }
        let title = self.t("Reference");
        let text = if self.lang == Lang::Russian {
            include_str!("../assets/docs/GLSL_Instructions_Advanced_ru.txt")
        } else {
            include_str!("../assets/docs/GLSL_Instructions_Advanced.txt")
        };
        let mut open = self.show_reference;
        egui::Window::new(title)
            .open(&mut open)
            .default_size([560.0, 420.0])
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.monospace(text);
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
        assert_eq!(mmss(0.0), "00:00");
        assert_eq!(mmss(65.0), "01:05");
        assert_eq!(mmss(-3.0), "00:00");
    }
}
