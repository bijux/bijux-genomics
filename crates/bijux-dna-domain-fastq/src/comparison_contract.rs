use bijux_dna_core::ids::StageId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageComparisonContract {
    pub stage_id: StageId,
    pub comparison_input_artifact_ids: &'static [&'static str],
    pub cohort_artifact_id: &'static str,
    pub comparison_artifact_id: &'static str,
    pub normalization_artifact_id: &'static str,
}

type StageComparisonContractRecord = (
    &'static str,
    &'static [&'static str],
    &'static str,
    &'static str,
    &'static str,
);

const STAGE_COMPARISON_CONTRACTS: &[StageComparisonContractRecord] = &[
    (
        "fastq.detect_adapters",
        &["adapter_report", "adapter_evidence_dir"],
        "adapter_inspection_tool_benchmark_cohort_json",
        "adapter_inspection_tool_comparison_json",
        "adapter_inspection_tool_normalization_json",
    ),
    (
        "fastq.trim_reads",
        &["trimmed_reads_r1", "trimmed_reads_r2", "report_json"],
        "trim_tool_benchmark_cohort_json",
        "trim_tool_comparison_json",
        "trim_tool_normalization_json",
    ),
    (
        "fastq.trim_polyg_tails",
        &["trimmed_reads_r1", "trimmed_reads_r2", "report_json"],
        "polyg_trim_tool_benchmark_cohort_json",
        "polyg_trim_tool_comparison_json",
        "polyg_trim_tool_normalization_json",
    ),
    (
        "fastq.deplete_rrna",
        &["rrna_report_json"],
        "rrna_depletion_tool_benchmark_cohort_json",
        "rrna_depletion_tool_comparison_json",
        "rrna_depletion_tool_normalization_json",
    ),
    (
        "fastq.filter_reads",
        &["filtered_reads_r1", "filtered_reads_r2"],
        "filter_tool_benchmark_cohort_json",
        "filter_tool_comparison_json",
        "filter_tool_normalization_json",
    ),
    (
        "fastq.deplete_host",
        &["host_depletion_report_json"],
        "host_depletion_tool_benchmark_cohort_json",
        "host_depletion_tool_comparison_json",
        "host_depletion_tool_normalization_json",
    ),
    (
        "fastq.merge_pairs",
        &["report_json"],
        "merge_tool_benchmark_cohort_json",
        "merge_tool_comparison_json",
        "merge_tool_normalization_json",
    ),
    (
        "fastq.deplete_reference_contaminants",
        &["contaminant_screen_report_json"],
        "contaminant_depletion_tool_benchmark_cohort_json",
        "contaminant_depletion_tool_comparison_json",
        "contaminant_depletion_tool_normalization_json",
    ),
    (
        "fastq.filter_low_complexity",
        &["filter_report_json"],
        "low_complexity_tool_benchmark_cohort_json",
        "low_complexity_tool_comparison_json",
        "low_complexity_tool_normalization_json",
    ),
    (
        "fastq.remove_duplicates",
        &["dedup_reads_r1", "dedup_reads_r2", "report_json"],
        "dedup_tool_benchmark_cohort_json",
        "dedup_tool_comparison_json",
        "dedup_tool_normalization_json",
    ),
    (
        "fastq.extract_umis",
        &["report_json"],
        "umi_extraction_tool_benchmark_cohort_json",
        "umi_extraction_tool_comparison_json",
        "umi_extraction_tool_normalization_json",
    ),
    (
        "fastq.profile_read_lengths",
        &["length_distribution_tsv", "length_distribution_json"],
        "read_length_tool_benchmark_cohort_json",
        "read_length_tool_comparison_json",
        "read_length_tool_normalization_json",
    ),
    (
        "fastq.report_qc",
        &["multiqc_report", "multiqc_data"],
        "qc_aggregation_tool_benchmark_cohort_json",
        "qc_aggregation_tool_comparison_json",
        "qc_aggregation_tool_normalization_json",
    ),
    (
        "fastq.correct_errors",
        &["corrected_reads_r1", "corrected_reads_r2", "report_json"],
        "correction_tool_benchmark_cohort_json",
        "correction_tool_comparison_json",
        "correction_tool_normalization_json",
    ),
    (
        "fastq.normalize_primers",
        &["primer_orientation_report", "primer_stats_json"],
        "primer_normalization_tool_benchmark_cohort_json",
        "primer_normalization_tool_comparison_json",
        "primer_normalization_tool_normalization_json",
    ),
    (
        "fastq.trim_terminal_damage",
        &["trimmed_reads_r1", "trimmed_reads_r2", "report_json"],
        "terminal_damage_tool_benchmark_cohort_json",
        "terminal_damage_tool_comparison_json",
        "terminal_damage_tool_normalization_json",
    ),
    (
        "fastq.profile_overrepresented_sequences",
        &[
            "overrepresented_sequences_tsv",
            "overrepresented_sequences_json",
        ],
        "overrepresented_sequence_tool_benchmark_cohort_json",
        "overrepresented_sequence_tool_comparison_json",
        "overrepresented_sequence_tool_normalization_json",
    ),
    (
        "fastq.validate_reads",
        &["validation_report"],
        "validation_tool_benchmark_cohort_json",
        "validation_tool_comparison_json",
        "validation_tool_normalization_json",
    ),
    (
        "fastq.screen_taxonomy",
        &["classification_report_json"],
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
                comparison_input_artifact_ids,
                cohort_artifact_id,
                comparison_artifact_id,
                normalization_artifact_id,
            )| {
                StageComparisonContract {
                    stage_id: StageId::new(*stage_id),
                    comparison_input_artifact_ids,
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
pub fn comparison_input_artifact_ids_for_stage(stage_id: &StageId) -> Vec<&'static str> {
    comparison_contract_for_stage(stage_id)
        .map(|contract| contract.comparison_input_artifact_ids.to_vec())
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
