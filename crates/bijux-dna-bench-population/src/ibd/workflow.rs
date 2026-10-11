//! Owner: bijux-dna-bench-population
//! Sequence admitted inputs, scientific execution, normalized evidence and report publication.
use crate::execution::stage_result::path_relative_to_repo;
use crate::ibd::admission::binding::resolve_governed_vcf_ibd_smoke_contract;
use crate::ibd::execution::exclusion::benchmark_output_lock;
use crate::ibd::execution::timing::timestamp_marker;
use crate::ibd::model::DEFAULT_OUTPUT_JSON_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_TSV_NAME;
use crate::ibd::model::DEFAULT_STAGE_RESULT_NAME;
use crate::ibd::model::GOVERNED_IBD_SEGMENT_MARKER_MINIMUM;
use crate::ibd::model::LOCAL_VCF_IBD_SMOKE_COMMAND;
use crate::ibd::publication::build_ibd_tsv;
use crate::inputs::PopulationBenchmarkInputs;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_model::validate_benchmark_local_vcf_ibd_smoke_report;
use bijux_dna_bench_model::BenchmarkLocalVcfIbdSmokeReport as LocalVcfIbdSmokeReport;
use bijux_dna_bench_model::BENCHMARK_LOCAL_VCF_IBD_SMOKE_SCHEMA_V1;
use bijux_dna_stages_vcf::pipeline::run_ibd_stage;
use bijux_dna_stages_vcf::pipeline::IbdStageParams;
use std::path::Path;
use std::time::Instant;

/// Execute the admitted population benchmark and return its persisted report.
///
/// # Errors
/// Rejects incompatible inputs and propagates scientific execution or publication failure.
pub fn run_local_vcf_ibd_smoke(
    inputs: &PopulationBenchmarkInputs,
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfIbdSmokeReport> {
    let _lock = benchmark_output_lock(repo_root)?;
    run_local_vcf_ibd_smoke_unlocked(inputs, repo_root, tool_id)
}

pub(crate) fn run_local_vcf_ibd_smoke_unlocked(
    inputs: &PopulationBenchmarkInputs,
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfIbdSmokeReport> {
    let contract = resolve_governed_vcf_ibd_smoke_contract(inputs, repo_root, tool_id)?;
    let prepared = super::execution::inputs::prepare_inputs(repo_root, &contract)?;
    let super::execution::inputs::PreparedIbdInputs {
        fixture_manifest_path,
        output_root,
        stage_root,
        input_vcf_path,
        sample_metadata_path,
        ..
    } = &prepared;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = run_ibd_stage(
        input_vcf_path,
        stage_root,
        &IbdStageParams {
            toolchain: contract.tool.clone(),
            min_variant_density_per_mb: 0.00001,
            max_missingness: 1.0,
            min_samples: 2,
            min_segment_cm: 1.0,
            min_markers_per_segment: GOVERNED_IBD_SEGMENT_MARKER_MINIMUM,
            ..IbdStageParams::default()
        },
    )
    .with_context(|| format!("run governed VCF IBD smoke from {}", input_vcf_path.display()))?;

    let super::observations::stage::ObservedIbdStage {
        source_ibd_input_path,
        source_ibd_segments_path,
        source_ibd_merged_segments_path,
        source_ibd_filtered_segments_path,
        source_ibd_summary_path,
        source_ibd_metrics_path,
        source_logs_path,
        rows,
        status,
        execution_mode,
        germline_tool_ok,
        ibdseq_tool_ok,
        ibdhap_tool_ok,
        insufficient_overlap_probe,
    } = super::observations::stage::observe_stage(repo_root, &prepared, &stage_outputs, &contract)?;

    let ibd_tsv_path = output_root.join(DEFAULT_OUTPUT_TSV_NAME);
    bijux_dna_infra::atomic_write_bytes(&ibd_tsv_path, build_ibd_tsv(&rows).as_bytes())?;
    let ibd_json_path = output_root.join(DEFAULT_OUTPUT_JSON_NAME);
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let pair_count = u64::try_from(rows.len()).map_err(|_| anyhow!("pair count overflow"))?;
    let report = LocalVcfIbdSmokeReport {
        schema_version: BENCHMARK_LOCAL_VCF_IBD_SMOKE_SCHEMA_V1.to_string(),
        command: format!("{LOCAL_VCF_IBD_SMOKE_COMMAND} --tool-id {}", contract.tool),
        stage_id: contract.stage.clone(),
        tool_id: contract.tool.clone(),
        corpus_id: contract.corpus.clone(),
        input_fixture_id: contract.input_fixture.clone(),
        fixture_manifest_path: path_relative_to_repo(repo_root, fixture_manifest_path),
        input_vcf_path: path_relative_to_repo(repo_root, input_vcf_path),
        sample_metadata_path: path_relative_to_repo(repo_root, sample_metadata_path),
        output_root: path_relative_to_repo(repo_root, output_root),
        ibd_tsv_path: path_relative_to_repo(repo_root, &ibd_tsv_path),
        ibd_json_path: path_relative_to_repo(repo_root, &ibd_json_path),
        source_ibd_input_path: path_relative_to_repo(repo_root, &source_ibd_input_path),
        source_ibd_segments_path: path_relative_to_repo(repo_root, &source_ibd_segments_path),
        source_ibd_merged_segments_path: path_relative_to_repo(
            repo_root,
            &source_ibd_merged_segments_path,
        ),
        source_ibd_filtered_segments_path: path_relative_to_repo(
            repo_root,
            &source_ibd_filtered_segments_path,
        ),
        source_ibd_summary_path: path_relative_to_repo(repo_root, &source_ibd_summary_path),
        source_ibd_metrics_path: path_relative_to_repo(repo_root, &source_ibd_metrics_path),
        source_logs_path: path_relative_to_repo(repo_root, &source_logs_path),
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        started_at: started_at.clone(),
        finished_at: finished_at.clone(),
        elapsed_seconds,
        exit_code: 0,
        execution_mode,
        germline_tool_ok,
        ibdseq_tool_ok,
        ibdhap_tool_ok,
        pair_count,
        rows,
        insufficient_overlap_probe,
        status,
    };
    validate_benchmark_local_vcf_ibd_smoke_report(&report)?;
    bijux_dna_infra::atomic_write_json(&ibd_json_path, &report)?;

    let stage_result_manifest = super::execution::stage_result::stage_result(repo_root, &report)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;

    Ok(report)
}
