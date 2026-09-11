use super::declarations::{
    COMPLETION_RULES_STAGE_ID, LOCAL_DAG_WATCHDOG_SIMULATION_SCHEMA_VERSION,
};
use super::model::{
    LocalDagWatchdogCompletionCheck, LocalDagWatchdogSimulationNode,
    LocalDagWatchdogSimulationReport,
};
use super::paths::path_relative_to_repo;
use super::scenario::LocalDagWatchdogScenario;
use super::status::LocalDagWatchdogNodeStatus;
use anyhow::{anyhow, Result};
use std::path::Path;
pub(super) fn build_completion_rules_report(
    repo_root: &Path,
    config_path: &Path,
    dag_report_path: &Path,
    output_path: &Path,
    dag_report: &bijux_dna_bench_model::BenchmarkLocalPipelineDagValidationReport,
) -> Result<LocalDagWatchdogSimulationReport> {
    let dag_node = dag_report
        .nodes
        .iter()
        .find(|node| node.stage_id == COMPLETION_RULES_STAGE_ID)
        .ok_or_else(|| {
            anyhow!("completion-rules scenario requires stage `{COMPLETION_RULES_STAGE_ID}`")
        })?;

    let completion_checks = completion_cases();

    let nodes = completion_checks
        .iter()
        .enumerate()
        .map(|(index, check)| {
            let status = if check.complete {
                LocalDagWatchdogNodeStatus::Completed
            } else if check.exit_code == 0 {
                LocalDagWatchdogNodeStatus::Incomplete
            } else {
                LocalDagWatchdogNodeStatus::Failed
            };
            LocalDagWatchdogSimulationNode {
                node_id: check.node_id.clone(),
                sample_id: "sample_primary".to_string(),
                stage_id: check.stage_id.clone(),
                status: status.as_str().to_string(),
                dependency_count: dag_node.dependency_count,
                depends_on: dag_node.depends_on.clone(),
                start_second: index as u64,
                finish_second: index as u64 + 1,
                duration_seconds: 1,
            }
        })
        .collect::<Vec<_>>();

    let completion_rules_proven = proves_completion_contract(&completion_checks);

    if !completion_rules_proven {
        return Err(anyhow!(
            "completion-rules simulation did not prove that completion requires zero exit code, declared outputs, and a result manifest"
        ));
    }

    let simulated_makespan_seconds = nodes.iter().map(|node| node.finish_second).max().unwrap_or(0);

    Ok(LocalDagWatchdogSimulationReport {
        schema_version: LOCAL_DAG_WATCHDOG_SIMULATION_SCHEMA_VERSION,
        scenario: LocalDagWatchdogScenario::CompletionRules.as_str().to_string(),
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
        reused_valid_node_ids: Vec::new(),
        invalid_node_ids: Vec::new(),
        missing_node_ids: Vec::new(),
        planned_node_ids: Vec::new(),
        partial_resume_proven: false,
        completion_check_stage_id: Some(COMPLETION_RULES_STAGE_ID.to_string()),
        completion_checks,
        completion_rules_proven,
        nodes,
    })
}
fn completion_cases() -> Vec<LocalDagWatchdogCompletionCheck> {
    let completion_checks = vec![
        LocalDagWatchdogCompletionCheck {
            case_id: "zero_exit_missing_outputs_and_manifest".to_string(),
            node_id: format!(
                "sample_primary::{}::{}",
                COMPLETION_RULES_STAGE_ID, "zero_exit_missing_outputs_and_manifest"
            ),
            stage_id: COMPLETION_RULES_STAGE_ID.to_string(),
            exit_code: 0,
            declared_outputs_exist: false,
            result_manifest_exists: false,
            complete: false,
        },
        LocalDagWatchdogCompletionCheck {
            case_id: "zero_exit_outputs_only".to_string(),
            node_id: format!(
                "sample_primary::{}::{}",
                COMPLETION_RULES_STAGE_ID, "zero_exit_outputs_only"
            ),
            stage_id: COMPLETION_RULES_STAGE_ID.to_string(),
            exit_code: 0,
            declared_outputs_exist: true,
            result_manifest_exists: false,
            complete: false,
        },
        LocalDagWatchdogCompletionCheck {
            case_id: "zero_exit_manifest_only".to_string(),
            node_id: format!(
                "sample_primary::{}::{}",
                COMPLETION_RULES_STAGE_ID, "zero_exit_manifest_only"
            ),
            stage_id: COMPLETION_RULES_STAGE_ID.to_string(),
            exit_code: 0,
            declared_outputs_exist: false,
            result_manifest_exists: true,
            complete: false,
        },
        LocalDagWatchdogCompletionCheck {
            case_id: "nonzero_exit_with_outputs_and_manifest".to_string(),
            node_id: format!(
                "sample_primary::{}::{}",
                COMPLETION_RULES_STAGE_ID, "nonzero_exit_with_outputs_and_manifest"
            ),
            stage_id: COMPLETION_RULES_STAGE_ID.to_string(),
            exit_code: 17,
            declared_outputs_exist: true,
            result_manifest_exists: true,
            complete: false,
        },
        LocalDagWatchdogCompletionCheck {
            case_id: "zero_exit_outputs_and_manifest".to_string(),
            node_id: format!(
                "sample_primary::{}::{}",
                COMPLETION_RULES_STAGE_ID, "zero_exit_outputs_and_manifest"
            ),
            stage_id: COMPLETION_RULES_STAGE_ID.to_string(),
            exit_code: 0,
            declared_outputs_exist: true,
            result_manifest_exists: true,
            complete: true,
        },
    ];
    completion_checks
}
fn proves_completion_contract(completion_checks: &[LocalDagWatchdogCompletionCheck]) -> bool {
    let completion_rules_proven = completion_checks.iter().all(|check| {
        check.complete
            == (check.exit_code == 0
                && check.declared_outputs_exist
                && check.result_manifest_exists)
    }) && completion_checks
        .iter()
        .any(|check| check.exit_code == 0 && !check.declared_outputs_exist && !check.complete)
        && completion_checks.iter().any(|check| {
            check.exit_code == 0
                && check.declared_outputs_exist
                && !check.result_manifest_exists
                && !check.complete
        })
        && completion_checks.iter().any(|check| {
            check.exit_code != 0
                && check.declared_outputs_exist
                && check.result_manifest_exists
                && !check.complete
        })
        && completion_checks.iter().any(|check| {
            check.exit_code == 0
                && check.declared_outputs_exist
                && check.result_manifest_exists
                && check.complete
        });
    completion_rules_proven
}
