use std::collections::BTreeMap;
use std::sync::OnceLock;

use bijux_dna_core::ids::{StageId, ToolId};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolIntegrationLevel {
    GovernedContract,
    PlannedContract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageToolBinding {
    pub stage_id: StageId,
    pub tool_id: ToolId,
    pub integration_level: ToolIntegrationLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkScenario {
    pub scenario_id: String,
    pub stage_id: StageId,
    pub description: String,
    pub fairness_rules: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DomainIndexContract {
    stage_tool_integration: BTreeMap<String, BTreeMap<String, ToolIntegrationLevel>>,
    #[serde(default)]
    reference_index_compatibility: BTreeMap<String, Vec<String>>,
    benchmark_scenarios: BTreeMap<String, BenchmarkScenarioRecord>,
}

#[derive(Debug, Deserialize)]
struct BenchmarkScenarioRecord {
    stage_id: String,
    description: String,
    fairness_rules: Vec<String>,
}

fn domain_index_contract() -> &'static DomainIndexContract {
    static CONTRACT: OnceLock<DomainIndexContract> = OnceLock::new();
    CONTRACT.get_or_init(|| {
        serde_yaml::from_str(include_str!("../../../domain/fastq/index.yaml"))
            .expect("parse domain/fastq/index.yaml integration contract")
    })
}

#[must_use]
pub fn stage_tool_bindings() -> Vec<StageToolBinding> {
    domain_index_contract()
        .stage_tool_integration
        .iter()
        .flat_map(|(stage_id, bindings)| {
            bindings.iter().map(move |(tool_id, integration_level)| StageToolBinding {
                stage_id: StageId::new(stage_id.clone()),
                tool_id: ToolId::new(tool_id.clone()),
                integration_level: *integration_level,
            })
        })
        .collect()
}

#[must_use]
pub fn stage_tool_bindings_for_stage(stage_id: &StageId) -> Vec<StageToolBinding> {
    stage_tool_bindings()
        .into_iter()
        .filter(|binding| binding.stage_id == *stage_id)
        .collect()
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
        })
        .collect()
}

#[must_use]
pub fn benchmark_scenarios_for_stage(stage_id: &StageId) -> Vec<BenchmarkScenario> {
    benchmark_scenarios()
        .into_iter()
        .filter(|scenario| scenario.stage_id == *stage_id)
        .collect()
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
pub fn is_reference_index_backend_compatible(
    tool_id: &ToolId,
    index_tool_id: &ToolId,
) -> bool {
    reference_index_backends_for_tool(tool_id)
        .into_iter()
        .any(|backend| backend == *index_tool_id)
}
