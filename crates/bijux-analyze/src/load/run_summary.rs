//! Owner: bijux-analyze
//! Run summary loader.

use std::path::Path;

use super::AnalyzeError;
use crate::model::RunSummaryV1;
use serde_json;

/// Load a run summary from JSON.
///
/// # Errors
/// Returns an error if the file is missing, unreadable, or has an invalid schema.
pub fn load_run_summary(path: &Path) -> std::result::Result<RunSummaryV1, AnalyzeError> {
    if !path.exists() {
        return Err(AnalyzeError::MissingFile {
            path: path.display().to_string(),
        });
    }
    let raw = std::fs::read_to_string(path).map_err(|err| AnalyzeError::InvalidJson {
        message: err.to_string(),
    })?;
    let summary: RunSummaryV1 =
        serde_json::from_str(&raw).map_err(|err| AnalyzeError::InvalidJson {
            message: err.to_string(),
        })?;
    if summary.schema_version != "bijux.run_summary.v1" {
        return Err(AnalyzeError::InvalidSchemaVersion {
            found: summary.schema_version,
            expected: "bijux.run_summary.v1".to_string(),
        });
    }
    Ok(summary)
}
