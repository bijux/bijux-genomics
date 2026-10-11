//! Publish declared execution outputs using actual execution addresses.
use crate::damage_filter::execution::artifacts::StageArtifacts;
use crate::damage_filter::model::GovernedVcfDamageFilterSmokeContract;
use crate::damage_filter::model::DAMAGE_FILTER_COUNTS_NAME;
use crate::damage_filter::model::DAMAGE_FILTER_MANIFEST_NAME;
use crate::damage_filter::model::DAMAGE_FILTER_SUMMARY_NAME;
use crate::damage_filter::model::DAMAGE_FILTER_WARNINGS_NAME;
use crate::damage_filter::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::damage_filter::model::DEFAULT_OUTPUT_VCF_NAME;
use crate::damage_filter::model::DEFAULT_STAGE_RESULT_NAME;
use crate::damage_filter::model::LOCAL_VCF_DAMAGE_FILTER_SMOKE_COMMAND;
use crate::damage_filter::publication::stage_result::build_stage_result_manifest;
use crate::publication::ReportContext;
use crate::stage_result::validate_stage_result_manifest;
use anyhow::Result;

pub(crate) fn publish_stage_result(
    context: &ReportContext<'_>,
    contract: &GovernedVcfDamageFilterSmokeContract,
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
    let summary_path = &artifacts.summary_path;
    let counts_path = &artifacts.counts_path;
    let warnings_path = &artifacts.warnings_path;
    let damage_manifest_path = &artifacts.damage_manifest_path;
    let stage_result_manifest = build_stage_result_manifest(
        repo_root,
        contract,
        &format!("{LOCAL_VCF_DAMAGE_FILTER_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "damage_filtered_vcf",
                DEFAULT_OUTPUT_VCF_NAME.to_string(),
                output_vcf.as_path(),
                "vcf_output",
            ),
            (
                "damage_filtered_vcf_tbi",
                format!("{DEFAULT_OUTPUT_VCF_NAME}.tbi"),
                output_tbi.as_path(),
                "vcf_index",
            ),
            (
                "damage_filter_summary_json",
                DAMAGE_FILTER_SUMMARY_NAME.to_string(),
                summary_path.as_path(),
                "report_output",
            ),
            (
                "damage_filter_counts_json",
                DAMAGE_FILTER_COUNTS_NAME.to_string(),
                counts_path.as_path(),
                "report_output",
            ),
            (
                "warnings_json",
                DAMAGE_FILTER_WARNINGS_NAME.to_string(),
                warnings_path.as_path(),
                "report_output",
            ),
            (
                "damage_genotype_manifest_json",
                DAMAGE_FILTER_MANIFEST_NAME.to_string(),
                damage_manifest_path.as_path(),
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
