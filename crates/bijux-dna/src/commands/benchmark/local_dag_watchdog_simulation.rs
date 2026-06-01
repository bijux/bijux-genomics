use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_pipeline_dag::{
    validate_pipeline_dag_path, DEFAULT_FASTQ_CORE_PREPROCESS_PIPELINE_CONFIG_PATH,
};

pub(crate) const DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH: &str =
    "target/local-ready/dag-sim/no-global-wait.json";
const LOCAL_DAG_WATCHDOG_SIMULATION_SCHEMA_VERSION: &str =
    "bijux.bench.local_dag_watchdog_simulation.v1";
const FASTQ_CORE_PREPROCESS_PIPELINE_REPORT_PATH: &str =
    "target/local-ready/pipeline-dag/fastq-core-preprocess.json";
const NO_GLOBAL_WAIT_SLOW_BRANCH_STAGE_ID: &str = "fastq.profile_read_lengths";

#[derive(Debug, Clone, Copy)]
pub(crate) enum LocalDagWatchdogScenario {
    NoGlobalWait,
}

impl LocalDagWatchdogScenario {
    fn as_str(self) -> &'static str {
        match self {
            Self::NoGlobalWait => "no_global_wait",
        }
    }

    fn default_output_relative_path(self) -> &'static str {
        match self {
            Self::NoGlobalWait => DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalDagWatchdogSimulationNode {
    pub(crate) node_id: String,
    pub(crate) stage_id: String,
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
    pub(crate) simulated_makespan_seconds: u64,
    pub(crate) slow_branch_stage_id: String,
    pub(crate) slow_branch_finish_second: u64,
    pub(crate) ready_while_slow_branch_running_stage_ids: Vec<String>,
    pub(crate) no_global_wait_proven: bool,
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
    };

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
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
            stage_id: node.stage_id.clone(),
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
        simulated_makespan_seconds,
        slow_branch_stage_id: NO_GLOBAL_WAIT_SLOW_BRANCH_STAGE_ID.to_string(),
        slow_branch_finish_second,
        ready_while_slow_branch_running_stage_ids,
        no_global_wait_proven,
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

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}
