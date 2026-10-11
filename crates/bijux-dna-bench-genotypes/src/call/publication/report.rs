//! Project admitted scientific observations into the compatible public report.
use crate::bam::execution::native::CalledArtifacts;
use crate::bam::execution::native::PreparedBam;
use crate::bam::model::DEFAULT_STAGE_RESULT_NAME;
use crate::call::model::GovernedVcfCallSmokeContract;
use crate::call::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::call::model::LOCAL_VCF_CALL_SMOKE_COMMAND;
use crate::call::observations::stage::StageEvidence;
use crate::publication::ReportContext;
use crate::stage_result::path_relative_to_repo;
use anyhow::Result;
use bijux_dna_bench_model::validate_benchmark_local_vcf_call_smoke_report;
use bijux_dna_bench_model::BenchmarkLocalVcfCallSmokeReport as LocalVcfCallSmokeReport;
use bijux_dna_bench_model::BENCHMARK_LOCAL_VCF_CALL_SMOKE_SCHEMA_V1;

pub(crate) fn build_report(
    context: &ReportContext<'_>,
    contract: &GovernedVcfCallSmokeContract,
    prepared: &PreparedBam,
    artifacts: &CalledArtifacts,
    evidence: StageEvidence,
) -> Result<LocalVcfCallSmokeReport> {
    let repo_root = context.paths.repository;
    let published = &context.paths;
    let published_output_root = context.paths.destination;
    let output_root = context.paths.execution;
    let output_vcf = &artifacts.vcf;
    let output_tbi = &artifacts.index;
    let materialized_reference = &prepared.reference;
    let metrics_path = output_root.join(DEFAULT_OUTPUT_METRICS_NAME);
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let elapsed_seconds = context.elapsed_seconds;
    let StageEvidence { metrics, validation } = evidence;
    let report = LocalVcfCallSmokeReport {
        schema_version: BENCHMARK_LOCAL_VCF_CALL_SMOKE_SCHEMA_V1.to_string(),
        command: format!("{LOCAL_VCF_CALL_SMOKE_COMMAND} --tool-id {}", contract.tool_id.clone()),
        stage_id: contract.stage_id.clone(),
        resolved_stage_id: contract.resolved_stage_id.clone(),
        tool_id: contract.tool_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        sample_id: contract.sample_id.clone(),
        sample_name: contract.sample_name.clone(),
        input_bam: contract.input_bam.clone(),
        input_bam_index: contract.input_bam_index.clone(),
        reference: contract.reference.clone(),
        materialized_reference_path: published.path(materialized_reference)?,
        output_root: path_relative_to_repo(repo_root, published_output_root),
        output_vcf_path: published.path(output_vcf)?,
        output_tbi_path: published.path(output_tbi)?,
        metrics_path: published.path(&metrics_path)?,
        stage_result_manifest_path: published.path(&stage_result_manifest_path)?,
        started_at: context.started_at.to_owned(),
        finished_at: context.finished_at.to_owned(),
        elapsed_seconds,
        exit_code: metrics.exit_code,
        variant_count: metrics.variant_count,
        snp_count: metrics.snp_count,
        indel_count: metrics.indel_count,
        sample_count: metrics.sample_count,
        parseable: true,
        validation_checks: validation.checks,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
    };
    validate_benchmark_local_vcf_call_smoke_report(&report)?;

    Ok(report)
}
