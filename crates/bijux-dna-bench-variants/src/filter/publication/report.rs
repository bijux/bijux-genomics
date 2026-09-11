//! Project validated evidence into the existing public report schema.
use crate::filter::execution::artifacts::StageArtifacts;
use crate::filter::execution::inputs::PreparedInputs;
use crate::filter::model::GovernedVcfFilterSmokeContract;
use crate::filter::model::LocalVcfFilterSmokeReport;
use crate::filter::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::filter::model::DEFAULT_STAGE_RESULT_NAME;
use crate::filter::model::LOCAL_VCF_FILTER_SMOKE_COMMAND;
use crate::filter::model::LOCAL_VCF_FILTER_SMOKE_SCHEMA_VERSION;
use crate::filter::observations::stage::StageEvidence;
use crate::publication::ReportContext;
use anyhow::Result;

pub(crate) fn build_report(
    context: &ReportContext<'_>,
    contract: &GovernedVcfFilterSmokeContract,
    prepared: &PreparedInputs,
    artifacts: &StageArtifacts,
    evidence: StageEvidence,
) -> Result<LocalVcfFilterSmokeReport> {
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
    let filter_breakdown_path = &artifacts.filter_breakdown_path;
    let filter_breakdown_tsv_path = &artifacts.filter_breakdown_tsv_path;
    let filter_explain_path = &artifacts.filter_explain_path;
    let report = LocalVcfFilterSmokeReport {
        schema_version: LOCAL_VCF_FILTER_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_FILTER_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
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
        filter_breakdown_path: published.path(filter_breakdown_path)?,
        filter_breakdown_tsv_path: published.path(filter_breakdown_tsv_path)?,
        filter_explain_path: published.path(filter_explain_path)?,
        stage_result_manifest_path: published.path(&stage_result_manifest_path)?,
        started_at: started_at.to_owned(),
        finished_at: finished_at.to_owned(),
        elapsed_seconds,
        exit_code: metrics.exit_code,
        input_variants: metrics.input_variants,
        pass_variants: metrics.pass_variants,
        failed_variants: metrics.failed_variants,
        filter_ids: metrics.filter_ids,
        depth_threshold: metrics.depth_threshold,
        quality_threshold: metrics.quality_threshold,
        missingness_threshold: metrics.missingness_threshold,
        sample_count: metrics.sample_count,
        parseable: true,
        validation_checks: validation.checks,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
    };
    Ok(report)
}
