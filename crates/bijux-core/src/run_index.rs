use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunIndexEntry {
    pub run_id: String,
    pub domain: String,
    pub pipeline: String,
    pub stages: Vec<String>,
    pub tools: Vec<String>,
    pub objective: Option<String>,
    pub platform: String,
    pub success: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunIndexLine {
    pub schema_version: u32,
    pub run: RunIndexEntry,
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
        .with_context(|| format!("open run index {}", index_path.display()))?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let parsed: RunIndexLine = serde_json::from_str(&line)?;
        entries.push(parsed.run);
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
        entries.retain(|entry| entry.stages.iter().any(|s| s == stage));
    }
    if let Some(tool) = &query.tool {
        entries.retain(|entry| entry.tools.iter().any(|t| t == tool));
    }
    if let Some(objective) = &query.objective {
        entries.retain(|entry| entry.objective.as_deref() == Some(objective.as_str()));
    }
    if let Some(success) = query.success {
        entries.retain(|entry| entry.success == success);
    }
    Ok(entries)
}
