//! Playlist helpers (port of `Player.list_files / list_filenames`).

use crate::formats::is_media_file;

/// Full paths of playable files in `folder` (non-recursive, like `os.listdir`).
pub fn list_files(folder: &std::path::Path) -> Vec<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(is_media_file)
                .unwrap_or(false)
        })
        .collect();
    files.sort();
    files
}

/// Numbered display names (`"1) name.mkv"`), mirrors `list_filenames`.
pub fn list_filenames(files: &[std::path::PathBuf]) -> Vec<String> {
    files
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            format!("{}) {name}", i + 1)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_only_media_sorted() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["b.mkv", "a.mp4", "note.txt", "song.flac", "subs.srt"] {
            std::fs::write(dir.path().join(name), b"x").unwrap();
        }
        let files = list_files(dir.path());
        let names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["a.mp4", "b.mkv", "song.flac"]);

        let display = list_filenames(&files);
        assert_eq!(display[0], "1) a.mp4");
        assert_eq!(display[2], "3) song.flac");
    }

    #[test]
    fn missing_dir_is_empty() {
        let files = list_files(std::path::Path::new("/no/such/dir/xyz"));
        assert!(files.is_empty());
    }
}
