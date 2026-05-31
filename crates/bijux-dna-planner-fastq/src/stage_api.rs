use bijux_dna_core::ids::{StageId, ToolId};

pub use crate::qc_contract::{
    governed_qc_bench_contributor_stage_ids, governed_qc_default_tool_ids,
    governed_qc_output_ids_for_stage, governed_qc_producer_stage_ids,
};
pub use crate::selection::args;
pub use crate::selection::{allowed_tools_for_stage, default_tool_for_stage};
pub use crate::tool_adapters::fastq;
pub use crate::tool_adapters::fastq::StageInfo;
pub use crate::STAGE_REPORT_AGGREGATE;
pub use crate::TOOL_SEQKIT;
pub use bijux_dna_core::prelude::RawFailure;
pub use bijux_dna_domain_fastq::banks;
pub use bijux_dna_domain_fastq::banks::{
    adapter_bank_context, contaminant_bank_context, polyx_bank_context, polyx_unsupported_warning,
    resolve_adapter_selection, resolve_contaminant_selection, resolve_effective_adapters,
    resolve_effective_contaminants, resolve_effective_polyx, resolve_polyx_selection,
    AdapterSelection, DEFAULT_ADAPTER_PRESET, DEFAULT_CONTAMINANT_PRESET, DEFAULT_POLYX_PRESET,
};
pub use bijux_dna_domain_fastq::ToolIntegrationLevel;
pub use bijux_dna_domain_fastq::{
    benchmark_scenarios_for_stage, ensure_umi_headers, inspect_headers, log_header_warnings,
    preflight_stage, stage_tool_binding, stage_tool_bindings_for_stage, FastqArtifact,
    FastqArtifactKind,
};
pub use bijux_dna_stages_fastq::stage_specs::*;
pub use bijux_dna_stages_fastq::{
    observer_stage_tool_bindings, runtime_interpretation_for_stage_tool, RuntimeInterpretationLevel,
};

