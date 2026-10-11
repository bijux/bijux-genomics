//! Publish stage outputs using their retained native execution addresses.
use crate::imputation::execution::artifacts::ImputationArtifacts;
use crate::imputation::execution::inputs::PreparedImputationInputs;
use crate::imputation::model::DEFAULT_OUTPUT_ACCEPT_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_INFO_HIST_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_LOGS_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_MAF_BINS_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_MANIFEST_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_OVERLAP_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_PANEL_ASSETS_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_PANEL_MISMATCH_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_QC_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_QC_TSV_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_VCF_NAME;
use crate::imputation::model::DEFAULT_OUTPUT_WARNINGS_NAME;
use crate::imputation::model::LOCAL_VCF_IMPUTE_SMOKE_COMMAND;
use crate::panel_contracts::build_stage_result_manifest;
use crate::panel_contracts::GovernedVcfPanelWorkflowSmokeContract;
use crate::panel_contracts::DEFAULT_STAGE_RESULT_NAME;
use crate::publication::ReportContext;
use crate::stage_result::validate_stage_result_manifest;
use anyhow::Result;

pub(crate) fn publish_stage_result(
    context: &ReportContext<'_>,
    contract: &GovernedVcfPanelWorkflowSmokeContract,
    prepared: &PreparedImputationInputs,
    artifacts: &ImputationArtifacts,
) -> Result<()> {
    let metrics_path = context.paths.execution.join(DEFAULT_OUTPUT_METRICS_NAME);
    let stage_result_manifest_path = context.paths.execution.join(DEFAULT_STAGE_RESULT_NAME);
    let stage_result_manifest = build_stage_result_manifest(
        context.paths.repository,
        contract,
        &format!("{LOCAL_VCF_IMPUTE_SMOKE_COMMAND} --tool-id {}", contract.tool),
        &[
            (
                "imputed_vcf",
                DEFAULT_OUTPUT_VCF_NAME.to_string(),
                artifacts.output_vcf.as_path(),
                "vcf_output",
            ),
            (
                "imputed_tbi",
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
                "imputation_qc_json",
                DEFAULT_OUTPUT_QC_NAME.to_string(),
                artifacts.imputation_qc.as_path(),
                "report_output",
            ),
            (
                "imputation_qc_tsv",
                DEFAULT_OUTPUT_QC_TSV_NAME.to_string(),
                artifacts.imputation_qc_tsv.as_path(),
                "table_output",
            ),
            (
                "imputation_manifest_json",
                DEFAULT_OUTPUT_MANIFEST_NAME.to_string(),
                artifacts.imputation_manifest.as_path(),
                "report_output",
            ),
            (
                "overlap_stats_json",
                DEFAULT_OUTPUT_OVERLAP_NAME.to_string(),
                artifacts.overlap_stats.as_path(),
                "report_output",
            ),
            (
                "warnings_json",
                DEFAULT_OUTPUT_WARNINGS_NAME.to_string(),
                artifacts.warnings.as_path(),
                "report_output",
            ),
            (
                "imputation_accept_json",
                DEFAULT_OUTPUT_ACCEPT_NAME.to_string(),
                artifacts.imputation_accept.as_path(),
                "report_output",
            ),
            (
                "panel_mismatch_diagnostics_json",
                DEFAULT_OUTPUT_PANEL_MISMATCH_NAME.to_string(),
                artifacts.panel_mismatch.as_path(),
                "report_output",
            ),
            (
                "maf_bin_quality_tsv",
                DEFAULT_OUTPUT_MAF_BINS_NAME.to_string(),
                artifacts.maf_bins.as_path(),
                "table_output",
            ),
            (
                "info_hist_json",
                DEFAULT_OUTPUT_INFO_HIST_NAME.to_string(),
                artifacts.info_hist.as_path(),
                "report_output",
            ),
            (
                "logs_txt",
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
