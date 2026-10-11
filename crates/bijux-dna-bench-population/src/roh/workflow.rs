//! Owner: bijux-dna-bench-population
//! Sequence admitted inputs, scientific execution, normalized evidence and report publication.
use crate::execution::stage_result::path_relative_to_repo;
use crate::inputs::PopulationBenchmarkInputs;
use crate::roh::admission::binding::resolve_governed_vcf_roh_smoke_contract;
use crate::roh::execution::timing::timestamp_marker;
use crate::roh::model::LocalVcfRohSmokeReport;
use crate::roh::model::DEFAULT_OUTPUT_JSON_NAME;
use crate::roh::model::DEFAULT_OUTPUT_TSV_NAME;
use crate::roh::model::DEFAULT_STAGE_RESULT_NAME;
use crate::roh::model::LOCAL_VCF_ROH_SMOKE_COMMAND;
use crate::roh::model::LOCAL_VCF_ROH_SMOKE_SCHEMA_VERSION;
use crate::roh::publication::build_roh_tsv;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::run_roh_stage;
use bijux_dna_stages_vcf::pipeline::RohStageParams;
use std::path::Path;
use std::time::Instant;

/// Execute the admitted population benchmark and return its persisted report.
///
/// # Errors
/// Rejects incompatible inputs and propagates scientific execution or publication failure.
pub fn run_local_vcf_roh_smoke(
    inputs: &PopulationBenchmarkInputs,
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfRohSmokeReport> {
    let contract = resolve_governed_vcf_roh_smoke_contract(inputs, tool_id)?;
    let prepared = super::execution::inputs::prepare_inputs(repo_root, &contract)?;
    let super::execution::inputs::PreparedRohInputs {
        fixture_manifest_path,
        output_root,
        stage_root,
        input_vcf_path,
        sample_metadata_path,
        ..
    } = &prepared;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_roh_stage(
        input_vcf_path,
        stage_root,
        &RohStageParams {
            toolchain: contract.tool.clone(),
            min_snp_density_per_mb: 0.00001,
            max_missingness: 1.0,
            min_segment_kb: 0,
            max_gap_bp: 10_000_000,
            ..RohStageParams::default()
        },
    )
    .with_context(|| format!("run governed VCF ROH smoke from {}", input_vcf_path.display()))?;

    let super::observations::stage::ObservedRohStage {
        source_roh_segments_path,
        source_roh_per_sample_path,
        source_roh_report_path,
        source_metrics_path,
        source_roh_summary_path,
        source_roh_metrics_path,
        source_logs_path,
        segments,
        per_sample_summary,
        execution_mode,
        tool_ok,
        segment_count,
        sample_count,
        total_length,
    } = super::observations::stage::observe_stage(&prepared, &stage_outputs)?;

    let roh_tsv_path = output_root.join(DEFAULT_OUTPUT_TSV_NAME);
    bijux_dna_infra::atomic_write_bytes(&roh_tsv_path, build_roh_tsv(&segments).as_bytes())?;
    let roh_json_path = output_root.join(DEFAULT_OUTPUT_JSON_NAME);
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let report = LocalVcfRohSmokeReport {
        schema_version: LOCAL_VCF_ROH_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_ROH_SMOKE_COMMAND} --tool-id {}", contract.tool),
        stage_id: contract.stage.clone(),
        tool_id: contract.tool.clone(),
        corpus_id: contract.corpus.clone(),
        input_fixture_id: contract.input_fixture.clone(),
        fixture_manifest_path: path_relative_to_repo(repo_root, fixture_manifest_path),
        input_vcf_path: path_relative_to_repo(repo_root, input_vcf_path),
        sample_metadata_path: path_relative_to_repo(repo_root, sample_metadata_path),
        output_root: path_relative_to_repo(repo_root, output_root),
        roh_tsv_path: path_relative_to_repo(repo_root, &roh_tsv_path),
        roh_json_path: path_relative_to_repo(repo_root, &roh_json_path),
        source_roh_segments_path: path_relative_to_repo(repo_root, &source_roh_segments_path),
        source_roh_per_sample_path: path_relative_to_repo(repo_root, &source_roh_per_sample_path),
        source_roh_report_path: path_relative_to_repo(repo_root, &source_roh_report_path),
        source_metrics_path: path_relative_to_repo(repo_root, &source_metrics_path),
        source_roh_summary_path: path_relative_to_repo(repo_root, &source_roh_summary_path),
        source_roh_metrics_path: path_relative_to_repo(repo_root, &source_roh_metrics_path),
        source_logs_path: path_relative_to_repo(repo_root, &source_logs_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at: started_at.clone(),
        finished_at: finished_at.clone(),
        elapsed_seconds,
        exit_code: 0,
        execution_mode,
        tool_ok,
        sample_count,
        segment_count,
        total_length,
        segments,
        per_sample_summary,
        status: "complete".to_string(),
    };
    bijux_dna_infra::atomic_write_json(&roh_json_path, &report)?;

    let stage_result_manifest = super::execution::stage_result::stage_result(repo_root, &report)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;

    Ok(report)
}
