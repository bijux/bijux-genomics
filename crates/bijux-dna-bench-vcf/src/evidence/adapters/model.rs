use serde::Serialize;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfAdaptersReadyGoalCheck {
    pub goal_id: u32,
    pub surface: String,
    pub output_path: Option<String>,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfAdaptersReadyReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub checked_goal_count: usize,
    pub passed_goal_count: usize,
    pub failed_goal_count: usize,
    pub failing_goal_ids: Vec<u32>,
    pub benchmark_ready_pair_count: usize,
    pub adapter_complete_pair_count: usize,
    pub output_complete_pair_count: usize,
    pub rendered_command_pair_count: usize,
    pub ok: bool,
    pub checks: Vec<VcfAdaptersReadyGoalCheck>,
}
