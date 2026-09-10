//! Adapt admitted execution and observed evidence to the public report schema.
use crate::panel_contracts::GovernedVcfPanelWorkflowSmokeContract;
use crate::panel_contracts::DEFAULT_STAGE_RESULT_NAME;
use crate::phasing::execution::artifacts::PhasingArtifacts;
use crate::phasing::execution::inputs::PreparedPhasingInputs;
use crate::phasing::model::LocalVcfPhasingSmokeReport;
use crate::phasing::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::phasing::model::GOVERNED_VCF_PHASING_INPUT_FIXTURE_ID;
use crate::phasing::model::LOCAL_VCF_PHASING_SMOKE_COMMAND;
use crate::phasing::model::LOCAL_VCF_PHASING_SMOKE_SCHEMA_VERSION;
use crate::phasing::observations::stage::PhasingEvidence;
use crate::publication::ReportContext;

pub(crate) fn build_report(
    context: &ReportContext<'_>,
    contract: &GovernedVcfPanelWorkflowSmokeContract,
    prepared: &PreparedPhasingInputs,
    artifacts: &PhasingArtifacts,
    evidence: &PhasingEvidence,
) -> anyhow::Result<LocalVcfPhasingSmokeReport> {
    let metrics_path = context.paths.execution.join(DEFAULT_OUTPUT_METRICS_NAME);
    let stage_result_manifest_path = context.paths.execution.join(DEFAULT_STAGE_RESULT_NAME);
    Ok(LocalVcfPhasingSmokeReport {
        schema_version: LOCAL_VCF_PHASING_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_PHASING_SMOKE_COMMAND} --tool-id {}", contract.tool),
        stage_id: contract.stage.clone(),
        tool_id: contract.tool.clone(),
        corpus_id: contract.corpus.clone(),
        input_fixture_id: GOVERNED_VCF_PHASING_INPUT_FIXTURE_ID.to_string(),
        panel_id: contract.panel.clone(),
        map_id: contract.map.clone(),
        input_vcf_path: context.paths.path(&prepared.input_vcf_path)?,
        output_root: context.paths.path(context.paths.execution)?,
        output_vcf_path: context.paths.path(&artifacts.output_vcf)?,
        output_tbi_path: context.paths.path(&artifacts.output_tbi)?,
        panel_assets_path: context.paths.path(&prepared.panel_assets_path)?,
        phasing_qc_path: context.paths.path(&artifacts.phasing_qc)?,
        phasing_manifest_path: context.paths.path(&artifacts.phasing_manifest)?,
        phase_block_stats_path: context.paths.path(&artifacts.phase_block_stats)?,
        switch_error_proxy_path: context.paths.path(&artifacts.switch_error_proxy)?,
        logs_path: context.paths.path(&artifacts.logs)?,
        metrics_path: context.paths.path(&metrics_path)?,
        stage_result_manifest_path: context.paths.path(&stage_result_manifest_path)?,
        started_at: context.started_at.to_owned(),
        finished_at: context.finished_at.to_owned(),
        elapsed_seconds: context.elapsed_seconds,
        exit_code: 0,
        input_genotypes: evidence.input_summary.genotype_count,
        phased_genotypes: evidence.output_summary.phased_genotypes,
        unphased_genotypes: evidence.output_summary.unphased_genotypes,
        phase_set_count: evidence.output_summary.phase_set_count,
        sample_count: evidence.sample_count,
        sample_ids: evidence.output_summary.sample_ids.clone(),
        parseable: true,
        validation_checks: evidence.validation.checks.clone(),
        gt_present: evidence.validation.gt_present,
        gl_present: evidence.validation.gl_present,
    })
}
