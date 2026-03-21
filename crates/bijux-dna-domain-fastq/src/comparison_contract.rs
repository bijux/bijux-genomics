use bijux_dna_core::ids::StageId;

use crate::stages::ports::stage_output_ids_in_manifest_order;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageComparisonContract {
    pub stage_id: StageId,
    pub comparison_input_artifact_ids: Vec<String>,
    pub cohort_artifact_id: &'static str,
    pub comparison_artifact_id: &'static str,
    pub normalization_artifact_id: &'static str,
}

type StageComparisonContractRecord = (&'static str, &'static str, &'static str, &'static str);

const STAGE_COMPARISON_CONTRACTS: &[StageComparisonContractRecord] = &[
    (
        "fastq.detect_adapters",
        "adapter_inspection_tool_benchmark_cohort_json",
        "adapter_inspection_tool_comparison_json",
        "adapter_inspection_tool_normalization_json",
    ),
    (
        "fastq.trim_reads",
        "trim_tool_benchmark_cohort_json",
        "trim_tool_comparison_json",
        "trim_tool_normalization_json",
    ),
    (
        "fastq.trim_polyg_tails",
        "polyg_trim_tool_benchmark_cohort_json",
        "polyg_trim_tool_comparison_json",
        "polyg_trim_tool_normalization_json",
    ),
    (
        "fastq.deplete_rrna",
        "rrna_depletion_tool_benchmark_cohort_json",
        "rrna_depletion_tool_comparison_json",
        "rrna_depletion_tool_normalization_json",
    ),
    (
        "fastq.filter_reads",
        "filter_tool_benchmark_cohort_json",
        "filter_tool_comparison_json",
        "filter_tool_normalization_json",
    ),
    (
        "fastq.deplete_host",
        "host_depletion_tool_benchmark_cohort_json",
        "host_depletion_tool_comparison_json",
        "host_depletion_tool_normalization_json",
    ),
    (
        "fastq.merge_pairs",
        "merge_tool_benchmark_cohort_json",
        "merge_tool_comparison_json",
        "merge_tool_normalization_json",
    ),
    (
        "fastq.deplete_reference_contaminants",
        "contaminant_depletion_tool_benchmark_cohort_json",
        "contaminant_depletion_tool_comparison_json",
        "contaminant_depletion_tool_normalization_json",
    ),
    (
        "fastq.filter_low_complexity",
        "low_complexity_tool_benchmark_cohort_json",
        "low_complexity_tool_comparison_json",
        "low_complexity_tool_normalization_json",
    ),
    (
        "fastq.remove_duplicates",
        "dedup_tool_benchmark_cohort_json",
        "dedup_tool_comparison_json",
        "dedup_tool_normalization_json",
    ),
    (
        "fastq.extract_umis",
        "umi_extraction_tool_benchmark_cohort_json",
        "umi_extraction_tool_comparison_json",
        "umi_extraction_tool_normalization_json",
    ),
    (
        "fastq.profile_read_lengths",
        "read_length_tool_benchmark_cohort_json",
        "read_length_tool_comparison_json",
        "read_length_tool_normalization_json",
    ),
    (
        "fastq.report_qc",
        "qc_aggregation_tool_benchmark_cohort_json",
        "qc_aggregation_tool_comparison_json",
        "qc_aggregation_tool_normalization_json",
    ),
    (
        "fastq.correct_errors",
        "correction_tool_benchmark_cohort_json",
        "correction_tool_comparison_json",
        "correction_tool_normalization_json",
    ),
    (
        "fastq.normalize_primers",
        "primer_normalization_tool_benchmark_cohort_json",
        "primer_normalization_tool_comparison_json",
        "primer_normalization_tool_normalization_json",
    ),
    (
        "fastq.trim_terminal_damage",
        "terminal_damage_tool_benchmark_cohort_json",
        "terminal_damage_tool_comparison_json",
        "terminal_damage_tool_normalization_json",
    ),
    (
        "fastq.profile_overrepresented_sequences",
        "overrepresented_sequence_tool_benchmark_cohort_json",
        "overrepresented_sequence_tool_comparison_json",
        "overrepresented_sequence_tool_normalization_json",
    ),
    (
        "fastq.validate_reads",
        "validation_tool_benchmark_cohort_json",
        "validation_tool_comparison_json",
        "validation_tool_normalization_json",
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
            |(
                stage_id,
                cohort_artifact_id,
                comparison_artifact_id,
                normalization_artifact_id,
            )| {
                StageComparisonContract {
                    stage_id: StageId::new(*stage_id),
                    comparison_input_artifact_ids: comparison_input_artifact_ids_for_manifest_stage(
                        stage_id,
                    ),
                    cohort_artifact_id,
                    comparison_artifact_id,
                    normalization_artifact_id,
                }
            },
        )
        .collect()
}

fn comparison_input_artifact_ids_for_manifest_stage(stage_id: &str) -> Vec<String> {
    let mut artifact_ids = stage_output_ids_in_manifest_order(stage_id).unwrap_or_default();
    prioritize_provenance_artifact(stage_id, &mut artifact_ids);
    artifact_ids
}

fn prioritize_provenance_artifact(stage_id: &str, artifact_ids: &mut Vec<String>) {
    let provenance_artifact_id = match stage_id {
        "fastq.validate_reads" => Some("validated_reads_manifest"),
        "fastq.report_qc" => Some("governed_qc_inputs_manifest"),
        _ => None,
    };
    let Some(provenance_artifact_id) = provenance_artifact_id else {
        return;
    };
    let Some(position) = artifact_ids
        .iter()
        .position(|artifact_id| artifact_id == provenance_artifact_id)
    else {
        return;
    };
    let provenance_artifact = artifact_ids.remove(position);
    artifact_ids.insert(0, provenance_artifact);
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
pub fn comparison_input_artifact_ids_for_stage(stage_id: &StageId) -> Vec<String> {
    comparison_contract_for_stage(stage_id)
        .map(|contract| contract.comparison_input_artifact_ids)
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
