use bijux_dna_core::ids::{StageId, ToolId};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolIntegrationLevel {
    GovernedContract,
    PlannedContract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageToolBinding {
    pub stage_id: StageId,
    pub tool_id: ToolId,
    pub integration_level: ToolIntegrationLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkScenario {
    pub scenario_id: String,
    pub stage_id: StageId,
    pub description: String,
    pub fairness_rules: Vec<String>,
    pub cohort_artifact_id: String,
    pub comparison_artifact_id: String,
    pub normalization_artifact_id: String,
}
