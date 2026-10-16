//! Anime4K Player — video player with Anime4K upscaling.

// Hide the console window on Windows (GUI app; mpv logs don't need a terminal).
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod anime4k;
mod app;
mod config;
mod fonts;
mod formats;
mod i18n;
mod icon;
mod mpv;
mod player;
mod playlist;
mod reference;
mod theme;
#[cfg(feature = "video")]
mod video;

use std::sync::Arc;

use app::{APP_NAME, Anime4KPlayerApp};
use config::ConfigManager;
use mpv::MpvBackend;

/// CLI input: optional media file, folder or URL (skips restoring the
/// last session). Parsed by hand — a single positional is all this app
/// needs, and it avoids pulling the whole `clap` tree (~14 crates).
fn parse_cli_input() -> Option<String> {
    let mut input: Option<String> = None;
    for arg in std::env::args_os().skip(1) {
        let s = arg.to_string_lossy().into_owned();
        match s.as_str() {
            "--" => continue,
            "-h" | "--help" => {
                println!(
                    "{APP_NAME}\n\nUsage: anime4k-player [input]\n\nArguments:\n  [input]  Media file, folder or URL to open (skips restoring the last session)"
                );
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("anime4k-player {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            s if s.starts_with('-') => {
                eprintln!("warning: unknown option '{s}' ignored");
            }
            _ if input.is_none() => {
                input = Some(s);
            }
            _ => {}
        }
    }
    input.filter(|s| !s.is_empty())
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
    eframe::icon_data::from_png_bytes(crate::icon::ICON_PNG)
        .map(Arc::new)
        .unwrap_or_else(|err| {
            eprintln!("cannot decode app icon: {err:#}");
            Arc::new(egui::IconData::default())
        })
}

fn main() -> eframe::Result<()> {
    let cli_input = parse_cli_input();
    let backend = create_backend();

    // Last window geometry (see `track_window_geometry`): restore position,
    // size and maximized state, falling back to defaults on first launch
    // or corrupt values.
    let cfg = ConfigManager::new("config.json");
    let num = |key: &str| cfg.get(key).and_then(serde_json::Value::as_f64);
    let size = match (num("windowW"), num("windowH")) {
        (Some(w), Some(h)) if w >= 400.0 && h >= 300.0 && w <= 7680.0 && h <= 4320.0 => {
            [w as f32, h as f32]
        }
        _ => [1100.0, 700.0],
    };
    let maximized = cfg
        .get("windowMaximized")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(APP_NAME)
        .with_min_inner_size([800.0, 600.0])
        .with_icon(load_app_icon());
    if maximized {
        // Maximized restore must NOT carry an explicit size/position: the
        // stored W/H is the last *windowed* rect, and combining it with
        // maximized leaves a broken quarter-screen window (winit quirk).
        viewport = viewport.with_maximized(true);
    } else {
        viewport = viewport.with_inner_size(size);
        if let (Some(x), Some(y)) = (num("windowX"), num("windowY"))
            && x.is_finite()
            && y.is_finite()
            && x > -10000.0
            && x < 10000.0
            && y > -10000.0
            && y < 10000.0
        {
            viewport = viewport.with_position([x as f32, y as f32]);
        }
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            Ok(Box::new(Anime4KPlayerApp::new(
                cc,
                backend.clone(),
                cli_input,
            )))
        }),
    )
}
