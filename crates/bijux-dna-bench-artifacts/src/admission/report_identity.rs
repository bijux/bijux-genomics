//! Publication ownership is checked independently from capability scientific payload schemas.

use anyhow::{ensure, Result};
use serde::Deserialize;
use std::path::Path;

use crate::model::ReportOwner;
use crate::paths::read_bounded;

#[derive(Deserialize)]
struct ReportIdentity {
    schema_version: String,
    stage_id: String,
    tool_id: String,
}

pub(crate) fn admit_report_identity(path: &Path, owner: &ReportOwner<'_>) -> Result<()> {
    ensure!(
        [owner.schema, owner.stage, owner.tool].iter().all(|identity| !identity.trim().is_empty()),
        "benchmark publication report owner identity must be nonempty"
    );
    // Additional report fields belong to capability models. Deriving this envelope still
    // rejects missing, non-string and repeated ownership fields instead of choosing a value.
    let bytes = read_bounded(path)?;
    ensure!(
        bytes.iter().find(|byte| !byte.is_ascii_whitespace()) == Some(&b'{'),
        "benchmark publication report must be a JSON object"
    );
    let report: ReportIdentity = serde_json::from_slice(&bytes)?;
    ensure!(
        report.schema_version == owner.schema
            && report.stage_id == owner.stage
            && report.tool_id == owner.tool,
        "benchmark publication report has a foreign owner"
    );
    Ok(())
}
