use anyhow::Result;
use bijux_dna_core::ids::StageId;

#[test]
fn benchmark_stages_publish_comparison_artifact_contracts() {
    let trim_stage = StageId::from_static("fastq.trim_reads");
    let trim_artifacts = bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&trim_stage);
    assert_eq!(
        trim_artifacts,
        vec![
            "trim_tool_benchmark_cohort_json",
            "trim_tool_comparison_json",
            "trim_tool_normalization_json",
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_contract_for_stage(&trim_stage)
            .expect("trim comparison contract")
            .comparison_artifact_id,
        "trim_tool_comparison_json"
    );

    let screen_stage = StageId::from_static("fastq.screen_taxonomy");
    let screen_artifacts =
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&screen_stage);
    assert_eq!(
        screen_artifacts,
        vec![
            "taxonomy_tool_benchmark_cohort_json",
            "taxonomy_tool_comparison_json",
            "taxonomy_tool_normalization_json",
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_contract_for_stage(&screen_stage)
            .expect("taxonomy comparison contract")
            .comparison_artifact_id,
        "taxonomy_tool_comparison_json"
    );

    let validate_stage = StageId::from_static("fastq.validate_reads");
    assert!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&validate_stage).is_empty(),
        "non-benchmark stages must not advertise comparison artifacts",
    );
}

#[test]
fn comparison_artifacts_stay_inside_fastq_artifact_vocabulary() -> Result<()> {
    let yaml = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .expect("workspace root")
            .join("domain/fastq/artifacts.yaml"),
    )?;
    for artifact_id in bijux_dna_domain_fastq::benchmark_comparison_artifact_ids() {
        assert!(
            yaml.contains(&format!("  - {artifact_id}")),
            "missing comparison artifact id {artifact_id} in domain/fastq/artifacts.yaml",
        );
    }
    Ok(())
}
