use bijux_dna_core::ids::{StageId, ToolId};

use crate::comparison_contract::comparison_contract_for_stage;
use crate::execution_support::{
    execution_support_for_stage, BenchmarkSupport, NormalizationSupport, PlanningSupport,
    RuntimeSupport,
};
use crate::integration_matrix::{
    benchmark_scenarios_for_stage, stage_tool_binding, stage_tool_bindings, ToolIntegrationLevel,
};

use super::model::{
    StageBenchmarkGovernance, StageToolBenchmarkContractMaturity, StageToolGovernanceProfile,
    StageToolNormalizationMaturity,
};

impl StageBenchmarkGovernance {
    #[must_use]
    pub fn has_governed_benchmark_contract(&self) -> bool {
        !self.scenarios.is_empty()
            && !self.comparison_input_artifact_ids.is_empty()
            && !self.comparison_artifact_ids.is_empty()
    }
}

impl StageToolGovernanceProfile {
    #[must_use]
    pub fn is_plannable(&self) -> bool {
        self.integration_level == ToolIntegrationLevel::GovernedContract
            && self.planning_support == Some(PlanningSupport::StageFamily)
    }

    #[must_use]
    pub fn is_runnable(&self) -> bool {
        self.integration_level == ToolIntegrationLevel::GovernedContract
            && self.runtime_support == Some(RuntimeSupport::Runnable)
            && self.admitted_runtime_tool
    }

    #[must_use]
    pub fn has_governed_benchmark_contract(&self) -> bool {
        stage_benchmark_governance(&self.stage_id)
            .is_some_and(|governance| governance.has_governed_benchmark_contract())
    }

    #[must_use]
    pub fn normalization_maturity(&self) -> StageToolNormalizationMaturity {
        if !self.is_runnable() {
            return StageToolNormalizationMaturity::None;
        }
        match self.normalization_support {
            Some(NormalizationSupport::None) | None => StageToolNormalizationMaturity::None,
            Some(NormalizationSupport::GenericEnvelope) => {
                StageToolNormalizationMaturity::GenericEnvelope
            }
            Some(NormalizationSupport::ObserverSpecialized | NormalizationSupport::Mixed) => {
                StageToolNormalizationMaturity::ObserverSpecialized
            }
        }
    }

    #[must_use]
    pub fn benchmark_contract_maturity(&self) -> StageToolBenchmarkContractMaturity {
        if !self.is_runnable() || !self.has_governed_benchmark_contract() {
            return StageToolBenchmarkContractMaturity::None;
        }
        match self.benchmark_support {
            Some(BenchmarkSupport::Comparable | BenchmarkSupport::Mixed) => {
                StageToolBenchmarkContractMaturity::BenchmarkComparable
            }
            Some(BenchmarkSupport::Cohort) => {
                StageToolBenchmarkContractMaturity::GovernedBenchmarkCohort
            }
            Some(BenchmarkSupport::None) | None => StageToolBenchmarkContractMaturity::None,
        }
    }
}

#[must_use]
pub fn stage_benchmark_governance(stage_id: &StageId) -> Option<StageBenchmarkGovernance> {
    let support = execution_support_for_stage(stage_id)?;
    let comparison_contract = comparison_contract_for_stage(stage_id);
    let mut scenarios = benchmark_scenarios_for_stage(stage_id);
    scenarios.sort_by(|left, right| left.scenario_id.cmp(&right.scenario_id));
    scenarios.dedup_by(|left, right| left.scenario_id == right.scenario_id);

    let comparison_input_artifact_ids = comparison_contract
        .as_ref()
        .map(|contract| {
            contract
                .comparison_input_artifact_ids
                .iter()
                .map(|artifact_id| (*artifact_id).clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let comparison_artifact_ids = comparison_contract
        .as_ref()
        .map(|contract| {
            vec![
                contract.cohort_artifact_id.clone(),
                contract.comparison_artifact_id.clone(),
                contract.normalization_artifact_id.clone(),
            ]
        })
        .unwrap_or_default();

    Some(StageBenchmarkGovernance {
        stage_id: stage_id.clone(),
        execution_status: Some(support.execution_status),
        benchmark_support: Some(support.benchmark_support),
        scenarios,
        comparison_input_artifact_ids,
        comparison_artifact_ids,
    })
}

#[must_use]
pub fn stage_tool_governance_profile(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Option<StageToolGovernanceProfile> {
    let binding = stage_tool_binding(stage_id, tool_id)?;
    let support = execution_support_for_stage(stage_id);
    let benchmark_governance = stage_benchmark_governance(stage_id);

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
        admitted_runtime_tool: support.as_ref().is_some_and(|record| {
            record
                .admitted_tools
                .iter()
                .any(|candidate| candidate == tool_id)
        }),
        benchmark_scenario_ids: benchmark_governance
            .as_ref()
            .map(|governance| {
                governance
                    .scenarios
                    .iter()
                    .map(|scenario| scenario.scenario_id.clone())
                    .collect()
            })
            .unwrap_or_default(),
        comparison_input_artifact_ids: benchmark_governance
            .as_ref()
            .map(|governance| governance.comparison_input_artifact_ids.clone())
            .unwrap_or_default(),
        comparison_artifact_ids: benchmark_governance
            .as_ref()
            .map(|governance| governance.comparison_artifact_ids.clone())
            .unwrap_or_default(),
    })
}

#[must_use]
pub fn stage_tool_governance_profiles_for_stage(
    stage_id: &StageId,
) -> Vec<StageToolGovernanceProfile> {
    stage_tool_bindings()
        .into_iter()
        .filter(|binding| binding.stage_id == *stage_id)
        .filter_map(|binding| stage_tool_governance_profile(&binding.stage_id, &binding.tool_id))
        .collect()
}
