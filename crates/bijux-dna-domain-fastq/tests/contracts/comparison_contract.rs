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
            "trim_tool_benchmark_cohort_json".to_string(),
            "trim_tool_comparison_json".to_string(),
            "trim_tool_normalization_json".to_string(),
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
        vec![
            "trimmed_reads_r1".to_string(),
            "trimmed_reads_r2".to_string(),
            "report_json".to_string()
        ]
    );

    let screen_stage = StageId::from_static("fastq.screen_taxonomy");
    let screen_artifacts = bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&screen_stage);
    assert_eq!(
        screen_artifacts,
        vec![
            "taxonomy_tool_benchmark_cohort_json".to_string(),
            "taxonomy_tool_comparison_json".to_string(),
            "taxonomy_tool_normalization_json".to_string(),
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
            "adapter_inspection_tool_benchmark_cohort_json".to_string(),
            "adapter_inspection_tool_comparison_json".to_string(),
            "adapter_inspection_tool_normalization_json".to_string(),
        ],
        "adapter inspection now participates in governed benchmark comparisons",
    );

    let filter_stage = StageId::from_static("fastq.filter_reads");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&filter_stage),
        vec![
            "filter_tool_benchmark_cohort_json".to_string(),
            "filter_tool_comparison_json".to_string(),
            "filter_tool_normalization_json".to_string(),
        ]
    );

    let merge_stage = StageId::from_static("fastq.merge_pairs");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&merge_stage),
        vec![
            "merge_tool_benchmark_cohort_json".to_string(),
            "merge_tool_comparison_json".to_string(),
            "merge_tool_normalization_json".to_string(),
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&merge_stage),
        vec![
            "report_json".to_string(),
            "merged_reads".to_string(),
            "unmerged_reads_r1".to_string(),
            "unmerged_reads_r2".to_string(),
        ]
    );

    let low_complexity_stage = StageId::from_static("fastq.filter_low_complexity");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&low_complexity_stage),
        vec![
            "low_complexity_tool_benchmark_cohort_json".to_string(),
            "low_complexity_tool_comparison_json".to_string(),
            "low_complexity_tool_normalization_json".to_string(),
        ]
    );

    let dedup_stage = StageId::from_static("fastq.remove_duplicates");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&dedup_stage),
        vec![
            "dedup_tool_benchmark_cohort_json".to_string(),
            "dedup_tool_comparison_json".to_string(),
            "dedup_tool_normalization_json".to_string(),
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&dedup_stage),
        vec![
            "report_json".to_string(),
            "duplicate_provenance_json".to_string(),
            "duplicate_classes_tsv".to_string(),
            "dedup_reads_r1".to_string(),
            "dedup_reads_r2".to_string(),
        ]
    );

    let read_length_stage = StageId::from_static("fastq.profile_read_lengths");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&read_length_stage),
        vec![
            "read_length_tool_benchmark_cohort_json".to_string(),
            "read_length_tool_comparison_json".to_string(),
            "read_length_tool_normalization_json".to_string(),
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&read_length_stage),
        vec![
            "report_json".to_string(),
            "length_distribution_tsv".to_string(),
            "length_distribution_json".to_string(),
        ]
    );

    let correction_stage = StageId::from_static("fastq.correct_errors");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&correction_stage),
        vec![
            "correction_tool_benchmark_cohort_json".to_string(),
            "correction_tool_comparison_json".to_string(),
            "correction_tool_normalization_json".to_string(),
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&correction_stage),
        vec![
            "report_json".to_string(),
            "corrected_reads_r1".to_string(),
            "corrected_reads_r2".to_string(),
        ]
    );

    let normalize_primers_stage = StageId::from_static("fastq.normalize_primers");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&normalize_primers_stage),
        vec![
            "primer_normalization_tool_benchmark_cohort_json".to_string(),
            "primer_normalization_tool_comparison_json".to_string(),
            "primer_normalization_tool_normalization_json".to_string(),
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&normalize_primers_stage),
        vec![
            "report_json".to_string(),
            "primer_orientation_report".to_string(),
            "primer_stats_json".to_string(),
            "normalized_reads_r1".to_string(),
            "normalized_reads_r2".to_string(),
        ]
    );

    let terminal_damage_stage = StageId::from_static("fastq.trim_terminal_damage");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&terminal_damage_stage),
        vec![
            "terminal_damage_tool_benchmark_cohort_json".to_string(),
            "terminal_damage_tool_comparison_json".to_string(),
            "terminal_damage_tool_normalization_json".to_string(),
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&terminal_damage_stage),
        vec![
            "trimmed_reads_r1".to_string(),
            "trimmed_reads_r2".to_string(),
            "report_json".to_string()
        ]
    );

    let normalize_abundance_stage = StageId::from_static("fastq.normalize_abundance");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&normalize_abundance_stage),
        vec![
            "normalize_abundance_tool_benchmark_cohort_json".to_string(),
            "normalize_abundance_tool_comparison_json".to_string(),
            "normalize_abundance_tool_normalization_json".to_string(),
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&normalize_abundance_stage),
        vec![
            "report_json".to_string(),
            "normalized_abundance_tsv".to_string(),
        ]
    );

    let polyg_stage = StageId::from_static("fastq.trim_polyg_tails");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&polyg_stage),
        vec![
            "polyg_trim_tool_benchmark_cohort_json".to_string(),
            "polyg_trim_tool_comparison_json".to_string(),
            "polyg_trim_tool_normalization_json".to_string(),
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&polyg_stage),
        vec![
            "trimmed_reads_r1".to_string(),
            "trimmed_reads_r2".to_string(),
            "report_json".to_string()
        ]
    );

    let overrepresented_stage = StageId::from_static("fastq.profile_overrepresented_sequences");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&overrepresented_stage),
        vec![
            "overrepresented_sequence_tool_benchmark_cohort_json".to_string(),
            "overrepresented_sequence_tool_comparison_json".to_string(),
            "overrepresented_sequence_tool_normalization_json".to_string(),
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&overrepresented_stage),
        vec![
            "report_json".to_string(),
            "overrepresented_sequences_tsv".to_string(),
            "overrepresented_sequences_json".to_string(),
        ]
    );

    let validation_stage = StageId::from_static("fastq.validate_reads");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&validation_stage),
        vec![
            "validation_tool_benchmark_cohort_json".to_string(),
            "validation_tool_comparison_json".to_string(),
            "validation_tool_normalization_json".to_string(),
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&validation_stage),
        vec![
            "validated_reads_manifest".to_string(),
            "validation_report".to_string()
        ]
    );

    let report_qc_stage = StageId::from_static("fastq.report_qc");
    assert_eq!(
        bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&report_qc_stage),
        vec![
            "report_json".to_string(),
            "governed_qc_inputs_manifest".to_string(),
        ]
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
        "fastq.profile_overrepresented_sequences",
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
                output_ids.contains(&artifact_id),
                "comparison input {artifact_id} must remain a governed output of {stage_id}",
            );
        }
    }
}
