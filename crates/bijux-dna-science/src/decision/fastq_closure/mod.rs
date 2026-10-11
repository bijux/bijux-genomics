//! Closure decisions over declared environment, source archives, paper records and QA blockers.
mod blockers;
mod projections;
use crate::domain::compilation::QaCoverageBlockerEntry;
use crate::domain::{FastqClosureGateRow, FastqEnvironmentRow};
use blockers::{closure_blockers, closure_warnings};
pub(crate) use projections::{
    build_fastq_default_binding_risk_rows, build_fastq_missing_closure_prerequisite_rows,
    build_fastq_truth_delta_rows,
};
use std::collections::{BTreeMap, BTreeSet};
pub(crate) fn build_fastq_closure_gate_rows(
    environment_rows: &[FastqEnvironmentRow],
    download_rows: &[crate::domain::FastqDownloadBacklogRow],
    paper_rows: &[crate::domain::FastqPaperArchiveRow],
    qa_coverage_blockers: &[QaCoverageBlockerEntry],
) -> Vec<FastqClosureGateRow> {
    let download_by_tool =
        download_rows.iter().map(|row| (row.tool_id.as_str(), row)).collect::<BTreeMap<_, _>>();
    let paper_by_tool = paper_rows.iter().fold(
        BTreeMap::<&str, Vec<&crate::domain::FastqPaperArchiveRow>>::new(),
        |mut acc, row| {
            acc.entry(row.tool_id.as_str()).or_default().push(row);
            acc
        },
    );
    let qa_blockers_by_stage = qa_coverage_blockers.iter().fold(
        BTreeMap::<&str, BTreeSet<&str>>::new(),
        |mut acc, blocker| {
            acc.entry(blocker.stage_id.as_str()).or_default().insert(blocker.blocker.as_str());
            acc
        },
    );

    let mut rows = environment_rows
        .iter()
        .map(|row| {
            let blockers =
                closure_blockers(row, &download_by_tool, &paper_by_tool, &qa_blockers_by_stage);
            let warnings = closure_warnings(row);
            let effective_closure_status = if blockers.is_empty() {
                "world_class_closed"
            } else if row.execution_status == "closed" {
                "declared_closed_with_gaps"
            } else {
                "not_closed"
            }
            .to_string();
            FastqClosureGateRow {
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                is_default: row.is_default,
                requested_execution_status: row.execution_status.clone(),
                effective_closure_status,
                world_class_closed: blockers.is_empty(),
                blocking_reasons: blockers.join(";"),
                warning_reasons: warnings.join(";"),
            }
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        (&left.stage_id, !left.is_default, &left.tool_id).cmp(&(
            &right.stage_id,
            !right.is_default,
            &right.tool_id,
        ))
    });
    rows
}
