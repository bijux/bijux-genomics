//! Owner: bijux-dna-analyze
//! Run summary loader.

use std::path::Path;

use super::AnalyzeError;
use super::support::read_required_json;
use crate::model::RunSummaryV1;

/// Load a run summary from JSON.
///
/// # Errors
/// Returns an error if the file is missing, unreadable, or has an invalid schema.
pub fn load_run_summary(path: &Path) -> std::result::Result<RunSummaryV1, AnalyzeError> {
    let summary: RunSummaryV1 = read_required_json(path)?;
    if summary.schema_version != "bijux.run_summary.v1" {
        return Err(AnalyzeError::InvalidSchemaVersion {
            found: summary.schema_version,
            expected: "bijux.run_summary.v1".to_string(),
        });
    }
    Ok(summary)
}
