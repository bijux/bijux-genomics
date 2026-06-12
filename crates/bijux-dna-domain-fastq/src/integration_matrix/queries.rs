use std::collections::BTreeSet;
use std::sync::OnceLock;

use bijux_dna_core::ids::{StageId, ToolId};
use serde::Deserialize;

use super::contract::domain_index_contract;
use super::model::{BenchmarkScenario, StageToolBinding, ToolIntegrationLevel};

#[derive(Debug, Deserialize)]
struct ToolPlannedStageRecord {
    tool_id: String,
    #[serde(default)]
    planned_stage_ids: Vec<String>,
}

#[must_use]
pub fn stage_tool_bindings() -> Vec<StageToolBinding> {
    static BINDINGS: OnceLock<Vec<StageToolBinding>> = OnceLock::new();
    BINDINGS
        .get_or_init(|| {
            let mut bindings = domain_index_contract()
                .stage_tool_integration
                .iter()
                .flat_map(|(stage_id, bindings)| {
                    bindings.iter().map(move |(tool_id, integration_level)| StageToolBinding {
                        stage_id: StageId::new(stage_id.clone()),
                        tool_id: ToolId::new(tool_id.clone()),
                        integration_level: *integration_level,
                    })
                })
                .collect::<Vec<_>>();

            let existing_keys = bindings
                .iter()
                .map(|binding| (binding.stage_id.to_string(), binding.tool_id.to_string()))
                .collect::<BTreeSet<_>>();
            bindings.extend(planned_stage_tool_bindings().into_iter().filter(|planned| {
                !existing_keys
                    .contains(&(planned.stage_id.to_string(), planned.tool_id.to_string()))
            }));
            bindings
        })
        .clone()
}

fn planned_stage_tool_bindings() -> Vec<StageToolBinding> {
    static PLANNED_BINDINGS: OnceLock<Vec<StageToolBinding>> = OnceLock::new();
    PLANNED_BINDINGS
        .get_or_init(|| {
            [
                include_str!("../../../../domain/fastq/tools/adapterremoval.yaml"),
                include_str!("../../../../domain/fastq/tools/atropos.yaml"),
                include_str!("../../../../domain/fastq/tools/bayeshammer.yaml"),
                include_str!("../../../../domain/fastq/tools/bbduk.yaml"),
                include_str!("../../../../domain/fastq/tools/bbmerge.yaml"),
                include_str!("../../../../domain/fastq/tools/bijux_dna.yaml"),
                include_str!("../../../../domain/fastq/tools/bowtie2.yaml"),
                include_str!("../../../../domain/fastq/tools/bowtie2_build.yaml"),
                include_str!("../../../../domain/fastq/tools/centrifuge.yaml"),
                include_str!("../../../../domain/fastq/tools/clumpify.yaml"),
                include_str!("../../../../domain/fastq/tools/cutadapt.yaml"),
                include_str!("../../../../domain/fastq/tools/dada2.yaml"),
                include_str!("../../../../domain/fastq/tools/diamond.yaml"),
                include_str!("../../../../domain/fastq/tools/dustmasker.yaml"),
                include_str!("../../../../domain/fastq/tools/fastp.yaml"),
                include_str!("../../../../domain/fastq/tools/fastq_scan.yaml"),
                include_str!("../../../../domain/fastq/tools/fastqc.yaml"),
                include_str!("../../../../domain/fastq/tools/fastqvalidator.yaml"),
                include_str!("../../../../domain/fastq/tools/fastuniq.yaml"),
                include_str!("../../../../domain/fastq/tools/fastx_clipper.yaml"),
                include_str!("../../../../domain/fastq/tools/flash2.yaml"),
                include_str!("../../../../domain/fastq/tools/fqtools.yaml"),
                include_str!("../../../../domain/fastq/tools/kaiju.yaml"),
                include_str!("../../../../domain/fastq/tools/kraken2.yaml"),
                include_str!("../../../../domain/fastq/tools/krakenuniq.yaml"),
                include_str!("../../../../domain/fastq/tools/leehom.yaml"),
                include_str!("../../../../domain/fastq/tools/lighter.yaml"),
                include_str!("../../../../domain/fastq/tools/multiqc.yaml"),
                include_str!("../../../../domain/fastq/tools/musket.yaml"),
                include_str!("../../../../domain/fastq/tools/pear.yaml"),
                include_str!("../../../../domain/fastq/tools/prinseq.yaml"),
                include_str!("../../../../domain/fastq/tools/rcorrector.yaml"),
                include_str!("../../../../domain/fastq/tools/seqfu.yaml"),
                include_str!("../../../../domain/fastq/tools/seqkit.yaml"),
                include_str!("../../../../domain/fastq/tools/seqkit_stats.yaml"),
                include_str!("../../../../domain/fastq/tools/seqpurge.yaml"),
                include_str!("../../../../domain/fastq/tools/seqtk.yaml"),
                include_str!("../../../../domain/fastq/tools/skewer.yaml"),
                include_str!("../../../../domain/fastq/tools/sortmerna.yaml"),
                include_str!("../../../../domain/fastq/tools/star.yaml"),
                include_str!("../../../../domain/fastq/tools/trim_galore.yaml"),
                include_str!("../../../../domain/fastq/tools/trimmomatic.yaml"),
                include_str!("../../../../domain/fastq/tools/umi_tools.yaml"),
                include_str!("../../../../domain/fastq/tools/vsearch.yaml"),
                include_str!("../../../../domain/fastq/tools/alientrimmer.yaml"),
            ]
            .into_iter()
            .flat_map(|raw| {
                let manifest: ToolPlannedStageRecord = bijux_dna_infra::formats::parse_yaml(raw)
                    .unwrap_or_else(|err| panic!("parse fastq planned tool manifest: {err}"));
                manifest.planned_stage_ids.into_iter().map(move |stage_id| StageToolBinding {
                    stage_id: StageId::new(stage_id),
                    tool_id: ToolId::new(manifest.tool_id.clone()),
                    integration_level: ToolIntegrationLevel::PlannedContract,
                })
            })
            .collect()
        })
        .clone()
}

