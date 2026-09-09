//! Owner: bijux-dna-bench
//! Interpret observed BAM correction report fields without filesystem access.

use super::fields::{
    format_f64, parse_bool, parse_f64, required_bool, required_f64, required_str, required_tsv,
    required_u64, summarize_metrics,
};
use super::merge::insert_evidence;
use super::{BamEvidenceAggregate, EvidenceRows};
use anyhow::Result;
use bijux_dna_core::prelude::numeric::checked_f64_from_u64;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) fn merge_overlap_correction_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    insert_evidence(
        rows,
        required_str(root, "stage_id")?,
        required_str(root, "method")?,
        BamEvidenceAggregate {
            contract_correctness_score: Some(if required_bool(root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "overlap correction report preserved governed corrected pair outputs".to_string(),
            ),
            retained_reads: Some(required_u64(root, "pair_count")?),
            scientific_metric_summary: summarize_metrics(vec![
                format!("pair_count={}", required_u64(root, "pair_count")?),
                format!("corrected_pairs={}", required_u64(root, "corrected_pairs")?),
                format!(
                    "corrected_overlap_bases={}",
                    required_u64(root, "corrected_overlap_bases")?
                ),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_recalibration_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let contract_ok = required_bool(root, "expectation_matched")?
        && required_bool(root, "output_bam_present")?
        && required_bool(root, "recalibration_report_present")?;
    insert_evidence(
        rows,
        required_str(root, "stage_id")?,
        required_str(root, "tool_id")?,
        BamEvidenceAggregate {
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "recalibration smoke report preserved governed recalibrated BAM and report outputs"
                    .to_string(),
            ),
            coverage_metric_value: Some(required_f64(root, "observed_mean_coverage")?),
            coverage_metric_basis: Some("observed_mean_coverage".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("status={}", required_str(root, "status")?),
                format!(
                    "observed_mean_coverage={}",
                    format_f64(required_f64(root, "observed_mean_coverage")?)
                ),
                format!(
                    "observed_breadth_1x={}",
                    format_f64(required_f64(root, "observed_breadth_1x")?)
                ),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_gc_bias_summary(
    tsv_rows: &[BTreeMap<String, String>],
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    if tsv_rows.is_empty() {
        return Ok(());
    }
    let mut matched = true;
    let mut deviation_sum = 0.0;
    let mut count = 0_u64;
    for row in tsv_rows {
        matched &= parse_bool(required_tsv(row, "case_expectation_matched")?)?;
        let normalized_coverage = parse_f64(required_tsv(row, "normalized_coverage")?)?;
        deviation_sum += (normalized_coverage - 1.0).abs();
        count += 1;
    }
    let mean_deviation = deviation_sum / checked_f64_from_u64(count, "GC bias row count")?;
    insert_evidence(
        rows,
        bijux_dna_core::id_catalog::BAM_GC_BIAS,
        "picard",
        BamEvidenceAggregate {
            contract_correctness_score: Some(if matched { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "GC bias TSV preserved governed normalized coverage and window surfaces"
                    .to_string(),
            ),
            scientific_metric_summary: summarize_metrics(vec![
                format!("mean_abs_normalized_coverage_deviation={}", format_f64(mean_deviation)),
                format!("gc_bin_count={count}"),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}
