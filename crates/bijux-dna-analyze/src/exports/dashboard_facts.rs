use std::path::Path;

use anyhow::Result;
use bijux_dna_infra::atomic_write_bytes;
use bijux_dna_runtime::FactsRowV1;

use crate::model::{DashboardFactRow, JsonBlob};

/// Write a deterministic dashboard facts JSONL file.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_dashboard_facts_jsonl(path: &Path, rows: &[FactsRowV1]) -> Result<()> {
    let mut ordered: Vec<DashboardFactRow> = rows
        .iter()
        .map(|row| DashboardFactRow {
            schema_version: "bijux.dashboard.fact.v1".to_string(),
            run_id: row.run_id.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            tool_version: row.tool_version.clone(),
            image_digest: row.image_digest.clone(),
            params_hash: row.params_hash.clone(),
            input_hash: row.input_hash.clone(),
            runtime_s: row.runtime_s,
            memory_mb: row.memory_mb,
            exit_code: row.exit_code,
            bank_hashes: JsonBlob::new(row.bank_hashes.clone()),
            metrics: JsonBlob::new(row.metrics.clone()),
            reports: JsonBlob::new(row.reports.clone()),
            artifacts: JsonBlob::new(row.artifacts.clone()),
            trace_id: row.trace_id.clone(),
            span_id: row.span_id.clone(),
        })
        .collect();
    ordered.sort_by(|a, b| a.key().cmp(&b.key()));
    let mut payload = String::new();
    for row in ordered {
        payload.push_str(&serde_json::to_string(&row)?);
        payload.push('\n');
    }
    atomic_write_bytes(path, payload.as_bytes()).map_err(anyhow::Error::from)?;
    Ok(())
}
