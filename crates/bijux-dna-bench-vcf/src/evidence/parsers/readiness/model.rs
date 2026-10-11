use serde::Serialize;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfParsersReportReadyGoalCheck {
    pub goal_id: u32,
    pub surface: String,
    pub output_path: Option<String>,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfParsersReportReadyReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub checked_goal_count: usize,
    pub passed_goal_count: usize,
    pub failed_goal_count: usize,
    pub failing_goal_ids: Vec<u32>,
    pub parser_fixture_row_count: usize,
    pub active_parser_fixture_row_count: usize,
    pub expected_result_row_count: usize,
    pub report_map_row_count: usize,
    pub comparable_metric_row_count: usize,
    pub ok: bool,
    pub checks: Vec<VcfParsersReportReadyGoalCheck>,
}
