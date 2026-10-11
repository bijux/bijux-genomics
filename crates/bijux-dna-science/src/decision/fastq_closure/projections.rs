//! Traceable closure gaps and default-binding risk projections from evaluated decisions.
use crate::domain::{
    FastqClosureGateRow, FastqDefaultBindingRiskRow, FastqMissingClosurePrerequisiteRow,
    FastqTruthDeltaRow,
};
pub(crate) fn build_fastq_truth_delta_rows(
    rows: &[FastqClosureGateRow],
) -> Vec<FastqTruthDeltaRow> {
    let mut deltas = Vec::new();
    for row in rows {
        if row.requested_execution_status == "closed" && !row.world_class_closed {
            deltas.push(FastqTruthDeltaRow {
                entity_type: "fastq_stage_tool_binding".to_string(),
                entity_id: format!("{}:{}", row.stage_id, row.tool_id),
                layer: "closure_gate".to_string(),
                expected_status: "world_class_closed".to_string(),
                observed_status: row.effective_closure_status.clone(),
                reason: row.blocking_reasons.clone(),
            });
        }
    }
    deltas
}

pub(crate) fn build_fastq_missing_closure_prerequisite_rows(
    rows: &[FastqClosureGateRow],
) -> Vec<FastqMissingClosurePrerequisiteRow> {
    let mut missing = Vec::new();
    for row in rows {
        for reason in row.blocking_reasons.split(';').filter(|reason| !reason.trim().is_empty()) {
            missing.push(FastqMissingClosurePrerequisiteRow {
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                prerequisite: reason.to_string(),
                severity: if row.is_default { "blocking" } else { "advisory" }.to_string(),
                detail: format!(
                    "{} cannot be treated as world-class closed for {} until {} is resolved",
                    row.tool_id, row.stage_id, reason
                ),
            });
        }
    }
    missing.sort_by(|left, right| {
        (&left.stage_id, &left.tool_id, &left.prerequisite).cmp(&(
            &right.stage_id,
            &right.tool_id,
            &right.prerequisite,
        ))
    });
    missing
}

pub(crate) fn build_fastq_default_binding_risk_rows(
    rows: &[FastqClosureGateRow],
) -> Vec<FastqDefaultBindingRiskRow> {
    rows.iter()
        .filter(|row| row.is_default)
        .map(|row| FastqDefaultBindingRiskRow {
            stage_id: row.stage_id.clone(),
            default_tool_id: row.tool_id.clone(),
            requested_execution_status: row.requested_execution_status.clone(),
            effective_closure_status: row.effective_closure_status.clone(),
            risk_class: if row.world_class_closed {
                "world_class_closed".to_string()
            } else if row.blocking_reasons.contains("pending_container_digest") {
                "immutable_container_blocked".to_string()
            } else if row.blocking_reasons.contains("missing_upstream_archive")
                || row.blocking_reasons.contains("missing_paper_archive")
            {
                "archive_evidence_blocked".to_string()
            } else {
                "closure_prerequisite_blocked".to_string()
            },
            blocking_reasons: row.blocking_reasons.clone(),
            warning_reasons: row.warning_reasons.clone(),
        })
        .collect()
}
