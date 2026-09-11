use super::declarations::{
    FAILURE_ISOLATION_CONTINUED_SAMPLE_ID, FAILURE_ISOLATION_FAILED_SAMPLE_ID,
    FAILURE_ISOLATION_FAILED_STAGE_ID, LOCAL_DAG_WATCHDOG_SIMULATION_SCHEMA_VERSION,
};
use super::model::{LocalDagWatchdogSimulationNode, LocalDagWatchdogSimulationReport};
use super::paths::path_relative_to_repo;
use super::scenario::LocalDagWatchdogScenario;
use super::status::LocalDagWatchdogNodeStatus;
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::path::Path;
pub(super) fn build_failure_isolation_report(
    repo_root: &Path,
    config_path: &Path,
    dag_report_path: &Path,
    output_path: &Path,
    dag_report: &bijux_dna_bench_model::BenchmarkLocalPipelineDagValidationReport,
) -> Result<LocalDagWatchdogSimulationReport> {
    let sample_ids = [FAILURE_ISOLATION_FAILED_SAMPLE_ID, FAILURE_ISOLATION_CONTINUED_SAMPLE_ID];
    let nodes = simulate_nodes(dag_report, sample_ids);

    let Some(failed_node) = nodes.iter().find(|node| {
        node.sample_id == FAILURE_ISOLATION_FAILED_SAMPLE_ID
            && node.stage_id == FAILURE_ISOLATION_FAILED_STAGE_ID
            && node.status == LocalDagWatchdogNodeStatus::Failed.as_str()
    }) else {
        return Err(anyhow!("failure-isolation scenario did not produce the injected failed node"));
    };

    let failure_second = failed_node.finish_second;
    let continued_unrelated_node_ids = nodes
        .iter()
        .filter(|node| {
            node.sample_id == FAILURE_ISOLATION_CONTINUED_SAMPLE_ID
                && node.status == LocalDagWatchdogNodeStatus::Completed.as_str()
                && node.finish_second > failure_second
        })
        .map(|node| node.node_id.clone())
        .collect::<Vec<_>>();
    let blocked_node_ids = nodes
        .iter()
        .filter(|node| {
            node.sample_id == FAILURE_ISOLATION_FAILED_SAMPLE_ID
                && node.status == LocalDagWatchdogNodeStatus::Blocked.as_str()
        })
        .map(|node| node.node_id.clone())
        .collect::<Vec<_>>();
    let failure_isolation_proven = continued_unrelated_node_ids
        .iter()
        .any(|node_id| node_id == "sample_beta::fastq.trim_reads")
        && continued_unrelated_node_ids
            .iter()
            .any(|node_id| node_id == "sample_beta::fastq.report_qc")
        && blocked_node_ids.iter().any(|node_id| node_id == "sample_alpha::fastq.trim_reads");

    if !failure_isolation_proven {
        return Err(anyhow!(
            "failure-isolation simulation did not show unrelated sample work continuing after the injected failure"
        ));
    }

    let simulated_makespan_seconds = nodes.iter().map(|node| node.finish_second).max().unwrap_or(0);

    Ok(LocalDagWatchdogSimulationReport {
        schema_version: LOCAL_DAG_WATCHDOG_SIMULATION_SCHEMA_VERSION,
        scenario: LocalDagWatchdogScenario::FailureIsolation.as_str().to_string(),
        config_path: path_relative_to_repo(repo_root, config_path),
        dag_report_path: path_relative_to_repo(repo_root, dag_report_path),
        output_path: path_relative_to_repo(repo_root, output_path),
        pipeline_id: dag_report.pipeline_id.clone(),
        node_count: nodes.len(),
        sample_count: sample_ids.len(),
        simulated_makespan_seconds,
        slow_branch_stage_id: String::new(),
        slow_branch_finish_second: 0,
        ready_while_slow_branch_running_stage_ids: Vec::new(),
        no_global_wait_proven: false,
        failed_sample_id: Some(FAILURE_ISOLATION_FAILED_SAMPLE_ID.to_string()),
        failed_stage_id: Some(FAILURE_ISOLATION_FAILED_STAGE_ID.to_string()),
        failure_second: Some(failure_second),
        continued_unrelated_node_ids,
        blocked_node_ids,
        failure_isolation_proven,
        reused_valid_node_ids: Vec::new(),
        invalid_node_ids: Vec::new(),
        missing_node_ids: Vec::new(),
        planned_node_ids: Vec::new(),
        partial_resume_proven: false,
        completion_check_stage_id: None,
        completion_checks: Vec::new(),
        completion_rules_proven: false,
        nodes,
    })
}
fn simulate_nodes(
    dag_report: &bijux_dna_bench_model::BenchmarkLocalPipelineDagValidationReport,
    sample_ids: [&str; 2],
) -> Vec<LocalDagWatchdogSimulationNode> {
    let mut nodes = Vec::with_capacity(dag_report.nodes.len() * sample_ids.len());
    let mut finish_times = BTreeMap::<String, u64>::new();
    let mut status_index = BTreeMap::<String, LocalDagWatchdogNodeStatus>::new();

    for sample_id in sample_ids {
        for node in &dag_report.nodes {
            let qualified_node_id = format!("{sample_id}::{}", node.node_id);
            let qualified_dependencies = node
                .depends_on
                .iter()
                .map(|dependency| format!("{sample_id}::{dependency}"))
                .collect::<Vec<_>>();
            let dependency_statuses = qualified_dependencies
                .iter()
                .filter_map(|dependency| status_index.get(dependency).copied())
                .collect::<Vec<_>>();
            let dependency_failed = dependency_statuses.iter().any(|status| {
                matches!(
                    status,
                    LocalDagWatchdogNodeStatus::Failed | LocalDagWatchdogNodeStatus::Blocked
                )
            });

            let (status, start_second, finish_second, duration_seconds) = if sample_id
                == FAILURE_ISOLATION_FAILED_SAMPLE_ID
                && node.stage_id == FAILURE_ISOLATION_FAILED_STAGE_ID
            {
                let start_second = qualified_dependencies
                    .iter()
                    .filter_map(|dependency| finish_times.get(dependency).copied())
                    .max()
                    .unwrap_or(0);
                let duration_seconds = failure_isolation_duration_seconds(&node.stage_id);
                (
                    LocalDagWatchdogNodeStatus::Failed,
                    start_second,
                    start_second + duration_seconds,
                    duration_seconds,
                )
            } else if dependency_failed {
                (LocalDagWatchdogNodeStatus::Blocked, 0, 0, 0)
            } else {
                let start_second = qualified_dependencies
                    .iter()
                    .filter_map(|dependency| finish_times.get(dependency).copied())
                    .max()
                    .unwrap_or(0);
                let duration_seconds = failure_isolation_duration_seconds(&node.stage_id);
                (
                    LocalDagWatchdogNodeStatus::Completed,
                    start_second,
                    start_second + duration_seconds,
                    duration_seconds,
                )
            };

            finish_times.insert(qualified_node_id.clone(), finish_second);
            status_index.insert(qualified_node_id.clone(), status);
            nodes.push(LocalDagWatchdogSimulationNode {
                node_id: qualified_node_id,
                sample_id: sample_id.to_string(),
                stage_id: node.stage_id.clone(),
                status: status.as_str().to_string(),
                dependency_count: node.dependency_count,
                depends_on: qualified_dependencies,
                start_second,
                finish_second,
                duration_seconds,
            });
        }
    }

    nodes
}

fn failure_isolation_duration_seconds(stage_id: &str) -> u64 {
    match stage_id {
        bijux_dna_core::id_catalog::FASTQ_PROFILE_READ_LENGTHS => 6,
        bijux_dna_core::id_catalog::FASTQ_DETECT_ADAPTERS => 2,
        _ => 1,
    }
}
