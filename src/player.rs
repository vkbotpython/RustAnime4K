//! Playback state machine.
//!
//! UI-independent: owns the file list, current index, volume rules and the
//! Anime4K preset name. Talks to mpv through [`crate::mpv::MpvBackend`] so it
//! is fully unit-testable with the mock backend.

use std::path::{Path, PathBuf};

use crate::mpv::MpvBackend;
use crate::playlist::{list_filenames, list_files};

pub const MAX_VOLUME: i32 = 100;
pub const MAX_VOLUME_PLUS: i32 = 150;
/// Positions below this are not restored on startup.
pub const MIN_RESTORE_POSITION: f64 = 10.0;

#[derive(Debug, Default)]
pub struct PlayerState {
    pub files: Vec<PathBuf>,
    pub current: Option<usize>,
    pub fullscreen: bool,
    pub preset_name: String,
    pub volume: i32,
    pub volume_plus: bool,
}

impl PlayerState {
    pub fn new(volume: i32, volume_plus: bool) -> Self {
        Self {
            volume: volume.clamp(0, Self::max_volume(volume_plus)),
            volume_plus,
            ..Default::default()
        }
    }

    pub fn max_volume(volume_plus: bool) -> i32 {
        if volume_plus {
            MAX_VOLUME_PLUS
        } else {
            MAX_VOLUME
        }
    }

    pub fn max_volume_self(&self) -> i32 {
        Self::max_volume(self.volume_plus)
    }

    pub fn filenames_only(&self) -> Vec<String> {
        list_filenames(&self.files)
    }

    pub fn current_file(&self) -> Option<&PathBuf> {
        self.current.and_then(|i| self.files.get(i))
    }

    fn set_single(&mut self, file: PathBuf) {
        self.files = vec![file];
        self.current = Some(0);
    }

    /// Opens a single file for playback.
    pub fn open_file(&mut self, mpv: &dyn MpvBackend, file: &Path) {
        mpv.stop();
        self.set_single(file.to_path_buf());
        mpv.play(&file.to_string_lossy());
    }

    /// Opens a URL for playback.
    pub fn open_url(&mut self, mpv: &dyn MpvBackend, url: &str) {
        let url = url.trim().to_string();
        mpv.stop();
        self.files = vec![PathBuf::from(&url)];
        self.current = Some(0);
        mpv.play(&url);
    }

    /// Scans the folder, selects `file` if given.
    pub fn open_folder(&mut self, mpv: &dyn MpvBackend, folder: &Path, file: Option<&Path>) {
        mpv.stop();
        self.files = list_files(folder);
        let index = file
            .and_then(|f| self.files.iter().position(|p| p == f))
            .unwrap_or(0);
        if self.files.is_empty() {
            self.current = None;
            return;
        }
        self.current = Some(index);
        mpv.play(&self.files[index].to_string_lossy());
    }

    /// Adds a file to the playlist (drag&drop of extra files).
    pub fn add_file(&mut self, file: PathBuf) {
        self.files.push(file);
        if self.current.is_none() {
            self.current = Some(0);
        }
    }

    /// Advances to the next file.
    pub fn next(&mut self, mpv: &dyn MpvBackend) -> bool {
        let next = match self.current {
            Some(i) if i + 1 < self.files.len() => i + 1,
            _ => return false,
        };
        self.current = Some(next);
        mpv.play(&self.files[next].to_string_lossy());
        true
    }

    /// Returns to the previous file.
    pub fn prev(&mut self, mpv: &dyn MpvBackend) -> bool {
        let prev = match self.current {
            Some(i) if i > 0 => i - 1,
            Some(_) => {
                self.current = Some(0);
                return true;
            }
            None => return false,
        };
        self.current = Some(prev);
        mpv.play(&self.files[prev].to_string_lossy());
        true
    }

    /// Plays the file at `index` (click on a playlist row).
    pub fn play_index(&mut self, mpv: &dyn MpvBackend, index: usize) -> bool {
        if index >= self.files.len() {
            return false;
        }
        self.current = Some(index);
        mpv.play(&self.files[index].to_string_lossy());
        true
    }

