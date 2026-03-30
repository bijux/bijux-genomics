use bijux_dna_core::ids::{StageId, ToolId};

use super::contract::domain_index_contract;
use super::model::{BenchmarkScenario, StageToolBinding};

#[must_use]
pub fn stage_tool_bindings() -> Vec<StageToolBinding> {
    domain_index_contract()
        .stage_tool_integration
        .iter()
        .flat_map(|(stage_id, bindings)| {
            bindings
                .iter()
                .map(move |(tool_id, integration_level)| StageToolBinding {
                    stage_id: StageId::new(stage_id.clone()),
                    tool_id: ToolId::new(tool_id.clone()),
                    integration_level: *integration_level,
                })
        })
        .collect()
}

#[must_use]
pub fn stage_tool_bindings_for_stage(stage_id: &StageId) -> Vec<StageToolBinding> {
    stage_tool_bindings()
        .into_iter()
        .filter(|binding| binding.stage_id == *stage_id)
        .collect()
}

#[must_use]
pub fn stage_tool_binding(stage_id: &StageId, tool_id: &ToolId) -> Option<StageToolBinding> {
    stage_tool_bindings()
        .into_iter()
        .find(|binding| binding.stage_id == *stage_id && binding.tool_id == *tool_id)
}

#[must_use]
pub fn benchmark_scenarios() -> Vec<BenchmarkScenario> {
    domain_index_contract()
        .benchmark_scenarios
        .iter()
        .map(|(scenario_id, scenario)| BenchmarkScenario {
            scenario_id: scenario_id.clone(),
            stage_id: StageId::new(scenario.stage_id.clone()),
            description: scenario.description.clone(),
            fairness_rules: scenario.fairness_rules.clone(),
            cohort_artifact_id: scenario.cohort_artifact_id.clone(),
            comparison_artifact_id: scenario.comparison_artifact_id.clone(),
            normalization_artifact_id: scenario.normalization_artifact_id.clone(),
        })
        .collect()
}

#[must_use]
pub fn benchmark_scenarios_for_stage(stage_id: &StageId) -> Vec<BenchmarkScenario> {
    benchmark_scenarios()
        .into_iter()
        .filter(|scenario| scenario.stage_id == *stage_id)
        .collect()
}

#[must_use]
pub fn reference_index_backends_for_tool(tool_id: &ToolId) -> Vec<ToolId> {
    domain_index_contract()
        .reference_index_compatibility
        .get(tool_id.as_str())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(ToolId::new)
        .collect()
}

#[must_use]
pub fn is_reference_index_backend_compatible(tool_id: &ToolId, index_tool_id: &ToolId) -> bool {
    reference_index_backends_for_tool(tool_id)
        .into_iter()
        .any(|backend| backend == *index_tool_id)
}
