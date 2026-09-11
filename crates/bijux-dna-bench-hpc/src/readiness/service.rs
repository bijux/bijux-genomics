use crate::readiness::campaign::service::evaluate_hpc_campaign_goals;
use crate::readiness::context::ReadinessContext;
use crate::readiness::corpora::evaluate_corpus_goals;
use crate::readiness::harness::evaluate_benchmark_harness_goals;
use crate::readiness::paths::path_relative_to_repo;
use crate::readiness::pipelines::evaluate_pipeline_dag_goals;
use crate::readiness::slurm::evaluate_slurm_goals;
use crate::readiness::stages::inventory::evaluate_bam_inventory_goal;
use crate::readiness::stages::inventory::evaluate_fastq_inventory_goal;
use crate::readiness::stages::materialization::evaluate_local_stage_goals;
use crate::readiness::watchdogs::evaluate_watchdog_goals;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_model::validate_benchmark_local_hpc_submission_readiness;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessReport as BenchLocalHpcSubmissionReadyReport;
use bijux_dna_bench_model::BENCHMARK_LOCAL_HPC_SUBMISSION_READINESS_SCHEMA_V1;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// Assess all governed local checks and publish the complete validated diagnostic.
///
/// # Errors
/// Returns an output publication error or a failed readiness assessment after its report is written.
pub fn validate_hpc_submission_ready(
    context: &ReadinessContext<'_>,
    output_path: PathBuf,
) -> Result<BenchLocalHpcSubmissionReadyReport> {
    let repo_root = context.repo_root;

    let absolute_output_path =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    if let Some(parent) = absolute_output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }

    let mut checks = Vec::new();
    let fastq_inventory = evaluate_fastq_inventory_goal(context, &mut checks);
    let bam_inventory = evaluate_bam_inventory_goal(context, &mut checks);
    evaluate_local_stage_goals(context, &mut checks);
    evaluate_benchmark_harness_goals(context, &mut checks, fastq_inventory, bam_inventory);
    evaluate_corpus_goals(context, &mut checks);
    evaluate_pipeline_dag_goals(context, &mut checks);
    evaluate_watchdog_goals(context, &mut checks);
    evaluate_hpc_campaign_goals(context, &mut checks);
    evaluate_slurm_goals(context, &mut checks);

    let failing_goal_ids = checks
        .iter()
        .filter(|check| !check.ok)
        .map(|check| check.goal_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let report = BenchLocalHpcSubmissionReadyReport {
        schema_version: BENCHMARK_LOCAL_HPC_SUBMISSION_READINESS_SCHEMA_V1.to_string(),
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        checked_goal_count: checks.len(),
        passed_goal_count: checks.iter().filter(|check| check.ok).count(),
        failed_goal_count: failing_goal_ids.len(),
        ok: failing_goal_ids.is_empty(),
        failing_goal_ids,
        checks,
    };
    validate_benchmark_local_hpc_submission_readiness(&report)?;
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;

    if report.ok {
        Ok(report)
    } else {
        Err(anyhow!("local HPC submission readiness failed; see {}", report.output_path))
    }
}
