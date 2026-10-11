//! Owner: bijux-dna-bench
//! Interpret VCF relatedness reports without filesystem access.

use super::fields::{
    format_f64, optional_f64, required_array, required_f64, required_i64, required_str,
    required_u64, summarize_metrics,
};
use super::merge::insert_evidence;
use super::{EvidenceRows, VcfEvidenceAggregate};
use anyhow::Result;
use bijux_dna_core::prelude::numeric::checked_f64_from_u64;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn merge_demography_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id = super::fields::required_stage(root, bijux_dna_core::id_catalog::VCF_DEMOGRAPHY)?;
    let estimates = required_array(root, "ne_estimates")?;
    let contract_ok = required_i64(root, "exit_code")? == 0
        && required_str(root, "status")? == "complete"
        && !estimates.is_empty();
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "demography smoke preserved governed trajectory and inference outputs".to_string(),
            ),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("method={}", required_str(root, "method")?),
                format!("inference_status={}", required_str(root, "inference_status")?),
                format!("time_bin_count={}", required_array(root, "time_bins")?.len()),
                format!("estimate_count={}", estimates.len()),
            ])),
            runtime_seconds: optional_f64(root, "elapsed_seconds"),
            runtime_source: Some("local_smoke_report".to_string()),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_ibd_summary(root: &Value, path: &Path, rows: &mut EvidenceRows) -> Result<()> {
    let stage_id = super::fields::required_stage(root, bijux_dna_core::id_catalog::VCF_IBD)?;
    let pair_count = required_u64(root, "pair_count")?;
    let contract_ok = required_i64(root, "exit_code")? == 0
        && required_str(root, "status")? == "complete"
        && pair_count > 0;
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            truth_correctness_basis: Some(
                "nonzero_pair_coverage_not_independent_ibd_truth".to_string(),
            ),
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "IBD smoke preserved governed pair and normalized segment outputs".to_string(),
            ),
            population_metric_value: Some(checked_f64_from_u64(pair_count, "pair count")?),
            population_metric_basis: Some("pair_count".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("pair_count={pair_count}"),
                format!("pair_row_count={}", required_array(root, "rows")?.len()),
                format!("execution_mode={}", required_str(root, "execution_mode")?),
                "accuracy_claim=integration_only_not_independent_ibd_truth".to_string(),
            ])),
            runtime_seconds: optional_f64(root, "elapsed_seconds"),
            runtime_source: Some("local_smoke_report".to_string()),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_roh_summary(root: &Value, path: &Path, rows: &mut EvidenceRows) -> Result<()> {
    let stage_id = super::fields::required_stage(root, bijux_dna_core::id_catalog::VCF_ROH)?;
    let contract_ok =
        required_i64(root, "exit_code")? == 0 && required_str(root, "status")? == "complete";
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "ROH smoke preserved governed segment and per-sample summary outputs".to_string(),
            ),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("segment_count={}", required_u64(root, "segment_count")?),
                format!("sample_count={}", required_u64(root, "sample_count")?),
                format!("total_length={}", format_f64(required_f64(root, "total_length")?)),
            ])),
            runtime_seconds: optional_f64(root, "elapsed_seconds"),
            runtime_source: Some("local_smoke_report".to_string()),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}
