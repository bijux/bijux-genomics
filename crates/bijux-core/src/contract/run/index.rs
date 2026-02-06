use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::contract::ContractVersion;
use crate::foundation::{BijuxError, Result};
use crate::ids::{PipelineId, RunId, StageId, ToolId};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunIndexEntry {
    pub run_id: RunId,
    pub domain: String,
    pub pipeline: PipelineId,
    pub stages: Vec<StageId>,
    pub tools: Vec<ToolId>,
    pub objective: Option<String>,
    pub platform: String,
    pub success: bool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunIndexLine {
    pub schema_version: u32,
    #[serde(default = "ContractVersion::v1")]
    pub contract_version: ContractVersion,
    pub run: Option<RunIndexEntry>,
    pub stage: Option<StageIndexRow>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageIndexRow {
    pub run_id: RunId,
    pub stage_id: StageId,
    pub tool_id: ToolId,
    pub params_hash: String,
    pub input_hash: String,
    pub output_hashes: Vec<String>,
    pub artifacts: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct RunQuery {
    pub stage: Option<String>,
    pub tool: Option<String>,
    pub objective: Option<String>,
    pub success: Option<bool>,
}

/// List all runs from `index.jsonl`.
///
/// # Errors
/// Returns an error if the index cannot be read.
pub fn list_runs(index_path: &Path) -> Result<Vec<RunIndexEntry>> {
    let file = File::open(index_path)
        .map_err(|err| BijuxError::Io(format!("open run index {}: {err}", index_path.display())))?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let parsed: RunIndexLine = serde_json::from_str(&line)?;
        if let Some(run) = parsed.run {
            entries.push(run);
        }
    }
    Ok(entries)
}

/// Query runs from `index.jsonl`.
///
/// # Errors
/// Returns an error if the index cannot be read.
pub fn query_runs(index_path: &Path, query: &RunQuery) -> Result<Vec<RunIndexEntry>> {
    let mut entries = list_runs(index_path)?;
    if let Some(stage) = &query.stage {
        entries.retain(|entry| entry.stages.iter().any(|s| s.as_str() == stage));
    }
    if let Some(tool) = &query.tool {
        entries.retain(|entry| entry.tools.iter().any(|t| t.as_str() == tool));
    }
    if let Some(objective) = &query.objective {
        entries.retain(|entry| entry.objective.as_deref() == Some(objective.as_str()));
    }
    if let Some(success) = query.success {
        entries.retain(|entry| entry.success == success);
    }
    Ok(entries)
}

/// Return the latest `limit` runs from the index.
///
/// # Errors
/// Returns an error if the index cannot be read.
pub fn query_latest_runs(index_path: &Path, limit: usize) -> Result<Vec<RunIndexEntry>> {
    let mut entries = list_runs(index_path)?;
    entries.sort_by(|a, b| a.run_id.cmp(&b.run_id));
    let len = entries.len();
    if limit >= len {
        return Ok(entries);
    }
    Ok(entries[len - limit..].to_vec())
}

/// Find a run by id.
///
/// # Errors
/// Returns an error if the index cannot be read.
pub fn query_run(index_path: &Path, run_id: &str) -> Result<Option<RunIndexEntry>> {
    let entries = list_runs(index_path)?;
    Ok(entries
        .into_iter()
        .find(|entry| entry.run_id.as_str() == run_id))
}

/// Query stage rows from `index.jsonl`.
///
/// # Errors
/// Returns an error if the index cannot be read.
pub fn query_stage_rows(
    index_path: &Path,
    stage: Option<&str>,
    tool: Option<&str>,
) -> Result<Vec<StageIndexRow>> {
    let file = File::open(index_path)
        .map_err(|err| BijuxError::Io(format!("open run index {}: {err}", index_path.display())))?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let parsed: RunIndexLine = serde_json::from_str(&line)?;
        let Some(row) = parsed.stage else { continue };
        if let Some(stage_id) = stage {
            if row.stage_id.as_str() != stage_id {
                continue;
            }
        }
        if let Some(tool_id) = tool {
            if row.tool_id.as_str() != tool_id {
                continue;
            }
        }
        rows.push(row);
    }
    Ok(rows)
}
