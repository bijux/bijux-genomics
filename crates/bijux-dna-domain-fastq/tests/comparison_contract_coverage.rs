use bijux_dna_core::ids::StageId;

#[test]
fn detect_adapters_exposes_stage_family_comparison_contract() {
    let contract = bijux_dna_domain_fastq::comparison_contract_for_stage(&StageId::from_static(
        "fastq.detect_adapters",
    ))
    .expect("detect_adapters comparison contract must exist");

    assert_eq!(
        contract.cohort_artifact_id,
        "adapter_inspection_tool_benchmark_cohort_json"
    );
    assert_eq!(
        contract.comparison_artifact_id,
        "adapter_inspection_tool_comparison_json"
    );
    assert_eq!(
        contract.normalization_artifact_id,
        "adapter_inspection_tool_normalization_json"
    );
}

#[test]
fn deplete_rrna_exposes_stage_family_comparison_contract() {
    let contract = bijux_dna_domain_fastq::comparison_contract_for_stage(&StageId::from_static(
        "fastq.deplete_rrna",
    ))
    .expect("deplete_rrna comparison contract must exist");

    assert_eq!(
        contract.cohort_artifact_id,
        "rrna_depletion_tool_benchmark_cohort_json"
    );
    assert_eq!(
        contract.comparison_artifact_id,
        "rrna_depletion_tool_comparison_json"
    );
    assert_eq!(
        contract.normalization_artifact_id,
        "rrna_depletion_tool_normalization_json"
    );
}

#[test]
fn deplete_host_exposes_stage_family_comparison_contract() {
    let contract = bijux_dna_domain_fastq::comparison_contract_for_stage(&StageId::from_static(
        "fastq.deplete_host",
    ))
    .expect("deplete_host comparison contract must exist");

    assert_eq!(
        contract.cohort_artifact_id,
        "host_depletion_tool_benchmark_cohort_json"
    );
    assert_eq!(
        contract.comparison_artifact_id,
        "host_depletion_tool_comparison_json"
    );
    assert_eq!(
        contract.normalization_artifact_id,
        "host_depletion_tool_normalization_json"
    );
}

#[test]
fn deplete_reference_contaminants_exposes_stage_family_comparison_contract() {
    let contract = bijux_dna_domain_fastq::comparison_contract_for_stage(&StageId::from_static(
        "fastq.deplete_reference_contaminants",
    ))
    .expect("deplete_reference_contaminants comparison contract must exist");

    assert_eq!(
        contract.cohort_artifact_id,
        "contaminant_depletion_tool_benchmark_cohort_json"
    );
    assert_eq!(
        contract.comparison_artifact_id,
        "contaminant_depletion_tool_comparison_json"
    );
    assert_eq!(
        contract.normalization_artifact_id,
        "contaminant_depletion_tool_normalization_json"
    );
}

#[test]
fn extract_umis_exposes_stage_family_comparison_contract() {
    let contract = bijux_dna_domain_fastq::comparison_contract_for_stage(&StageId::from_static(
        "fastq.extract_umis",
    ))
    .expect("extract_umis comparison contract must exist");

    assert_eq!(
        contract.cohort_artifact_id,
        "umi_extraction_tool_benchmark_cohort_json"
    );
    assert_eq!(
        contract.comparison_artifact_id,
        "umi_extraction_tool_comparison_json"
    );
    assert_eq!(
        contract.normalization_artifact_id,
        "umi_extraction_tool_normalization_json"
    );
}

#[test]
fn report_qc_exposes_stage_family_comparison_contract() {
    let contract = bijux_dna_domain_fastq::comparison_contract_for_stage(&StageId::from_static(
        "fastq.report_qc",
    ))
    .expect("report_qc comparison contract must exist");

    assert_eq!(
        contract.cohort_artifact_id,
        "qc_aggregation_tool_benchmark_cohort_json"
    );
    assert_eq!(
        contract.comparison_artifact_id,
        "qc_aggregation_tool_comparison_json"
    );
    assert_eq!(
        contract.normalization_artifact_id,
        "qc_aggregation_tool_normalization_json"
    );
    assert_eq!(
        contract.comparison_input_artifact_ids,
        &[
            "governed_qc_inputs_manifest",
            "multiqc_report",
            "multiqc_data",
        ]
    );
}

#[test]
fn validate_reads_exposes_lineage_aware_comparison_contract() {
    let contract = bijux_dna_domain_fastq::comparison_contract_for_stage(&StageId::from_static(
        "fastq.validate_reads",
    ))
    .expect("validate_reads comparison contract must exist");

    assert_eq!(
        contract.comparison_input_artifact_ids,
        &["validated_reads_manifest", "validation_report"]
    );
}

#[test]
fn trim_polyg_tails_exposes_stage_family_comparison_contract() {
    let contract = bijux_dna_domain_fastq::comparison_contract_for_stage(&StageId::from_static(
        "fastq.trim_polyg_tails",
    ))
    .expect("trim_polyg_tails comparison contract must exist");

    assert_eq!(
        contract.cohort_artifact_id,
        "polyg_trim_tool_benchmark_cohort_json"
    );
    assert_eq!(
        contract.comparison_artifact_id,
        "polyg_trim_tool_comparison_json"
    );
    assert_eq!(
        contract.normalization_artifact_id,
        "polyg_trim_tool_normalization_json"
    );
    assert_eq!(
        contract.comparison_input_artifact_ids,
        &["trimmed_reads_r1", "trimmed_reads_r2", "report_json"]
    );
}
