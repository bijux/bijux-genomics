//! Owner: bijux-dna-bench
//! Interpret VCF imputation reports without filesystem access.

use super::fields::{
    format_f64, fraction_u64, optional_f64, required_f64, required_i64, required_str, required_u64,
    summarize_metrics,
};
use super::merge::insert_evidence;
use super::{EvidenceRows, VcfEvidenceAggregate};
use anyhow::Result;
use bijux_dna_core::prelude::numeric::checked_f64_from_u64;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn merge_imputation_metrics_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id =
        super::fields::required_stage(root, bijux_dna_core::id_catalog::VCF_IMPUTATION_METRICS)?;
    let concordance = required_f64(root, "concordance")?;
    let dosage_r2 = required_f64(root, "dosage_r2")?;
    let mean_info_score = required_f64(root, "mean_info_score")?;
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(concordance),
            truth_correctness_basis: Some("concordance".to_string()),
            contract_correctness_score: Some(if required_i64(root, "exit_code")? == 0 { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "imputation metrics smoke preserved governed concordance and dosage quality outputs".to_string(),
            ),
            phasing_imputation_metric_value: Some(dosage_r2),
            phasing_imputation_metric_basis: Some("dosage_r2".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("concordance={}", format_f64(concordance)),
                format!("dosage_r2={}", format_f64(dosage_r2)),
                format!("mean_info_score={}", format_f64(mean_info_score)),
                format!("low_confidence_sites={}", required_u64(root, "low_confidence_sites")?),
                format!("masked_truth_sites={}", required_u64(root, "masked_truth_sites")?),
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

pub(crate) fn merge_impute_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id = super::fields::required_stage(root, bijux_dna_core::id_catalog::VCF_IMPUTE)?;
    let masked_truth_match_count = required_u64(root, "masked_truth_match_count")?;
    let masked_truth_site_count = required_u64(root, "masked_truth_site_count")?;
    let imputed_genotypes = required_u64(root, "imputed_genotypes")?;
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(
                masked_truth_match_count,
                masked_truth_site_count.max(1),
            )?),
            truth_correctness_basis: Some("masked_truth_match_fraction".to_string()),
            contract_correctness_score: Some(if required_i64(root, "exit_code")? == 0 {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "imputation smoke preserved governed masked-truth and completion outputs"
                    .to_string(),
            ),
            phasing_imputation_metric_value: Some(fraction_u64(
                masked_truth_match_count,
                masked_truth_site_count.max(1),
            )?),
            phasing_imputation_metric_basis: Some("masked_truth_match_fraction".to_string()),
            genotype_truth_metric_value: Some(checked_f64_from_u64(
                imputed_genotypes,
                "imputed genotypes",
            )?),
            genotype_truth_metric_basis: Some("imputed_genotypes".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("missing_before={}", required_u64(root, "missing_before")?),
                format!("missing_after={}", required_u64(root, "missing_after")?),
                format!("low_confidence_count={}", required_u64(root, "low_confidence_count")?),
                format!("unresolved_count={}", required_u64(root, "unresolved_count")?),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_phasing_summary(
    metrics: &Value,
    metrics_path: &Path,
    stage_result: Option<&Value>,
    stage_result_path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id = super::fields::required_stage(metrics, bijux_dna_core::id_catalog::VCF_PHASING)?;
    let input_genotypes = required_u64(metrics, "input_genotypes")?;
    let phased_genotypes = required_u64(metrics, "phased_genotypes")?;
    let contract_ok = if let Some(stage_result) = stage_result {
        stage_result.get("runtime").and_then(|value| value.get("status")).and_then(Value::as_str)
            == Some("succeeded")
    } else {
        required_i64(metrics, "exit_code")? == 0
    };
    let runtime_seconds = stage_result
        .and_then(|value| value.get("runtime"))
        .and_then(|value| value.get("elapsed_seconds"))
        .and_then(Value::as_f64);
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(metrics, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(phased_genotypes, input_genotypes.max(1))?),
            truth_correctness_basis: Some("phased_genotype_fraction".to_string()),
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "phasing smoke preserved governed phased VCF, QC, and manifest outputs".to_string(),
            ),
            phasing_imputation_metric_value: Some(fraction_u64(
                phased_genotypes,
                input_genotypes.max(1),
            )?),
            phasing_imputation_metric_basis: Some("phased_genotype_fraction".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("phase_set_count={}", required_u64(metrics, "phase_set_count")?),
                format!("sample_count={}", required_u64(metrics, "sample_count")?),
                format!("unphased_genotypes={}", required_u64(metrics, "unphased_genotypes")?),
            ])),
            runtime_seconds,
            runtime_source: runtime_seconds.map(|_| "stage_result_manifest".to_string()),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        metrics_path,
    );
    if stage_result.is_some() {
        insert_evidence(
            rows,
            stage_id,
            super::fields::required_identifier(metrics, "tool_id")?,
            VcfEvidenceAggregate {
                source_paths: BTreeSet::new(),
                ..VcfEvidenceAggregate::default()
            },
            stage_result_path,
        );
    }
    Ok(())
}

pub(crate) fn merge_prepare_reference_panel_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id = super::fields::required_stage(
        root,
        bijux_dna_core::id_catalog::VCF_PREPARE_REFERENCE_PANEL,
    )?;
    let contract_ok = required_i64(root, "exit_code")? == 0
        && required_str(root, "normalization_status")? == "sorted_indexed_deduplicated";
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "reference-panel preparation smoke preserved governed normalized panel outputs"
                    .to_string(),
            ),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("panel_id={}", required_str(root, "panel_id")?),
                format!("map_id={}", required_str(root, "map_id")?),
                format!("input_variants={}", required_u64(root, "input_variants")?),
                format!("output_variants={}", required_u64(root, "output_variants")?),
                format!(
                    "duplicate_sites_removed={}",
                    required_u64(root, "duplicate_sites_removed")?
                ),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}
