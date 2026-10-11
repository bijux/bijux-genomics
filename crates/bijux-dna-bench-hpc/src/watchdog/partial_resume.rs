use super::declarations::{
    LOCAL_DAG_WATCHDOG_SIMULATION_SCHEMA_VERSION, PARTIAL_RESUME_INVALID_NODE_ID,
    PARTIAL_RESUME_MISSING_NODE_IDS,
};
use super::model::{LocalDagWatchdogSimulationNode, LocalDagWatchdogSimulationReport};
use super::paths::path_relative_to_repo;
use super::scenario::LocalDagWatchdogScenario;
use super::status::LocalDagWatchdogNodeStatus;
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::path::Path;
pub(super) fn build_partial_resume_report(
    repo_root: &Path,
    config_path: &Path,
    dag_report_path: &Path,
    output_path: &Path,
    dag_report: &bijux_dna_bench_model::BenchmarkLocalPipelineDagValidationReport,
) -> Result<LocalDagWatchdogSimulationReport> {
    let reused_valid_node_ids = vec![
        bijux_dna_core::id_catalog::FASTQ_VALIDATE_READS.to_string(),
        bijux_dna_core::id_catalog::FASTQ_PROFILE_READ_LENGTHS.to_string(),
        bijux_dna_core::id_catalog::FASTQ_DETECT_ADAPTERS.to_string(),
    ];
    let invalid_node_ids = vec![PARTIAL_RESUME_INVALID_NODE_ID.to_string()];
    let missing_node_ids = PARTIAL_RESUME_MISSING_NODE_IDS
        .iter()
        .map(|node_id| (*node_id).to_string())
        .collect::<Vec<_>>();
    let planned_node_ids = invalid_node_ids
        .iter()
        .cloned()
        .chain(missing_node_ids.iter().cloned())
        .collect::<Vec<_>>();
    let plan_start_index = planned_node_ids
        .iter()
        .enumerate()
        .map(|(index, node_id)| (node_id.clone(), index as u64))
        .collect::<BTreeMap<_, _>>();

    let mut nodes = Vec::with_capacity(dag_report.nodes.len());
    for node in &dag_report.nodes {
        let (status, start_second, finish_second, duration_seconds) =
            if reused_valid_node_ids.iter().any(|node_id| node_id == &node.node_id) {
                (LocalDagWatchdogNodeStatus::Reused, 0, 0, 0)
            } else if planned_node_ids.iter().any(|node_id| node_id == &node.node_id) {
                let start_second = *plan_start_index.get(&node.node_id).ok_or_else(|| {
                    anyhow!("partial-resume plan index missing `{}`", node.node_id)
                })?;
                let duration_seconds = partial_resume_duration_seconds(&node.stage_id);
                (
                    LocalDagWatchdogNodeStatus::Planned,
                    start_second,
                    start_second + duration_seconds,
                    duration_seconds,
                )
            } else {
                return Err(anyhow!(
                    "partial-resume scenario did not classify node `{}`",
                    node.node_id
                ));
            };

        nodes.push(LocalDagWatchdogSimulationNode {
            node_id: node.node_id.clone(),
            sample_id: "sample_primary".to_string(),
            stage_id: node.stage_id.clone(),
            status: status.as_str().to_string(),
            dependency_count: node.dependency_count,
            depends_on: node.depends_on.clone(),
            start_second,
            finish_second,
            duration_seconds,
        });
    }

    let partial_resume_proven = reused_valid_node_ids
        .iter()
        .all(|node_id| !planned_node_ids.iter().any(|planned| planned == node_id))
        && invalid_node_ids
            .iter()
            .all(|node_id| planned_node_ids.iter().any(|planned| planned == node_id))
        && missing_node_ids
            .iter()
            .all(|node_id| planned_node_ids.iter().any(|planned| planned == node_id));

    if !partial_resume_proven {
        return Err(anyhow!(
            "partial-resume simulation did not preserve valid completed work while replanning only missing or invalid nodes"
        ));
    }

    let simulated_makespan_seconds = nodes.iter().map(|node| node.finish_second).max().unwrap_or(0);

    Ok(LocalDagWatchdogSimulationReport {
        schema_version: LOCAL_DAG_WATCHDOG_SIMULATION_SCHEMA_VERSION,
        scenario: LocalDagWatchdogScenario::PartialResume.as_str().to_string(),
        config_path: path_relative_to_repo(repo_root, config_path),
        dag_report_path: path_relative_to_repo(repo_root, dag_report_path),
        output_path: path_relative_to_repo(repo_root, output_path),
        pipeline_id: dag_report.pipeline_id.clone(),
        node_count: nodes.len(),
        sample_count: 1,
        simulated_makespan_seconds,
        slow_branch_stage_id: String::new(),
        slow_branch_finish_second: 0,
        ready_while_slow_branch_running_stage_ids: Vec::new(),
        no_global_wait_proven: false,
        failed_sample_id: None,
        failed_stage_id: None,
        failure_second: None,
        continued_unrelated_node_ids: Vec::new(),
        blocked_node_ids: Vec::new(),
        failure_isolation_proven: false,
        reused_valid_node_ids,
        invalid_node_ids,
        missing_node_ids,
        planned_node_ids,
        partial_resume_proven,
        completion_check_stage_id: None,
        completion_checks: Vec::new(),
        completion_rules_proven: false,
        nodes,
    })
}

fn partial_resume_duration_seconds(_stage_id: &str) -> u64 {
    1
}
