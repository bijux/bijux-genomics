use std::path::Path;

#[must_use]
pub(super) fn file_name_from_url(url: &str) -> String {
    Path::new(url)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown_file".to_string())
}
