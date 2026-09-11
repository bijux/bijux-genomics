//! Compose admitted likelihood propagation and retained publication.
use crate::gl_propagation::admission::bindings::resolve_governed_vcf_gl_propagation_smoke_contract;
use crate::gl_propagation::execution::PreparedInputs;
use crate::gl_propagation::execution::StageArtifacts;
use crate::gl_propagation::model::LocalVcfGlPropagationSmokeReport;
use crate::gl_propagation::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::gl_propagation::model::DEFAULT_VCF_GL_PROPAGATION_SMOKE_ROOT;
use crate::gl_propagation::model::LOCAL_VCF_GL_PROPAGATION_SMOKE_METRICS_SCHEMA_VERSION;
use crate::inputs::GenotypeBenchmarkInputs;
use crate::publication::timestamp_marker;
use crate::publication::ReportContext;
use anyhow::Result;
use std::path::Path;
use std::time::Instant;

/// Execute an admitted genotype benchmark and publish its scientific evidence.
///
/// # Errors
/// Returns declaration, fixture, native execution, scientific validation or publication failures.
pub fn run_local_vcf_gl_propagation_smoke(
    inputs: &GenotypeBenchmarkInputs,
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfGlPropagationSmokeReport> {
    let contract = resolve_governed_vcf_gl_propagation_smoke_contract(inputs, tool_id)?;
    let published_output_root =
        repo_root.join(DEFAULT_VCF_GL_PROPAGATION_SMOKE_ROOT).join(&contract.tool_id);
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
    let prepared = PreparedInputs::prepare(&output_root)?;
    let started_at = timestamp_marker();
    let started = Instant::now();
    let native = prepared.execute()?;
    let artifacts = StageArtifacts::materialize(&native, &output_root)?;
    let evidence = super::observations::stage::observe(&contract, &prepared, &artifacts)?;
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
    super::publication::execution_result::publish_index(&context, &contract, &artifacts)?;
    let report = super::publication::report::build_report(
        &context, &contract, &prepared, &artifacts, evidence,
    )?;
    let plan = crate::publication::PublicationPlan::admit(
        repo_root,
        &output_root,
        &published_output_root,
        &crate::publication::ReportOwner {
            report_name: DEFAULT_OUTPUT_METRICS_NAME,
            schema: LOCAL_VCF_GL_PROPAGATION_SMOKE_METRICS_SCHEMA_VERSION,
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
