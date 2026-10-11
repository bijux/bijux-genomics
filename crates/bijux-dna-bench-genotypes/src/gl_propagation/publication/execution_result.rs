//! Persist the admitted benchmark output index.
use crate::gl_propagation::execution::StageArtifacts;
use crate::gl_propagation::model::GovernedVcfGlPropagationSmokeContract;
use crate::gl_propagation::model::DEFAULT_OUTPUT_BCF_NAME;
use crate::gl_propagation::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::gl_propagation::model::DEFAULT_OUTPUT_REPORT_NAME;
use crate::gl_propagation::model::DEFAULT_OUTPUT_VCF_NAME;
use crate::gl_propagation::model::DEFAULT_STAGE_RESULT_NAME;
use crate::gl_propagation::model::LOCAL_VCF_GL_PROPAGATION_SMOKE_COMMAND;
use crate::gl_propagation::publication::stage_result::build_stage_result_manifest;
use crate::publication::ReportContext;
use crate::stage_result::validate_stage_result_manifest;
use anyhow::Result;

pub(crate) fn publish_index(
    context: &ReportContext<'_>,
    contract: &GovernedVcfGlPropagationSmokeContract,
    artifacts: &StageArtifacts,
) -> Result<()> {
    let repo_root = context.paths.repository;
    let output_root = context.paths.execution;
    let output_vcf = &artifacts.vcf;
    let output_tbi = &artifacts.index;
    let bcf_path = &artifacts.bcf;
    let bcf_index_path = &artifacts.bcf_index;
    let report_path = &artifacts.report;
    let metrics_path = output_root.join(DEFAULT_OUTPUT_METRICS_NAME);
    let started_at = context.started_at;
    let finished_at = context.finished_at;
    let elapsed_seconds = context.elapsed_seconds;
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let stage_result_manifest = build_stage_result_manifest(
        repo_root,
        contract,
        &format!("{LOCAL_VCF_GL_PROPAGATION_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "gl_propagated_vcf",
                DEFAULT_OUTPUT_VCF_NAME.to_string(),
                output_vcf.as_path(),
                "vcf_output",
            ),
            (
                "gl_propagated_vcf_tbi",
                format!("{DEFAULT_OUTPUT_VCF_NAME}.tbi"),
                output_tbi.as_path(),
                "vcf_index",
            ),
            (
                "gl_propagated_bcf",
                DEFAULT_OUTPUT_BCF_NAME.to_string(),
                bcf_path.as_path(),
                "bcf_output",
            ),
            (
                "bcf_index",
                format!("{DEFAULT_OUTPUT_BCF_NAME}.csi"),
                bcf_index_path.as_path(),
                "bcf_index",
            ),
            (
                "gl_propagation_report",
                DEFAULT_OUTPUT_REPORT_NAME.to_string(),
                report_path.as_path(),
                "report_output",
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
