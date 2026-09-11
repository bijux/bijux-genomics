use super::{checks, evidence::DryRunEvidence, DryRunContext};
use anyhow::Result;
use bijux_dna_bench_model::{
    validate_benchmark_local_hpc_dry_run_readiness,
    BenchmarkLocalHpcDryRunReadinessReport as LocalHpcDryRunReadyReport,
    BENCHMARK_LOCAL_HPC_DRY_RUN_READINESS_SCHEMA_V1,
};
use std::collections::BTreeSet;
use std::path::Path;
pub(super) fn build_hpc_dry_run_ready(
    context: &DryRunContext<'_>,
    absolute_output_path: &Path,
) -> Result<LocalHpcDryRunReadyReport> {
    context.output.admit(context.repo_root, absolute_output_path)?;
    let evidence = DryRunEvidence::acquire(context);
    let checks = vec![
        checks::asset_coverage(&evidence, &context.paths),
        checks::scratch_coverage(&evidence, &context.paths),
        checks::execution_resolution(&evidence, &context.paths),
        checks::stage_array(&evidence, &context.paths),
        checks::pipeline_array(&evidence, &context.paths),
        checks::dependency_blocking(&evidence, &context.paths),
        checks::resume_policy(&evidence, &context.paths),
        checks::collection_states(&evidence, &context.paths),
        checks::candidate_admission(&evidence, &context.paths),
    ];
    let failing_goal_ids = checks
        .iter()
        .filter(|check| !check.ok)
        .map(|check| check.goal_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let summary = super::summary::summary(&evidence);
    let behavior = super::behavior::behavior(&evidence, &checks);

    let report = LocalHpcDryRunReadyReport {
        schema_version: BENCHMARK_LOCAL_HPC_DRY_RUN_READINESS_SCHEMA_V1.to_string(),
        output_path: context.output.display(context.repo_root, absolute_output_path),
        checked_goal_count: checks.len(),
        passed_goal_count: checks.iter().filter(|check| check.ok).count(),
        failed_goal_count: failing_goal_ids.len(),
        ready_for_first_hpc_run: failing_goal_ids.is_empty() && behavior.proven,
        failing_goal_ids,
        summary,
        behavior,
        checks,
    };
    validate_benchmark_local_hpc_dry_run_readiness(&report)?;
    Ok(report)
}
