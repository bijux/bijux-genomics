//! Owner: bijux-dna-bench
//! Merge BAM observations in declared source priority order.

use super::{BamEvidenceAggregate, EvidenceRows};
use std::path::Path;

pub(super) fn insert_evidence(
    rows: &mut EvidenceRows,
    stage_id: &str,
    tool_id: &str,
    evidence: BamEvidenceAggregate,
    source_path: &Path,
) {
    let entry = rows.entry((stage_id.to_string(), tool_id.to_string())).or_default();
    entry.source_paths.insert(source_path.display().to_string());
    merge_option(&mut entry.truth_correctness_score, evidence.truth_correctness_score);
    merge_string(&mut entry.truth_correctness_basis, evidence.truth_correctness_basis);
    merge_option(&mut entry.contract_correctness_score, evidence.contract_correctness_score);
    merge_string(&mut entry.contract_correctness_basis, evidence.contract_correctness_basis);
    merge_option(&mut entry.retained_reads, evidence.retained_reads);
    merge_option(&mut entry.dropped_reads, evidence.dropped_reads);
    merge_option(&mut entry.alignment_qc_metric_value, evidence.alignment_qc_metric_value);
    merge_string(&mut entry.alignment_qc_metric_basis, evidence.alignment_qc_metric_basis);
    merge_option(&mut entry.coverage_metric_value, evidence.coverage_metric_value);
    merge_string(&mut entry.coverage_metric_basis, evidence.coverage_metric_basis);
    merge_option(&mut entry.damage_metric_value, evidence.damage_metric_value);
    merge_string(&mut entry.damage_metric_basis, evidence.damage_metric_basis);
    merge_option(&mut entry.authenticity_metric_value, evidence.authenticity_metric_value);
    merge_string(&mut entry.authenticity_metric_basis, evidence.authenticity_metric_basis);
    merge_string(&mut entry.scientific_metric_summary, evidence.scientific_metric_summary);
}

fn merge_option<T: Copy>(target: &mut Option<T>, incoming: Option<T>) {
    if target.is_none() {
        *target = incoming;
    }
}

fn merge_string(target: &mut Option<String>, incoming: Option<String>) {
    if target.is_none() {
        *target = incoming;
    }
}
