use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use super::corpus::BenchCorpusId;
use super::objective::Objective;
use super::ranking::StageSelection;

#[derive(Debug, Serialize)]
pub struct SelectionReport {
    pub objective: String,
    pub corpus_id: String,
    pub stages: Vec<StageSelection>,
}

/// Write the selection report to disk.
///
/// # Errors
/// Returns an error if the report cannot be serialized or written.
pub fn write_selection_report(
    out_dir: &Path,
    objective: Objective,
    corpus_id: BenchCorpusId,
    stages: Vec<StageSelection>,
) -> Result<()> {
    let report = SelectionReport {
        objective: objective.as_str().to_string(),
        corpus_id: corpus_id.as_str().to_string(),
        stages,
    };
    let path = out_dir.join("selection_report.json");
    let payload = serde_json::to_string_pretty(&report).context("serialize selection report")?;
    std::fs::write(&path, payload).context("write selection_report.json")?;
    Ok(())
}
