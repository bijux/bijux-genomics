//! Compose typed inputs, native stage execution, scientific observations and publication.
use crate::inputs::HaplotypeBenchmarkInputs;
use crate::reference_panel::admission::bindings::resolve_governed_vcf_prepare_reference_panel_smoke_contract;
use crate::reference_panel::execution::timing::timestamp_marker;
use crate::reference_panel::model::LocalVcfPrepareReferencePanelSmokeReport;
use crate::reference_panel::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::reference_panel::model::DEFAULT_VCF_PREPARE_PANEL_SMOKE_ROOT;
use crate::reference_panel::model::LOCAL_VCF_PREPARE_PANEL_SMOKE_METRICS_SCHEMA_VERSION;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;
use std::time::Instant;

/// Execute the admitted benchmark and publish its validated artifacts.
///
/// # Errors
/// Returns admission, execution, scientific validation or publication failures.
pub fn run_local_vcf_prepare_reference_panel_smoke(
    inputs: &HaplotypeBenchmarkInputs,
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfPrepareReferencePanelSmokeReport> {
    let contract = resolve_governed_vcf_prepare_reference_panel_smoke_contract(inputs, tool_id)?;
    let published_output_root =
        repo_root.join(DEFAULT_VCF_PREPARE_PANEL_SMOKE_ROOT).join(&contract.tool);
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
    let prepared = super::execution::inputs::prepare_inputs(&output_root)?;
    let started_at = timestamp_marker();
    let started = Instant::now();
    let stage_outputs = super::execution::stage::execute_stage(&prepared, &contract)?;

    let artifacts = super::execution::artifacts::materialize_outputs(&stage_outputs, &output_root)?;

    let (mut evidence, manifest_evidence) = super::observations::evidence::observe_panel(
        &artifacts.panel_vcf,
        &artifacts.panel_tbi,
        &artifacts.panel_manifest,
        &prepared.raw_panel_path,
    )?;
    evidence.sample_consistent =
        super::observations::validation::validate_panel(&evidence, &manifest_evidence)?;

    let metrics =
        super::publication::metrics::build_metrics(&contract, &evidence, &published, &artifacts)?;
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
    super::publication::execution_result::publish_stage_result(&context, &contract, &artifacts)?;

    let report = super::publication::report::build_report(
        &context, &contract, &prepared, &artifacts, &evidence,
    )?;
    let plan = crate::publication::PublicationPlan::admit(
        repo_root,
        &output_root,
        &published_output_root,
        &crate::publication::ReportOwner {
            report_name: DEFAULT_OUTPUT_METRICS_NAME,
            schema: LOCAL_VCF_PREPARE_PANEL_SMOKE_METRICS_SCHEMA_VERSION,
            stage: &report.stage_id,
            tool: &report.tool_id,
        },
        &crate::publication::SCHEMA,
    )?;
    // Native manifests and checksums name this execution; keep it before replacing stable files.
    let _retained_execution = execution_dir.keep();
    plan.publish()?;
    Ok(report)
}
