//! Project preserved likelihood metrics into the public report schema.
use crate::gl_propagation::execution::PreparedInputs;
use crate::gl_propagation::execution::StageArtifacts;
use crate::gl_propagation::model::GovernedVcfGlPropagationSmokeContract;
use crate::gl_propagation::model::LocalVcfGlPropagationSmokeReport;
use crate::gl_propagation::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::gl_propagation::model::DEFAULT_STAGE_RESULT_NAME;
use crate::gl_propagation::model::LOCAL_VCF_GL_PROPAGATION_SMOKE_COMMAND;
use crate::gl_propagation::model::LOCAL_VCF_GL_PROPAGATION_SMOKE_SCHEMA_VERSION;
use crate::gl_propagation::observations::stage::StageEvidence;
use crate::publication::ReportContext;
use crate::stage_result::path_relative_to_repo;
use anyhow::Result;

pub(crate) fn build_report(
    context: &ReportContext<'_>,
    contract: &GovernedVcfGlPropagationSmokeContract,
    prepared: &PreparedInputs,
    artifacts: &StageArtifacts,
    evidence: StageEvidence,
) -> Result<LocalVcfGlPropagationSmokeReport> {
    let repo_root = context.paths.repository;
    let output_root = context.paths.execution;
    let output_vcf = &artifacts.vcf;
    let output_tbi = &artifacts.index;
    let bcf_path = &artifacts.bcf;
    let bcf_index_path = &artifacts.bcf_index;
    let report_path = &artifacts.report;
    let metrics_path = output_root.join(DEFAULT_OUTPUT_METRICS_NAME);
    let published = &context.paths;
    let published_output_root = context.paths.destination;
    let input_vcf = &prepared.vcf;
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let elapsed_seconds = context.elapsed_seconds;
    let StageEvidence { metrics, validation } = evidence;
    let report = LocalVcfGlPropagationSmokeReport {
        schema_version: LOCAL_VCF_GL_PROPAGATION_SMOKE_SCHEMA_VERSION,
        command: format!(
            "{LOCAL_VCF_GL_PROPAGATION_SMOKE_COMMAND} --tool-id {}",
            contract.tool_id.clone()
        ),
        stage_id: contract.stage_id.clone(),
        tool_id: contract.tool_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        sample_name: contract.sample_name.clone(),
        input_vcf_path: published.path(input_vcf)?,
        output_root: path_relative_to_repo(repo_root, published_output_root),
        output_vcf_path: published.path(output_vcf)?,
        output_tbi_path: published.path(output_tbi)?,
        output_bcf_path: published.path(bcf_path)?,
        output_bcf_csi_path: published.path(bcf_index_path)?,
        report_path: published.path(report_path)?,
        metrics_path: published.path(&metrics_path)?,
        stage_result_manifest_path: published.path(&stage_result_manifest_path)?,
        started_at: context.started_at.to_owned(),
        finished_at: context.finished_at.to_owned(),
        elapsed_seconds,
        exit_code: metrics.exit_code,
        input_likelihood_fields: metrics.input_likelihood_fields,
        output_likelihood_fields: metrics.output_likelihood_fields,
        lost_fields: metrics.lost_fields,
        site_count_before: metrics.site_count_before,
        site_count_after: metrics.site_count_after,
        sample_count: metrics.sample_count,
        parseable: true,
        validation_checks: validation.checks,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
    };
    Ok(report)
}
