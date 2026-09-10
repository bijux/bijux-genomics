//! Compose imputation execution, normalized quality and owned artifact publication.
use crate::imputation::run_local_vcf_impute_smoke;
use crate::imputation_metrics::execution::timing::timestamp_marker;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_REPORT_NAME;
use crate::imputation_metrics::model::DEFAULT_STAGE_RESULT_NAME;
use crate::imputation_metrics::model::DEFAULT_VCF_IMPUTATION_METRICS_SMOKE_ROOT;
use crate::imputation_metrics::observations::reports::read_json;
use crate::imputation_metrics::observations::variants::summarize_imputation_metrics;
use crate::inputs::HaplotypeBenchmarkInputs;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_model::validate_benchmark_local_vcf_imputation_metrics_smoke_report;
use bijux_dna_bench_model::BenchmarkLocalVcfImputationMetricsSmokeReport as LocalVcfImputationMetricsSmokeReport;
use bijux_dna_bench_model::BENCHMARK_LOCAL_VCF_IMPUTATION_METRICS_SMOKE_SCHEMA_V1;
use std::path::Path;
use std::time::Instant;

/// Execute the admitted benchmark and publish its validated artifacts.
///
/// # Errors
/// Returns admission, execution, scientific validation or publication failures.
pub fn run_local_vcf_imputation_metrics_smoke(
    inputs: &HaplotypeBenchmarkInputs,
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfImputationMetricsSmokeReport> {
    let source_report = run_local_vcf_impute_smoke(inputs, repo_root, tool_id)?;
    let published_output_root =
        repo_root.join(DEFAULT_VCF_IMPUTATION_METRICS_SMOKE_ROOT).join(&source_report.tool_id);
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

    bijux_dna_infra::ensure_dir(&output_root)
        .with_context(|| format!("create {}", output_root.display()))?;

    let artifacts =
        super::execution::source::materialize_source(repo_root, &output_root, &source_report)?;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let source_imputation_qc = read_json(&artifacts.source_imputation_qc)?;
    let summary = summarize_imputation_metrics(&source_imputation_qc)?;

    let imputation_metrics_path = output_root.join(DEFAULT_OUTPUT_REPORT_NAME);
    let report_payload = super::publication::report::build_report_payload(
        &source_report,
        &summary,
        &published,
        &artifacts,
        &started_at,
    )?;
    super::publication::orchestration::publish_orchestration(
        &source_report,
        &summary,
        &published,
        &artifacts,
    )?;

    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let report = LocalVcfImputationMetricsSmokeReport {
        finished_at: finished_at.clone(),
        elapsed_seconds,
        stage_result_manifest_path: published.path(&stage_result_manifest_path)?,
        ..report_payload
    };
    validate_benchmark_local_vcf_imputation_metrics_smoke_report(&report)?;
    bijux_dna_infra::atomic_write_json(&imputation_metrics_path, &report)?;

    let context = crate::publication::ReportContext {
        paths: published,
        started_at: &started_at,
        finished_at: &finished_at,
        elapsed_seconds,
    };
    super::publication::stage_result::publish_stage_result(&context, &source_report, &artifacts)?;
    let plan = crate::publication::PublicationPlan::admit(
        repo_root,
        &output_root,
        &published_output_root,
        &crate::publication::ReportOwner {
            report_name: DEFAULT_OUTPUT_REPORT_NAME,
            schema: BENCHMARK_LOCAL_VCF_IMPUTATION_METRICS_SMOKE_SCHEMA_V1,
            stage: &report.stage_id,
            tool: &report.tool_id,
        },
    )?;
    // Native manifests and checksums name this execution; keep it before replacing stable files.
    let _retained_execution = execution_dir.keep();
    plan.publish()?;
    Ok(report)
}
