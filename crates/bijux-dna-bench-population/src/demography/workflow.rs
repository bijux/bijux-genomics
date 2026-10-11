//! Owner: bijux-dna-bench-population
//! Sequence admitted inputs, scientific execution, normalized evidence and report publication.
use crate::demography::admission::binding::resolve_governed_vcf_demography_smoke_contract;
use crate::demography::execution::timing::timestamp_marker;
use crate::demography::model::LocalVcfDemographySmokeReport;
use crate::demography::model::DEFAULT_OUTPUT_JSON_NAME;
use crate::demography::model::DEFAULT_STAGE_RESULT_NAME;
use crate::demography::model::GOVERNED_VCF_DEMOGRAPHY_MIN_SEGMENTS;
use crate::demography::model::LOCAL_VCF_DEMOGRAPHY_SMOKE_COMMAND;
use crate::demography::model::LOCAL_VCF_DEMOGRAPHY_SMOKE_SCHEMA_VERSION;
use crate::execution::stage_result::path_relative_to_repo;
use crate::ibd::benchmark_output_lock;
use crate::inputs::PopulationBenchmarkInputs;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::run_demography_stage;
use bijux_dna_stages_vcf::pipeline::DemographyStageParams;
use std::path::Path;
use std::time::Instant;

/// Execute the admitted population benchmark and return its persisted report.
///
/// # Errors
/// Rejects incompatible inputs and propagates scientific execution or publication failure.
pub fn run_local_vcf_demography_smoke(
    inputs: &PopulationBenchmarkInputs,
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfDemographySmokeReport> {
    let _lock = benchmark_output_lock(repo_root)?;
    run_local_vcf_demography_smoke_unlocked(inputs, repo_root, tool_id)
}

pub(crate) fn run_local_vcf_demography_smoke_unlocked(
    inputs: &PopulationBenchmarkInputs,
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfDemographySmokeReport> {
    let contract = resolve_governed_vcf_demography_smoke_contract(inputs, tool_id)?;
    let prepared = super::execution::inputs::prepare_inputs(inputs, repo_root, &contract)?;
    let super::execution::inputs::PreparedDemographyInputs {
        output_root,
        stage_root,
        input_ibd_path,
        ..
    } = &prepared;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_demography_stage(
        input_ibd_path,
        stage_root,
        &DemographyStageParams {
            min_segments: GOVERNED_VCF_DEMOGRAPHY_MIN_SEGMENTS,
            ..DemographyStageParams::default()
        },
    )
    .with_context(|| {
        format!("run governed VCF demography smoke from {}", input_ibd_path.display())
    })?;

    let super::observations::stage::ObservedDemographyStage {
        source_upstream_ibd_report_path,
        source_upstream_filtered_segments_path,
        source_ne_trajectory_path,
        source_demography_contract_path,
        source_demography_metrics_path,
        source_logs_path,
        evidence,
        insufficient_data_probe,
    } = super::observations::stage::observe_stage(repo_root, &prepared, &stage_outputs)?;

    let demography_json_path = output_root.join(DEFAULT_OUTPUT_JSON_NAME);
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let report = LocalVcfDemographySmokeReport {
        schema_version: LOCAL_VCF_DEMOGRAPHY_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_DEMOGRAPHY_SMOKE_COMMAND} --tool-id {}", contract.tool),
        stage_id: contract.stage.clone(),
        tool_id: contract.tool.clone(),
        corpus_id: contract.corpus.clone(),
        input_fixture_id: contract.input_fixture.clone(),
        output_root: path_relative_to_repo(repo_root, output_root),
        demography_json_path: path_relative_to_repo(repo_root, &demography_json_path),
        input_ibd: path_relative_to_repo(repo_root, input_ibd_path),
        source_upstream_ibd_report_path: path_relative_to_repo(
            repo_root,
            &source_upstream_ibd_report_path,
        ),
        source_upstream_filtered_segments_path: path_relative_to_repo(
            repo_root,
            &source_upstream_filtered_segments_path,
        ),
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
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at,
        finished_at,
        elapsed_seconds,
        exit_code: 0,
        method: evidence.method,
        inference_status: evidence.inference_status,
        tool_ok: evidence.tool_ok,
        time_bins: evidence.time_bins,
        ne_estimates: evidence.ne_estimates,
        status: evidence.status,
        insufficient_reason: evidence.insufficient_reason,
        insufficient_data_probe,
    };
    bijux_dna_infra::atomic_write_json(&demography_json_path, &report)?;

    let stage_result_manifest = super::execution::stage_result::stage_result(repo_root, &report)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;

    Ok(report)
}
