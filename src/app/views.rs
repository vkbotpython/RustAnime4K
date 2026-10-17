//! Панели: меню, видео, транспорт, треки, плейлист.

use super::helpers::{fmt_time, is_url_like, pick_sidecar_file, read_clipboard_text};
use super::state::{Anime4KPlayerApp, MediaInfoSig};
use super::state::{SPEEDS, TIME_SCALE};
use crate::anime4k;
use crate::mpv::TrackKind;
use serde_json::json;
use std::time::Instant;

/// Альфа подложки фулскрин-оверлеев (0-255): слегка прозрачная, видео под
/// панелью едва просвечивает, а текст и кнопки остаются родными.
const OVERLAY_ALPHA: u8 = 225;
/// Ширина выезжающего плейлиста в фулскрине (фиксированная: таскать край
/// панели поверх видео неудобно, да и не нужно).
const OVERLAY_PLAYLIST_WIDTH: f32 = 320.0;

impl Anime4KPlayerApp {
    /// Подложка фулскрин-оверлеев: цвет окна текущей темы, но слегка
    /// прозрачный. Прозрачным делается только фон — кнопки и текст поверх
    /// остаются непрозрачными и читаемыми в любой теме.
    fn overlay_fill(ui: &egui::Ui) -> egui::Color32 {
        // `window_fill` хранится premultiplied (под Mica/Acrylic — ещё и
        // полупрозрачным): каналы нужно размножить обратно, иначе подложка
        // потемнеет.
        let [r, g, b, _] = ui.visuals().window_fill.to_srgba_unmultiplied();
        egui::Color32::from_rgba_unmultiplied(r, g, b, OVERLAY_ALPHA)
    }

