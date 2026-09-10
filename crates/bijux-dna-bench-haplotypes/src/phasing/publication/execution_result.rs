//! Publish stage outputs using their retained native execution addresses.
use crate::panel_contracts::build_stage_result_manifest;
use crate::panel_contracts::GovernedVcfPanelWorkflowSmokeContract;
use crate::panel_contracts::DEFAULT_STAGE_RESULT_NAME;
use crate::phasing::execution::artifacts::PhasingArtifacts;
use crate::phasing::execution::inputs::PreparedPhasingInputs;
use crate::phasing::model::DEFAULT_OUTPUT_LOGS_NAME;
use crate::phasing::model::DEFAULT_OUTPUT_MANIFEST_NAME;
use crate::phasing::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::phasing::model::DEFAULT_OUTPUT_PANEL_ASSETS_NAME;
use crate::phasing::model::DEFAULT_OUTPUT_PHASE_BLOCK_STATS_NAME;
use crate::phasing::model::DEFAULT_OUTPUT_QC_NAME;
use crate::phasing::model::DEFAULT_OUTPUT_SWITCH_PROXY_NAME;
use crate::phasing::model::DEFAULT_OUTPUT_VCF_NAME;
use crate::phasing::model::LOCAL_VCF_PHASING_SMOKE_COMMAND;
use crate::publication::ReportContext;
use crate::stage_result::validate_stage_result_manifest;
use anyhow::Result;

pub(crate) fn publish_stage_result(
    context: &ReportContext<'_>,
    contract: &GovernedVcfPanelWorkflowSmokeContract,
    prepared: &PreparedPhasingInputs,
    artifacts: &PhasingArtifacts,
) -> Result<()> {
    let metrics_path = context.paths.execution.join(DEFAULT_OUTPUT_METRICS_NAME);
    let stage_result_manifest_path = context.paths.execution.join(DEFAULT_STAGE_RESULT_NAME);
    let stage_result_manifest = build_stage_result_manifest(
        context.paths.repository,
        contract,
        &format!("{LOCAL_VCF_PHASING_SMOKE_COMMAND} --tool-id {}", contract.tool),
        &[
            (
                "phased_vcf",
                DEFAULT_OUTPUT_VCF_NAME.to_string(),
                artifacts.output_vcf.as_path(),
                "vcf_output",
            ),
            (
                "phased_vcf_tbi",
                format!("{DEFAULT_OUTPUT_VCF_NAME}.tbi"),
                artifacts.output_tbi.as_path(),
                "index_output",
            ),
            (
                "panel_assets_json",
                DEFAULT_OUTPUT_PANEL_ASSETS_NAME.to_string(),
                prepared.panel_assets_path.as_path(),
                "report_output",
            ),
            (
                "phasing_qc",
                DEFAULT_OUTPUT_QC_NAME.to_string(),
                artifacts.phasing_qc.as_path(),
                "report_output",
            ),
            (
                "phasing_manifest",
                DEFAULT_OUTPUT_MANIFEST_NAME.to_string(),
                artifacts.phasing_manifest.as_path(),
                "report_output",
            ),
            (
                "phase_block_stats",
                DEFAULT_OUTPUT_PHASE_BLOCK_STATS_NAME.to_string(),
                artifacts.phase_block_stats.as_path(),
                "table_output",
            ),
            (
                "switch_error_proxy",
                DEFAULT_OUTPUT_SWITCH_PROXY_NAME.to_string(),
                artifacts.switch_error_proxy.as_path(),
                "table_output",
            ),
            (
                "phasing_log",
                DEFAULT_OUTPUT_LOGS_NAME.to_string(),
                artifacts.logs.as_path(),
                "log_output",
            ),
            (
                "metrics_json",
                DEFAULT_OUTPUT_METRICS_NAME.to_string(),
                metrics_path.as_path(),
                "report_output",
            ),
        ],
        context.started_at,
        context.finished_at,
        context.elapsed_seconds,
    );
    validate_stage_result_manifest(&stage_result_manifest)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;

    Ok(())
}
