use bijux_dna_core::ids::{StageId, ToolId};

use super::model::{
    BenchmarkReadinessLevel, RuntimeNormalizationLevel, StageToolBenchmarkContractMaturity,
    StageToolCapabilityContract, StageToolMaturityLevel, StageToolNormalizationMaturity,
};
use super::profiles::stage_tool_governance_profile;

#[must_use]
pub fn stage_tool_capability_contract(
    stage_id: &StageId,
    tool_id: &ToolId,
    runtime_normalization: RuntimeNormalizationLevel,
) -> Option<StageToolCapabilityContract> {
    let governance = stage_tool_governance_profile(stage_id, tool_id)?;
    let plannable = governance.is_plannable();
    let runnable = governance.is_runnable();
    let parse_normalized = match governance.normalization_maturity() {
        StageToolNormalizationMaturity::None => false,
        StageToolNormalizationMaturity::GenericEnvelope => true,
        StageToolNormalizationMaturity::ObserverSpecialized => {
            runtime_normalization == RuntimeNormalizationLevel::ObserverSpecialized
        }
    };
    let benchmark_contract_maturity = governance.benchmark_contract_maturity();
    let benchmark_normalized = parse_normalized
        && runtime_normalization == RuntimeNormalizationLevel::ObserverSpecialized
        && matches!(
            benchmark_contract_maturity,
            StageToolBenchmarkContractMaturity::GovernedBenchmarkCohort
                | StageToolBenchmarkContractMaturity::BenchmarkComparable
        );
    let comparable = parse_normalized
        && runtime_normalization == RuntimeNormalizationLevel::ObserverSpecialized
        && benchmark_contract_maturity == StageToolBenchmarkContractMaturity::BenchmarkComparable;

    Some(StageToolCapabilityContract {
        stage_id: governance.stage_id,
        tool_id: governance.tool_id,
        integration_level: governance.integration_level,
        execution_status: governance.execution_status,
        benchmark_scenario_ids: governance.benchmark_scenario_ids,
        declared: true,
        plannable,
        runnable,
        parse_normalized,
        benchmark_normalized,
        comparable,
    })
}

#[must_use]
pub fn benchmark_readiness_for_stage_tool(
    stage_id: &StageId,
    tool_id: &ToolId,
    runtime_normalization: RuntimeNormalizationLevel,
) -> Option<BenchmarkReadinessLevel> {
    let capability = stage_tool_capability_contract(stage_id, tool_id, runtime_normalization)?;
    Some(if !capability.runnable {
        BenchmarkReadinessLevel::PlannedContract
    } else if capability.comparable {
        BenchmarkReadinessLevel::ObserverSpecializedBenchmark
    } else if capability.benchmark_normalized {
        BenchmarkReadinessLevel::GovernedBenchmarkCohort
    } else {
        BenchmarkReadinessLevel::GovernedExecution
    })
}

#[must_use]
pub fn stage_tool_maturity(
    stage_id: &StageId,
    tool_id: &ToolId,
    runtime_normalization: RuntimeNormalizationLevel,
) -> Option<StageToolMaturityLevel> {
    let capability = stage_tool_capability_contract(stage_id, tool_id, runtime_normalization)?;
    Some(if !capability.runnable {
        StageToolMaturityLevel::PlannedBinding
    } else if capability.comparable {
        StageToolMaturityLevel::BenchmarkComparable
    } else if runtime_normalization == RuntimeNormalizationLevel::ObserverSpecialized {
        StageToolMaturityLevel::ObserverNormalized
    } else if capability.benchmark_normalized {
        StageToolMaturityLevel::GenericNormalized
    } else {
        StageToolMaturityLevel::GovernedExecution
    })
}
