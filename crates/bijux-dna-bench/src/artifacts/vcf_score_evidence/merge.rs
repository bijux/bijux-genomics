//! Owner: bijux-dna-bench
//! Merge VCF evidence in declared source priority order.

use super::{EvidenceRows, VcfEvidenceAggregate};
use std::path::Path;

pub(super) fn insert_evidence(
    rows: &mut EvidenceRows,
    stage_id: &str,
    tool_id: &str,
    evidence: VcfEvidenceAggregate,
    source_path: &Path,
) {
    let entry = rows.entry((stage_id.to_string(), tool_id.to_string())).or_default();
    entry.source_paths.insert(source_path.display().to_string());
    merge_option(&mut entry.truth_correctness_score, evidence.truth_correctness_score);
    merge_string(&mut entry.truth_correctness_basis, evidence.truth_correctness_basis);
    merge_option(&mut entry.contract_correctness_score, evidence.contract_correctness_score);
    merge_string(&mut entry.contract_correctness_basis, evidence.contract_correctness_basis);
    merge_option(&mut entry.genotype_truth_metric_value, evidence.genotype_truth_metric_value);
    merge_string(&mut entry.genotype_truth_metric_basis, evidence.genotype_truth_metric_basis);
    merge_option(&mut entry.missingness_metric_value, evidence.missingness_metric_value);
    merge_string(&mut entry.missingness_metric_basis, evidence.missingness_metric_basis);
    merge_option(
        &mut entry.phasing_imputation_metric_value,
        evidence.phasing_imputation_metric_value,
    );
    merge_string(
        &mut entry.phasing_imputation_metric_basis,
        evidence.phasing_imputation_metric_basis,
    );
    merge_option(&mut entry.population_metric_value, evidence.population_metric_value);
    merge_string(&mut entry.population_metric_basis, evidence.population_metric_basis);
    merge_string(&mut entry.scientific_metric_summary, evidence.scientific_metric_summary);
    merge_option(&mut entry.runtime_seconds, evidence.runtime_seconds);
    merge_string(&mut entry.runtime_source, evidence.runtime_source);
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