    pub(crate) fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menu_bar").show(ui, |ui| {
            self.menu_contents(ui);
        });
    }

    /// Верхнее меню поверх видео для фулскрина: висит на полупрозрачной
    /// подложке и не отнимает у картинки место (см. `controls_overlay`).
    pub(crate) fn menu_overlay(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let width = ctx.viewport_rect().width().max(0.0);
        egui::Area::new(egui::Id::unique("menu_overlay"))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .movable(false)
            .show(&ctx, |ui| {
                let fill = Self::overlay_fill(ui);
                egui::Frame::NONE
                    .fill(fill)
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        let w = (width - 16.0).max(0.0);
                        ui.set_min_width(w);
                        ui.set_max_width(w);
                        self.menu_contents(ui);
                    });
            });
    }

    /// Кнопки верхнего меню (общие для `menu_bar` и `menu_overlay`).
    pub(crate) fn menu_contents(&mut self, ui: &mut egui::Ui) {
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
                    for name in anime4k::ultra_hq_sorted_names() {
                        if ui.button(format!("{} {name}", self.t("Mode"))).clicked() {
                            self.apply_anime4k_hq(name);
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
    }

    /// Right-click menu.
    pub(crate) fn context_menu(&mut self, ui: &mut egui::Ui) {
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
                for name in anime4k::ultra_hq_sorted_names() {
                    if ui.button(format!("{} {name}", self.t("Mode"))).clicked() {
                        self.apply_anime4k_hq(name);
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
    pub(crate) fn video_central_panel(fullscreen: bool) -> egui::CentralPanel {
        let mut panel = egui::CentralPanel::default();
        if fullscreen {
            panel = panel.frame(egui::Frame::NONE);
        }
        panel
    }

    pub(crate) fn video_panel(&mut self, ui: &mut egui::Ui) {
        #[cfg(feature = "video")]
        if self.video.is_some() {
            // Borrow, don't clone: the old code built 2 `String` per frame
            // (`status.clone()` + `last_error().to_string()`) at 60fps.
            let has_status = !self.status.is_empty();
            let has_error = self
                .video
                .as_ref()
                .is_some_and(|r| r.last_error().is_some());
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
                    // Zero-alloc overlay: `&str` straight into the painter,
                    // no `status.clone()` / `to_string()` per frame. The
                    // painter copies glyphs into its own tessellator anyway.
                    let err_text: Option<&str> = self.video.as_ref().and_then(|r| r.last_error());
                    if let Some(err) = err_text {
                        ui.painter().text(
                            rect.left_bottom() + egui::vec2(8.0, -8.0),
                            egui::Align2::LEFT_BOTTOM,
                            err,
                            egui::FontId::proportional(13.0),
                            egui::Color32::LIGHT_RED,
                        );
                    } else if has_status {
                        ui.painter().text(
                            rect.left_bottom() + egui::vec2(8.0, -8.0),
                            egui::Align2::LEFT_BOTTOM,
                            self.status.as_str(),
                            egui::FontId::proportional(13.0),
                            egui::Color32::LIGHT_GRAY,
                        );
                    }
                    // Keep flags used (avoids unused warnings on some cfgs).
                    let _ = has_error;
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
                ui.painter().text(
                    rect.left_bottom() + egui::vec2(8.0, -8.0),
                    egui::Align2::LEFT_BOTTOM,
                    self.status.as_str(),
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

    /// Cached media-info row. Rebuilds only when snapshot inputs change;
    /// per-frame callers get the stored string instead of ~8 fresh `String`.
    pub(crate) fn cached_media_info(&mut self) -> &str {
        #[cfg(feature = "video")]
        let video_ms = self.video_ms_ema as i64;
        #[cfg(not(feature = "video"))]
        let video_ms = 0;
        // Zero-alloc fast path: compare by reference, cloning only when the
        // row actually needs a rebuild (was: 4 `String` clones per frame).
        let matches = match self.media_info_sig.as_ref() {
            Some(old) => {
                old.preset.as_str() == self.player.preset_name.as_str()
                    && old.video_format.as_deref() == self.snap.video_format.as_deref()
                    && old.audio_codec.as_deref() == self.snap.audio_codec.as_deref()
                    && old.width == self.snap.width
                    && old.height == self.snap.height
                    && old.fps_bits == self.snap.fps.map(f64::to_bits)
                    && old.frame_drops == self.snap.frame_drops
                    && old.hwdec.as_deref() == self.snap.hwdec.as_deref()
                    && old.cache_pause == self.snap.cache_pause
                    && old.has_current == self.player.current.is_some()
                    && old.ui_ms == self.ui_ms_ema as i64
                    && old.video_ms == video_ms
                    && old.lang == self.lang
            }
            None => false,
        };
        if !matches {
            let sig = MediaInfoSig {
                preset: self.player.preset_name.clone(),
                video_format: self.snap.video_format.clone(),
                audio_codec: self.snap.audio_codec.clone(),
                width: self.snap.width,
                height: self.snap.height,
                fps_bits: self.snap.fps.map(f64::to_bits),
                frame_drops: self.snap.frame_drops,
                hwdec: self.snap.hwdec.clone(),
                cache_pause: self.snap.cache_pause,
                has_current: self.player.current.is_some(),
                ui_ms: self.ui_ms_ema as i64,
                video_ms,
                lang: self.lang,
            };
            let codec = sig
                .video_format
                .clone()
                .or_else(|| sig.audio_codec.clone())
                .unwrap_or_default()
                .to_uppercase();
            let resolution = match (sig.width, sig.height) {
                (Some(w), Some(h)) => format!("{w}x{h}"),
                _ => String::new(),
            };
            let fps = match sig.fps_bits.map(f64::from_bits) {
                Some(f) if !resolution.is_empty() => format!("{f:.1} FPS"),
                _ => String::new(),
            };
            let drops = match sig.frame_drops {
                Some(n) => format!("{}: {n}", self.t("Frames lost")),
                None => String::new(),
            };
            let mut parts = [sig.preset.clone(), codec, resolution, fps, drops]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
            // Live perf readout while playing (see ui_ms_ema/video_ms_ema).
            if sig.has_current {
                parts.push(format!("UI {:.0}ms", self.ui_ms_ema));
                #[cfg(feature = "video")]
                parts.push(format!("V {:.0}ms", self.video_ms_ema));
                if sig.cache_pause {
                    parts.push("CACHE-WAIT".to_string());
                }
                if let Some(hw) = &sig.hwdec {
                    parts.push(format!("HW {hw}"));
                }
            }
            self.media_info_cache = parts.join(" | ");
            self.media_info_sig = Some(sig);
        }
        &self.media_info_cache
    }

    pub(crate) fn controls_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("controls").show(ui, |ui| {
            self.controls_contents(ui);
        });
    }

    /// Нижняя панель поверх видео для фулскрина: лежит на картинке на
    /// слегка прозрачной подложке вместо того, чтобы сжимать видео вверх
    /// (обычный `Panel::bottom` отнимает место у `CentralPanel`).
    pub(crate) fn controls_overlay(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let width = ctx.viewport_rect().width().max(0.0);
        egui::Area::new(egui::Id::unique("controls_overlay"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .movable(false)
            .show(&ctx, |ui| {
                let fill = Self::overlay_fill(ui);
                egui::Frame::NONE
                    .fill(fill)
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        let w = (width - 16.0).max(0.0);
                        ui.set_min_width(w);
                        ui.set_max_width(w);
                        self.controls_contents(ui);
                    });
            });
    }

    /// Содержимое нижней панели (общее для `controls_panel` и оверлея).
    pub(crate) fn controls_contents(&mut self, ui: &mut egui::Ui) {
        let duration = self.snap.duration.unwrap_or(0.0);
        let pos = self.snap.time_pos.unwrap_or(0.0);
        let max = (duration * TIME_SCALE).max(1.0);
        let mut slider_val = pos * TIME_SCALE;

        // Cached time labels: `fmt_time` allocates per call, and this runs
        // at 24-60fps. Display granularity is 1s, so rebuild only when the
        // whole second flips (~1 alloc/sec instead of ~120/sec).
        let pos_tick = pos.max(0.0) as i64;
        if self.time_pos_tick != pos_tick {
            self.time_pos_tick = pos_tick;
            self.time_pos_text = fmt_time(pos);
        }
        let dur_tick = duration.max(0.0) as i64;
        if self.time_dur_tick != dur_tick {
            self.time_dur_tick = dur_tick;
            self.time_dur_text = fmt_time(duration);
        }
        // Cached volume label: was `format!("{}%")` every frame.
        if self.volume_text_vol != self.player.volume {
            self.volume_text_vol = self.player.volume;
            self.volume_text = format!("{}%", self.player.volume);
        }

        // Row 1: full-width seek bar, times pinned to the window edges.
        ui.horizontal(|ui| {
            ui.monospace(self.time_pos_text.as_str());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.monospace(self.time_dur_text.as_str());
                ui.spacing_mut().slider_width = ui.available_width();
                let hover = format!("{} / {}", self.time_pos_text, self.time_dur_text);
                let resp = ui
                    .add(
                        egui::Slider::new(&mut slider_val, 0.0..=max)
                            .show_value(false)
                            .trailing_fill(true)
                            .step_by(0.5),
                    )
                    .on_hover_text(hover);
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
            // Hover still formats per frame (1 alloc), but the visible label
            // below is cached; `vol` only changes at 0.7Hz + drags.
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
            ui.monospace(self.volume_text.as_str())
                .on_hover_text(self.t("Volume level"));
        });
        ui.horizontal(|ui| {
            ui.monospace(self.cached_media_info());
        });
        if self.show_audio_menu {
            self.track_menu(ui, TrackKind::Audio);
        }
        if self.show_sub_menu {
            self.track_menu(ui, TrackKind::Sub);
        }
    }

    pub(crate) fn track_menu(&mut self, ui: &mut egui::Ui, kind: TrackKind) {
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
            let is_empty = match kind {
                TrackKind::Audio => self.snap.audio_rows.is_empty(),
                TrackKind::Sub => self.snap.sub_rows.is_empty(),
            };
            if is_empty {
                ui.label(no_tracks);
            } else {
                let mut disable_hit = false;
                if ui.button(disable).clicked() {
                    disable_hit = true;
                }
                let mut hit: Option<i64> = None;
                {
                    let rows = match kind {
                        TrackKind::Audio => &self.snap.audio_rows,
                        TrackKind::Sub => &self.snap.sub_rows,
                    };
                    for (id, label) in rows.iter() {
                        if ui.button(label.as_str()).clicked() {
                            hit = Some(*id);
                        }
                    }
                }
                if disable_hit {
                    match kind {
                        TrackKind::Audio => self.mpv.set_audio_track(0),
                        TrackKind::Sub => self.mpv.set_sub_track(0),
                    }
                }
                if let Some(id) = hit {
                    match kind {
                        TrackKind::Audio => self.mpv.set_audio_track(id),
                        TrackKind::Sub => self.mpv.set_sub_track(id),
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
                    // External file: force a track-list refetch on the next
                    // tick instead of waiting for a count change.
                    self.tracks_count_cache = None;
                    match kind {
                        TrackKind::Audio => self.add_audio_path.clear(),
                        TrackKind::Sub => self.add_sub_path.clear(),
                    }
                }
            });
        });
    }

    pub(crate) fn playlist_panel(&mut self, ui: &mut egui::Ui) {
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
                self.playlist_contents(ui);
            });
    }

    /// Плейлист поверх видео для фулскрина: выезжает справа на слегка
    /// прозрачной подложке и не сжимает картинку (см. `controls_overlay`).
    pub(crate) fn playlist_overlay(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let vp = ctx.viewport_rect();
        let width = OVERLAY_PLAYLIST_WIDTH.min(vp.width()).max(0.0);
        let height = vp.height().max(0.0);
        egui::Area::new(egui::Id::unique("playlist_overlay"))
            .anchor(egui::Align2::RIGHT_CENTER, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .movable(false)
            .show(&ctx, |ui| {
                let fill = Self::overlay_fill(ui);
                egui::Frame::NONE
                    .fill(fill)
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        let (w, h) = ((width - 16.0).max(0.0), (height - 16.0).max(0.0));
                        ui.set_min_width(w);
                        ui.set_max_width(w);
                        ui.set_min_height(h);
                        ui.set_max_height(h);
                        self.playlist_contents(ui);
                    });
            });
    }

    /// Содержимое плейлиста (общее для `playlist_panel` и оверлея).
    pub(crate) fn playlist_contents(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.t("Playlist"));
        let current = self.player.current;
        let mut chosen: Option<usize> = None;
        // Single list: cached names (rebuilt only when files change),
        // so per-frame cost is O(visible) instead of O(N) allocs.
        // NB: a vertical ScrollArea shrinks horizontally to fit its
        // content by default. The panel frame then covers only the
        // text width and anything dragged wider stays unpainted
        // (black strip). Forcing full width keeps the frame over
        // the whole allocation.
        //
        // Huge folders (10k files) would still create O(N) widgets per
        // frame: `ScrollArea` culls *painting* but not widget creation.
        // Window the list around the current file so per-frame cost stays
        // bounded; the footer reports the hidden count.
        const MAX_PLAYLIST_ROWS: usize = 2000;
        const PLAYLIST_WINDOW: usize = 1000;
        {
            let names = self.player.cached_filenames();
            let total = names.len();
            let (start, end, truncated) = if total > MAX_PLAYLIST_ROWS {
                let anchor = current.unwrap_or(0).min(total.saturating_sub(1));
                let start = anchor
                    .saturating_sub(PLAYLIST_WINDOW / 2)
                    .min(total - MAX_PLAYLIST_ROWS);
                (start, start + MAX_PLAYLIST_ROWS, true)
            } else {
                (0, total, false)
            };
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Stretch rows across the full width so the highlight
                    // and the frame cover the whole panel, not just text.
                    ui.set_width(ui.available_width());
                    if start > 0 {
                        ui.weak(format!("… {} ↑", start));
                    }
                    for (i, name) in names.iter().enumerate().skip(start).take(end - start) {
                        let selected = current == Some(i);
                        if ui.selectable_label(selected, name).clicked() {
                            chosen = Some(i);
                        }
                    }
                    if truncated {
                        ui.weak(format!("… {} ↓", total - end));
                    }
                });
        }
        if let Some(i) = chosen {
            let ctx = ui.ctx().clone();
            self.do_play_index(&ctx, i);
        }
        // Source info pinned to the bottom: footer only, no second list
        // (the old duplicate ScrollArea doubled widget cost per frame and
        // its clicks were dropped — `chosen` was never consumed there).
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
                        egui::Label::new(egui::RichText::new(parent.to_string_lossy()).monospace())
                            .truncate(),
                    );
                }
            }
        });
    }
}
