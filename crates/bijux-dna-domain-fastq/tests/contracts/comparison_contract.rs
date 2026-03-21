use anyhow::Result;
use bijux_dna_core::ids::StageId;
use std::collections::BTreeSet;

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
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&trim_stage),
        vec!["trimmed_reads_r1", "trimmed_reads_r2", "report_json"]
    );

    let screen_stage = StageId::from_static("fastq.screen_taxonomy");
    let screen_artifacts = bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&screen_stage);
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

    let detect_adapters_stage = StageId::from_static("fastq.detect_adapters");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&detect_adapters_stage),
        vec![
            "adapter_inspection_tool_benchmark_cohort_json",
            "adapter_inspection_tool_comparison_json",
            "adapter_inspection_tool_normalization_json",
        ],
        "adapter inspection now participates in governed benchmark comparisons",
    );

    let filter_stage = StageId::from_static("fastq.filter_reads");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&filter_stage),
        vec![
            "filter_tool_benchmark_cohort_json",
            "filter_tool_comparison_json",
            "filter_tool_normalization_json",
        ]
    );

    let merge_stage = StageId::from_static("fastq.merge_pairs");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&merge_stage),
        vec![
            "merge_tool_benchmark_cohort_json",
            "merge_tool_comparison_json",
            "merge_tool_normalization_json",
        ]
    );

    let low_complexity_stage = StageId::from_static("fastq.filter_low_complexity");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&low_complexity_stage),
        vec![
            "low_complexity_tool_benchmark_cohort_json",
            "low_complexity_tool_comparison_json",
            "low_complexity_tool_normalization_json",
        ]
    );

    let dedup_stage = StageId::from_static("fastq.remove_duplicates");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&dedup_stage),
        vec![
            "dedup_tool_benchmark_cohort_json",
            "dedup_tool_comparison_json",
            "dedup_tool_normalization_json",
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&dedup_stage),
        vec!["dedup_reads_r1", "dedup_reads_r2", "report_json"]
    );

    let read_length_stage = StageId::from_static("fastq.profile_read_lengths");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&read_length_stage),
        vec![
            "read_length_tool_benchmark_cohort_json",
            "read_length_tool_comparison_json",
            "read_length_tool_normalization_json",
        ]
    );

    let correction_stage = StageId::from_static("fastq.correct_errors");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&correction_stage),
        vec![
            "correction_tool_benchmark_cohort_json",
            "correction_tool_comparison_json",
            "correction_tool_normalization_json",
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&correction_stage),
        vec!["corrected_reads_r1", "corrected_reads_r2", "report_json"]
    );

    let normalize_primers_stage = StageId::from_static("fastq.normalize_primers");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&normalize_primers_stage),
        vec![
            "primer_normalization_tool_benchmark_cohort_json",
            "primer_normalization_tool_comparison_json",
            "primer_normalization_tool_normalization_json",
        ]
    );

    let terminal_damage_stage = StageId::from_static("fastq.trim_terminal_damage");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&terminal_damage_stage),
        vec![
            "terminal_damage_tool_benchmark_cohort_json",
            "terminal_damage_tool_comparison_json",
            "terminal_damage_tool_normalization_json",
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&terminal_damage_stage),
        vec!["trimmed_reads_r1", "trimmed_reads_r2", "report_json"]
    );

    let polyg_stage = StageId::from_static("fastq.trim_polyg_tails");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&polyg_stage),
        vec![
            "polyg_trim_tool_benchmark_cohort_json",
            "polyg_trim_tool_comparison_json",
            "polyg_trim_tool_normalization_json",
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&polyg_stage),
        vec!["trimmed_reads_r1", "trimmed_reads_r2", "report_json"]
    );

    let overrepresented_stage = StageId::from_static("fastq.profile_overrepresented_sequences");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&overrepresented_stage),
        vec![
            "overrepresented_sequence_tool_benchmark_cohort_json",
            "overrepresented_sequence_tool_comparison_json",
            "overrepresented_sequence_tool_normalization_json",
        ]
    );

    let validation_stage = StageId::from_static("fastq.validate_reads");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&validation_stage),
        vec![
            "validation_tool_benchmark_cohort_json",
            "validation_tool_comparison_json",
            "validation_tool_normalization_json",
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&validation_stage),
        vec!["validation_report", "validated_reads_manifest"]
    );

    let report_qc_stage = StageId::from_static("fastq.report_qc");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&report_qc_stage),
        vec!["multiqc_report", "multiqc_data"]
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

#[test]
fn comparison_inputs_remain_inside_governed_stage_outputs() {
    for stage_id in [
        "fastq.trim_reads",
        "fastq.trim_polyg_tails",
        "fastq.remove_duplicates",
        "fastq.report_qc",
        "fastq.correct_errors",
        "fastq.trim_terminal_damage",
        "fastq.validate_reads",
    ] {
        let output_ids = bijux_dna_domain_fastq::stage_output_ids(stage_id)
            .expect("stage outputs must exist for governed benchmark stage");
        let output_ids = output_ids.into_iter().collect::<BTreeSet<_>>();
        let comparison_inputs = bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(
            &StageId::new(stage_id.to_string()),
        );
        assert!(
            !comparison_inputs.is_empty(),
            "comparison stage {stage_id} must publish governed comparison inputs",
        );
        for artifact_id in comparison_inputs {
            assert!(
                output_ids.contains(artifact_id),
                "comparison input {artifact_id} must remain a governed output of {stage_id}",
            );
        }
    }
}
