use std::path::Path;

/// Read a UTF-8 text file into memory.
///
/// # Panics
/// Panics if `path` cannot be read as a string.
#[must_use]
pub fn read_text(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}
