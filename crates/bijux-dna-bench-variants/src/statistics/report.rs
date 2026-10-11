use super::{
    execution::{ObservedStats, PreparedInput},
    model::{
        GovernedVcfStatsSmokeContract, LocalVcfStatsSmokeReport, DEFAULT_OUTPUT_METRICS_NAME,
        DEFAULT_STAGE_RESULT_NAME, LOCAL_VCF_STATS_SMOKE_COMMAND,
        LOCAL_VCF_STATS_SMOKE_SCHEMA_VERSION,
    },
};
use crate::publication::ReportContext;
use anyhow::Result;
pub(super) fn build_report(
    context: &ReportContext<'_>,
    contract: &GovernedVcfStatsSmokeContract,
    prepared: &PreparedInput,
    observed: &ObservedStats,
) -> Result<LocalVcfStatsSmokeReport> {
    let output_root = context.paths.execution;
    let metrics = &observed.metrics;
    Ok(LocalVcfStatsSmokeReport {
        schema_version: LOCAL_VCF_STATS_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_STATS_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id.clone(),
        tool_id: contract.tool_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        sample_name: contract.sample_name.clone(),
        input_vcf_path: context.paths.path(&prepared.input_vcf)?,
        output_root: context.paths.path(output_root)?,
        stats_json_path: context.paths.path(&observed.stats_json_path)?,
        bcftools_stats_path: context.paths.path(&observed.bcftools_stats_path)?,
        metrics_path: context.paths.path(&output_root.join(DEFAULT_OUTPUT_METRICS_NAME))?,
        stage_result_manifest_path: context
            .paths
            .path(&output_root.join(DEFAULT_STAGE_RESULT_NAME))?,
        started_at: context.started_at.to_owned(),
        finished_at: context.finished_at.to_owned(),
        elapsed_seconds: context.elapsed_seconds,
        exit_code: metrics.exit_code,
        variant_count: metrics.variant_count,
        snp_count: metrics.snp_count,
        indel_count: metrics.indel_count,
        transition_count: metrics.transition_count,
        transversion_count: metrics.transversion_count,
        ti_tv: metrics.ti_tv,
        sample_count: metrics.sample_count,
    })
}
