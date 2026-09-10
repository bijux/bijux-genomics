//! Owner: bijux-dna-bench-population
//! Observe normalized stage outputs and preserve failure-probe classifications.
use crate::demography::fixtures::failure_probe::run_insufficient_data_probe;
use crate::demography::model::DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_CONTRACT_NAME;
use crate::demography::model::DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_METRICS_NAME;
use crate::demography::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::demography::model::DEFAULT_OUTPUT_SOURCE_NE_TRAJECTORY_NAME;
use crate::demography::model::DEFAULT_OUTPUT_SOURCE_UPSTREAM_FILTERED_SEGMENTS_NAME;
use crate::demography::model::DEFAULT_OUTPUT_SOURCE_UPSTREAM_IBD_REPORT_NAME;
use crate::demography::observations::evidence::load_demography_evidence;
use crate::demography::observations::evidence::validate_primary_demography_evidence;
use crate::demography::LocalVcfDemographyInsufficientProbe;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;

pub(crate) struct ObservedDemographyStage {
    pub(crate) source_upstream_ibd_report_path: std::path::PathBuf,
    pub(crate) source_upstream_filtered_segments_path: std::path::PathBuf,
    pub(crate) source_ne_trajectory_path: std::path::PathBuf,
    pub(crate) source_demography_contract_path: std::path::PathBuf,
    pub(crate) source_demography_metrics_path: std::path::PathBuf,
    pub(crate) source_logs_path: std::path::PathBuf,
    pub(crate) evidence: crate::demography::model::NormalizedDemographyEvidence,
    pub(crate) insufficient_data_probe: LocalVcfDemographyInsufficientProbe,
}

pub(crate) fn observe_stage(
    repo_root: &Path,
    prepared: &super::super::execution::inputs::PreparedDemographyInputs,
    stage_outputs: &bijux_dna_stages_vcf::pipeline::DemographyStageOutputs,
) -> Result<ObservedDemographyStage> {
    let super::super::execution::inputs::PreparedDemographyInputs {
        upstream_ibd_report,
        upstream_ibd_report_source,
        output_root,
        probe_root,
        input_ibd_path,
        ..
    } = prepared;
    let source_upstream_ibd_report_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_UPSTREAM_IBD_REPORT_NAME);
    bijux_dna_infra::copy_file(upstream_ibd_report_source, &source_upstream_ibd_report_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                upstream_ibd_report_source.display(),
                source_upstream_ibd_report_path.display()
            )
        })?;
    let source_upstream_filtered_segments_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_UPSTREAM_FILTERED_SEGMENTS_NAME);
    bijux_dna_infra::copy_file(input_ibd_path, &source_upstream_filtered_segments_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                input_ibd_path.display(),
                source_upstream_filtered_segments_path.display()
            )
        })?;
    let source_ne_trajectory_path = output_root.join(DEFAULT_OUTPUT_SOURCE_NE_TRAJECTORY_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.ne_trajectory_tsv, &source_ne_trajectory_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.ne_trajectory_tsv.display(),
                source_ne_trajectory_path.display()
            )
        })?;
    let source_demography_contract_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_CONTRACT_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.demography_json, &source_demography_contract_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.demography_json.display(),
                source_demography_contract_path.display()
            )
        })?;
    let source_demography_metrics_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_METRICS_NAME);
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
    let source_logs_path = output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.logs_txt, &source_logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), source_logs_path.display())
    })?;

    let evidence = load_demography_evidence(
        &source_demography_contract_path,
        &source_demography_metrics_path,
    )?;
    validate_primary_demography_evidence(&evidence)?;
    let insufficient_data_probe = run_insufficient_data_probe(
        repo_root,
        probe_root,
        &upstream_ibd_report.insufficient_overlap_probe.source_ibd_filtered_segments_path,
    )?;

    Ok(ObservedDemographyStage {
        source_upstream_ibd_report_path,
        source_upstream_filtered_segments_path,
        source_ne_trajectory_path,
        source_demography_contract_path,
        source_demography_metrics_path,
        source_logs_path,
        evidence,
        insufficient_data_probe,
    })
}
