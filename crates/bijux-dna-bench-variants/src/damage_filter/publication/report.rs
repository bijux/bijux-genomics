//! Project validated evidence into the existing public report schema.
use crate::damage_filter::execution::artifacts::StageArtifacts;
use crate::damage_filter::execution::inputs::PreparedInputs;
use crate::damage_filter::model::GovernedVcfDamageFilterSmokeContract;
use crate::damage_filter::model::LocalVcfDamageFilterSmokeReport;
use crate::damage_filter::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::damage_filter::model::DEFAULT_STAGE_RESULT_NAME;
use crate::damage_filter::model::LOCAL_VCF_DAMAGE_FILTER_SMOKE_COMMAND;
use crate::damage_filter::model::LOCAL_VCF_DAMAGE_FILTER_SMOKE_SCHEMA_VERSION;
use crate::damage_filter::observations::stage::StageEvidence;
use crate::publication::ReportContext;
use anyhow::Result;

pub(crate) fn build_report(
    context: &ReportContext<'_>,
    contract: &GovernedVcfDamageFilterSmokeContract,
    prepared: &PreparedInputs,
    artifacts: &StageArtifacts,
    evidence: StageEvidence,
) -> Result<LocalVcfDamageFilterSmokeReport> {
    let StageEvidence { metrics, validation } = evidence;
    let output_root = context.paths.execution;
    let published = &context.paths;
    let input_vcf = &prepared.input_vcf;
    let metrics_path = context.paths.execution.join(DEFAULT_OUTPUT_METRICS_NAME);
    let stage_result_manifest_path = context.paths.execution.join(DEFAULT_STAGE_RESULT_NAME);
    let started_at = context.started_at;
    let finished_at = context.finished_at;
    let elapsed_seconds = context.elapsed_seconds;
    let output_vcf = &artifacts.output_vcf;
    let output_tbi = &artifacts.output_tbi;
    let summary_path = &artifacts.summary_path;
    let counts_path = &artifacts.counts_path;
    let warnings_path = &artifacts.warnings_path;
    let damage_manifest_path = &artifacts.damage_manifest_path;
    let report = LocalVcfDamageFilterSmokeReport {
        schema_version: LOCAL_VCF_DAMAGE_FILTER_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_DAMAGE_FILTER_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id.clone(),
        tool_id: contract.tool_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        sample_name: contract.sample_name.clone(),
        input_vcf_path: published.path(input_vcf)?,
        output_root: published.path(output_root)?,
        output_vcf_path: published.path(output_vcf)?,
        output_tbi_path: published.path(output_tbi)?,
        metrics_path: published.path(&metrics_path)?,
        summary_path: published.path(summary_path)?,
        counts_path: published.path(counts_path)?,
        warnings_path: published.path(warnings_path)?,
        damage_manifest_path: published.path(damage_manifest_path)?,
        stage_result_manifest_path: published.path(&stage_result_manifest_path)?,
        started_at: started_at.to_owned(),
        finished_at: finished_at.to_owned(),
        elapsed_seconds,
        exit_code: metrics.exit_code,
        input_variants: metrics.input_variants,
        removed_variants: metrics.removed_variants,
        retained_variants: metrics.retained_variants,
        low_quality_filtered_variants: metrics.low_quality_filtered_variants,
        damage_ratio_filtered_variants: metrics.damage_ratio_filtered_variants,
        terminal_damage_filtered_variants: metrics.terminal_damage_filtered_variants,
        damage_context_rule: metrics.damage_context_rule,
        terminal_context_count: metrics.terminal_context_count,
        sample_count: metrics.sample_count,
        parseable: true,
        validation_checks: validation.checks,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
    };
    Ok(report)
}
