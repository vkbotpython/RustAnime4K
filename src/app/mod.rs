//! Окно плеера: состояние, панели, ввод и диалоги.

mod dialogs;
mod helpers;
mod input;
mod playback;
mod session;
mod state;
mod views;

pub use self::state::{APP_NAME, Anime4KPlayerApp};

use std::time::{Duration, Instant};

use self::state::DOUBLE_CLICK_REVERT_WINDOW;

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
        // session can never eat much disk (startup starts fresh, see `mpv`).
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
        // render() itself never blocks (video-timing-offset=0, see mpv):
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
