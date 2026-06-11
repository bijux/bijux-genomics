use std::collections::BTreeSet;

use anyhow::{anyhow, Result};
use bijux_dna_domain_vcf::VcfDomainStage;
use bijux_dna_stages_vcf::stage_specs::{
    vcf_domain_stage_adapter_id, vcf_domain_stage_expected_output_ids, vcf_domain_stage_parser_id,
};

use super::local_vcf_stage_matrix::{build_vcf_stage_matrix_rows, VcfStageMatrixRow};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExplicitVcfBenchmarkBinding {
    stage: VcfDomainStage,
    tool_id: &'static str,
    corpus_id: &'static str,
    asset_profile_id: &'static str,
}

const EXPLICIT_VCF_BENCHMARK_BINDINGS: &[ExplicitVcfBenchmarkBinding] = &[
    ExplicitVcfBenchmarkBinding {
        stage: VcfDomainStage::Qc,
        tool_id: "bcftools",
        corpus_id: "vcf_production_regression",
        asset_profile_id: "vcf_cohort",
    },
    ExplicitVcfBenchmarkBinding {
        stage: VcfDomainStage::Qc,
        tool_id: "plink",
        corpus_id: "vcf_production_regression",
        asset_profile_id: "vcf_cohort",
    },
    ExplicitVcfBenchmarkBinding {
        stage: VcfDomainStage::Pca,
        tool_id: "eigensoft",
        corpus_id: "vcf_production_regression",
        asset_profile_id: "vcf_cohort",
    },
];

pub(crate) fn collect_vcf_benchmark_binding_rows() -> Result<Vec<VcfStageMatrixRow>> {
    let mut rows = build_vcf_stage_matrix_rows()?;
    rows.extend(explicit_vcf_benchmark_binding_rows()?);
    rows.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.corpus_id.cmp(&right.corpus_id))
            .then_with(|| left.asset_profile_id.cmp(&right.asset_profile_id))
    });

    let binding_keys = rows
        .iter()
        .map(|row| {
            (
                row.stage_id.clone(),
                row.tool_id.clone(),
                row.corpus_id.clone(),
                row.asset_profile_id.clone(),
            )
        })
        .collect::<BTreeSet<_>>();
    if binding_keys.len() != rows.len() {
        return Err(anyhow!(
            "VCF benchmark bindings must keep exactly one row per stage/tool/corpus/asset binding"
        ));
    }
    Ok(rows)
}

pub(crate) fn collect_vcf_qc_benchmark_binding_rows() -> Result<Vec<VcfStageMatrixRow>> {
    Ok(collect_vcf_benchmark_binding_rows()?
        .into_iter()
        .filter(|row| row.stage_id == VcfDomainStage::Qc.as_str())
        .collect())
}

fn explicit_vcf_benchmark_binding_rows() -> Result<Vec<VcfStageMatrixRow>> {
    EXPLICIT_VCF_BENCHMARK_BINDINGS
        .iter()
        .map(|binding| {
            let adapter_id = vcf_domain_stage_adapter_id(binding.stage).ok_or_else(|| {
                anyhow!(
                    "VCF explicit benchmark binding `{}` / `{}` is missing an adapter contract id",
                    binding.stage.as_str(),
                    binding.tool_id
                )
            })?;
            let parser_id = vcf_domain_stage_parser_id(binding.stage).ok_or_else(|| {
                anyhow!(
                    "VCF explicit benchmark binding `{}` / `{}` is missing a parser contract id",
                    binding.stage.as_str(),
                    binding.tool_id
                )
            })?;
            let expected_outputs =
                vcf_domain_stage_expected_output_ids(binding.stage).ok_or_else(|| {
                    anyhow!(
                        "VCF explicit benchmark binding `{}` / `{}` is missing expected output ids",
                        binding.stage.as_str(),
                        binding.tool_id
                    )
                })?;

            Ok(VcfStageMatrixRow {
                stage_id: binding.stage.as_str().to_string(),
                tool_id: binding.tool_id.to_string(),
                corpus_id: binding.corpus_id.to_string(),
                asset_profile_id: binding.asset_profile_id.to_string(),
                adapter_id: adapter_id.to_string(),
                parser_id: parser_id.to_string(),
                expected_outputs: expected_outputs
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect(),
            })
        })
        .collect()
}