#[must_use]
pub fn stage_tool_bindings_for_stage(stage_id: &StageId) -> Vec<StageToolBinding> {
    stage_tool_bindings().into_iter().filter(|binding| binding.stage_id == *stage_id).collect()
}

#[must_use]
pub fn registered_tool_ids_for_stage(stage_id: &StageId) -> Vec<ToolId> {
    tool_ids_for_stage_by_level(stage_id, None)
}

#[must_use]
pub fn governed_tool_ids_for_stage(stage_id: &StageId) -> Vec<ToolId> {
    tool_ids_for_stage_by_level(stage_id, Some(ToolIntegrationLevel::GovernedContract))
}

#[must_use]
pub fn planned_tool_ids_for_stage(stage_id: &StageId) -> Vec<ToolId> {
    tool_ids_for_stage_by_level(stage_id, Some(ToolIntegrationLevel::PlannedContract))
}

fn tool_ids_for_stage_by_level(
    stage_id: &StageId,
    level: Option<ToolIntegrationLevel>,
) -> Vec<ToolId> {
    let mut tools = stage_tool_bindings()
        .into_iter()
        .filter(|binding| binding.stage_id == *stage_id)
        .filter(|binding| level.is_none_or(|level| binding.integration_level == level))
        .map(|binding| binding.tool_id)
        .collect::<Vec<_>>();
    tools.sort();
    tools.dedup();
    tools
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
    benchmark_scenarios().into_iter().filter(|scenario| scenario.stage_id == *stage_id).collect()
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
    reference_index_backends_for_tool(tool_id).into_iter().any(|backend| backend == *index_tool_id)
}

#[must_use]
pub fn stage_sanity_metrics_for_stage(stage_id: &StageId) -> Vec<String> {
    domain_index_contract().stage_sanity_metrics.get(stage_id.as_str()).cloned().unwrap_or_default()
}
