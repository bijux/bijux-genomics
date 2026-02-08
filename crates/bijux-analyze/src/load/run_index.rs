//! Owner: bijux-analyze
//! Run index loader.

use std::path::Path;

use bijux_core::contract::RunIndexLine;
use serde_json;

use super::AnalyzeError;

/// Load a run index from JSONL.
///
/// # Errors
/// Returns an error if the file is missing, unreadable, or contains invalid rows.
pub fn load_run_index(path: &Path) -> std::result::Result<Vec<RunIndexLine>, AnalyzeError> {
    if !path.exists() {
        return Err(AnalyzeError::MissingFile {
            path: path.display().to_string(),
        });
    }
    let raw = std::fs::read_to_string(path).map_err(|err| AnalyzeError::InvalidJson {
        message: err.to_string(),
    })?;
    let mut rows = Vec::new();
    for (idx, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let parsed_row: RunIndexLine =
            serde_json::from_str(line).map_err(|err| AnalyzeError::InvalidJsonlRow {
                line: idx + 1,
                message: err.to_string(),
            })?;
        if parsed_row.schema_version != 1 {
            return Err(AnalyzeError::InvalidSchemaVersion {
                found: parsed_row.schema_version.to_string(),
                expected: "1".to_string(),
            });
        }
        rows.push(parsed_row);
    }
    Ok(rows)
}