pub type StagePlanJson = bijux_dna_stage_contract::StagePlanJsonV1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageToolCapability {
    pub stage_id: StageId,
    pub tool_id: ToolId,
    pub integration_level: ToolIntegrationLevel,
    pub execution_status: Option<bijux_dna_domain_fastq::ExecutionStatus>,
    pub runtime_interpretation: RuntimeInterpretationLevel,
    pub benchmark_scenarios: Vec<String>,
    pub execution: StageToolExecutionCapability,
    pub normalization: StageToolNormalizationCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageToolExecutionCapability {
    pub declared: bool,
    pub plannable: bool,
    pub runnable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageToolNormalizationCapability {
    pub parse_normalized: bool,
    pub benchmark_normalized: bool,
    pub comparable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkReadinessLevel {
    PlannedContract,
    GovernedExecution,
    GovernedBenchmarkCohort,
    ObserverSpecializedBenchmark,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageToolBenchmarkProfile {
    pub stage_id: StageId,
    pub tool_id: ToolId,
    pub integration_level: ToolIntegrationLevel,
    pub runtime_interpretation: RuntimeInterpretationLevel,
    pub benchmark_scenarios: Vec<String>,
    pub readiness: BenchmarkReadinessLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCohort {
    pub scenario_id: String,
    pub stage_id: StageId,
    pub description: String,
    pub fairness_rules: Vec<String>,
    pub tool_ids: Vec<ToolId>,
    pub observer_specialized_tools: Vec<ToolId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolsetExecutionMode {
    DefaultChoice,
    GovernedExecution,
    BenchmarkCohort,
    AllBindings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageToolMaturityLevel {
    PlannedBinding,
    GovernedExecution,
    GenericNormalized,
    ObserverNormalized,
    BenchmarkComparable,
}

fn runtime_contract_levels(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> (RuntimeInterpretationLevel, bijux_dna_domain_fastq::RuntimeNormalizationLevel) {
    let runtime_interpretation = runtime_interpretation_for_stage_tool(stage_id, tool_id)
        .unwrap_or(RuntimeInterpretationLevel::GenericEnvelope);
    let runtime_normalization =
        if runtime_interpretation == RuntimeInterpretationLevel::ObserverSpecialized {
            bijux_dna_domain_fastq::RuntimeNormalizationLevel::ObserverSpecialized
        } else {
            bijux_dna_domain_fastq::RuntimeNormalizationLevel::GenericEnvelope
        };
    (runtime_interpretation, runtime_normalization)
}

#[must_use]
pub fn tool_supports_input_layout(stage_id: &StageId, tool_id: &ToolId, paired_end: bool) -> bool {
    bijux_dna_domain_fastq::tool_supports_input_layout(stage_id, tool_id, paired_end)
}

#[must_use]
pub fn stage_accepts_input_layout(
    stage_id: &StageId,
    layout: bijux_dna_domain_fastq::FastqReadLayout,
) -> bool {
    bijux_dna_domain_fastq::stage_accepts_input_layout(stage_id.as_str(), layout)
}

#[must_use]
pub fn filter_tools_for_input_layout(
    stage_id: &StageId,
    tool_ids: Vec<ToolId>,
    paired_end: bool,
) -> Vec<ToolId> {
    bijux_dna_domain_fastq::filter_tools_for_input_layout(stage_id, tool_ids, paired_end)
}

/// Build the governed local-smoke case plans for
/// `fastq.estimate_library_complexity_prealign`.
///
/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_estimate_library_complexity_prealign_smoke_plans(
    repo_root: &std::path::Path,
) -> anyhow::Result<Vec<crate::LocalEstimateLibraryComplexityPrealignSmokeCasePlan>> {
    crate::planner::local_estimate_library_complexity_prealign_smoke_plans(repo_root)
}

#[must_use]
pub fn stage_tool_capability(stage_id: &StageId, tool_id: &ToolId) -> Option<StageToolCapability> {
    let (runtime_interpretation, runtime_normalization) =
        runtime_contract_levels(stage_id, tool_id);
    let capability = bijux_dna_domain_fastq::stage_tool_capability_contract(
        stage_id,
        tool_id,
        runtime_normalization,
    )?;

    Some(StageToolCapability {
        stage_id: capability.stage_id,
        tool_id: capability.tool_id,
        integration_level: capability.integration_level,
        execution_status: capability.execution_status,
        runtime_interpretation,
        benchmark_scenarios: capability.benchmark_scenario_ids,
        execution: StageToolExecutionCapability {
            declared: capability.declared,
            plannable: capability.plannable,
            runnable: capability.runnable,
        },
        normalization: StageToolNormalizationCapability {
            parse_normalized: capability.parse_normalized,
            benchmark_normalized: capability.benchmark_normalized,
            comparable: capability.comparable,
        },
    })
}

#[must_use]
pub fn stage_tool_capabilities_for_stage(stage_id: &StageId) -> Vec<StageToolCapability> {
    stage_tool_bindings_for_stage(stage_id)
        .into_iter()
        .filter_map(|binding| stage_tool_capability(&binding.stage_id, &binding.tool_id))
        .collect()
}

#[must_use]
pub fn stage_tool_capabilities() -> Vec<StageToolCapability> {
    stage_tool_bindings()
        .into_iter()
        .filter_map(|binding| stage_tool_capability(&binding.stage_id, &binding.tool_id))
        .collect()
}

#[must_use]
pub fn benchmark_profile_for_stage_tool(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Option<StageToolBenchmarkProfile> {
    let capability = stage_tool_capability(stage_id, tool_id)?;
    let (_, runtime_normalization) = runtime_contract_levels(stage_id, tool_id);
    let readiness = bijux_dna_domain_fastq::benchmark_readiness_for_stage_tool(
        stage_id,
        tool_id,
        runtime_normalization,
    )
    .map(|level| match level {
        bijux_dna_domain_fastq::BenchmarkReadinessLevel::PlannedContract => {
            BenchmarkReadinessLevel::PlannedContract
        }
        bijux_dna_domain_fastq::BenchmarkReadinessLevel::GovernedExecution => {
            BenchmarkReadinessLevel::GovernedExecution
        }
        bijux_dna_domain_fastq::BenchmarkReadinessLevel::GovernedBenchmarkCohort => {
            BenchmarkReadinessLevel::GovernedBenchmarkCohort
        }
        bijux_dna_domain_fastq::BenchmarkReadinessLevel::ObserverSpecializedBenchmark => {
            BenchmarkReadinessLevel::ObserverSpecializedBenchmark
        }
    })?;
    Some(StageToolBenchmarkProfile {
        stage_id: capability.stage_id,
        tool_id: capability.tool_id,
        integration_level: capability.integration_level,
        runtime_interpretation: capability.runtime_interpretation,
        benchmark_scenarios: capability.benchmark_scenarios,
        readiness,
    })
}

#[must_use]
pub fn benchmark_profiles_for_stage(stage_id: &StageId) -> Vec<StageToolBenchmarkProfile> {
    stage_tool_bindings_for_stage(stage_id)
        .into_iter()
        .filter_map(|binding| benchmark_profile_for_stage_tool(&binding.stage_id, &binding.tool_id))
        .collect()
}

#[must_use]
pub fn benchmark_cohorts_for_stage(stage_id: &StageId) -> Vec<BenchmarkCohort> {
    let profiles = benchmark_profiles_for_stage(stage_id);
    bijux_dna_domain_fastq::stage_benchmark_governance(stage_id)
        .map(|governance| governance.scenarios)
        .unwrap_or_default()
        .into_iter()
        .map(|scenario| {
            let cohort_profiles = profiles
                .iter()
                .filter(|profile| {
                    profile
                        .benchmark_scenarios
                        .iter()
                        .any(|scenario_id| scenario_id == &scenario.scenario_id)
                        && matches!(
                            profile.readiness,
                            BenchmarkReadinessLevel::GovernedBenchmarkCohort
                                | BenchmarkReadinessLevel::ObserverSpecializedBenchmark
                        )
                })
                .collect::<Vec<_>>();
            let observer_specialized_tools = cohort_profiles
                .iter()
                .filter(|profile| {
                    profile.runtime_interpretation
                        == RuntimeInterpretationLevel::ObserverSpecialized
                })
                .map(|profile| profile.tool_id.clone())
                .collect::<Vec<_>>();
            BenchmarkCohort {
                scenario_id: scenario.scenario_id,
                stage_id: scenario.stage_id,
                description: scenario.description,
                fairness_rules: scenario.fairness_rules,
                tool_ids: cohort_profiles.iter().map(|profile| profile.tool_id.clone()).collect(),
                observer_specialized_tools,
            }
        })
        .collect()
}

#[must_use]
pub fn benchmark_cohort_for_stage_scenario(
    stage_id: &StageId,
    scenario_id: &str,
) -> Option<BenchmarkCohort> {
    benchmark_cohorts_for_stage(stage_id)
        .into_iter()
        .find(|cohort| cohort.scenario_id == scenario_id)
}

#[must_use]
pub fn toolset_for_stage_benchmark_scenario(stage_id: &StageId, scenario_id: &str) -> Vec<ToolId> {
    benchmark_cohort_for_stage_scenario(stage_id, scenario_id)
        .map(|cohort| cohort.tool_ids)
        .unwrap_or_default()
}

#[must_use]
pub fn benchmark_default_scenario_toolset(stage_id: &StageId) -> Vec<ToolId> {
    let mut cohorts = benchmark_cohorts_for_stage(stage_id);
    cohorts.sort_by(|left, right| left.scenario_id.cmp(&right.scenario_id));
    if cohorts.len() != 1 {
        return Vec::new();
    }
    cohorts.remove(0).tool_ids
}

#[must_use]
pub fn toolset_for_stage(stage_id: &StageId, mode: ToolsetExecutionMode) -> Vec<ToolId> {
    match mode {
        ToolsetExecutionMode::DefaultChoice => {
            default_tool_for_stage(stage_id).into_iter().collect()
        }
        ToolsetExecutionMode::GovernedExecution => stage_tool_capabilities_for_stage(stage_id)
            .into_iter()
            .filter(|capability| capability.execution.runnable)
            .map(|capability| capability.tool_id)
            .collect(),
        ToolsetExecutionMode::BenchmarkCohort => benchmark_default_scenario_toolset(stage_id),
        ToolsetExecutionMode::AllBindings => stage_tool_capabilities_for_stage(stage_id)
            .into_iter()
            .map(|capability| capability.tool_id)
            .collect(),
    }
}

/// Build the governed local-ready `fastq.index_reference` plan from repository-owned config.
///
/// # Errors
/// Returns an error if the local-ready config, local runtime profile, governed FASTQ tool YAML,
/// or reference FASTA inputs cannot be resolved into a deterministic `StagePlanV1`.
pub fn local_index_reference_plan(
    repo_root: &std::path::Path,
) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> {
    crate::planner::local_index_reference_plan(repo_root)
}

/// Build the governed local-smoke `fastq.validate_reads` plans from repository-owned config.
///
/// # Errors
/// Returns an error if the local-smoke config, governed FASTQ tool YAML, or local FASTQ fixtures
/// cannot be resolved into deterministic `StagePlanV1` values.
pub fn local_validate_reads_smoke_plans(
    repo_root: &std::path::Path,
) -> anyhow::Result<Vec<crate::planner::LocalValidateReadsSmokeCasePlan>> {
    crate::planner::local_validate_reads_smoke_plans(repo_root)
}

/// Build the governed local-smoke `fastq.detect_adapters` plans from repository-owned config.
///
/// # Errors
/// Returns an error if the local-smoke config, governed FASTQ tool YAML, or local FASTQ fixtures
/// cannot be resolved into deterministic `StagePlanV1` values.
pub fn local_detect_adapters_smoke_plans(
    repo_root: &std::path::Path,
) -> anyhow::Result<Vec<crate::planner::LocalDetectAdaptersSmokeCasePlan>> {
    crate::planner::local_detect_adapters_smoke_plans(repo_root)
}

/// Build the governed local-smoke `fastq.detect_duplicates_premerge` plans from
/// repository-owned config.
///
/// # Errors
/// Returns an error if the local-smoke config, governed FASTQ tool YAML, or local FASTQ fixtures
/// cannot be resolved into deterministic `StagePlanV1` values.
pub fn local_detect_duplicates_premerge_smoke_plans(
    repo_root: &std::path::Path,
) -> anyhow::Result<Vec<crate::planner::LocalDetectDuplicatesPremergeSmokeCasePlan>> {
    crate::planner::local_detect_duplicates_premerge_smoke_plans(repo_root)
}

/// Build the governed local-smoke `fastq.profile_read_lengths` plans from repository-owned
/// config.
///
/// # Errors
/// Returns an error if the local-smoke config, governed FASTQ tool YAML, or local FASTQ fixtures
/// cannot be resolved into deterministic `StagePlanV1` values.
pub fn local_profile_read_lengths_smoke_plans(
    repo_root: &std::path::Path,
) -> anyhow::Result<Vec<crate::planner::LocalProfileReadLengthsSmokeCasePlan>> {
    crate::planner::local_profile_read_lengths_smoke_plans(repo_root)
}

#[must_use]
pub fn stage_tool_maturity(stage_id: &StageId, tool_id: &ToolId) -> Option<StageToolMaturityLevel> {
    let (_, runtime_normalization) = runtime_contract_levels(stage_id, tool_id);
    bijux_dna_domain_fastq::stage_tool_maturity(stage_id, tool_id, runtime_normalization).map(
        |level| match level {
            bijux_dna_domain_fastq::StageToolMaturityLevel::PlannedBinding => {
                StageToolMaturityLevel::PlannedBinding
            }
            bijux_dna_domain_fastq::StageToolMaturityLevel::GovernedExecution => {
                StageToolMaturityLevel::GovernedExecution
            }
            bijux_dna_domain_fastq::StageToolMaturityLevel::GenericNormalized => {
                StageToolMaturityLevel::GenericNormalized
            }
            bijux_dna_domain_fastq::StageToolMaturityLevel::ObserverNormalized => {
                StageToolMaturityLevel::ObserverNormalized
            }
            bijux_dna_domain_fastq::StageToolMaturityLevel::BenchmarkComparable => {
                StageToolMaturityLevel::BenchmarkComparable
            }
        },
    )
}

#[must_use]
pub fn adapter_bank_path() -> std::path::PathBuf {
    bijux_dna_domain_fastq::adapter_bank_path()
}
