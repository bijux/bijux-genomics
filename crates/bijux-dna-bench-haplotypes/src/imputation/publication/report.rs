//! Adapt admitted execution and observed evidence to the public report schema.
use crate::imputation::execution::artifacts::ImputationArtifacts;
use crate::imputation::execution::inputs::PreparedImputationInputs;
use crate::imputation::model::LocalVcfImputeSmokeReport;
use crate::imputation::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::imputation::model::GOVERNED_VCF_IMPUTE_INPUT_FIXTURE_ID;
use crate::imputation::model::LOCAL_VCF_IMPUTE_SMOKE_COMMAND;
use crate::imputation::model::LOCAL_VCF_IMPUTE_SMOKE_SCHEMA_VERSION;
use crate::imputation::observations::evidence::ImputationEvidence;
use crate::panel_contracts::GovernedVcfPanelWorkflowSmokeContract;
use crate::panel_contracts::DEFAULT_STAGE_RESULT_NAME;
use crate::publication::ReportContext;

pub(crate) fn build_report(
    context: &ReportContext<'_>,
    contract: &GovernedVcfPanelWorkflowSmokeContract,
    prepared: &PreparedImputationInputs,
    artifacts: &ImputationArtifacts,
    evidence: &ImputationEvidence,
) -> anyhow::Result<LocalVcfImputeSmokeReport> {
    let metrics_path = context.paths.execution.join(DEFAULT_OUTPUT_METRICS_NAME);
    let stage_result_manifest_path = context.paths.execution.join(DEFAULT_STAGE_RESULT_NAME);
    Ok(LocalVcfImputeSmokeReport {
        schema_version: LOCAL_VCF_IMPUTE_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_IMPUTE_SMOKE_COMMAND} --tool-id {}", contract.tool),
        stage_id: contract.stage.clone(),
        tool_id: contract.tool.clone(),
        corpus_id: contract.corpus.clone(),
        input_fixture_id: GOVERNED_VCF_IMPUTE_INPUT_FIXTURE_ID.to_string(),
        panel_id: contract.panel.clone(),
        map_id: contract.map.clone(),
        input_vcf_path: context.paths.path(&prepared.input_vcf_path)?,
        truth_vcf_path: context.paths.path(&prepared.truth_vcf_path)?,
        output_root: context.paths.path(context.paths.execution)?,
        output_vcf_path: context.paths.path(&artifacts.output_vcf)?,
        output_tbi_path: context.paths.path(&artifacts.output_tbi)?,
        panel_assets_path: context.paths.path(&prepared.panel_assets_path)?,
        imputation_qc_path: context.paths.path(&artifacts.imputation_qc)?,
        imputation_qc_tsv_path: context.paths.path(&artifacts.imputation_qc_tsv)?,
        imputation_manifest_path: context.paths.path(&artifacts.imputation_manifest)?,
        overlap_stats_path: context.paths.path(&artifacts.overlap_stats)?,
        warnings_path: context.paths.path(&artifacts.warnings)?,
        imputation_accept_path: context.paths.path(&artifacts.imputation_accept)?,
        panel_mismatch_path: context.paths.path(&artifacts.panel_mismatch)?,
        maf_bins_path: context.paths.path(&artifacts.maf_bins)?,
        info_hist_path: context.paths.path(&artifacts.info_hist)?,
        logs_path: context.paths.path(&artifacts.logs)?,
        metrics_path: context.paths.path(&metrics_path)?,
        stage_result_manifest_path: context.paths.path(&stage_result_manifest_path)?,
        started_at: context.started_at.to_owned(),
        finished_at: context.finished_at.to_owned(),
        elapsed_seconds: context.elapsed_seconds,
        exit_code: 0,
        variant_count: evidence.output_summary.variant_count,
        missing_before: evidence.quality.missing_before,
        missing_after: evidence.quality.missing_after,
        imputed_genotypes: evidence.quality.imputed_genotypes,
        low_confidence_count: evidence.quality.low_confidence_count,
        masked_truth_site_count: evidence.quality.masked_truth_site_count,
        masked_truth_match_count: evidence.quality.masked_truth_match_count,
        unresolved_count: evidence.quality.unresolved_count,
        not_imputable_reasons: evidence.quality.not_imputable_reasons.clone(),
        sample_count: evidence.sample_count,
        sample_ids: evidence.output_summary.sample_ids.clone(),
        masked_sample_gt: evidence.output_summary.masked_sample_gt.clone(),
        donor_sample_gt: evidence.output_summary.donor_sample_gt.clone(),
        parseable: true,
        validation_checks: evidence.validation.checks.clone(),
        gt_present: evidence.validation.gt_present,
        gl_present: evidence.validation.gl_present,
    })
}
