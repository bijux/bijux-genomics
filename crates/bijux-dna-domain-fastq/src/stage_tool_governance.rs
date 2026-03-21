use bijux_dna_core::ids::{StageId, ToolId};

use crate::comparison_contract::comparison_contract_for_stage;
use crate::execution_support::{
    execution_support_for_stage, BenchmarkSupport, ExecutionStatus, NormalizationSupport,
    PlanningSupport, RuntimeSupport,
};
use crate::integration_matrix::{
    benchmark_scenarios_for_stage, stage_tool_binding, stage_tool_bindings, ToolIntegrationLevel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageToolGovernanceProfile {
    pub stage_id: StageId,
    pub tool_id: ToolId,
    pub integration_level: ToolIntegrationLevel,
    pub execution_status: Option<ExecutionStatus>,
    pub planning_support: Option<PlanningSupport>,
    pub runtime_support: Option<RuntimeSupport>,
    pub normalization_support: Option<NormalizationSupport>,
    pub benchmark_support: Option<BenchmarkSupport>,
    pub default_tool: bool,
    pub admitted_runtime_tool: bool,
    pub benchmark_scenario_ids: Vec<String>,
    pub comparison_input_artifact_ids: Vec<String>,
    pub comparison_artifact_ids: Vec<String>,
}

impl StageToolGovernanceProfile {
    #[must_use]
    pub fn is_plannable(self: &Self) -> bool {
        self.integration_level == ToolIntegrationLevel::GovernedContract
            && self.planning_support == Some(PlanningSupport::StageFamily)
    }

    #[must_use]
    pub fn is_runnable(self: &Self) -> bool {
        self.integration_level == ToolIntegrationLevel::GovernedContract
            && self.runtime_support == Some(RuntimeSupport::Runnable)
            && self.admitted_runtime_tool
    }

    #[must_use]
    pub fn has_governed_benchmark_contract(self: &Self) -> bool {
        !self.benchmark_scenario_ids.is_empty()
            && !self.comparison_input_artifact_ids.is_empty()
            && !self.comparison_artifact_ids.is_empty()
    }
}

#[must_use]
pub fn stage_tool_governance_profile(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Option<StageToolGovernanceProfile> {
    let binding = stage_tool_binding(stage_id, tool_id)?;
    let support = execution_support_for_stage(stage_id);
    let comparison_contract = comparison_contract_for_stage(stage_id);
    let mut benchmark_scenario_ids = benchmark_scenarios_for_stage(stage_id)
        .into_iter()
        .map(|scenario| scenario.scenario_id)
        .collect::<Vec<_>>();
    benchmark_scenario_ids.sort();
    benchmark_scenario_ids.dedup();

    let mut comparison_input_artifact_ids = comparison_contract
        .as_ref()
        .map(|contract| {
            contract
                .comparison_input_artifact_ids
                .iter()
                .map(|artifact_id| (*artifact_id).to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    comparison_input_artifact_ids.sort();
    comparison_input_artifact_ids.dedup();

    let mut comparison_artifact_ids = comparison_contract
        .as_ref()
        .map(|contract| {
            vec![
                contract.cohort_artifact_id.to_string(),
                contract.comparison_artifact_id.to_string(),
                contract.normalization_artifact_id.to_string(),
            ]
        })
        .unwrap_or_default();
    comparison_artifact_ids.sort();
    comparison_artifact_ids.dedup();

    Some(StageToolGovernanceProfile {
        stage_id: stage_id.clone(),
        tool_id: tool_id.clone(),
        integration_level: binding.integration_level,
        execution_status: support.as_ref().map(|record| record.execution_status),
        planning_support: support.as_ref().map(|record| record.planning_support),
        runtime_support: support.as_ref().map(|record| record.runtime_support),
        normalization_support: support.as_ref().map(|record| record.normalization_support),
        benchmark_support: support.as_ref().map(|record| record.benchmark_support),
        default_tool: support
            .as_ref()
            .and_then(|record| record.default_tool.as_ref())
            == Some(tool_id),
        admitted_runtime_tool: support
            .as_ref()
            .map(|record| record.admitted_tools.iter().any(|candidate| candidate == tool_id))
            .unwrap_or(false),
        benchmark_scenario_ids,
        comparison_input_artifact_ids,
        comparison_artifact_ids,
    })
}

#[must_use]
pub fn stage_tool_governance_profiles_for_stage(stage_id: &StageId) -> Vec<StageToolGovernanceProfile> {
    stage_tool_bindings()
        .into_iter()
        .filter(|binding| binding.stage_id == *stage_id)
        .filter_map(|binding| stage_tool_governance_profile(&binding.stage_id, &binding.tool_id))
        .collect()
}
