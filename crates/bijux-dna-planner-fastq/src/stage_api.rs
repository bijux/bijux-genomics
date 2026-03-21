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
    pub declared: bool,
    pub plannable: bool,
    pub runnable: bool,
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
    pub generic_envelope_tools: Vec<ToolId>,
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

#[must_use]
pub fn stage_tool_capability(stage_id: &StageId, tool_id: &ToolId) -> Option<StageToolCapability> {
    let governance = bijux_dna_domain_fastq::stage_tool_governance_profile(stage_id, tool_id)?;
    let runtime_interpretation = runtime_interpretation_for_stage_tool(stage_id, tool_id)
        .unwrap_or(RuntimeInterpretationLevel::GenericEnvelope);
    let declared = true;
    let plannable = governance.is_plannable();
    let runnable = governance.is_runnable();
    let parse_normalized = match governance.normalization_maturity() {
        bijux_dna_domain_fastq::StageToolNormalizationMaturity::None => false,
        bijux_dna_domain_fastq::StageToolNormalizationMaturity::GenericEnvelope => true,
        bijux_dna_domain_fastq::StageToolNormalizationMaturity::ObserverSpecialized => {
            runtime_interpretation == RuntimeInterpretationLevel::ObserverSpecialized
        }
    };
    let benchmark_contract_maturity = governance.benchmark_contract_maturity();
    let benchmark_normalized = parse_normalized
        && runtime_interpretation == RuntimeInterpretationLevel::ObserverSpecialized
        && matches!(
            benchmark_contract_maturity,
            bijux_dna_domain_fastq::StageToolBenchmarkContractMaturity::GovernedBenchmarkCohort
                | bijux_dna_domain_fastq::StageToolBenchmarkContractMaturity::BenchmarkComparable
        );
    let comparable = parse_normalized
        && runtime_interpretation == RuntimeInterpretationLevel::ObserverSpecialized
        && benchmark_contract_maturity
            == bijux_dna_domain_fastq::StageToolBenchmarkContractMaturity::BenchmarkComparable;

    Some(StageToolCapability {
        stage_id: governance.stage_id,
        tool_id: governance.tool_id,
        integration_level: governance.integration_level,
        execution_status: governance.execution_status,
        runtime_interpretation,
        benchmark_scenarios: governance.benchmark_scenario_ids,
        declared,
        plannable,
        runnable,
        parse_normalized,
        benchmark_normalized,
        comparable,
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
    let readiness = if !capability.runnable {
        BenchmarkReadinessLevel::PlannedContract
    } else if capability.comparable {
        BenchmarkReadinessLevel::ObserverSpecializedBenchmark
    } else if capability.benchmark_normalized {
        BenchmarkReadinessLevel::GovernedBenchmarkCohort
    } else {
        BenchmarkReadinessLevel::GovernedExecution
    };
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
            let generic_envelope_tools = cohort_profiles
                .iter()
                .filter(|profile| {
                    profile.runtime_interpretation == RuntimeInterpretationLevel::GenericEnvelope
                })
                .map(|profile| profile.tool_id.clone())
                .collect::<Vec<_>>();
            BenchmarkCohort {
                scenario_id: scenario.scenario_id,
                stage_id: scenario.stage_id,
                description: scenario.description,
                fairness_rules: scenario.fairness_rules,
                tool_ids: cohort_profiles
                    .iter()
                    .map(|profile| profile.tool_id.clone())
                    .collect(),
                observer_specialized_tools,
                generic_envelope_tools,
            }
        })
        .collect()
}

#[must_use]
pub fn toolset_for_stage(stage_id: &StageId, mode: ToolsetExecutionMode) -> Vec<ToolId> {
    match mode {
        ToolsetExecutionMode::DefaultChoice => {
            default_tool_for_stage(stage_id).into_iter().collect()
        }
        ToolsetExecutionMode::GovernedExecution => stage_tool_capabilities_for_stage(stage_id)
            .into_iter()
            .filter(|capability| capability.runnable)
            .map(|capability| capability.tool_id)
            .collect(),
        ToolsetExecutionMode::BenchmarkCohort => {
            let mut tool_ids = stage_tool_capabilities_for_stage(stage_id)
                .into_iter()
                .filter(|capability| capability.benchmark_normalized)
                .map(|capability| capability.tool_id)
                .collect::<Vec<_>>();
            tool_ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
            tool_ids.dedup_by(|left, right| left == right);
            tool_ids
        }
        ToolsetExecutionMode::AllBindings => stage_tool_capabilities_for_stage(stage_id)
            .into_iter()
            .map(|capability| capability.tool_id)
            .collect(),
    }
}

#[must_use]
pub fn stage_tool_maturity(stage_id: &StageId, tool_id: &ToolId) -> Option<StageToolMaturityLevel> {
    let capability = stage_tool_capability(stage_id, tool_id)?;
    Some(if !capability.runnable {
        StageToolMaturityLevel::PlannedBinding
    } else if capability.comparable {
        StageToolMaturityLevel::BenchmarkComparable
    } else if capability.runtime_interpretation == RuntimeInterpretationLevel::ObserverSpecialized {
        StageToolMaturityLevel::ObserverNormalized
    } else if capability.benchmark_normalized {
        StageToolMaturityLevel::GenericNormalized
    } else {
        StageToolMaturityLevel::GovernedExecution
    })
}

pub fn adapter_bank_path() -> std::path::PathBuf {
    bijux_dna_domain_fastq::adapter_bank_path()
}
