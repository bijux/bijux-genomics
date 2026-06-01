use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_pipeline_dag::{
    validate_pipeline_dag_path, DEFAULT_FASTQ_CORE_PREPROCESS_PIPELINE_CONFIG_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH: &str =
    "target/local-ready/dag-sim/no-global-wait.json";
pub(crate) const DEFAULT_FAILURE_ISOLATION_REPORT_PATH: &str =
    "target/local-ready/dag-sim/failure-isolation.json";
const LOCAL_DAG_WATCHDOG_SIMULATION_SCHEMA_VERSION: &str =
    "bijux.bench.local_dag_watchdog_simulation.v1";
const FASTQ_CORE_PREPROCESS_PIPELINE_REPORT_PATH: &str =
    "target/local-ready/pipeline-dag/fastq-core-preprocess.json";
const NO_GLOBAL_WAIT_SLOW_BRANCH_STAGE_ID: &str = "fastq.profile_read_lengths";
const FAILURE_ISOLATION_FAILED_SAMPLE_ID: &str = "sample_alpha";
const FAILURE_ISOLATION_CONTINUED_SAMPLE_ID: &str = "sample_beta";
const FAILURE_ISOLATION_FAILED_STAGE_ID: &str = "fastq.detect_adapters";

#[derive(Debug, Clone, Copy)]
pub(crate) enum LocalDagWatchdogScenario {
    NoGlobalWait,
    FailureIsolation,
}

impl LocalDagWatchdogScenario {
    fn as_str(self) -> &'static str {
        match self {
            Self::NoGlobalWait => "no_global_wait",
            Self::FailureIsolation => "failure_isolation",
        }
    }

    fn default_output_relative_path(self) -> &'static str {
        match self {
            Self::NoGlobalWait => DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH,
            Self::FailureIsolation => DEFAULT_FAILURE_ISOLATION_REPORT_PATH,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum LocalDagWatchdogNodeStatus {
    Completed,
    Failed,
    Blocked,
}

impl LocalDagWatchdogNodeStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalDagWatchdogSimulationNode {
    pub(crate) node_id: String,
    pub(crate) sample_id: String,
    pub(crate) stage_id: String,
    pub(crate) status: String,
    pub(crate) dependency_count: usize,
    pub(crate) depends_on: Vec<String>,
    pub(crate) start_second: u64,
    pub(crate) finish_second: u64,
    pub(crate) duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalDagWatchdogSimulationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) scenario: String,
    pub(crate) config_path: String,
    pub(crate) dag_report_path: String,
    pub(crate) output_path: String,
    pub(crate) pipeline_id: String,
    pub(crate) node_count: usize,
    pub(crate) sample_count: usize,
    pub(crate) simulated_makespan_seconds: u64,
    pub(crate) slow_branch_stage_id: String,
    pub(crate) slow_branch_finish_second: u64,
    pub(crate) ready_while_slow_branch_running_stage_ids: Vec<String>,
    pub(crate) no_global_wait_proven: bool,
    pub(crate) failed_sample_id: Option<String>,
    pub(crate) failed_stage_id: Option<String>,
    pub(crate) failure_second: Option<u64>,
    pub(crate) continued_unrelated_node_ids: Vec<String>,
    pub(crate) blocked_node_ids: Vec<String>,
    pub(crate) failure_isolation_proven: bool,
    pub(crate) nodes: Vec<LocalDagWatchdogSimulationNode>,
}

pub(crate) fn simulate_dag_watchdog_path(
    repo_root: &Path,
    scenario: LocalDagWatchdogScenario,
    output_path: &Path,
) -> Result<LocalDagWatchdogSimulationReport> {
    let config_path = repo_root.join(DEFAULT_FASTQ_CORE_PREPROCESS_PIPELINE_CONFIG_PATH);
    let dag_report_path = repo_root.join(FASTQ_CORE_PREPROCESS_PIPELINE_REPORT_PATH);
    let dag_report = validate_pipeline_dag_path(repo_root, &config_path, &dag_report_path)?;

    let report = match scenario {
        LocalDagWatchdogScenario::NoGlobalWait => {
            build_no_global_wait_report(repo_root, &config_path, &dag_report_path, output_path, &dag_report)?
        }
        LocalDagWatchdogScenario::FailureIsolation => build_failure_isolation_report(
            repo_root,
            &config_path,
            &dag_report_path,
            output_path,
            &dag_report,
        )?,
    };

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}

pub(crate) fn run_simulate_dag_watchdog(
    args: &parse::BenchLocalSimulateDagWatchdogArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let scenario = match args.scenario {
        parse::BenchLocalDagWatchdogScenarioArg::NoGlobalWait => {
            LocalDagWatchdogScenario::NoGlobalWait
        }
        parse::BenchLocalDagWatchdogScenarioArg::FailureIsolation => {
            LocalDagWatchdogScenario::FailureIsolation
        }
    };
    let output_path = match &args.output {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(scenario.default_output_relative_path()),
    };
    let report = simulate_dag_watchdog_path(&repo_root, scenario, &output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

fn build_no_global_wait_report(
    repo_root: &Path,
    config_path: &Path,
    dag_report_path: &Path,
    output_path: &Path,
    dag_report: &crate::commands::benchmark::local_pipeline_dag::LocalPipelineDagValidationReport,
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
        .any(|stage_id| stage_id == "fastq.trim_reads")
        && ready_while_slow_branch_running_stage_ids
            .iter()
            .any(|stage_id| stage_id == "fastq.filter_reads");

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
        nodes,
    })
}

fn build_failure_isolation_report(
    repo_root: &Path,
    config_path: &Path,
    dag_report_path: &Path,
    output_path: &Path,
    dag_report: &crate::commands::benchmark::local_pipeline_dag::LocalPipelineDagValidationReport,
) -> Result<LocalDagWatchdogSimulationReport> {
    let sample_ids = [
        FAILURE_ISOLATION_FAILED_SAMPLE_ID,
        FAILURE_ISOLATION_CONTINUED_SAMPLE_ID,
    ];
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
            let dependency_failed = dependency_statuses
                .iter()
                .any(|status| matches!(status, LocalDagWatchdogNodeStatus::Failed | LocalDagWatchdogNodeStatus::Blocked));

            let (status, start_second, finish_second, duration_seconds) =
                if sample_id == FAILURE_ISOLATION_FAILED_SAMPLE_ID
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

    let Some(failed_node) = nodes.iter().find(|node| {
        node.sample_id == FAILURE_ISOLATION_FAILED_SAMPLE_ID
            && node.stage_id == FAILURE_ISOLATION_FAILED_STAGE_ID
            && node.status == LocalDagWatchdogNodeStatus::Failed.as_str()
    }) else {
        return Err(anyhow!(
            "failure-isolation scenario did not produce the injected failed node"
        ));
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
        && blocked_node_ids
            .iter()
            .any(|node_id| node_id == "sample_alpha::fastq.trim_reads");

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
        nodes,
    })
}

