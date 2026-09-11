//! Compose admitted calling, scientific observation and retained report publication.
use crate::bam::admission::bindings::resolve_governed_vcf_call_bam_smoke_contract;
use crate::bam::execution::native::CalledArtifacts;
use crate::bam::execution::native::CallingMode;
use crate::bam::execution::native::PreparedBam;
use crate::call_gl::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::call_gl::model::DEFAULT_OUTPUT_VCF_NAME;
use crate::call_gl::model::DEFAULT_VCF_CALL_GL_SMOKE_ROOT;
use crate::call_gl::model::GOVERNED_VCF_CALL_GL_STAGE_ID;
use crate::inputs::GenotypeBenchmarkInputs;
use crate::publication::timestamp_marker;
use crate::publication::ReportContext;
use anyhow::Result;
use bijux_dna_bench_model::BenchmarkLocalVcfCallGlSmokeReport as LocalVcfCallGlSmokeReport;
use bijux_dna_bench_model::BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_METRICS_SCHEMA_V1;
use std::path::Path;
use std::time::Instant;

/// Execute an admitted genotype benchmark and publish its scientific evidence.
///
/// # Errors
/// Returns declaration, fixture, native execution, scientific validation or publication failures.
pub fn run_local_vcf_call_gl_smoke(
    inputs: &GenotypeBenchmarkInputs,
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfCallGlSmokeReport> {
    let contract = resolve_governed_vcf_call_bam_smoke_contract(
        inputs,
        repo_root,
        GOVERNED_VCF_CALL_GL_STAGE_ID,
        tool_id,
        "gl_sites_vcf",
    )?;
    let published_output_root =
        repo_root.join(DEFAULT_VCF_CALL_GL_SMOKE_ROOT).join(&contract.tool_id);
    let execution_dir = bijux_dna_infra::temp_dir_in(
        &crate::publication::execution_parent(&published_output_root)?,
        "run-",
    )?;
    let output_root = execution_dir.path().to_path_buf();
    let published = crate::publication::PublishedArtifactPaths {
        repository: repo_root,
        execution: &output_root,
        destination: &published_output_root,
    };
    let prepared =
        PreparedBam::prepare(repo_root, &output_root, &contract.input_bam, &contract.reference)?;
    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs =
        prepared.execute(CallingMode::Gl, &contract.tool_id, &contract.sample_name, false)?;
    let artifacts =
        CalledArtifacts::materialize(&stage_outputs, &output_root, DEFAULT_OUTPUT_VCF_NAME)?;

    let evidence = super::observations::stage::observe(&contract, &artifacts)?;
    let metrics_path = output_root.join(DEFAULT_OUTPUT_METRICS_NAME);
    bijux_dna_infra::atomic_write_json(&metrics_path, &evidence.metrics)?;

    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let context = ReportContext {
        paths: published,
        started_at: &started_at,
        finished_at: &finished_at,
        elapsed_seconds,
    };
    super::publication::stage_result::publish_index(&context, &contract, &artifacts)?;
    let report = super::publication::report::build_report(
        &context, &contract, &prepared, &artifacts, evidence,
    )?;
    let plan = crate::publication::PublicationPlan::admit(
        repo_root,
        &output_root,
        &published_output_root,
        &crate::publication::ReportOwner {
            report_name: DEFAULT_OUTPUT_METRICS_NAME,
            schema: BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_METRICS_SCHEMA_V1,
            stage: &report.stage_id,
            tool: &report.tool_id,
        },
        &crate::publication::SCHEMA,
    )?;
    // Native manifests and replay evidence name this physical execution directory.
    let _retained_execution = execution_dir.keep();
    plan.publish()?;
    Ok(report)
}
