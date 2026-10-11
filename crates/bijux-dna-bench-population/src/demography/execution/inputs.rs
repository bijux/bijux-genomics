//! Owner: bijux-dna-bench-population
//! Admit upstream cohort inputs and prepare explicitly owned stage paths.
use crate::demography::execution::artifacts::ensure_file_exists;
use crate::demography::model::DEFAULT_INPUT_IBD_NAME;
use crate::demography::model::DEFAULT_VCF_DEMOGRAPHY_SMOKE_ROOT;
use crate::demography::model::GOVERNED_VCF_DEMOGRAPHY_UPSTREAM_TOOL_ID;
use crate::ibd::run_local_vcf_ibd_smoke_unlocked;
use crate::inputs::PopulationBenchmarkInputs;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;

pub(crate) struct PreparedDemographyInputs {
    pub(crate) upstream_ibd_report: bijux_dna_bench_model::BenchmarkLocalVcfIbdSmokeReport,
    pub(crate) upstream_ibd_report_source: std::path::PathBuf,
    pub(crate) output_root: std::path::PathBuf,
    pub(crate) stage_root: std::path::PathBuf,
    pub(crate) probe_root: std::path::PathBuf,
    pub(crate) input_ibd_path: std::path::PathBuf,
}

pub(crate) fn prepare_inputs(
    inputs: &PopulationBenchmarkInputs,
    repo_root: &Path,
    contract: &crate::demography::model::GovernedVcfDemographySmokeContract,
) -> Result<PreparedDemographyInputs> {
    let upstream_ibd_report = run_local_vcf_ibd_smoke_unlocked(
        inputs,
        repo_root,
        GOVERNED_VCF_DEMOGRAPHY_UPSTREAM_TOOL_ID,
    )?;
    let upstream_ibd_report_source = repo_root.join(&upstream_ibd_report.ibd_json_path);
    let upstream_filtered_segments_source =
        repo_root.join(&upstream_ibd_report.source_ibd_filtered_segments_path);
    ensure_file_exists(&upstream_ibd_report_source, "upstream VCF IBD smoke report")?;
    ensure_file_exists(&upstream_filtered_segments_source, "upstream VCF IBD filtered segments")?;

    let output_root = repo_root.join(DEFAULT_VCF_DEMOGRAPHY_SMOKE_ROOT).join(&contract.tool);
    bijux_dna_infra::ensure_dir(&output_root)
        .with_context(|| format!("create {}", output_root.display()))?;
    let artifacts_root = output_root.join("artifacts");
    let input_root = artifacts_root.join("input");
    let stage_root = artifacts_root.join("stage");
    let probe_root = artifacts_root.join("probe");
    bijux_dna_infra::ensure_dir(&input_root)
        .with_context(|| format!("create {}", input_root.display()))?;
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    bijux_dna_infra::ensure_dir(&probe_root)
        .with_context(|| format!("create {}", probe_root.display()))?;

    let input_ibd_path = input_root.join(DEFAULT_INPUT_IBD_NAME);
    bijux_dna_infra::copy_file(&upstream_filtered_segments_source, &input_ibd_path).with_context(
        || {
            format!(
                "copy {} to {}",
                upstream_filtered_segments_source.display(),
                input_ibd_path.display()
            )
        },
    )?;

    Ok(PreparedDemographyInputs {
        upstream_ibd_report,
        upstream_ibd_report_source,
        output_root,
        stage_root,
        probe_root,
        input_ibd_path,
    })
}
