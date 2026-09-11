//! Publish declared execution outputs using actual execution addresses.
use crate::filter::execution::artifacts::StageArtifacts;
use crate::filter::model::GovernedVcfFilterSmokeContract;
use crate::filter::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::filter::model::DEFAULT_OUTPUT_VCF_NAME;
use crate::filter::model::DEFAULT_STAGE_RESULT_NAME;
use crate::filter::model::FILTER_BREAKDOWN_JSON_NAME;
use crate::filter::model::FILTER_BREAKDOWN_TSV_NAME;
use crate::filter::model::FILTER_EXPLAIN_JSON_NAME;
use crate::filter::model::LOCAL_VCF_FILTER_SMOKE_COMMAND;
use crate::filter::publication::stage_result::build_stage_result_manifest;
use crate::publication::ReportContext;
use crate::stage_result::validate_stage_result_manifest;
use anyhow::Result;

pub(crate) fn publish_stage_result(
    context: &ReportContext<'_>,
    contract: &GovernedVcfFilterSmokeContract,
    artifacts: &StageArtifacts,
) -> Result<()> {
    let repo_root = context.paths.repository;
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
    let stage_result_manifest = build_stage_result_manifest(
        repo_root,
        contract,
        &format!("{LOCAL_VCF_FILTER_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "filtered_vcf",
                DEFAULT_OUTPUT_VCF_NAME.to_string(),
                output_vcf.as_path(),
                "vcf_output",
            ),
            (
                "filtered_vcf_tbi",
                format!("{DEFAULT_OUTPUT_VCF_NAME}.tbi"),
                output_tbi.as_path(),
                "vcf_index",
            ),
            (
                "filter_breakdown_json",
                FILTER_BREAKDOWN_JSON_NAME.to_string(),
                filter_breakdown_path.as_path(),
                "report_output",
            ),
            (
                "filter_breakdown_tsv",
                FILTER_BREAKDOWN_TSV_NAME.to_string(),
                filter_breakdown_tsv_path.as_path(),
                "report_output",
            ),
            (
                "filter_explain_json",
                FILTER_EXPLAIN_JSON_NAME.to_string(),
                filter_explain_path.as_path(),
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
