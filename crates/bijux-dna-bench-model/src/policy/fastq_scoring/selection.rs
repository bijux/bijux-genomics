//! Evidence matching and failure attribution for FASTQ stage/tool bindings.

use crate::{
    FastqScoreEvidence, FastqScoreExpectedRow, FastqScoreMemoryRow, FastqScoreRuntimeRow,
    StageScoringRow,
};
use std::cmp::Ordering;
use std::collections::BTreeMap;

pub(super) fn classify_failure_class(
    binding_rows: &[FastqScoreExpectedRow],
    evidence: Option<&FastqScoreEvidence>,
    runtime_row: Option<&FastqScoreRuntimeRow>,
    external_artifact_binding: bool,
) -> String {
    if binding_rows.is_empty() {
        return "missing_output".to_string();
    }
    if binding_rows.iter().any(|row| row.row_status == "unsupported_pair") {
        return "unsupported_pair".to_string();
    }
    if binding_rows.iter().any(|row| row.row_status == "missing_result") {
        return "missing_output".to_string();
    }
    if external_artifact_binding {
        return "blocked_external_access".to_string();
    }
    if let Some(runtime_row) = runtime_row {
        match runtime_row.execution_status.as_str() {
            "failed" => return "command_failed".to_string(),
            "unavailable" => return "tool_not_found".to_string(),
            _ => {}
        }
    }
    if evidence.is_none() {
        return "insufficient_data".to_string();
    }
    "none".to_string()
}

pub(super) fn describe_row_reason(
    failure_class: &str,
    evidence: Option<&FastqScoreEvidence>,
    runtime_row: Option<&FastqScoreRuntimeRow>,
    memory_row: Option<&FastqScoreMemoryRow>,
    stage_row: &StageScoringRow,
    tool_id: &str,
    failure_catalog: &BTreeMap<String, String>,
) -> String {
    if failure_class != "none" {
        let detail = failure_catalog
            .get(failure_class)
            .cloned()
            .unwrap_or_else(|| "unknown failure classification".to_string());
        if failure_class == "insufficient_data" && evidence.is_none() {
            return format!(
                "{detail}; no real FASTQ smoke or micro evidence row was found for `{}` / `{}`",
                stage_row.stage_id, tool_id
            );
        }
        return detail;
    }
    let evidence_count = evidence.map_or(0, |row| row.source_paths.len());
    match (runtime_row, memory_row) {
        (Some(runtime_row), Some(memory_row)) => format!(
            "scored from {evidence_count} real evidence surface(s) with micro execution status `{}` and memory source `{}`",
            runtime_row.execution_status, memory_row.memory_source
        ),
        (Some(runtime_row), None) => format!(
            "scored from {evidence_count} real evidence surface(s); micro runtime status is `{}` but no memory row matched this binding",
            runtime_row.execution_status
        ),
        _ => format!(
            "scored from {evidence_count} real evidence surface(s); runtime and memory ranking remain unavailable for this binding"
        ),
    }
}

pub(super) fn select_runtime_row(
    stage_id: &str,
    tool_id: &str,
    runtime_rows: &BTreeMap<(String, String), Vec<FastqScoreRuntimeRow>>,
) -> Option<FastqScoreRuntimeRow> {
    runtime_rows
        .get(&(stage_id.to_owned(), tool_id.to_owned()))
        .and_then(|rows| best_runtime_row(rows))
}

pub(super) fn select_memory_row(
    stage_id: &str,
    tool_id: &str,
    memory_rows: &BTreeMap<(String, String), Vec<FastqScoreMemoryRow>>,
) -> Option<FastqScoreMemoryRow> {
    memory_rows
        .get(&(stage_id.to_owned(), tool_id.to_owned()))
        .and_then(|rows| best_memory_row(rows))
}

fn best_runtime_row(rows: &[FastqScoreRuntimeRow]) -> Option<FastqScoreRuntimeRow> {
    let mut candidates = rows.to_vec();
    candidates.sort_by(|left, right| {
        execution_status_rank(&left.execution_status)
            .cmp(&execution_status_rank(&right.execution_status))
            .then_with(|| option_f64_cmp(right.elapsed_seconds, left.elapsed_seconds))
    });
    candidates.into_iter().next()
}

fn best_memory_row(rows: &[FastqScoreMemoryRow]) -> Option<FastqScoreMemoryRow> {
    let mut candidates = rows.to_vec();
    candidates.sort_by(|left, right| {
        execution_status_rank(&left.execution_status)
            .cmp(&execution_status_rank(&right.execution_status))
            .then_with(|| option_f64_cmp(right.observed_memory_mb, left.observed_memory_mb))
            .then_with(|| option_f64_cmp(right.declared_memory_mb, left.declared_memory_mb))
    });
    candidates.into_iter().next()
}

fn execution_status_rank(value: &str) -> usize {
    match value {
        "succeeded" => 0,
        "container_needed" => 1,
        "unavailable" => 2,
        "failed" => 3,
        _ => 4,
    }
}

fn option_f64_cmp(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => Ordering::Equal,
    }
}
