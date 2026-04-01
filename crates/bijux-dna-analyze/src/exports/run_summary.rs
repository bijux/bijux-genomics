use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_infra::atomic_write_bytes;
use bijux_dna_runtime::FactsRowV1;

use crate::model::{
    stable_sort_records, JsonBlob, RunSummaryDeltas, RunSummaryStageRow, RunSummaryV1,
};

use super::support::{stage_outputs_for_row, summarize_facts};

/// Write a deterministic run summary JSON from facts rows.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_run_summary_json(path: &Path, rows: &[FactsRowV1]) -> Result<()> {
    let summary = summarize_facts(rows);
    let facts_path = Some("facts.jsonl".to_string());
    let report_path = Some("report.json".to_string());
    let telemetry_path = Some("telemetry/events.jsonl".to_string());
    let mut stage_rows: Vec<RunSummaryStageRow> = rows
        .iter()
        .map(|row| RunSummaryStageRow {
            run_id: row.run_id.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            tool_version: row.tool_version.clone(),
            image_digest: row.image_digest.clone(),
            params_hash: row.params_hash.clone(),
            input_hash: row.input_hash.clone(),
            bank_hashes: JsonBlob::from(row.bank_hashes.clone()),
            runtime_s: row.runtime_s,
            memory_mb: row.memory_mb,
            exit_code: row.exit_code,
            reports: JsonBlob::from(row.reports.clone()),
            deltas: RunSummaryDeltas {
                reads_in: row.reads_in,
                reads_out: row.reads_out,
                bases_in: row.bases_in,
                bases_out: row.bases_out,
                pairs_in: row.pairs_in,
                pairs_out: row.pairs_out,
            },
        })
        .collect();
    let mut final_outputs = Vec::new();
    for row in rows {
        if row.stage_id == "fastq.report_qc" {
            final_outputs.extend(stage_outputs_for_row(row));
        }
    }
    final_outputs.sort();
    final_outputs.dedup();
    stable_sort_records(&mut stage_rows, |row| {
        (
            row.run_id.as_str(),
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            row.params_hash.as_str(),
            row.input_hash.as_str(),
        )
    });
    let payload = RunSummaryV1 {
        schema_version: "bijux.run_summary.v1".to_string(),
        facts_path,
        report_path,
        telemetry_path,
        final_outputs,
        runs: summary.runs,
        stages: summary.stages,
        total_runtime_s: summary.total_runtime_s,
        avg_runtime_s: summary.avg_runtime_s,
        stage_rows,
    };
    atomic_write_bytes(path, &serde_json::to_vec_pretty(&payload)?)
        .map_err(anyhow::Error::from)
        .with_context(|| format!("write run summary {}", path.display()))?;
    Ok(())
}
