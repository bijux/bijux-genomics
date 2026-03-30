use bijux_dna_core::ids::{StageId, ToolId};

use crate::execution_support::{
    BenchmarkSupport, ExecutionStatus, NormalizationSupport, PlanningSupport, RuntimeSupport,
};
use crate::integration_matrix::{BenchmarkScenario, ToolIntegrationLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeNormalizationLevel {
    GenericEnvelope,
    ObserverSpecialized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageToolNormalizationMaturity {
    None,
    GenericEnvelope,
    ObserverSpecialized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageToolBenchmarkContractMaturity {
    None,
    GovernedBenchmarkCohort,
    BenchmarkComparable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkReadinessLevel {
    PlannedContract,
    GovernedExecution,
    GovernedBenchmarkCohort,
    ObserverSpecializedBenchmark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageToolMaturityLevel {
    PlannedBinding,
    GovernedExecution,
    GenericNormalized,
    ObserverNormalized,
    BenchmarkComparable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageBenchmarkGovernance {
    pub stage_id: StageId,
    pub execution_status: Option<ExecutionStatus>,
    pub benchmark_support: Option<BenchmarkSupport>,
    pub scenarios: Vec<BenchmarkScenario>,
    pub comparison_input_artifact_ids: Vec<String>,
    pub comparison_artifact_ids: Vec<String>,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct StageToolCapabilityContract {
    pub stage_id: StageId,
    pub tool_id: ToolId,
    pub integration_level: ToolIntegrationLevel,
    pub execution_status: Option<ExecutionStatus>,
    pub benchmark_scenario_ids: Vec<String>,
    pub declared: bool,
    pub plannable: bool,
    pub runnable: bool,
    pub parse_normalized: bool,
    pub benchmark_normalized: bool,
    pub comparable: bool,
}
