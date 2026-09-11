//! Project admitted scientific observations into the compatible public report.
use crate::bam::execution::native::CalledArtifacts;
use crate::bam::execution::native::PreparedBam;
use crate::bam::model::GovernedVcfCallBamSmokeContract;
use crate::bam::model::DEFAULT_STAGE_RESULT_NAME;
use crate::call_pseudohaploid::model::LocalVcfCallPseudohaploidSmokeReport;
use crate::call_pseudohaploid::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::call_pseudohaploid::model::LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_COMMAND;
use crate::call_pseudohaploid::model::LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_SCHEMA_VERSION;
use crate::call_pseudohaploid::observations::stage::StageEvidence;
use crate::publication::ReportContext;
use crate::stage_result::path_relative_to_repo;
use anyhow::Result;

pub(crate) fn build_report(
    context: &ReportContext<'_>,
    contract: &GovernedVcfCallBamSmokeContract,
    prepared: &PreparedBam,
    artifacts: &CalledArtifacts,
    evidence: StageEvidence,
) -> Result<LocalVcfCallPseudohaploidSmokeReport> {
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
    let report = LocalVcfCallPseudohaploidSmokeReport {
        schema_version: LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_SCHEMA_VERSION,
        command: format!(
            "{LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_COMMAND} --tool-id {}",
            contract.tool_id.clone()
        ),
        stage_id: contract.stage_id.clone(),
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
        target_sites: metrics.target_sites,
        covered_sites: metrics.covered_sites,
        called_sites: metrics.called_sites,
        missing_sites: metrics.missing_sites,
        sampling_policy: metrics.sampling_policy,
        seed_usage: metrics.seed_usage,
        random_seed: metrics.random_seed,
        sample_count: metrics.sample_count,
        parseable: true,
        validation_checks: validation.checks,
        haploid_compatible: true,
        gt_present: validation.gt_present,
        gl_present: validation.gl_present,
        raw_output_sha256: metrics.raw_output_sha256,
        raw_replay_output_sha256: metrics.raw_replay_output_sha256,
        canonical_output_sha256: metrics.canonical_output_sha256,
        canonical_replay_output_sha256: metrics.canonical_replay_output_sha256,
        determinism_scope: metrics.determinism_scope,
        raw_replay_match: metrics.raw_replay_match,
        deterministic_replay_match: metrics.deterministic_replay_match,
    };
    Ok(report)
}
