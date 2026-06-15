use bijux_dna_core::ids::StageId;

fn comparison_contract(stage_id: &'static str) -> bijux_dna_domain_fastq::StageComparisonContract {
    bijux_dna_domain_fastq::comparison_contract_for_stage(&StageId::from_static(stage_id))
        .unwrap_or_else(|| panic!("{stage_id} comparison contract must exist"))
}

#[test]
fn detect_adapters_exposes_stage_family_comparison_contract() {
    let contract = comparison_contract("fastq.detect_adapters");

    assert_eq!(contract.cohort_artifact_id, "adapter_inspection_tool_benchmark_cohort_json");
    assert_eq!(contract.comparison_artifact_id, "adapter_inspection_tool_comparison_json");
    assert_eq!(contract.normalization_artifact_id, "adapter_inspection_tool_normalization_json");
}

#[test]
fn deplete_rrna_exposes_stage_family_comparison_contract() {
    let contract = comparison_contract("fastq.deplete_rrna");

    assert_eq!(contract.cohort_artifact_id, "rrna_depletion_tool_benchmark_cohort_json");
    assert_eq!(contract.comparison_artifact_id, "rrna_depletion_tool_comparison_json");
    assert_eq!(contract.normalization_artifact_id, "rrna_depletion_tool_normalization_json");
    assert_eq!(
        contract.comparison_input_artifact_ids,
        &[
            "rrna_report_json",
            "rrna_report_tsv",
            "rrna_filtered_reads_r1",
            "rrna_filtered_reads_r2",
            "rrna_removed_reads_r1",
            "rrna_removed_reads_r2",
        ]
    );
}

#[test]
fn deplete_host_exposes_stage_family_comparison_contract() {
    let contract = comparison_contract("fastq.deplete_host");

    assert_eq!(contract.cohort_artifact_id, "host_depletion_tool_benchmark_cohort_json");
    assert_eq!(contract.comparison_artifact_id, "host_depletion_tool_comparison_json");
    assert_eq!(contract.normalization_artifact_id, "host_depletion_tool_normalization_json");
    assert_eq!(
        contract.comparison_input_artifact_ids,
        &[
            "host_depletion_report_json",
            "host_depleted_reads_r1",
            "host_depleted_reads_r2",
            "removed_host_reads_r1",
            "removed_host_reads_r2",
        ]
    );
}

#[test]
fn deplete_reference_contaminants_exposes_stage_family_comparison_contract() {
    let contract = comparison_contract("fastq.deplete_reference_contaminants");

    assert_eq!(contract.cohort_artifact_id, "contaminant_depletion_tool_benchmark_cohort_json");
    assert_eq!(contract.comparison_artifact_id, "contaminant_depletion_tool_comparison_json");
    assert_eq!(contract.normalization_artifact_id, "contaminant_depletion_tool_normalization_json");
    assert_eq!(
        contract.comparison_input_artifact_ids,
        &[
            "contaminant_screen_report_json",
            "contaminant_screened_reads_r1",
            "contaminant_screened_reads_r2",
            "removed_contaminant_reads_r1",
            "removed_contaminant_reads_r2",
        ]
    );
}

#[test]
fn extract_umis_exposes_stage_family_comparison_contract() {
    let contract = comparison_contract("fastq.extract_umis");

    assert_eq!(contract.cohort_artifact_id, "umi_extraction_tool_benchmark_cohort_json");
    assert_eq!(contract.comparison_artifact_id, "umi_extraction_tool_comparison_json");
    assert_eq!(contract.normalization_artifact_id, "umi_extraction_tool_normalization_json");
}

#[test]
fn report_qc_exposes_stage_family_comparison_contract() {
    let contract = comparison_contract("fastq.report_qc");

    assert_eq!(contract.cohort_artifact_id, "qc_aggregation_tool_benchmark_cohort_json");
    assert_eq!(contract.comparison_artifact_id, "qc_aggregation_tool_comparison_json");
    assert_eq!(contract.normalization_artifact_id, "qc_aggregation_tool_normalization_json");
    assert_eq!(
        contract.comparison_input_artifact_ids,
        &["report_json", "governed_qc_inputs_manifest", "multiqc_report", "multiqc_data",]
    );
}

#[test]
fn validate_reads_exposes_lineage_aware_comparison_contract() {
    let contract = comparison_contract("fastq.validate_reads");

    assert_eq!(
        contract.comparison_input_artifact_ids,
        &["validation_report", "validated_reads_manifest"]
    );
}

#[test]
fn trim_polyg_tails_exposes_stage_family_comparison_contract() {
    let contract = comparison_contract("fastq.trim_polyg_tails");

    assert_eq!(contract.cohort_artifact_id, "polyg_trim_tool_benchmark_cohort_json");
    assert_eq!(contract.comparison_artifact_id, "polyg_trim_tool_comparison_json");
    assert_eq!(contract.normalization_artifact_id, "polyg_trim_tool_normalization_json");
    assert_eq!(
        contract.comparison_input_artifact_ids,
        &["trimmed_reads_r1", "trimmed_reads_r2", "report_json"]
    );
}
