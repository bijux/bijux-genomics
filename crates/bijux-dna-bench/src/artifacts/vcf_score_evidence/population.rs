//! Owner: bijux-dna-bench
//! Interpret VCF population reports without filesystem access.

use super::fields::{
    fraction_u64, optional_f64, required_array, required_bool, required_i64, required_object,
    required_str, required_u64, required_u64_from_object, summarize_metrics,
};
use super::merge::insert_evidence;
use super::{EvidenceRows, VcfEvidenceAggregate};
use anyhow::Result;
use bijux_dna_core::prelude::numeric::checked_f64_from_u64;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn merge_admixture_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id = super::fields::required_stage(root, bijux_dna_core::id_catalog::VCF_ADMIXTURE)?;
    let sample_count = required_u64(root, "sample_count")?;
    let population_count = required_u64(root, "population_count")?;
    let selected_k = required_u64(root, "selected_k")?;
    let status = required_str(root, "status")?;
    let tool_ok = required_bool(root, "tool_ok")?;
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(
                selected_k.min(population_count),
                population_count.max(1),
            )?),
            truth_correctness_basis: Some("selected_k_over_population_count".to_string()),
            contract_correctness_score: Some(if tool_ok && status == "complete" {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "admixture smoke report preserved governed selected_k and cluster outputs"
                    .to_string(),
            ),
            population_metric_value: Some(checked_f64_from_u64(
                population_count,
                "population count",
            )?),
            population_metric_basis: Some("population_count".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("selected_k={selected_k}"),
                format!("sample_count={sample_count}"),
                format!("population_count={population_count}"),
                format!("status={status}"),
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

pub(crate) fn merge_pca_summary(root: &Value, path: &Path, rows: &mut EvidenceRows) -> Result<()> {
    let stage_id = super::fields::required_stage(root, bijux_dna_core::id_catalog::VCF_PCA)?;
    let excluded_samples = required_array(root, "excluded_samples")?;
    let unexpected_samples = required_array(root, "unexpected_samples")?;
    let tool_ok = required_bool(root, "tool_ok")?;
    let variant_count = required_u64(root, "variant_count")?;
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(
                if tool_ok && excluded_samples.is_empty() && unexpected_samples.is_empty() {
                    1.0
                } else {
                    0.0
                },
            ),
            truth_correctness_basis: Some("sample_complete_projection".to_string()),
            contract_correctness_score: Some(if required_i64(root, "exit_code")? == 0 {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "PCA smoke preserved governed eigenvalue and coordinate outputs".to_string(),
            ),
            population_metric_value: Some(checked_f64_from_u64(variant_count, "variant count")?),
            population_metric_basis: Some("variant_count".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("sample_count={}", required_u64(root, "sample_count")?),
                format!("variant_count={variant_count}"),
                format!("eigenvalue_count={}", required_array(root, "eigenvalues")?.len()),
                format!("execution_mode={}", required_str(root, "execution_mode")?),
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

pub(crate) fn merge_population_structure_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id =
        super::fields::required_stage(root, bijux_dna_core::id_catalog::VCF_POPULATION_STRUCTURE)?;
    let distance_summary = required_object(root, "distance_summary")?;
    let pair_count = required_u64_from_object(distance_summary, "pair_count")?;
    let sample_count = required_u64_from_object(distance_summary, "sample_count")?;
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(if required_str(root, "status")? == "complete" { 1.0 } else { 0.0 }),
            truth_correctness_basis: Some("status_complete".to_string()),
            contract_correctness_score: Some(if required_i64(root, "exit_code")? == 0 { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "population-structure smoke preserved governed distance summaries and consumed PCA/admixture evidence".to_string(),
            ),
            population_metric_value: Some(checked_f64_from_u64(pair_count, "pair count")?),
            population_metric_basis: Some("pair_count".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("sample_count={sample_count}"),
                format!("pair_count={pair_count}"),
                format!(
                    "within_population_pair_count={}",
                    required_u64_from_object(distance_summary, "within_population_pair_count")?
                ),
                format!(
                    "cross_population_pair_count={}",
                    required_u64_from_object(distance_summary, "cross_population_pair_count")?
                ),
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
