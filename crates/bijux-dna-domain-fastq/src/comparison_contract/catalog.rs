use bijux_dna_core::ids::StageId;

use crate::{benchmark_scenarios_for_stage, BenchmarkScenario};

use super::priorities::comparison_input_artifact_ids_for_manifest_stage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageComparisonContract {
    pub stage_id: StageId,
    pub comparison_input_artifact_ids: Vec<String>,
    pub cohort_artifact_id: String,
    pub comparison_artifact_id: String,
    pub normalization_artifact_id: String,
}

fn comparison_contract(scenario: BenchmarkScenario) -> StageComparisonContract {
    StageComparisonContract {
        comparison_input_artifact_ids: comparison_input_artifact_ids_for_manifest_stage(
            scenario.stage_id.as_str(),
        ),
        stage_id: scenario.stage_id,
        cohort_artifact_id: scenario.cohort_artifact_id,
        comparison_artifact_id: scenario.comparison_artifact_id,
        normalization_artifact_id: scenario.normalization_artifact_id,
    }
}

fn stage_comparison_contracts() -> Vec<StageComparisonContract> {
    crate::benchmark_scenarios().into_iter().map(comparison_contract).collect()
}

#[must_use]
pub fn comparison_contract_for_stage(stage_id: &StageId) -> Option<StageComparisonContract> {
    let mut stage_scenarios = benchmark_scenarios_for_stage(stage_id);
    if stage_scenarios.len() != 1 {
        return None;
    }
    stage_scenarios.pop().map(comparison_contract)
}

#[must_use]
pub fn comparison_artifact_ids_for_stage(stage_id: &StageId) -> Vec<String> {
    comparison_contract_for_stage(stage_id)
        .map(|contract| {
            vec![
                contract.cohort_artifact_id,
                contract.comparison_artifact_id,
                contract.normalization_artifact_id,
            ]
        })
        .unwrap_or_default()
}

#[must_use]
pub fn comparison_input_artifact_ids_for_stage(stage_id: &StageId) -> Vec<String> {
    comparison_contract_for_stage(stage_id)
        .map(|contract| contract.comparison_input_artifact_ids)
        .unwrap_or_default()
}

#[must_use]
pub fn benchmark_comparison_artifact_ids() -> Vec<String> {
    stage_comparison_contracts()
        .into_iter()
        .flat_map(|contract| {
            [
                contract.cohort_artifact_id,
                contract.comparison_artifact_id,
                contract.normalization_artifact_id,
            ]
        })
        .collect()
}
