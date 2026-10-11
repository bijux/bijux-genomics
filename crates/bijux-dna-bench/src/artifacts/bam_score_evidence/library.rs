//! Owner: bijux-dna-bench
//! Interpret observed BAM library report fields without filesystem access.

use super::fields::{
    format_f64, optional_str, optional_u64, required_bool, required_f64, required_str,
    required_u64, summarize_metrics,
};
use super::merge::insert_evidence;
use super::{BamEvidenceAggregate, EvidenceRows};
use anyhow::Result;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn merge_complexity_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id = super::fields::required_stage(root, bijux_dna_core::id_catalog::BAM_COMPLEXITY)?;
    insert_evidence(
        rows,
        stage_id,
        required_str(root, "method")?,
        BamEvidenceAggregate {
            contract_correctness_score: Some(if required_bool(root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "complexity smoke report preserved governed library complexity summary outputs"
                    .to_string(),
            ),
            scientific_metric_summary: summarize_metrics(vec![
                format!(
                    "estimated_library_size={}",
                    optional_u64(root, "estimated_library_size")
                        .map_or_else(String::new, |value| value.to_string())
                ),
                format!(
                    "saturation_estimate={}",
                    optional_str(root, "saturation_estimate").unwrap_or("")
                ),
                format!("observed_total_reads={}", required_u64(root, "observed_total_reads")?),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_endogenous_content_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id =
        super::fields::required_stage(root, bijux_dna_core::id_catalog::BAM_ENDOGENOUS_CONTENT)?;
    insert_evidence(
        rows,
        stage_id,
        "samtools",
        BamEvidenceAggregate {
            contract_correctness_score: Some(if required_bool(root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "endogenous content report preserved governed endogenous fraction outputs"
                    .to_string(),
            ),
            retained_reads: Some(required_u64(root, "endogenous_reads")?),
            coverage_metric_value: Some(required_f64(root, "endogenous_fraction")?),
            coverage_metric_basis: Some("endogenous_fraction".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!(
                    "endogenous_fraction={}",
                    format_f64(required_f64(root, "endogenous_fraction")?)
                ),
                format!("mapped_reads={}", required_u64(root, "mapped_reads")?),
                format!("total_reads={}", required_u64(root, "total_reads")?),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_insert_size_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id =
        super::fields::required_stage(root, bijux_dna_core::id_catalog::BAM_INSERT_SIZE)?;
    insert_evidence(
        rows,
        stage_id,
        required_str(root, "method")?,
        BamEvidenceAggregate {
            contract_correctness_score: Some(if required_bool(root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "insert size report preserved governed distribution summary outputs".to_string(),
            ),
            scientific_metric_summary: summarize_metrics(vec![
                format!("read_pairs={}", required_u64(root, "read_pairs")?),
                format!("mean_insert_size={}", format_f64(required_f64(root, "mean_insert_size")?)),
                format!(
                    "median_insert_size={}",
                    format_f64(required_f64(root, "median_insert_size")?)
                ),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}
