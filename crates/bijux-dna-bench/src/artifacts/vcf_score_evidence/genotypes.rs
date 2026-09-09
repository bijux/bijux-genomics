//! Owner: bijux-dna-bench
//! Interpret VCF genotypes reports without filesystem access.

use super::fields::{
    format_u64_as_string, fraction_u64, required_array, required_bool, required_i64, required_str,
    required_u64, summarize_metrics, unique_strings,
};
use super::merge::insert_evidence;
use super::{EvidenceRows, VcfEvidenceAggregate};
use anyhow::Result;
use bijux_dna_core::prelude::numeric::checked_f64_from_u64;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

/// The two governed call report count contracts have distinct stage and field identities.
#[derive(Debug, Clone, Copy)]
pub(crate) enum CallCountContract {
    Variants,
    DiploidGenotypes,
}

impl CallCountContract {
    fn fields(self) -> (&'static str, &'static str) {
        match self {
            Self::Variants => (bijux_dna_core::id_catalog::VCF_CALL, "variant_count"),
            Self::DiploidGenotypes => {
                (bijux_dna_core::id_catalog::VCF_CALL_DIPLOID, "called_genotypes")
            }
        }
    }
}

pub(crate) fn merge_call_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
    contract: CallCountContract,
) -> Result<()> {
    let (expected_stage, genotype_key) = contract.fields();
    let genotype_basis = genotype_key;
    let stage_id = super::fields::required_stage(root, expected_stage)?;
    let genotype_value = required_u64(root, genotype_key)?;
    let exit_code = required_i64(root, "exit_code")?;
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            contract_correctness_score: Some(if exit_code == 0 && genotype_value > 0 {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "VCF call smoke preserved governed output payloads and call counts".to_string(),
            ),
            genotype_truth_metric_value: Some(checked_f64_from_u64(genotype_value, genotype_key)?),
            genotype_truth_metric_basis: Some(genotype_basis.to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("sample_count={}", required_u64(root, "sample_count")?),
                format!("{genotype_key}={}", format_u64_as_string(genotype_value)),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_call_gl_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id = super::fields::required_stage(root, bijux_dna_core::id_catalog::VCF_CALL_GL)?;
    let sites_with_likelihoods = required_u64(root, "sites_with_likelihoods")?;
    let missing_likelihoods = required_u64(root, "missing_likelihoods")?;
    let total_sites = sites_with_likelihoods + missing_likelihoods;
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(
                sites_with_likelihoods,
                total_sites.max(1),
            )?),
            truth_correctness_basis: Some("likelihood_site_fraction".to_string()),
            contract_correctness_score: Some(if required_i64(root, "exit_code")? == 0 {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "genotype-likelihood smoke preserved governed likelihood fields".to_string(),
            ),
            genotype_truth_metric_value: Some(checked_f64_from_u64(
                sites_with_likelihoods,
                "sites with likelihoods",
            )?),
            genotype_truth_metric_basis: Some("sites_with_likelihoods".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("sample_count={}", required_u64(root, "sample_count")?),
                format!(
                    "samples_with_likelihoods={}",
                    required_u64(root, "samples_with_likelihoods")?
                ),
                format!("missing_likelihoods={missing_likelihoods}"),
                format!("likelihood_field={}", required_str(root, "likelihood_field")?),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_call_pseudohaploid_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id =
        super::fields::required_stage(root, bijux_dna_core::id_catalog::VCF_CALL_PSEUDOHAPLOID)?;
    let called_sites = required_u64(root, "called_sites")?;
    let target_sites = required_u64(root, "target_sites")?;
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(called_sites, target_sites.max(1))?),
            truth_correctness_basis: Some("called_site_fraction".to_string()),
            contract_correctness_score: Some(
                if required_i64(root, "exit_code")? == 0
                    && required_bool(root, "deterministic_replay_match")?
                {
                    1.0
                } else {
                    0.0
                },
            ),
            contract_correctness_basis: Some(
                "pseudohaploid smoke preserved governed deterministic replay and site counts"
                    .to_string(),
            ),
            genotype_truth_metric_value: Some(checked_f64_from_u64(called_sites, "called sites")?),
            genotype_truth_metric_basis: Some("called_sites".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("covered_sites={}", required_u64(root, "covered_sites")?),
                format!("missing_sites={}", required_u64(root, "missing_sites")?),
                format!("sample_count={}", required_u64(root, "sample_count")?),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_gl_propagation_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id =
        super::fields::required_stage(root, bijux_dna_core::id_catalog::VCF_GL_PROPAGATION)?;
    let sample_count = required_u64(root, "sample_count")?;
    let site_count_before = required_u64(root, "site_count_before")?;
    let site_count_after = required_u64(root, "site_count_after")?;
    let lost_fields = required_array(root, "lost_fields")?;
    insert_evidence(
        rows,
        stage_id,
        super::fields::required_identifier(root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(
                site_count_after,
                site_count_before.max(1),
            )?),
            truth_correctness_basis: Some("retained_site_fraction".to_string()),
            contract_correctness_score: Some(
                if required_i64(root, "exit_code")? == 0 && lost_fields.is_empty() {
                    1.0
                } else {
                    0.0
                },
            ),
            contract_correctness_basis: Some(
                "genotype-likelihood propagation preserved governed sample and site surfaces"
                    .to_string(),
            ),
            genotype_truth_metric_value: Some(checked_f64_from_u64(
                site_count_after,
                "site count after",
            )?),
            genotype_truth_metric_basis: Some("site_count_after".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("sample_count={sample_count}"),
                format!("site_count_before={site_count_before}"),
                format!("site_count_after={site_count_after}"),
                format!(
                    "output_likelihood_fields={}",
                    unique_strings(
                        required_array(root, "output_likelihood_fields")?
                            .iter()
                            .filter_map(Value::as_str)
                            .map(str::to_string),
                    )
                    .join(",")
                ),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}
