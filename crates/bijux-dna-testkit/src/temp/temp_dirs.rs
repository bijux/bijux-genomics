use std::path::PathBuf;

use tempfile::TempDir;

fn test_tmp_root() -> Option<PathBuf> {
    std::env::var("TEST_TMP_DIR").ok().map(PathBuf::from)
}

fn safe_test_name(test_name: &str) -> String {
    let mut out = String::with_capacity(test_name.len());
    let mut previous_was_dash = false;
    for ch in test_name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
            previous_was_dash = false;
        } else if !previous_was_dash {
            out.push('-');
            previous_was_dash = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "test".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Create a test temp directory rooted under `TEST_TMP_DIR` when available.
///
/// # Panics
/// Panics if the temporary directory cannot be created.
#[must_use]
pub fn tempdir_for(test_name: &str) -> TempDir {
    let prefix = format!("bijux-dna-{}-", safe_test_name(test_name));
    if let Some(root) = test_tmp_root() {
        if root.exists() {
            return tempfile::Builder::new()
                .prefix(&prefix)
                .tempdir_in(&root)
                .unwrap_or_else(|err| panic!("tempdir_in {}: {err}", root.display()));
        }
    }
    tempfile::Builder::new()
        .prefix(&prefix)
        .tempdir()
        .unwrap_or_else(|err| panic!("tempdir: {err}"))
}
