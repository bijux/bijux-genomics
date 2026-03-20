use bijux_dna_core::ids::StageId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageComparisonContract {
    pub stage_id: StageId,
    pub cohort_artifact_id: &'static str,
    pub comparison_artifact_id: &'static str,
    pub normalization_artifact_id: &'static str,
}

const STAGE_COMPARISON_CONTRACTS: &[(&str, &str, &str, &str)] = &[
    (
        "fastq.trim_reads",
        "trim_tool_benchmark_cohort_json",
        "trim_tool_comparison_json",
        "trim_tool_normalization_json",
    ),
    (
        "fastq.screen_taxonomy",
        "taxonomy_tool_benchmark_cohort_json",
        "taxonomy_tool_comparison_json",
        "taxonomy_tool_normalization_json",
    ),
];

fn stage_comparison_contracts() -> Vec<StageComparisonContract> {
    STAGE_COMPARISON_CONTRACTS
        .iter()
        .map(
            |(stage_id, cohort_artifact_id, comparison_artifact_id, normalization_artifact_id)| {
                StageComparisonContract {
                    stage_id: StageId::new(*stage_id),
                    cohort_artifact_id,
                    comparison_artifact_id,
                    normalization_artifact_id,
                }
            },
        )
        .collect()
}

#[must_use]
pub fn comparison_contract_for_stage(stage_id: &StageId) -> Option<StageComparisonContract> {
    stage_comparison_contracts()
        .into_iter()
        .find(|contract| contract.stage_id == *stage_id)
}

#[must_use]
pub fn comparison_artifact_ids_for_stage(stage_id: &StageId) -> Vec<&'static str> {
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
pub fn benchmark_comparison_artifact_ids() -> Vec<&'static str> {
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
