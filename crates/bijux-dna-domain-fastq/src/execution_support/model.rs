use bijux_dna_core::ids::{StageId, ToolId};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Closed,
    DeclaredOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanningSupport {
    DeclaredOnly,
    StageFamily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSupport {
    DeclaredOnly,
    Runnable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizationSupport {
    None,
    GenericEnvelope,
    ObserverSpecialized,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkSupport {
    None,
    Cohort,
    Comparable,
    Mixed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageExecutionSupport {
    pub stage_id: StageId,
    pub execution_status: ExecutionStatus,
    pub planning_support: PlanningSupport,
    pub runtime_support: RuntimeSupport,
    pub normalization_support: NormalizationSupport,
    pub benchmark_support: BenchmarkSupport,
    pub default_tool: Option<ToolId>,
    pub admitted_tools: Vec<ToolId>,
}

impl StageExecutionSupport {
    #[must_use]
    pub fn is_plannable(&self) -> bool {
        self.planning_support == PlanningSupport::StageFamily
    }

    #[must_use]
    pub fn is_runnable(&self) -> bool {
        self.runtime_support == RuntimeSupport::Runnable
    }

    #[must_use]
    pub fn has_generic_normalization(&self) -> bool {
        matches!(
            self.normalization_support,
            NormalizationSupport::GenericEnvelope | NormalizationSupport::Mixed
        )
    }

    #[must_use]
    pub fn has_observer_specialized_normalization(&self) -> bool {
        matches!(
            self.normalization_support,
            NormalizationSupport::ObserverSpecialized | NormalizationSupport::Mixed
        )
    }

    #[must_use]
    pub fn supports_benchmark_cohorts(&self) -> bool {
        matches!(
            self.benchmark_support,
            BenchmarkSupport::Cohort | BenchmarkSupport::Comparable | BenchmarkSupport::Mixed
        )
    }

    #[must_use]
    pub fn supports_comparable_benchmarks(&self) -> bool {
        matches!(self.benchmark_support, BenchmarkSupport::Comparable | BenchmarkSupport::Mixed)
    }
}
