//! Мелкие чистые функции: время, пути, диалоги выбора файлов.

use crate::config::ConfigManager;
use crate::formats::{AUDIO_FORMATS, SUBTITLES_FORMATS, VIDEO_FORMATS};
use crate::mpv::TrackKind;
use serde_json::Value;
use std::path::{Path, PathBuf};
/// `MM:SS`, or `HH:MM:SS` for videos >= 1h.
pub(crate) fn fmt_time(secs: f64) -> String {
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

pub(crate) fn shaders_dir() -> PathBuf {
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
pub(crate) fn missing_shader_files(dir: &Path, preset: &[String]) -> Vec<String> {
    preset
        .iter()
        .filter(|s| !dir.join(s).is_file())
        .cloned()
        .collect()
}

/// Parses the safe `name=value` subset of launch parameters
/// (arbitrary code is not supported).
pub(crate) fn parse_launch_params(text: &str) -> Vec<(String, String)> {
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

pub(crate) fn get_str(cfg: &ConfigManager, key: &str) -> Option<String> {
    cfg.get(key).and_then(Value::as_str).map(str::to_string)
}

pub(crate) fn get_bool(cfg: &ConfigManager, key: &str, default: bool) -> bool {
    cfg.get(key).and_then(Value::as_bool).unwrap_or(default)
}

/// Reads the system clipboard (for the `Paste` buttons). Returns empty string on failure.
pub(crate) fn read_clipboard_text() -> String {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.get_text())
        .unwrap_or_default()
}

pub(crate) fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

pub(crate) fn is_url_like(s: &str) -> bool {
    s.contains("://")
}

/// Display name for the window title / playlist
/// (`basename` for files, part after `/` for URLs).
pub(crate) fn display_name_of(path_str: &str) -> String {
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
pub(crate) fn pick_media_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Video", VIDEO_FORMATS)
        .add_filter("Audio", AUDIO_FORMATS)
        .pick_file()
}

/// Native "pick subtitle/audio sidecar file" dialog.
pub(crate) fn pick_sidecar_file(kind: &TrackKind) -> Option<PathBuf> {
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
}
