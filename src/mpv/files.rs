//! Поиск новых файлов (скриншоты mpv пишут файл сами).

/// File names (lossy) currently inside `dir`. Read failures yield an empty
/// list: the screenshot watch treats them as "nothing new yet".
pub(crate) fn dir_file_names(dir: &std::path::Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// First file in `dir` absent from `before` (the mpv screenshot that just
/// landed), if any.
pub(crate) fn find_new_file(before: &[String], dir: &std::path::Path) -> Option<String> {
    dir_file_names(dir)
        .into_iter()
        .find(|name| !before.iter().any(|b| b == name))
}
