//! Owner: bijux-dna-bench-population
//! Demography fixtures `failure_probe`.
use crate::demography::execution::artifacts::ensure_file_exists;
use crate::demography::model::{
    LocalVcfDemographyInsufficientProbe, DEFAULT_PROBE_INPUT_IBD_NAME,
    DEFAULT_PROBE_SOURCE_DEMOGRAPHY_CONTRACT_NAME, DEFAULT_PROBE_SOURCE_DEMOGRAPHY_METRICS_NAME,
    DEFAULT_PROBE_SOURCE_LOGS_NAME, DEFAULT_PROBE_SOURCE_NE_TRAJECTORY_NAME,
    GOVERNED_VCF_DEMOGRAPHY_MIN_SEGMENTS,
};
use crate::demography::observations::evidence::load_demography_evidence;
use crate::execution::stage_result::path_relative_to_repo;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::run_demography_stage;
use bijux_dna_stages_vcf::pipeline::DemographyStageParams;
use std::path::Path;

pub(crate) fn run_insufficient_data_probe(
    repo_root: &Path,
    probe_root: &Path,
    upstream_probe_filtered_segments_path: &str,
) -> Result<LocalVcfDemographyInsufficientProbe> {
    let upstream_probe_filtered_segments_source =
        repo_root.join(upstream_probe_filtered_segments_path);
    ensure_file_exists(
        &upstream_probe_filtered_segments_source,
        "upstream sparse-overlap IBD filtered segments",
    )?;
    let input_ibd_path = probe_root.join(DEFAULT_PROBE_INPUT_IBD_NAME);
    bijux_dna_infra::copy_file(&upstream_probe_filtered_segments_source, &input_ibd_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                upstream_probe_filtered_segments_source.display(),
                input_ibd_path.display()
            )
        })?;
    let stage_root = probe_root.join("stage");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let stage_outputs = run_demography_stage(
        &input_ibd_path,
        &stage_root,
        &DemographyStageParams {
            min_segments: GOVERNED_VCF_DEMOGRAPHY_MIN_SEGMENTS,
            ..DemographyStageParams::default()
        },
    )
    .with_context(|| {
        format!("run sparse-overlap demography probe from {}", input_ibd_path.display())
    })?;

    let source_ne_trajectory_path = probe_root.join(DEFAULT_PROBE_SOURCE_NE_TRAJECTORY_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.ne_trajectory_tsv, &source_ne_trajectory_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.ne_trajectory_tsv.display(),
                source_ne_trajectory_path.display()
            )
        })?;
    let source_demography_contract_path =
        probe_root.join(DEFAULT_PROBE_SOURCE_DEMOGRAPHY_CONTRACT_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.demography_json, &source_demography_contract_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.demography_json.display(),
                source_demography_contract_path.display()
            )
        })?;
    let source_demography_metrics_path =
        probe_root.join(DEFAULT_PROBE_SOURCE_DEMOGRAPHY_METRICS_NAME);
    bijux_dna_infra::copy_file(
        &stage_outputs.demography_metrics_json,
        &source_demography_metrics_path,
    )
    .with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.demography_metrics_json.display(),
            source_demography_metrics_path.display()
        )
    })?;
    let source_logs_path = probe_root.join(DEFAULT_PROBE_SOURCE_LOGS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.logs_txt, &source_logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), source_logs_path.display())
    })?;

    let evidence = load_demography_evidence(
        &source_demography_contract_path,
        &source_demography_metrics_path,
    )?;
    if evidence.status != "insufficient_data" {
        bail!(
            "sparse-overlap demography probe expected `insufficient_data`, found `{}`",
            evidence.status
        );
    }
    let insufficient_reason = evidence.insufficient_reason.clone().ok_or_else(|| {
        anyhow!("sparse-overlap demography probe must report `insufficient_data_reason`")
    })?;

    Ok(LocalVcfDemographyInsufficientProbe {
        input_ibd: path_relative_to_repo(repo_root, &input_ibd_path),
        method: evidence.method,
        status: evidence.status,
        insufficient_reason,
        time_bins: evidence.time_bins,
        ne_estimates: evidence.ne_estimates,
        source_ne_trajectory_path: path_relative_to_repo(repo_root, &source_ne_trajectory_path),
        source_demography_contract_path: path_relative_to_repo(
            repo_root,
            &source_demography_contract_path,
        ),
        source_demography_metrics_path: path_relative_to_repo(
            repo_root,
            &source_demography_metrics_path,
        ),
        source_logs_path: path_relative_to_repo(repo_root, &source_logs_path),
    })
}
