//! Owner: bijux-dna-bench-model
//! Summarize BAM scoring rows in the existing report envelope.

use crate::{
    BamToolScoreRow, BamToolScoreStatus, BamToolScoresReport, BAM_TOOL_SCORES_SCHEMA_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};

/// Build report counts from the supplied score rows.
#[must_use]
pub fn build_bam_tool_scores_report(
    output_path: String,
    config_path: String,
    rows: Vec<BamToolScoreRow>,
) -> BamToolScoresReport {
    let stage_count = rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len();
    let scored_row_count =
        rows.iter().filter(|row| row.score_status == BamToolScoreStatus::Scored).count();
    let insufficient_evidence_row_count = rows
        .iter()
        .filter(|row| row.score_status == BamToolScoreStatus::InsufficientEvidence)
        .count();
    let blocked_row_count =
        rows.iter().filter(|row| row.score_status == BamToolScoreStatus::Blocked).count();
    let failure_class_counts =
        rows.iter().fold(BTreeMap::<String, usize>::new(), |mut counts, row| {
            *counts.entry(row.failure_class.clone()).or_default() += 1;
            counts
        });
    BamToolScoresReport {
        schema_version: BAM_TOOL_SCORES_SCHEMA_VERSION,
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