fn no_global_wait_duration_seconds(stage_id: &str) -> u64 {
    match stage_id {
        "fastq.validate_reads" => 1,
        "fastq.profile_read_lengths" => 12,
        "fastq.detect_adapters" => 2,
        "fastq.trim_reads" => 1,
        "fastq.filter_reads" => 1,
        "fastq.profile_reads" => 1,
        "fastq.report_qc" => 1,
        _ => 1,
    }
}

fn failure_isolation_duration_seconds(stage_id: &str) -> u64 {
    match stage_id {
        "fastq.validate_reads" => 1,
        "fastq.profile_read_lengths" => 6,
        "fastq.detect_adapters" => 2,
        "fastq.trim_reads" => 1,
        "fastq.filter_reads" => 1,
        "fastq.profile_reads" => 1,
        "fastq.report_qc" => 1,
        _ => 1,
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        simulate_dag_watchdog_path, LocalDagWatchdogScenario, DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH,
    };

    fn repo_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn no_global_wait_simulation_proves_ready_nodes_do_not_wait_for_a_slow_branch() {
        let repo_root = repo_root();
        let output_path = repo_root.join(DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH);
        let report = simulate_dag_watchdog_path(
            &repo_root,
            LocalDagWatchdogScenario::NoGlobalWait,
            &output_path,
        )
        .expect("simulate no-global-wait watchdog report");

        assert_eq!(report.scenario, "no_global_wait");
        assert_eq!(report.pipeline_id, "fastq-core-preprocess");
        assert_eq!(report.node_count, 7);
        assert_eq!(report.slow_branch_stage_id, "fastq.profile_read_lengths");
        assert_eq!(report.slow_branch_finish_second, 13);
        assert!(report.no_global_wait_proven);
        assert!(
            report
                .ready_while_slow_branch_running_stage_ids
                .iter()
                .any(|stage_id| stage_id == "fastq.trim_reads"),
            "trim_reads must be allowed to start while the slow profiling branch is still running"
        );
        assert!(
            report
                .ready_while_slow_branch_running_stage_ids
                .iter()
                .any(|stage_id| stage_id == "fastq.filter_reads"),
            "filter_reads must stay unblocked by the unrelated slow profiling branch"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.trim_reads"
                    && node.start_second == 3
                    && node.finish_second == 4
            }),
            "trim_reads should start as soon as validate_reads and detect_adapters are done"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.profile_read_lengths"
                    && node.start_second == 1
                    && node.finish_second == 13
            }),
            "profile_read_lengths must remain the intentionally slow branch in the simulation"
        );
    }
}
