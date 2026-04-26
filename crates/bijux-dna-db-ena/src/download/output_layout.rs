use std::path::Path;

#[must_use]
pub(super) fn file_name_from_url(url: &str) -> String {
    let clean_url = url.trim().split(['?', '#']).next().unwrap_or_default();
    let clean_url = clean_url.trim_end_matches('/');

    Path::new(clean_url)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown_file".to_string())
}

#[cfg(test)]
mod tests {
    use super::file_name_from_url;

    #[test]
    fn file_name_from_url_ignores_query_and_fragment() {
        assert_eq!(
            file_name_from_url("https://example.org/path/read.fastq.gz?download=1#top"),
            "read.fastq.gz"
        );
    }

    #[test]
    fn file_name_from_url_uses_last_path_segment_for_trailing_slash_urls() {
        assert_eq!(file_name_from_url("ftp://example.org/path/read/"), "read");
    }
}
