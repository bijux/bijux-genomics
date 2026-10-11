use super::{
    execution::ObservedStats,
    manifest::build_stage_result_manifest,
    model::{
        GovernedVcfStatsSmokeContract, DEFAULT_OUTPUT_BCFTOOLS_STATS_NAME,
        DEFAULT_OUTPUT_METRICS_NAME, DEFAULT_OUTPUT_STATS_NAME, DEFAULT_STAGE_RESULT_NAME,
        LOCAL_VCF_STATS_SMOKE_COMMAND, LOCAL_VCF_STATS_SMOKE_METRICS_SCHEMA_VERSION,
    },
};
use crate::publication::{PublicationPlan, ReportContext, ReportOwner, SCHEMA};
use crate::stage_result::validate_stage_result_manifest;
use anyhow::Result;
pub(super) fn admit_publication(
    context: &ReportContext<'_>,
    contract: &GovernedVcfStatsSmokeContract,
    observed: &ObservedStats,
) -> Result<PublicationPlan> {
    let output_root = context.paths.execution;
    let metrics_path = output_root.join(DEFAULT_OUTPUT_METRICS_NAME);
    let manifest = build_stage_result_manifest(
        context.paths.repository,
        contract,
        &format!("{LOCAL_VCF_STATS_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &[
            (
                "stats_json",
                DEFAULT_OUTPUT_STATS_NAME.to_owned(),
                observed.stats_json_path.as_path(),
                "report_output",
            ),
            (
                "bcftools_stats_txt",
                DEFAULT_OUTPUT_BCFTOOLS_STATS_NAME.to_owned(),
                observed.bcftools_stats_path.as_path(),
                "report_output",
            ),
            (
                "metrics_json",
                DEFAULT_OUTPUT_METRICS_NAME.to_owned(),
                metrics_path.as_path(),
                "report_output",
            ),
        ],
        context.started_at,
        context.finished_at,
        context.elapsed_seconds,
    );
    validate_stage_result_manifest(&manifest)?;
    bijux_dna_infra::atomic_write_json(&output_root.join(DEFAULT_STAGE_RESULT_NAME), &manifest)?;
    PublicationPlan::admit(
        context.paths.repository,
        output_root,
        context.paths.destination,
        &ReportOwner {
            report_name: DEFAULT_OUTPUT_METRICS_NAME,
            schema: LOCAL_VCF_STATS_SMOKE_METRICS_SCHEMA_VERSION,
            stage: &contract.stage_id,
            tool: &contract.tool_id,
        },
        &SCHEMA,
    )
}
