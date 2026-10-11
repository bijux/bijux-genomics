use super::declarations::{
    LOCAL_DAG_WATCHDOG_SIMULATION_SCHEMA_VERSION, NO_GLOBAL_WAIT_SLOW_BRANCH_STAGE_ID,
};
use super::model::{LocalDagWatchdogSimulationNode, LocalDagWatchdogSimulationReport};
use super::paths::path_relative_to_repo;
use super::scenario::LocalDagWatchdogScenario;
use super::status::LocalDagWatchdogNodeStatus;
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::path::Path;
pub(super) fn build_no_global_wait_report(
    repo_root: &Path,
    config_path: &Path,
    dag_report_path: &Path,
    output_path: &Path,
    dag_report: &bijux_dna_bench_model::BenchmarkLocalPipelineDagValidationReport,
) -> Result<LocalDagWatchdogSimulationReport> {
    let mut finish_times = BTreeMap::<String, u64>::new();
    let mut stage_finish_times = BTreeMap::<String, u64>::new();
    let mut nodes = Vec::with_capacity(dag_report.nodes.len());

    for node in &dag_report.nodes {
        let start_second = node
            .depends_on
            .iter()
            .filter_map(|dependency| finish_times.get(dependency).copied())
            .max()
            .unwrap_or(0);
        let duration_seconds = no_global_wait_duration_seconds(&node.stage_id);
        let finish_second = start_second + duration_seconds;

        finish_times.insert(node.node_id.clone(), finish_second);
        stage_finish_times.insert(node.stage_id.clone(), finish_second);
        nodes.push(LocalDagWatchdogSimulationNode {
            node_id: node.node_id.clone(),
            sample_id: "sample_primary".to_string(),
            stage_id: node.stage_id.clone(),
            status: LocalDagWatchdogNodeStatus::Completed.as_str().to_string(),
            dependency_count: node.dependency_count,
            depends_on: node.depends_on.clone(),
            start_second,
            finish_second,
            duration_seconds,
        });
    }

    let Some(slow_branch_finish_second) =
        stage_finish_times.get(NO_GLOBAL_WAIT_SLOW_BRANCH_STAGE_ID).copied()
    else {
        return Err(anyhow!(
            "no-global-wait scenario requires stage `{NO_GLOBAL_WAIT_SLOW_BRANCH_STAGE_ID}`"
        ));
    };

    let ready_while_slow_branch_running_stage_ids = nodes
        .iter()
        .filter(|node| {
            node.stage_id != NO_GLOBAL_WAIT_SLOW_BRANCH_STAGE_ID
                && node.start_second > 0
                && node.start_second < slow_branch_finish_second
        })
        .map(|node| node.stage_id.clone())
        .collect::<Vec<_>>();

    let no_global_wait_proven = ready_while_slow_branch_running_stage_ids
        .iter()
        .any(|stage_id| stage_id == bijux_dna_core::id_catalog::FASTQ_TRIM)
        && ready_while_slow_branch_running_stage_ids
            .iter()
            .any(|stage_id| stage_id == bijux_dna_core::id_catalog::FASTQ_FILTER);

    if !no_global_wait_proven {
        return Err(anyhow!(
            "no-global-wait simulation did not observe independent ready stages while `{NO_GLOBAL_WAIT_SLOW_BRANCH_STAGE_ID}` was still running"
        ));
    }

    let simulated_makespan_seconds = nodes.iter().map(|node| node.finish_second).max().unwrap_or(0);

    Ok(LocalDagWatchdogSimulationReport {
        schema_version: LOCAL_DAG_WATCHDOG_SIMULATION_SCHEMA_VERSION,
        scenario: LocalDagWatchdogScenario::NoGlobalWait.as_str().to_string(),
        config_path: path_relative_to_repo(repo_root, config_path),
        dag_report_path: path_relative_to_repo(repo_root, dag_report_path),
        output_path: path_relative_to_repo(repo_root, output_path),
        pipeline_id: dag_report.pipeline_id.clone(),
        node_count: nodes.len(),
        sample_count: 1,
        simulated_makespan_seconds,
        slow_branch_stage_id: NO_GLOBAL_WAIT_SLOW_BRANCH_STAGE_ID.to_string(),
        slow_branch_finish_second,
        ready_while_slow_branch_running_stage_ids,
        no_global_wait_proven,
        failed_sample_id: None,
        failed_stage_id: None,
        failure_second: None,
        continued_unrelated_node_ids: Vec::new(),
        blocked_node_ids: Vec::new(),
        failure_isolation_proven: false,
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

fn no_global_wait_duration_seconds(stage_id: &str) -> u64 {
    match stage_id {
        bijux_dna_core::id_catalog::FASTQ_PROFILE_READ_LENGTHS => 12,
        bijux_dna_core::id_catalog::FASTQ_DETECT_ADAPTERS => 2,
        _ => 1,
    }
}
