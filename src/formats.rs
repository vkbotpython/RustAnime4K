//! Supported media file extensions (port of `main.py` format tuples).

pub const VIDEO_FORMATS: &[&str] = &[
    "mp4", "mkv", "webm", "avi", "mov", "wmv", "3gp", "ts", "mpeg", "gif",
];

pub const AUDIO_FORMATS: &[&str] = &["m4a", "mp3", "flac", "ogg", "aac", "opus", "wav"];

pub const SUBTITLES_FORMATS: &[&str] = &["ass", "idx", "srt", "ssa", "sub", "ttml", "vtt"];

/// Returns the lowercase extension of a file name, if any.
pub fn extension_of(file_name: &str) -> Option<String> {
    std::path::Path::new(file_name)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
}

/// True if the file is a playable media file (video or audio).
pub fn is_media_file(file_name: &str) -> bool {
    match extension_of(file_name) {
        Some(ext) => VIDEO_FORMATS.contains(&ext.as_str()) || AUDIO_FORMATS.contains(&ext.as_str()),
        None => false,
    }
}

/// True if the file is a subtitles file.
pub fn is_subtitles_file(file_name: &str) -> bool {
    match extension_of(file_name) {
        Some(ext) => SUBTITLES_FORMATS.contains(&ext.as_str()),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_detection() {
        assert!(is_media_file("movie.mkv"));
        assert!(is_media_file("song.FLAC"));
        assert!(is_media_file("clip.MP4"));
        assert!(!is_media_file("subs.srt"));
        assert!(!is_media_file("readme.txt"));
        assert!(!is_media_file("noext"));
    }

    #[test]
    fn subtitles_detection() {
        assert!(is_subtitles_file("subs.srt"));
        assert!(is_subtitles_file("subs.ASS"));
        assert!(!is_subtitles_file("movie.mkv"));
    }
}
