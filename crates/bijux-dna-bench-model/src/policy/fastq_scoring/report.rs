//! Aggregate scored-row outcomes into the persisted report.

use crate::{
    FastqToolScoreRow, FastqToolScoreStatus, FastqToolScoresReport,
    FASTQ_TOOL_SCORES_SCHEMA_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};

/// Summarize scored rows; output locations are caller-supplied metadata.
#[must_use]
pub fn build_fastq_tool_scores_report(
    output_path: String,
    config_path: String,
    rows: Vec<FastqToolScoreRow>,
) -> FastqToolScoresReport {
    let stage_count = rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len();
    let scored_row_count =
        rows.iter().filter(|row| row.score_status == FastqToolScoreStatus::Scored).count();
    let insufficient_evidence_row_count = rows
        .iter()
        .filter(|row| row.score_status == FastqToolScoreStatus::InsufficientEvidence)
        .count();
    let blocked_row_count =
        rows.iter().filter(|row| row.score_status == FastqToolScoreStatus::Blocked).count();
    let failure_class_counts =
        rows.iter().fold(BTreeMap::<String, usize>::new(), |mut counts, row| {
            *counts.entry(row.failure_class.clone()).or_default() += 1;
            counts
        });
    FastqToolScoresReport {
        schema_version: FASTQ_TOOL_SCORES_SCHEMA_VERSION,
        output_path,
        config_path,
        row_count: rows.len(),
        stage_count,
        tool_count,
        scored_row_count,
        insufficient_evidence_row_count,
        blocked_row_count,
        failure_class_counts,
        rows,
    }
}
