use super::{
    admission::resolve_governed_vcf_stats_smoke_contract,
    execution::{execute, prepare},
    model::{LocalVcfStatsSmokeReport, DEFAULT_OUTPUT_METRICS_NAME, DEFAULT_VCF_STATS_SMOKE_ROOT},
    publication::admit_publication,
    report::build_report,
    timing::timestamp_marker,
};
use crate::{
    inputs::VariantBenchmarkInputs,
    publication::{execution_parent, PublishedArtifactPaths, ReportContext},
};
use anyhow::Result;
use std::{path::Path, time::Instant};
/// Execute governed cohort statistics and retain native evidence before stable publication.
///
/// # Errors
/// Returns declaration, execution, scientific validation or publication failures.
pub fn run_local_vcf_stats_smoke(
    inputs: &VariantBenchmarkInputs,
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfStatsSmokeReport> {
    let contract = resolve_governed_vcf_stats_smoke_contract(inputs, tool_id)?;
    let published_output_root =
        repo_root.join(DEFAULT_VCF_STATS_SMOKE_ROOT).join(&contract.tool_id);
    let execution_dir =
        bijux_dna_infra::temp_dir_in(&execution_parent(&published_output_root)?, "run-")?;
    let output_root = execution_dir.path();
    let prepared = prepare(output_root)?;
    let started_at = timestamp_marker();
    let started = Instant::now();
    let observed = execute(&contract, &prepared, output_root)?;
    bijux_dna_infra::atomic_write_json(
        &output_root.join(DEFAULT_OUTPUT_METRICS_NAME),
        &observed.metrics,
    )?;
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let context = ReportContext {
        paths: PublishedArtifactPaths {
            repository: repo_root,
            execution: output_root,
            destination: &published_output_root,
        },
        started_at: &started_at,
        finished_at: &finished_at,
        elapsed_seconds,
    };
    let plan = admit_publication(&context, &contract, &observed)?;
    let report = build_report(&context, &contract, &prepared, &observed)?;
    let _retained_execution = execution_dir.keep();
    plan.publish()?;
    Ok(report)
}
