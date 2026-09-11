//! Project validated evidence into the existing public report schema.
use crate::postprocess::execution::artifacts::StageArtifacts;
use crate::postprocess::execution::inputs::PreparedInputs;
use crate::postprocess::model::GovernedVcfPostprocessSmokeContract;
use crate::postprocess::model::LocalVcfPostprocessSmokeReport;
use crate::postprocess::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::postprocess::model::DEFAULT_STAGE_RESULT_NAME;
use crate::postprocess::model::LOCAL_VCF_POSTPROCESS_SMOKE_COMMAND;
use crate::postprocess::model::LOCAL_VCF_POSTPROCESS_SMOKE_SCHEMA_VERSION;
use crate::postprocess::observations::stage::StageEvidence;
use crate::publication::ReportContext;
use anyhow::Result;

pub(crate) fn build_report(
    context: &ReportContext<'_>,
    contract: &GovernedVcfPostprocessSmokeContract,
    prepared: &PreparedInputs,
    artifacts: &StageArtifacts,
    evidence: StageEvidence,
) -> Result<LocalVcfPostprocessSmokeReport> {
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
    let validate_outputs_path = &artifacts.validate_outputs_path;
    let final_manifest_path = &artifacts.final_manifest_path;
    let normalization_contract_path = &artifacts.normalization_contract_path;
    let artifact_checksums_path = &artifacts.artifact_checksums_path;
    let logs_path = &artifacts.logs_path;
    let report = LocalVcfPostprocessSmokeReport {
        schema_version: LOCAL_VCF_POSTPROCESS_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_POSTPROCESS_SMOKE_COMMAND} --tool-id {}", contract.tool),
        stage_id: contract.stage.clone(),
        tool_id: contract.tool.clone(),
        corpus_id: contract.corpus.clone(),
        input_fixture_id: contract.input_fixture.clone(),
        input_vcf_path: published.path(input_vcf)?,
        output_root: published.path(output_root)?,
        output_vcf_path: published.path(output_vcf)?,
        output_tbi_path: published.path(output_tbi)?,
        validate_outputs_path: published.path(validate_outputs_path)?,
        final_manifest_path: published.path(final_manifest_path)?,
        normalization_contract_path: published.path(normalization_contract_path)?,
        artifact_checksums_path: published.path(artifact_checksums_path)?,
        logs_path: published.path(logs_path)?,
        metrics_path: published.path(&metrics_path)?,
        stage_result_manifest_path: published.path(&stage_result_manifest_path)?,
        started_at: started_at.to_owned(),
        finished_at: finished_at.to_owned(),
        elapsed_seconds,
        exit_code: metrics.exit_code,
        readable_vcf: metrics.readable_vcf,
        tabix_present: metrics.tabix_present,
        contigs_consistent_with_species_context: metrics.contigs_consistent_with_species_context,
        left_align_applied: metrics.left_align_applied,
        multiallelic_records_split: metrics.multiallelic_records_split,
        indels_normalized: metrics.indels_normalized,
        variant_ids_normalized: metrics.variant_ids_normalized,
        invalid_records_removed: metrics.invalid_records_removed,
        filter_standardized_to_pass: metrics.filter_standardized_to_pass,
        record_count: metrics.record_count,
        sample_count: metrics.sample_count,
        sample_ids: metrics.sample_ids,
        mq_removed: metrics.mq_removed,
        multiallelic_records_remaining: metrics.multiallelic_records_remaining,
        parseable: true,
        validation_checks: validation.checks,
        gt_present: validation.gt_present,
    };
    Ok(report)
}
