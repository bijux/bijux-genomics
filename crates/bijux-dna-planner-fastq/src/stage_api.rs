use bijux_dna_core::ids::{StageId, ToolId};

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
pub use bijux_dna_stages_fastq::RuntimeInterpretationLevel;

pub type StagePlanJson = bijux_dna_stage_contract::StagePlanJsonV1;

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

#[must_use]
pub fn benchmark_profile_for_stage_tool(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Option<StageToolBenchmarkProfile> {
    let binding = stage_tool_binding(stage_id, tool_id)?;
    let runtime_interpretation = bijux_dna_stages_fastq::runtime_interpretation_for_stage(stage_id)
        .unwrap_or(RuntimeInterpretationLevel::GenericEnvelope);
    let benchmark_scenarios = benchmark_scenarios_for_stage(stage_id)
        .into_iter()
        .map(|scenario| scenario.scenario_id)
        .collect::<Vec<_>>();
    let readiness = match (
        binding.integration_level,
        benchmark_scenarios.is_empty(),
        runtime_interpretation,
    ) {
        (ToolIntegrationLevel::PlannedContract, _, _) => BenchmarkReadinessLevel::PlannedContract,
        (
            ToolIntegrationLevel::GovernedContract,
            false,
            RuntimeInterpretationLevel::ObserverSpecialized,
        ) => BenchmarkReadinessLevel::ObserverSpecializedBenchmark,
        (
            ToolIntegrationLevel::GovernedContract,
            false,
            RuntimeInterpretationLevel::GenericEnvelope,
        ) => BenchmarkReadinessLevel::GovernedBenchmarkCohort,
        (ToolIntegrationLevel::GovernedContract, true, _) => {
            BenchmarkReadinessLevel::GovernedExecution
        }
    };
    Some(StageToolBenchmarkProfile {
        stage_id: stage_id.clone(),
        tool_id: tool_id.clone(),
        integration_level: binding.integration_level,
        runtime_interpretation,
        benchmark_scenarios,
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

pub fn adapter_bank_path() -> std::path::PathBuf {
    bijux_dna_domain_fastq::adapter_bank_path()
}
