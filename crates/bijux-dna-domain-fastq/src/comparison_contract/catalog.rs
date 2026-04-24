use bijux_dna_core::ids::StageId;

use crate::benchmark_scenarios_for_stage;

use super::priorities::comparison_input_artifact_ids_for_manifest_stage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageComparisonContract {
    pub stage_id: StageId,
    pub comparison_input_artifact_ids: Vec<String>,
    pub cohort_artifact_id: String,
    pub comparison_artifact_id: String,
    pub normalization_artifact_id: String,
}

fn stage_comparison_contracts() -> Vec<StageComparisonContract> {
    crate::benchmark_scenarios()
        .iter()
        .map(|scenario| StageComparisonContract {
            stage_id: scenario.stage_id.clone(),
            comparison_input_artifact_ids: comparison_input_artifact_ids_for_manifest_stage(
                scenario.stage_id.as_str(),
            ),
            cohort_artifact_id: scenario.cohort_artifact_id.clone(),
            comparison_artifact_id: scenario.comparison_artifact_id.clone(),
            normalization_artifact_id: scenario.normalization_artifact_id.clone(),
        })
        .collect()
}

#[must_use]
pub fn comparison_contract_for_stage(stage_id: &StageId) -> Option<StageComparisonContract> {
    let stage_scenarios = benchmark_scenarios_for_stage(stage_id);
    if stage_scenarios.len() > 1 {
        return None;
    }
    stage_comparison_contracts().into_iter().find(|contract| contract.stage_id == *stage_id)
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