    /// Closes the current file and clears the playlist.
    pub fn close(&mut self, mpv: &dyn MpvBackend) {
        mpv.stop();
        self.files.clear();
        self.current = None;
        self.preset_name.clear();
    }

    pub fn set_volume(&mut self, mpv: &dyn MpvBackend, volume: i32) {
        self.volume = volume.clamp(0, self.max_volume_self());
        mpv.set_volume(self.volume);
    }

    pub fn volume_up(&mut self, mpv: &dyn MpvBackend) {
        // Step 5 with snap to the max.
        let max = self.max_volume_self();
        let next = if self.volume > max - 5 {
            max
        } else {
            self.volume + 5
        };
        self.set_volume(mpv, next);
    }

    pub fn volume_down(&mut self, mpv: &dyn MpvBackend) {
        // Step 5, snap to 0.
        let next = if self.volume < 5 { 0 } else { self.volume - 5 };
        self.set_volume(mpv, next);
    }

    /// Disables Anime4K upscaling.
    pub fn disable_anime4k(&mut self, mpv: &dyn MpvBackend) {
        self.preset_name.clear();
        mpv.set_glsl_shaders("");
    }

    /// Applies an Anime4K preset.
    pub fn apply_preset(&mut self, mpv: &dyn MpvBackend, name: &str, glsl_shaders: &str) {
        self.preset_name = name.to_string();
        mpv.set_glsl_shaders(glsl_shaders);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mpv::MockMpv;

    fn file(name: &str) -> PathBuf {
        PathBuf::from(format!("/media/{name}"))
    }

    #[test]
    fn open_next_prev_close() {
        let mpv = MockMpv::new();
        let mut p = PlayerState::new(80, false);
        p.open_file(&mpv, &file("a.mkv"));
        assert_eq!(p.current_file(), Some(&file("a.mkv")));
        assert_eq!(mpv.last_played(), Some("/media/a.mkv".to_string()));

        p.add_file(file("b.mkv"));
        assert!(p.next(&mpv));
        assert_eq!(p.current, Some(1));
        assert!(!p.next(&mpv));
        assert!(p.prev(&mpv));
        assert_eq!(p.current, Some(0));

        p.close(&mpv);
        assert!(p.files.is_empty() && p.current.is_none());
    }

    #[test]
    fn volume_clamped() {
        let mpv = MockMpv::new();
        let mut p = PlayerState::new(80, false);
        p.set_volume(&mpv, 500);
        assert_eq!(p.volume, MAX_VOLUME);
        p.set_volume(&mpv, -5);
        assert_eq!(p.volume, 0);

        let mut p2 = PlayerState::new(80, true);
        p2.set_volume(&mpv, 500);
        assert_eq!(p2.volume, MAX_VOLUME_PLUS);
    }

    #[test]
    fn volume_step_snaps_to_max() {
        // Volume step snaps to max/0 (±5).
        let mpv = MockMpv::new();
        let mut p = PlayerState::new(96, false);
        p.volume_up(&mpv);
        assert_eq!(p.volume, 100);
        p.volume_down(&mpv);
        assert_eq!(p.volume, 95);

        let mut p2 = PlayerState::new(3, false);
        p2.volume_down(&mpv);
        assert_eq!(p2.volume, 0);

        let mut p3 = PlayerState::new(146, true);
        p3.volume_up(&mpv);
        assert_eq!(p3.volume, 150);
    }

    #[test]
    fn open_folder_selects_file() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["a.mp4", "b.mkv"] {
            std::fs::write(dir.path().join(name), b"x").unwrap();
        }
        let mpv = MockMpv::new();
        let mut p = PlayerState::new(80, false);
        let want = dir.path().join("b.mkv");
        p.open_folder(&mpv, dir.path(), Some(&want));
        assert_eq!(p.current_file(), Some(&want));
    }

    #[test]
    fn open_url_trims() {
        let mpv = MockMpv::new();
        let mut p = PlayerState::new(80, false);
        p.open_url(&mpv, "  https://example.com/v.mp4\n");
        assert_eq!(p.files.len(), 1);
        assert_eq!(
            mpv.last_played().as_deref(),
            Some("https://example.com/v.mp4")
        );
    }
}
