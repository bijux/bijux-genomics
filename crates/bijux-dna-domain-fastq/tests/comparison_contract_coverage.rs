use bijux_dna_core::ids::StageId;

#[test]
fn detect_adapters_exposes_stage_family_comparison_contract() {
    let contract =
        bijux_dna_domain_fastq::comparison_contract_for_stage(&StageId::from_static(
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
    let contract =
        bijux_dna_domain_fastq::comparison_contract_for_stage(&StageId::from_static(
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
