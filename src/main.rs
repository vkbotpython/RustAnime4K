//! Anime Player — Rust port: video player with Anime4K upscaling.

// Hide the console window on Windows (GUI app; mpv logs don't need a terminal).
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod anime4k;
mod app;
mod config;
mod formats;
mod i18n;
mod mpv;
mod player;
mod playlist;
mod reference;
mod theme;
#[cfg(feature = "video")]
mod video;

use std::sync::Arc;

use clap::Parser;

use app::{AnimePlayerApp, APP_NAME};
use mpv::MpvBackend;

#[derive(Parser, Debug)]
#[command(name = "anime-player", about = "Video player with Anime4K upscaling")]
struct Cli {
    /// Media file, folder or URL to open (skips restoring the last session).
    input: Option<String>,
}

fn create_backend() -> Arc<dyn MpvBackend> {
    #[cfg(feature = "video")]
    {
        match mpv::LibMpv::new() {
            Ok(backend) => return Arc::new(backend),
            Err(err) => eprintln!("libmpv init failed, falling back to mock backend: {err:#}"),
        }
    }
    Arc::new(mpv::MockMpv::new())
}

/// Title-bar / taskbar icon, embedded at compile time so it works without
/// the `assets/` folder next to the exe. Falls back to the default icon
/// if the PNG ever fails to decode.
fn load_app_icon() -> Arc<egui::IconData> {
    eframe::icon_data::from_png_bytes(include_bytes!("../assets/images/anime-player-icon.png"))
        .map(Arc::new)
        .unwrap_or_else(|err| {
            eprintln!("cannot decode app icon: {err:#}");
            Arc::new(egui::IconData::default())
        })
}

fn main() -> eframe::Result<()> {
    let cli = Cli::parse();
    let backend = create_backend();

    let viewport = egui::ViewportBuilder::default()
        .with_title(APP_NAME)
        .with_min_inner_size([800.0, 600.0])
        .with_inner_size([1100.0, 700.0])
        .with_icon(load_app_icon());

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            Ok(Box::new(AnimePlayerApp::new(
                cc,
                backend.clone(),
                cli.input,
            )))
        }),
    )
}
