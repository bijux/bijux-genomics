//! Compose typed inputs, native stage execution, scientific observations and publication.
use crate::imputation::execution::timing::timestamp_marker;
use crate::imputation::model::LocalVcfImputeSmokeReport;
use crate::imputation::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::imputation::model::DEFAULT_VCF_IMPUTE_SMOKE_ROOT;
use crate::imputation::model::GOVERNED_VCF_IMPUTE_STAGE_ID;
use crate::imputation::model::LOCAL_VCF_IMPUTE_SMOKE_METRICS_SCHEMA_VERSION;
use crate::inputs::HaplotypeBenchmarkInputs;
use crate::panel_contracts::resolve_governed_vcf_panel_workflow_smoke_contract;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;
use std::time::Instant;

/// Execute the admitted benchmark and publish its validated artifacts.
///
/// # Errors
/// Returns admission, execution, scientific validation or publication failures.
pub fn run_local_vcf_impute_smoke(
    inputs: &HaplotypeBenchmarkInputs,
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfImputeSmokeReport> {
    let contract = resolve_governed_vcf_panel_workflow_smoke_contract(
        inputs,
        GOVERNED_VCF_IMPUTE_STAGE_ID,
        tool_id,
        "imputed_vcf",
    )?;
    let published_output_root = repo_root.join(DEFAULT_VCF_IMPUTE_SMOKE_ROOT).join(&contract.tool);
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
    let prepared = super::execution::inputs::prepare_inputs(&output_root, &contract)?;
    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = super::execution::stage::execute_stage(&prepared, &contract)?;

    let artifacts = super::execution::artifacts::materialize_outputs(&stage_outputs, &output_root)?;

    let (validation, output_summary) = super::observations::validation::observe_variants(
        &artifacts.output_vcf,
        &artifacts.output_tbi,
    )?;
    let quality =
        super::observations::quality::observe_quality(&artifacts.imputation_qc, prepared.backend)?;
    let sample_count = u64::try_from(output_summary.sample_ids.len())
        .map_err(|_| anyhow!("sample count overflow"))?;
    let evidence = super::observations::evidence::ImputationEvidence {
        validation,
        output_summary,
        quality,
        sample_count,
    };

    let metrics = super::publication::metrics::build_metrics(&contract, &evidence);
    let metrics_path = output_root.join(DEFAULT_OUTPUT_METRICS_NAME);
    bijux_dna_infra::atomic_write_json(&metrics_path, &metrics)?;

    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let context = crate::publication::ReportContext {
        paths: published,
        started_at: &started_at,
        finished_at: &finished_at,
        elapsed_seconds,
    };
    super::publication::execution_result::publish_stage_result(
        &context, &contract, &prepared, &artifacts,
    )?;

    let report = super::publication::report::build_report(
        &context, &contract, &prepared, &artifacts, &evidence,
    )?;
    let plan = crate::publication::PublicationPlan::admit(
        repo_root,
        &output_root,
        &published_output_root,
        &crate::publication::ReportOwner {
            report_name: DEFAULT_OUTPUT_METRICS_NAME,
            schema: LOCAL_VCF_IMPUTE_SMOKE_METRICS_SCHEMA_VERSION,
            stage: &report.stage_id,
            tool: &report.tool_id,
        },
    )?;
    // Native manifests and checksums name this execution; keep it before replacing stable files.
    let _retained_execution = execution_dir.keep();
    plan.publish()?;
    Ok(report)
}
