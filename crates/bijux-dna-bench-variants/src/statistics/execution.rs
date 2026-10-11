use super::{
    fixtures::write_governed_stats_input_vcf,
    metrics::{build_metrics, read_stats_json, validate_stats_against_truth},
    model::{
        GovernedVcfStatsSmokeContract, LocalVcfStatsSmokeMetrics, VcfStatsTruthCounts,
        DEFAULT_INPUT_VCF_NAME, DEFAULT_OUTPUT_BCFTOOLS_STATS_NAME, DEFAULT_OUTPUT_STATS_NAME,
    },
    truth::summarize_variant_truth,
};
use anyhow::{Context, Result};
use bijux_dna_domain_vcf::params::VcfStatsParams;
use bijux_dna_stages_vcf::pipeline::run_stats_stage_real;
use std::{
    fs,
    path::{Path, PathBuf},
};
pub(super) struct PreparedInput {
    pub(super) input_vcf: PathBuf,
    pub(super) stage_root: PathBuf,
    pub(super) truth: VcfStatsTruthCounts,
}
pub(super) struct ObservedStats {
    pub(super) stats_json_path: PathBuf,
    pub(super) bcftools_stats_path: PathBuf,
    pub(super) metrics: LocalVcfStatsSmokeMetrics,
}
pub(super) fn prepare(output_root: &Path) -> Result<PreparedInput> {
    let artifacts_root = output_root.join("artifacts");
    let input_root = artifacts_root.join("input");
    let stage_root = artifacts_root.join("stage");
    bijux_dna_infra::ensure_dir(&input_root)
        .with_context(|| format!("create {}", input_root.display()))?;
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let input_vcf = input_root.join(DEFAULT_INPUT_VCF_NAME);
    write_governed_stats_input_vcf(&input_vcf)?;
    let truth = summarize_variant_truth(&input_vcf)?;
    Ok(PreparedInput { input_vcf, stage_root, truth })
}
pub(super) fn execute(
    contract: &GovernedVcfStatsSmokeContract,
    prepared: &PreparedInput,
    output_root: &Path,
) -> Result<ObservedStats> {
    let stage_outputs = run_stats_stage_real(
        &prepared.input_vcf,
        &prepared.stage_root,
        &VcfStatsParams { sample_name: contract.sample_name.clone(), ..VcfStatsParams::default() },
    )
    .with_context(|| {
        format!("run governed VCF stats smoke from {}", prepared.input_vcf.display())
    })?;
    let stats_json_path = output_root.join(DEFAULT_OUTPUT_STATS_NAME);
    fs::copy(&stage_outputs.stats_json, &stats_json_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.stats_json.display(), stats_json_path.display())
    })?;
    let bcftools_stats_path = output_root.join(DEFAULT_OUTPUT_BCFTOOLS_STATS_NAME);
    fs::copy(&stage_outputs.bcftools_stats_txt, &bcftools_stats_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.bcftools_stats_txt.display(),
            bcftools_stats_path.display()
        )
    })?;
    let stats = read_stats_json(&stats_json_path)?;
    validate_stats_against_truth(&stats, &prepared.truth, &contract.sample_name)?;
    let metrics = build_metrics(contract, &prepared.truth, &stats)?;
    Ok(ObservedStats { stats_json_path, bcftools_stats_path, metrics })
}
