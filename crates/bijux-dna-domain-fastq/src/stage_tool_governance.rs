use bijux_dna_core::ids::{StageId, ToolId};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::comparison_contract::comparison_contract_for_stage;
use crate::execution_support::{
    execution_support_for_stage, BenchmarkSupport, ExecutionStatus, NormalizationSupport,
    PlanningSupport, RuntimeSupport,
};
use crate::integration_matrix::{
    benchmark_scenarios_for_stage, stage_tool_binding, stage_tool_bindings, ToolIntegrationLevel,
};
use crate::{BenchmarkScenario, FastqArtifactKind};

#[derive(Debug, Clone, Deserialize, Default)]
struct ToolExecutionContractRecord {
    #[serde(default)]
    required_inputs: Vec<String>,
    #[serde(default)]
    optional_inputs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ToolStageContractRecord {
    #[serde(default)]
    required_inputs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ToolLayoutManifestRecord {
    tool_id: String,
    #[serde(default)]
    execution_contract: ToolExecutionContractRecord,
    #[serde(default)]
    stage_contracts: BTreeMap<String, ToolStageContractRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StageToolInputLayoutContract {
    supports_single_end: bool,
    supports_paired_end: bool,
}

fn tool_layout_manifests() -> &'static BTreeMap<(String, String), StageToolInputLayoutContract> {
    static CONTRACTS: OnceLock<BTreeMap<(String, String), StageToolInputLayoutContract>> =
        OnceLock::new();
    CONTRACTS.get_or_init(|| {
        let mut contracts = BTreeMap::new();
        for raw in [
            include_str!("../../../domain/fastq/tools/adapterremoval.yaml"),
            include_str!("../../../domain/fastq/tools/atropos.yaml"),
            include_str!("../../../domain/fastq/tools/bayeshammer.yaml"),
            include_str!("../../../domain/fastq/tools/bbduk.yaml"),
            include_str!("../../../domain/fastq/tools/bbmerge.yaml"),
            include_str!("../../../domain/fastq/tools/bowtie2.yaml"),
            include_str!("../../../domain/fastq/tools/bowtie2_build.yaml"),
            include_str!("../../../domain/fastq/tools/centrifuge.yaml"),
            include_str!("../../../domain/fastq/tools/clumpify.yaml"),
            include_str!("../../../domain/fastq/tools/cutadapt.yaml"),
            include_str!("../../../domain/fastq/tools/dada2.yaml"),
            include_str!("../../../domain/fastq/tools/diamond.yaml"),
            include_str!("../../../domain/fastq/tools/dustmasker.yaml"),
            include_str!("../../../domain/fastq/tools/fastp.yaml"),
            include_str!("../../../domain/fastq/tools/fastq_scan.yaml"),
            include_str!("../../../domain/fastq/tools/fastqc.yaml"),
            include_str!("../../../domain/fastq/tools/fastqvalidator.yaml"),
            include_str!("../../../domain/fastq/tools/fastuniq.yaml"),
            include_str!("../../../domain/fastq/tools/fastx_clipper.yaml"),
            include_str!("../../../domain/fastq/tools/flash2.yaml"),
            include_str!("../../../domain/fastq/tools/fqtools.yaml"),
            include_str!("../../../domain/fastq/tools/kaiju.yaml"),
            include_str!("../../../domain/fastq/tools/kraken2.yaml"),
            include_str!("../../../domain/fastq/tools/krakenuniq.yaml"),
            include_str!("../../../domain/fastq/tools/leehom.yaml"),
            include_str!("../../../domain/fastq/tools/lighter.yaml"),
            include_str!("../../../domain/fastq/tools/multiqc.yaml"),
            include_str!("../../../domain/fastq/tools/musket.yaml"),
            include_str!("../../../domain/fastq/tools/pear.yaml"),
            include_str!("../../../domain/fastq/tools/prinseq.yaml"),
            include_str!("../../../domain/fastq/tools/rcorrector.yaml"),
            include_str!("../../../domain/fastq/tools/seqfu.yaml"),
            include_str!("../../../domain/fastq/tools/seqkit.yaml"),
            include_str!("../../../domain/fastq/tools/seqkit_stats.yaml"),
            include_str!("../../../domain/fastq/tools/seqpurge.yaml"),
            include_str!("../../../domain/fastq/tools/seqtk.yaml"),
            include_str!("../../../domain/fastq/tools/skewer.yaml"),
            include_str!("../../../domain/fastq/tools/sortmerna.yaml"),
            include_str!("../../../domain/fastq/tools/star.yaml"),
            include_str!("../../../domain/fastq/tools/trim_galore.yaml"),
            include_str!("../../../domain/fastq/tools/trimmomatic.yaml"),
            include_str!("../../../domain/fastq/tools/umi_tools.yaml"),
            include_str!("../../../domain/fastq/tools/vsearch.yaml"),
            include_str!("../../../domain/fastq/tools/alientrimmer.yaml"),
        ] {
            let manifest: ToolLayoutManifestRecord = bijux_dna_infra::formats::parse_yaml(raw)
                .unwrap_or_else(|err| panic!("parse fastq tool layout manifest: {err}"));
            for (stage_id, stage_contract) in manifest.stage_contracts {
                let requires_reads_r2 = stage_contract
                    .required_inputs
                    .iter()
                    .chain(manifest.execution_contract.required_inputs.iter())
                    .any(|input| input == "reads_r2");
                let allows_reads_r2 = requires_reads_r2
                    || manifest
                        .execution_contract
                        .optional_inputs
                        .iter()
                        .any(|input| input == "reads_r2");
                contracts.insert(
                    (stage_id, manifest.tool_id.clone()),
                    StageToolInputLayoutContract {
                        supports_single_end: !requires_reads_r2,
                        supports_paired_end: allows_reads_r2,
                    },
                );
            }
        }
        contracts
    })
}

fn stage_tool_input_layout_contract(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Option<StageToolInputLayoutContract> {
    tool_layout_manifests()
        .get(&(stage_id.to_string(), tool_id.to_string()))
        .copied()
}

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

impl StageBenchmarkGovernance {
    #[must_use]
    pub fn has_governed_benchmark_contract(&self) -> bool {
        !self.scenarios.is_empty()
            && !self.comparison_input_artifact_ids.is_empty()
            && !self.comparison_artifact_ids.is_empty()
    }
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

#[must_use]
pub fn tool_supports_input_layout(stage_id: &StageId, tool_id: &ToolId, paired_end: bool) -> bool {
    let Some(contract) = crate::contract_for_stage(stage_id.as_str()) else {
        return false;
    };
    let required_kind = if paired_end {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    if !contract
        .accepted_input_kinds
        .iter()
        .any(|kind| kind == &required_kind)
    {
        return false;
    }
    match stage_tool_input_layout_contract(stage_id, tool_id) {
        Some(layout_contract) => {
            if paired_end {
                layout_contract.supports_paired_end
            } else {
                layout_contract.supports_single_end
            }
        }
        None => true,
    }
}

#[must_use]
pub fn filter_tools_for_input_layout(
    stage_id: &StageId,
    tool_ids: Vec<ToolId>,
    paired_end: bool,
) -> Vec<ToolId> {
    tool_ids
        .into_iter()
        .filter(|tool_id| tool_supports_input_layout(stage_id, tool_id, paired_end))
        .collect()
}

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
