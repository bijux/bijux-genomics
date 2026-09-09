//! Owner: bijux-dna-bench-model
//! Select VCF observations and classify unavailable or blocked evidence.

use crate::{VcfScoreEvidence, VcfScoreExpectedRow, VcfScoreMemoryRow, VcfScoreRuntimeRow};
use std::cmp::Ordering;
use std::collections::BTreeMap;

pub(super) fn classify_failure_class(
    binding_rows: &[VcfScoreExpectedRow],
    evidence: Option<&VcfScoreEvidence>,
    runtime_row: Option<&VcfScoreRuntimeRow>,
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
    if evidence.is_none() {
        if let Some(runtime_row) = runtime_row {
            match runtime_row.execution_status.as_str() {
                "failed" => return "command_failed".to_string(),
                "unavailable" => return "tool_not_found".to_string(),
                _ => {}
            }
        }
        return "insufficient_data".to_string();
    }
    "none".to_string()
}

pub(super) fn describe_row_reason(
    failure_class: &str,
    evidence: Option<&VcfScoreEvidence>,
    runtime_row: Option<&VcfScoreRuntimeRow>,
    memory_row: Option<&VcfScoreMemoryRow>,
    stage_row: &crate::StageScoringRow,
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
                "{detail}; no real VCF smoke or micro evidence row was found for `{}` / `{}`",
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
    stage_row: &crate::StageScoringRow,
    tool_id: &str,
    runtime_rows: &BTreeMap<(String, String), Vec<VcfScoreRuntimeRow>>,
    has_evidence: bool,
) -> Option<VcfScoreRuntimeRow> {
    if let Some(rows) = runtime_rows.get(&(stage_row.stage_id.clone(), tool_id.to_string())) {
        return best_runtime_row(rows);
    }
    if tool_id != stage_row.default_tool_id || !has_evidence {
        return None;
    }
    let stage_candidates = runtime_rows
        .iter()
        .filter(|((stage_id, _), _)| stage_id == &stage_row.stage_id)
        .flat_map(|(_, rows)| rows.iter().cloned())
        .collect::<Vec<_>>();
    if stage_candidates.len() == 1 {
        return stage_candidates.into_iter().next();
    }
    if stage_candidates
        .iter()
        .all(|row| !stage_row.benchmark_ready_tool_ids.iter().any(|tool| tool == &row.tool_id))
    {
        return best_runtime_row(&stage_candidates);
    }
    None
}

pub(super) fn select_memory_row(
    stage_row: &crate::StageScoringRow,
    tool_id: &str,
    memory_rows: &BTreeMap<(String, String), Vec<VcfScoreMemoryRow>>,
    has_evidence: bool,
) -> Option<VcfScoreMemoryRow> {
    if let Some(rows) = memory_rows.get(&(stage_row.stage_id.clone(), tool_id.to_string())) {
        return best_memory_row(rows);
    }
    if tool_id != stage_row.default_tool_id || !has_evidence {
        return None;
    }
    let stage_candidates = memory_rows
        .iter()
        .filter(|((stage_id, _), _)| stage_id == &stage_row.stage_id)
        .flat_map(|(_, rows)| rows.iter().cloned())
        .collect::<Vec<_>>();
    if stage_candidates.len() == 1 {
        return stage_candidates.into_iter().next();
    }
    if stage_candidates
        .iter()
        .all(|row| !stage_row.benchmark_ready_tool_ids.iter().any(|tool| tool == &row.tool_id))
    {
        return best_memory_row(&stage_candidates);
    }
    None
}

pub(super) fn best_runtime_row(rows: &[VcfScoreRuntimeRow]) -> Option<VcfScoreRuntimeRow> {
    let mut candidates = rows.to_vec();
    candidates.sort_by(|left, right| {
        execution_status_rank(&left.execution_status)
            .cmp(&execution_status_rank(&right.execution_status))
            .then_with(|| option_f64_cmp(right.elapsed_seconds, left.elapsed_seconds))
    });
    candidates.into_iter().next()
}

pub(super) fn best_memory_row(rows: &[VcfScoreMemoryRow]) -> Option<VcfScoreMemoryRow> {
    let mut candidates = rows.to_vec();
    candidates.sort_by(|left, right| {
        execution_status_rank(&left.execution_status)
            .cmp(&execution_status_rank(&right.execution_status))
            .then_with(|| option_f64_cmp(right.observed_memory_mb, left.observed_memory_mb))
            .then_with(|| option_f64_cmp(right.declared_memory_mb, left.declared_memory_mb))
    });
    candidates.into_iter().next()
}

pub(super) fn execution_status_rank(status: &str) -> usize {
    match status {
        "succeeded" => 0,
        "unavailable" => 1,
        "container_needed" => 2,
        "failed" => 3,
        _ => 4,
    }
}

pub(super) fn option_f64_cmp(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}
