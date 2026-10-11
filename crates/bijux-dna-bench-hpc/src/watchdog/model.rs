use serde::Serialize;
#[derive(Debug, Clone, Serialize)]
pub struct LocalDagWatchdogSimulationNode {
    pub node_id: String,
    pub sample_id: String,
    pub stage_id: String,
    pub status: String,
    pub dependency_count: usize,
    pub depends_on: Vec<String>,
    pub start_second: u64,
    pub finish_second: u64,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalDagWatchdogCompletionCheck {
    pub case_id: String,
    pub node_id: String,
    pub stage_id: String,
    pub exit_code: i32,
    pub declared_outputs_exist: bool,
    pub result_manifest_exists: bool,
    pub complete: bool,
}

#[derive(Debug, Clone, Serialize)]
// The persisted schema records four independent scenario verdicts.
#[allow(clippy::struct_excessive_bools)]
pub struct LocalDagWatchdogSimulationReport {
    pub schema_version: &'static str,
    pub scenario: String,
    pub config_path: String,
    pub dag_report_path: String,
    pub output_path: String,
    pub pipeline_id: String,
    pub node_count: usize,
    pub sample_count: usize,
    pub simulated_makespan_seconds: u64,
    pub slow_branch_stage_id: String,
    pub slow_branch_finish_second: u64,
    pub ready_while_slow_branch_running_stage_ids: Vec<String>,
    pub no_global_wait_proven: bool,
    pub failed_sample_id: Option<String>,
    pub failed_stage_id: Option<String>,
    pub failure_second: Option<u64>,
    pub continued_unrelated_node_ids: Vec<String>,
    pub blocked_node_ids: Vec<String>,
    pub failure_isolation_proven: bool,
    pub reused_valid_node_ids: Vec<String>,
    pub invalid_node_ids: Vec<String>,
    pub missing_node_ids: Vec<String>,
    pub planned_node_ids: Vec<String>,
    pub partial_resume_proven: bool,
    pub completion_check_stage_id: Option<String>,
    pub completion_checks: Vec<LocalDagWatchdogCompletionCheck>,
    pub completion_rules_proven: bool,
    pub nodes: Vec<LocalDagWatchdogSimulationNode>,
}
