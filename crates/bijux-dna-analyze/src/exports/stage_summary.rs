use std::fmt::Write;
use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_core::prelude::InvariantStatusV1;
use bijux_dna_infra::atomic_write_bytes;
use bijux_dna_runtime::FactsRowV1;

use crate::model::stable_sort_records;

use super::facts_support::{params_excerpt, stage_report_for_row, tool_invocation_for_stage};

/// Write a deterministic stage summary CSV from facts rows.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_stage_summary_csv(path: &Path, rows: &[FactsRowV1]) -> Result<()> {
    let mut ordered = rows.to_vec();
    stable_sort_records(&mut ordered, |row| {
        (
            row.run_id.as_str(),
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            row.params_hash.as_str(),
            "",
        )
    });
    let mut output = String::new();
    output.push_str("stage,tool,version,input_reads,output_reads,retention,runtime_s,memory_mb,key_params,verdict,notes\n");
    for row in ordered {
        let retention = match (row.reads_in, row.reads_out) {
            #[allow(clippy::cast_precision_loss)]
            (Some(ri), Some(ro)) if ri > 0 => Some(ro as f64 / ri as f64),
            _ => None,
        };
        let stage_report = stage_report_for_row(&row);
        let verdict = stage_report.as_ref().and_then(|report| report.verdict.as_ref()).map_or(
            "",
            |verdict| match verdict.verdict {
                InvariantStatusV1::Pass => "pass",
                InvariantStatusV1::Warn => "warn",
                InvariantStatusV1::Fail => "fail",
            },
        );
        let mut notes = Vec::new();
        if let Some(report) = stage_report.as_ref() {
            if let Some(verdict) = report.verdict.as_ref() {
                notes.extend(verdict.reasons.clone());
            }
            notes.extend(report.warnings.clone());
            notes.extend(report.errors.clone());
        }
        let key_params = stage_report.as_ref().and_then(tool_invocation_for_stage).map_or_else(
            || serde_json::json!({}),
            |invocation| {
                let params = if invocation.effective_params_json_normalized.is_null() {
                    invocation.parameters_json_normalized
                } else {
                    invocation.effective_params_json_normalized
                };
                params_excerpt(&params, 8)
            },
        );
        let key_params = serde_json::to_string(&key_params).unwrap_or_else(|_| "{}".to_string());
        let notes = notes.join(" | ");
        let reads_in = row.reads_in.map(|v| v.to_string()).unwrap_or_default();
        let reads_out = row.reads_out.map(|v| v.to_string()).unwrap_or_default();
        let retention = retention.map(|v| format!("{v:.4}")).unwrap_or_default();
        let _ = writeln!(
            output,
            "{},{},{},{},{},{},{:.2},{:.2},{},{},{}",
            csv_escape(&row.stage_id),
            csv_escape(&row.tool_id),
            csv_escape(&row.tool_version),
            csv_escape(reads_in.as_str()),
            csv_escape(reads_out.as_str()),
            csv_escape(&retention),
            row.runtime_s,
            row.memory_mb,
            csv_escape(&key_params),
            csv_escape(verdict),
            csv_escape(&notes),
        );
    }
    atomic_write_bytes(path, output.as_bytes())
        .map_err(anyhow::Error::from)
        .with_context(|| format!("write stage summary csv {}", path.display()))?;
    Ok(())
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        let escaped = value.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        value.to_string()
    }
}
