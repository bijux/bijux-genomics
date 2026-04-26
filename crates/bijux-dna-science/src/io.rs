use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use walkdir::WalkDir;

/// Read a UTF-8 text file.
///
/// # Errors
///
/// Returns an error when the file cannot be opened or decoded as UTF-8.
pub fn read_utf8(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

/// Ensure a directory and its parents exist.
///
/// # Errors
///
/// Returns an error when the directory cannot be created.
pub fn ensure_dir(path: &Path) -> Result<()> {
    bijux_dna_infra::ensure_dir(path).with_context(|| format!("create {}", path.display()))
}

/// Write UTF-8 text, creating the parent directory when needed.
///
/// # Errors
///
/// Returns an error when the parent directory or file cannot be written.
pub fn write_utf8(path: &Path, contents: &str) -> Result<()> {
    bijux_dna_infra::write_string(path, contents)
        .with_context(|| format!("write {}", path.display()))
}

/// List YAML files below a directory in deterministic order.
///
/// # Errors
///
/// Returns an error when the directory walk cannot be initialized.
pub fn list_yaml_files(path: &Path) -> Result<Vec<PathBuf>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut files = WalkDir::new(path)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|entry| {
            entry
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("yaml"))
        })
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}
