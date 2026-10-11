//! Owner: bijux-dna-bench-model
//! Summarize VCF scoring rows in the existing report envelope.

use crate::{
    VcfToolScoreRow, VcfToolScoreStatus, VcfToolScoresReport, VCF_TOOL_SCORES_SCHEMA_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};

/// Build report counts from supplied score rows.
#[must_use]
pub fn build_vcf_tool_scores_report(
    output_path: String,
    config_path: String,
    rows: Vec<VcfToolScoreRow>,
) -> VcfToolScoresReport {
    let stage_count = rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len();
    let scored_row_count =
        rows.iter().filter(|row| row.score_status == VcfToolScoreStatus::Scored).count();
    let insufficient_evidence_row_count = rows
        .iter()
        .filter(|row| row.score_status == VcfToolScoreStatus::InsufficientEvidence)
        .count();
    let blocked_row_count =
        rows.iter().filter(|row| row.score_status == VcfToolScoreStatus::Blocked).count();
    let failure_class_counts =
        rows.iter().fold(BTreeMap::<String, usize>::new(), |mut counts, row| {
            *counts.entry(row.failure_class.clone()).or_default() += 1;
            counts
        });
    VcfToolScoresReport {
        schema_version: VCF_TOOL_SCORES_SCHEMA_VERSION,
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
