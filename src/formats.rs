//! Supported media file extensions.

pub const VIDEO_FORMATS: &[&str] = &[
    "mp4", "mkv", "webm", "avi", "mov", "wmv", "3gp", "ts", "mpeg", "gif",
];

pub const AUDIO_FORMATS: &[&str] = &["m4a", "mp3", "flac", "ogg", "aac", "opus", "wav"];

pub const SUBTITLES_FORMATS: &[&str] = &["ass", "idx", "srt", "ssa", "sub", "ttml", "vtt"];

/// Raw (non-lowercased) extension slice, if any. Directory parts are
/// ignored so full paths and URLs work as well as bare names.
fn raw_extension(file_name: &str) -> Option<&str> {
    let file = file_name.rsplit(['/', '\\']).next().unwrap_or(file_name);
    // Mirror `Path::extension`: a leading dot starts the stem, it is not
    // an extension separator (".mkv" has no extension).
    let stem = file.strip_prefix('.').unwrap_or(file);
    let (_, ext) = stem.rsplit_once('.')?;
    (!ext.is_empty()).then_some(ext)
}

/// True if the file extension (case-insensitive) is in `formats`.
/// Allocation-free: no `to_lowercase()` per probed file.
pub fn extension_is_listed(file_name: &str, formats: &[&str]) -> bool {
    match raw_extension(file_name) {
        Some(ext) => formats.iter().any(|f| f.eq_ignore_ascii_case(ext)),
        None => false,
    }
}

/// True if the file is a playable media file (video or audio).
pub fn is_media_file(file_name: &str) -> bool {
    extension_is_listed(file_name, VIDEO_FORMATS) || extension_is_listed(file_name, AUDIO_FORMATS)
}

/// True if the file is a subtitles file.
pub fn is_subtitles_file(file_name: &str) -> bool {
    extension_is_listed(file_name, SUBTITLES_FORMATS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_detection() {
        assert!(is_media_file("movie.mkv"));
        assert!(is_media_file("song.FLAC"));
        assert!(is_media_file("clip.MP4"));
        assert!(is_media_file("/media/anime/EP01.MkV"));
        assert!(is_media_file("https://example.com/vid.webm"));
        assert!(is_media_file("D:\\anime\\ep02.AVI"));
        assert!(!is_media_file("subs.srt"));
        assert!(!is_media_file("readme.txt"));
        assert!(!is_media_file("noext"));
        assert!(!is_media_file("trailingdot."));
        assert!(!is_media_file(".mkv"));
        assert!(!is_media_file(".hidden"));
    }

    #[test]
    fn subtitles_detection() {
        assert!(is_subtitles_file("subs.srt"));
        assert!(is_subtitles_file("subs.ASS"));
        assert!(!is_subtitles_file("movie.mkv"));
    }
}
