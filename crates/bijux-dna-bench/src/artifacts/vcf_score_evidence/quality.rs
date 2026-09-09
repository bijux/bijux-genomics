//! Owner: bijux-dna-bench
//! Interpret VCF quality reports without filesystem access.

use super::fields::{
    checked_count_sum, format_f64, fraction_u64, required_array, required_bool, required_f64,
    required_i64, required_u64, summarize_metrics,
};
use super::merge::insert_evidence;
use super::{EvidenceRows, VcfEvidenceAggregate};
use anyhow::Result;
use bijux_dna_core::prelude::numeric::checked_f64_from_u64;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn merge_damage_filter_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id =
        super::fields::required_stage(root, bijux_dna_core::id_catalog::VCF_DAMAGE_FILTER)?;
    let retained_variants = required_u64(root, "retained_variants")?;
    let input_variants = required_u64(root, "input_variants")?;
    let removed_variants = required_u64(root, "removed_variants")?;
    let terminal_damage_filtered_variants =
        required_u64(root, "terminal_damage_filtered_variants")?;
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(retained_variants, input_variants.max(1))?),
            truth_correctness_basis: Some("retained_variant_fraction".to_string()),
            contract_correctness_score: Some(if required_i64(root, "exit_code")? == 0 {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "damage filter smoke preserved governed damage-aware variant filtering outputs"
                    .to_string(),
            ),
            genotype_truth_metric_value: Some(checked_f64_from_u64(
                retained_variants,
                "retained variants",
            )?),
            genotype_truth_metric_basis: Some("retained_variants".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("removed_variants={removed_variants}"),
                format!("retained_variants={retained_variants}"),
                format!("terminal_damage_filtered_variants={terminal_damage_filtered_variants}"),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_filter_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id = super::fields::required_stage(root, bijux_dna_core::id_catalog::VCF_FILTER)?;
    let input_variants = required_u64(root, "input_variants")?;
    let pass_variants = required_u64(root, "pass_variants")?;
    let failed_variants = required_u64(root, "failed_variants")?;
    let observed_variants = checked_count_sum(pass_variants, failed_variants, "filtered variants")?;
    let contract_ok = required_i64(root, "exit_code")? == 0 && input_variants == observed_variants;
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "VCF filter smoke preserved governed pass and failed variant counts".to_string(),
            ),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("pass_variants={pass_variants}"),
                format!("failed_variants={failed_variants}"),
                format!(
                    "quality_threshold={}",
                    format_f64(required_f64(root, "quality_threshold")?)
                ),
                format!("depth_threshold={}", format_f64(required_f64(root, "depth_threshold")?)),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_postprocess_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id =
        super::fields::required_stage(root, bijux_dna_core::id_catalog::VCF_POSTPROCESS)?;
    let contract_ok = required_i64(root, "exit_code")? == 0
        && required_bool(root, "readable_vcf")?
        && required_bool(root, "tabix_present")?
        && required_bool(root, "contigs_consistent_with_species_context")?;
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "postprocess smoke preserved governed readability, contig, and indexing outputs"
                    .to_string(),
            ),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("record_count={}", required_u64(root, "record_count")?),
                format!(
                    "multiallelic_records_split={}",
                    required_u64(root, "multiallelic_records_split")?
                ),
                format!("indels_normalized={}", required_u64(root, "indels_normalized")?),
                format!("variant_ids_normalized={}", required_u64(root, "variant_ids_normalized")?),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_qc_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
    tool_id: &str,
) -> Result<()> {
    super::fields::required_report_schema(root, "bijux.vcf.qc.v1")?;
    let missingness_post = required_f64(root, "missingness_post")?;
    let imputation_info_mean = required_f64(root, "imputation_info_mean")?;
    let rsq_mean = required_f64(root, "rsq_mean")?;
    let truth_score = ((1.0 - missingness_post).clamp(0.0, 1.0)
        + imputation_info_mean.clamp(0.0, 1.0)
        + rsq_mean.clamp(0.0, 1.0))
        / 3.0;
    insert_evidence(
        rows,
        bijux_dna_core::id_catalog::VCF_QC,
        tool_id,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(truth_score),
            truth_correctness_basis: Some("mean_missingness_info_rsq_score".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "QC smoke summary preserved governed missingness and imputation-quality outputs"
                    .to_string(),
            ),
            missingness_metric_value: Some((1.0 - missingness_post).clamp(0.0, 1.0)),
            missingness_metric_basis: Some("one_minus_missingness_post".to_string()),
            phasing_imputation_metric_value: Some(rsq_mean.clamp(0.0, 1.0)),
            phasing_imputation_metric_basis: Some("rsq_mean".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("missingness_pre={}", format_f64(required_f64(root, "missingness_pre")?)),
                format!("missingness_post={}", format_f64(missingness_post)),
                format!("imputation_info_mean={}", format_f64(imputation_info_mean)),
                format!("rsq_mean={}", format_f64(rsq_mean)),
                format!("excluded_samples={}", required_array(root, "excluded_samples")?.len()),
                format!("excluded_variants={}", required_array(root, "excluded_variants")?.len()),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_stats_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    super::fields::required_report_schema(root, "bijux.vcf.stats.v1")?;
    let stage_id = bijux_dna_core::id_catalog::VCF_STATS;
    let variants_total = required_u64(root, "variants_total")?;
    let snps = required_u64(root, "snps")?;
    let indels = required_u64(root, "indels")?;
    let missingness_post = required_f64(root, "missingness_post")?;
    let classified_variants = checked_count_sum(snps, indels, "classified variants")?;
    let contract_ok =
        required_f64(root, "annotation_coverage")? == 1.0 && variants_total == classified_variants;
    insert_evidence(
        rows,
        stage_id,
        "bcftools",
        VcfEvidenceAggregate {
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "stats smoke preserved governed variant totals and annotation coverage".to_string(),
            ),
            missingness_metric_value: Some((1.0 - missingness_post).clamp(0.0, 1.0)),
            missingness_metric_basis: Some("one_minus_missingness_post".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("variants_total={variants_total}"),
                format!("snps={snps}"),
                format!("indels={indels}"),
                format!("ti_tv={}", format_f64(required_f64(root, "ti_tv")?)),
                format!(
                    "heterozygosity_ratio={}",
                    format_f64(required_f64(root, "heterozygosity_ratio")?)
                ),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}
