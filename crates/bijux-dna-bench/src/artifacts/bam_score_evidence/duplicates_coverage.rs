//! Owner: bijux-dna-bench
//! Interpret observed BAM duplicates and coverage report fields without filesystem access.

use super::fields::{
    checked_observation_sum, format_f64, optional_str, parse_bool, parse_f64, parse_u64,
    required_bool, required_f64, required_str, required_tsv, required_u64, summarize_metrics,
};
use super::merge::insert_evidence;
use super::{BamEvidenceAggregate, EvidenceRows};
use anyhow::Result;
use bijux_dna_core::prelude::numeric::checked_f64_from_u64;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) fn merge_markdup_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let duplicate_fraction = required_f64(root, "duplicate_fraction")?;
    let duplicate_count = required_u64(root, "duplicate_count")?;
    let output_reads = required_u64(root, "output_reads")?;
    let removed_reads = required_u64(root, "removed_reads")?;
    insert_evidence(
        rows,
        required_str(root, "stage_id")?,
        "samtools",
        BamEvidenceAggregate {
            truth_correctness_score: Some((1.0 - duplicate_fraction).clamp(0.0, 1.0)),
            truth_correctness_basis: Some("one_minus_duplicate_fraction".to_string()),
            contract_correctness_score: Some(if required_bool(root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "markdup smoke report preserved governed duplicate marking outputs".to_string(),
            ),
            retained_reads: Some(output_reads),
            dropped_reads: Some(removed_reads),
            alignment_qc_metric_value: Some((1.0 - duplicate_fraction).clamp(0.0, 1.0)),
            alignment_qc_metric_basis: Some("one_minus_duplicate_fraction".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("duplicate_count={duplicate_count}"),
                format!("duplicate_fraction={}", format_f64(duplicate_fraction)),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_duplication_metrics_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let duplicate_fraction = required_f64(root, "duplicate_fraction")?;
    let duplicate_count = required_u64(root, "duplicate_count")?;
    let examined_reads = required_u64(root, "examined_reads")?;
    let estimated_library_size =
        optional_str(root, "estimated_library_size").unwrap_or("").to_string();
    insert_evidence(
        rows,
        required_str(root, "stage_id")?,
        "samtools",
        BamEvidenceAggregate {
            truth_correctness_score: Some((1.0 - duplicate_fraction).clamp(0.0, 1.0)),
            truth_correctness_basis: Some("one_minus_duplicate_fraction".to_string()),
            contract_correctness_score: Some(if required_bool(root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "duplication metrics report preserved governed duplicate count and fraction surfaces"
                    .to_string(),
            ),
            retained_reads: Some(examined_reads),
            alignment_qc_metric_value: Some((1.0 - duplicate_fraction).clamp(0.0, 1.0)),
            alignment_qc_metric_basis: Some("one_minus_duplicate_fraction".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("duplicate_count={duplicate_count}"),
                format!("duplicate_fraction={}", format_f64(duplicate_fraction)),
                format!("estimated_library_size={estimated_library_size}"),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_coverage_summary(
    tsv_rows: &[BTreeMap<String, String>],
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    if tsv_rows.is_empty() {
        return Ok(());
    }
    let mut breadth_sum = 0.0;
    let mut mean_depth_sum = 0.0;
    let mut covered_bases_total = 0_u64;
    let mut matched = true;
    let mut count = 0_u64;
    for row in tsv_rows {
        breadth_sum = checked_observation_sum(
            breadth_sum,
            parse_f64(required_tsv(row, "breadth_1x")?)?,
            "breadth_1x",
        )?;
        mean_depth_sum = checked_observation_sum(
            mean_depth_sum,
            parse_f64(required_tsv(row, "mean_depth")?)?,
            "mean_depth",
        )?;
        covered_bases_total = covered_bases_total
            .checked_add(parse_u64(required_tsv(row, "covered_bases")?)?)
            .ok_or_else(|| anyhow::anyhow!("BAM coverage covered_bases total overflow"))?;
        matched &= parse_bool(required_tsv(row, "case_expectation_matched")?)?;
        count = count
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("BAM coverage row count overflow"))?;
    }
    let mean_breadth = breadth_sum / checked_f64_from_u64(count, "coverage row count")?;
    let mean_depth = mean_depth_sum / checked_f64_from_u64(count, "coverage row count")?;
    insert_evidence(
        rows,
        bijux_dna_core::id_catalog::BAM_COVERAGE,
        "samtools",
        BamEvidenceAggregate {
            truth_correctness_score: Some(mean_breadth),
            truth_correctness_basis: Some("mean_breadth_1x".to_string()),
            contract_correctness_score: Some(if matched { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "coverage TSV preserved governed breadth, covered bases, and depth surfaces"
                    .to_string(),
            ),
            coverage_metric_value: Some(mean_depth),
            coverage_metric_basis: Some("mean_depth".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("breadth_1x_mean={}", format_f64(mean_breadth)),
                format!("covered_bases_total={covered_bases_total}"),
                format!("mean_depth_mean={}", format_f64(mean_depth)),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}
