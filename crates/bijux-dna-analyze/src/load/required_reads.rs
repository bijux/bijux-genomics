use std::fs::File;
use std::path::Path;

use serde::de::DeserializeOwned;

use super::AnalyzeError;

pub(super) fn ensure_exists(path: &Path) -> std::result::Result<(), AnalyzeError> {
    if path.exists() {
        Ok(())
    } else {
        Err(AnalyzeError::MissingFile { path: path.display().to_string() })
    }
}

pub(super) fn open_required_file(path: &Path) -> std::result::Result<File, AnalyzeError> {
    ensure_exists(path)?;
    File::open(path).map_err(|err| AnalyzeError::InvalidJson { message: err.to_string() })
}

pub(super) fn read_required_string(path: &Path) -> std::result::Result<String, AnalyzeError> {
    ensure_exists(path)?;
    std::fs::read_to_string(path)
        .map_err(|err| AnalyzeError::InvalidJson { message: err.to_string() })
}

pub(super) fn read_required_json<T: DeserializeOwned>(
    path: &Path,
) -> std::result::Result<T, AnalyzeError> {
    let raw = read_required_string(path)?;
    serde_json::from_str(&raw).map_err(|err| AnalyzeError::InvalidJson { message: err.to_string() })
}
