//! Publish the benchmark output index with immutable execution addresses.
use crate::bam::execution::native::CalledArtifacts;
use crate::bam::model::GovernedVcfCallBamSmokeContract;
use crate::bam::model::DEFAULT_STAGE_RESULT_NAME;
use crate::bam::publication::stage_result::build_stage_result_manifest;
use crate::call::model::GovernedVcfCallSmokeContract;
use crate::call::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::call::model::DEFAULT_OUTPUT_VCF_NAME;
use crate::call::model::LOCAL_VCF_CALL_SMOKE_COMMAND;
use crate::publication::ReportContext;
use crate::stage_result::validate_stage_result_manifest;
use anyhow::Result;

pub(crate) fn publish_index(
    context: &ReportContext<'_>,
    contract: &GovernedVcfCallSmokeContract,
    artifacts: &CalledArtifacts,
) -> Result<()> {
    let repo_root = context.paths.repository;
    let output_root = context.paths.execution;
    let output_vcf = &artifacts.vcf;
    let output_tbi = &artifacts.index;
    let metrics_path = output_root.join(DEFAULT_OUTPUT_METRICS_NAME);
    let started_at = context.started_at;
    let finished_at = context.finished_at;
    let elapsed_seconds = context.elapsed_seconds;
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let base_contract = GovernedVcfCallBamSmokeContract {
        stage_id: contract.stage_id.clone(),
        tool_id: contract.tool_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_bam: contract.input_bam.clone(),
        input_bam_index: contract.input_bam_index.clone(),
        reference: contract.reference.clone(),
        sample_id: contract.sample_id.clone(),
        sample_name: contract.sample_name.clone(),
    };
    let stage_result_manifest = build_stage_result_manifest(
        repo_root,
        &base_contract,
        &format!("{LOCAL_VCF_CALL_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            ("called_vcf", DEFAULT_OUTPUT_VCF_NAME.to_string(), output_vcf.as_path(), "vcf_output"),
            (
                "called_vcf_tbi",
                format!("{DEFAULT_OUTPUT_VCF_NAME}.tbi"),
                output_tbi.as_path(),
                "vcf_index",
            ),
            (
                "metrics_json",
                DEFAULT_OUTPUT_METRICS_NAME.to_string(),
                metrics_path.as_path(),
                "report_output",
            ),
        ],
        started_at,
        finished_at,
        elapsed_seconds,
    );
    validate_stage_result_manifest(&stage_result_manifest)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;

    Ok(())
}
