use anyhow::Result;
use bijux_dna_core::ids::StageId;
use std::collections::BTreeSet;

fn comparison_contract(stage_id: &StageId) -> bijux_dna_domain_fastq::StageComparisonContract {
    bijux_dna_domain_fastq::comparison_contract_for_stage(stage_id)
        .unwrap_or_else(|| panic!("comparison contract missing for {}", stage_id.as_str()))
}

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn assert_comparison_stage_contract(
    stage_name: &str,
    artifacts: &[&str],
    comparison_artifact_id: Option<&str>,
    inputs: Option<&[&str]>,
) {
    let stage_id = StageId::new(stage_name.to_string());
    assert_eq!(
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&stage_id),
        strings(artifacts),
    );
    if let Some(expected_comparison_artifact_id) = comparison_artifact_id {
        assert_eq!(
            comparison_contract(&stage_id).comparison_artifact_id,
            expected_comparison_artifact_id
        );
    }
    if let Some(expected_inputs) = inputs {
        assert_eq!(
            bijux_dna_domain_fastq::comparison_input_artifact_ids_for_stage(&stage_id),
            strings(expected_inputs),
        );
    }
}

#[test]
fn preprocess_stages_publish_comparison_artifact_contracts() {
    assert_comparison_stage_contract(
        "fastq.trim_reads",
        &[
            "trim_tool_benchmark_cohort_json",
            "trim_tool_comparison_json",
            "trim_tool_normalization_json",
        ],
        Some("trim_tool_comparison_json"),
        Some(&["trimmed_reads_r1", "trimmed_reads_r2", "report_json"]),
    );
    assert_comparison_stage_contract(
        "fastq.filter_reads",
        &[
            "filter_tool_benchmark_cohort_json",
            "filter_tool_comparison_json",
            "filter_tool_normalization_json",
        ],
        None,
        None,
    );
    assert_comparison_stage_contract(
        "fastq.merge_pairs",
        &[
            "merge_tool_benchmark_cohort_json",
            "merge_tool_comparison_json",
            "merge_tool_normalization_json",
        ],
        None,
        Some(&["report_json", "merged_reads", "unmerged_reads_r1", "unmerged_reads_r2"]),
    );
    assert_comparison_stage_contract(
        "fastq.filter_low_complexity",
        &[
            "low_complexity_tool_benchmark_cohort_json",
            "low_complexity_tool_comparison_json",
            "low_complexity_tool_normalization_json",
        ],
        None,
        None,
    );
    assert_comparison_stage_contract(
        "fastq.remove_duplicates",
        &[
            "dedup_tool_benchmark_cohort_json",
            "dedup_tool_comparison_json",
            "dedup_tool_normalization_json",
        ],
        None,
        Some(&[
            "report_json",
            "duplicate_provenance_json",
            "duplicate_classes_tsv",
            "dedup_reads_r1",
            "dedup_reads_r2",
        ]),
    );
    assert_comparison_stage_contract(
        "fastq.correct_errors",
        &[
            "correction_tool_benchmark_cohort_json",
            "correction_tool_comparison_json",
            "correction_tool_normalization_json",
        ],
        None,
        Some(&["report_json", "corrected_reads_r1", "corrected_reads_r2"]),
    );
}

#[test]
fn trim_backend_comparison_contract_exposes_required_tools_and_caveats() {
    let contract = bijux_dna_domain_fastq::trim_backend_comparison_contract();
    let required_tools = contract
        .required_tool_ids
        .iter()
        .map(bijux_dna_core::contract::ToolId::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        required_tools,
        BTreeSet::from(["adapterremoval", "cutadapt", "fastp", "fastx_clipper", "trimmomatic"])
    );
    assert!(
        contract
            .comparison_tool_profiles
            .iter()
            .any(|profile| profile.tool_id.as_str() == "fastx_clipper" && profile.legacy_backend),
        "trim comparison contract must flag at least one legacy backend"
    );
    assert!(
        contract.comparison_tool_profiles.iter().all(|profile| !profile.caveats.is_empty()),
        "every trim comparison backend must expose caveats"
    );
    assert_eq!(
        contract.normalized_metric_ids,
        vec![
            "reads_in",
            "reads_out",
            "bases_in",
            "bases_out",
            "pairs_in",
            "pairs_out",
            "mean_q_before",
            "mean_q_after",
        ]
    );
}

#[test]
fn profiling_stages_publish_comparison_artifact_contracts() {
    assert_comparison_stage_contract(
        "fastq.screen_taxonomy",
        &[
            "taxonomy_tool_benchmark_cohort_json",
            "taxonomy_tool_comparison_json",
            "taxonomy_tool_normalization_json",
        ],
        Some("taxonomy_tool_comparison_json"),
        None,
    );
    assert_comparison_stage_contract(
        "fastq.detect_adapters",
        &[
            "adapter_inspection_tool_benchmark_cohort_json",
            "adapter_inspection_tool_comparison_json",
            "adapter_inspection_tool_normalization_json",
        ],
        None,
        None,
    );
    assert_comparison_stage_contract(
        "fastq.profile_read_lengths",
        &[
            "read_length_tool_benchmark_cohort_json",
            "read_length_tool_comparison_json",
            "read_length_tool_normalization_json",
        ],
        None,
        Some(&["report_json", "length_distribution_tsv", "length_distribution_json"]),
    );
    assert_comparison_stage_contract(
        "fastq.profile_reads",
        &[
            "profile_reads_tool_benchmark_cohort_json",
            "profile_reads_tool_comparison_json",
            "profile_reads_tool_normalization_json",
        ],
        None,
        Some(&["qc_json", "qc_tsv", "qc_plots_dir"]),
    );
    assert_comparison_stage_contract(
        "fastq.profile_overrepresented_sequences",
        &[
            "overrepresented_sequence_tool_benchmark_cohort_json",
            "overrepresented_sequence_tool_comparison_json",
            "overrepresented_sequence_tool_normalization_json",
        ],
        None,
        Some(&["report_json", "overrepresented_sequences_tsv", "overrepresented_sequences_json"]),
    );
    assert_comparison_stage_contract(
        "fastq.validate_reads",
        &[
            "validation_tool_benchmark_cohort_json",
            "validation_tool_comparison_json",
            "validation_tool_normalization_json",
        ],
        None,
        Some(&["validation_report", "validated_reads_manifest"]),
    );
    assert_comparison_stage_contract(
        "fastq.report_qc",
        &[
            "qc_aggregation_tool_benchmark_cohort_json",
            "qc_aggregation_tool_comparison_json",
            "qc_aggregation_tool_normalization_json",
        ],
        Some("qc_aggregation_tool_comparison_json"),
        Some(&["report_json", "governed_qc_inputs_manifest", "multiqc_report", "multiqc_data"]),
    );
}

#[test]
fn edna_stages_publish_comparison_artifact_contracts() {
    assert_comparison_stage_contract(
        "fastq.normalize_primers",
        &[
            "primer_normalization_tool_benchmark_cohort_json",
            "primer_normalization_tool_comparison_json",
            "primer_normalization_tool_normalization_json",
        ],
        None,
        Some(&[
            "report_json",
            "primer_orientation_report",
            "primer_stats_json",
            "normalized_reads_r1",
            "normalized_reads_r2",
        ]),
    );
    assert_comparison_stage_contract(
        "fastq.trim_terminal_damage",
        &[
            "terminal_damage_tool_benchmark_cohort_json",
            "terminal_damage_tool_comparison_json",
            "terminal_damage_tool_normalization_json",
        ],
        None,
        Some(&["trimmed_reads_r1", "trimmed_reads_r2", "report_json"]),
    );
    assert_comparison_stage_contract(
        "fastq.normalize_abundance",
        &[
            "normalize_abundance_tool_benchmark_cohort_json",
            "normalize_abundance_tool_comparison_json",
            "normalize_abundance_tool_normalization_json",
        ],
        None,
        Some(&["report_json", "normalized_abundance_tsv"]),
    );
    assert_comparison_stage_contract(
        "fastq.remove_chimeras",
        &[
            "chimera_tool_benchmark_cohort_json",
            "chimera_tool_comparison_json",
            "chimera_tool_normalization_json",
        ],
        None,
        Some(&[
            "report_json",
            "uchime_report_tsv",
            "chimera_metrics_json",
            "chimera_filtered_reads",
            "chimeras_fasta",
        ]),
    );
    assert_comparison_stage_contract(
        "fastq.trim_polyg_tails",
        &[
            "polyg_trim_tool_benchmark_cohort_json",
            "polyg_trim_tool_comparison_json",
            "polyg_trim_tool_normalization_json",
        ],
        None,
        Some(&["trimmed_reads_r1", "trimmed_reads_r2", "report_json"]),
    );
}

#[test]
fn comparison_artifacts_stay_inside_fastq_artifact_vocabulary() -> Result<()> {
    let yaml = std::fs::read_to_string(workspace_root().join("domain/fastq/artifacts.yaml"))?;
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
        "fastq.profile_read_lengths",
        "fastq.profile_reads",
        "fastq.normalize_primers",
        "fastq.normalize_abundance",
        "fastq.remove_chimeras",
        "fastq.trim_terminal_damage",
        "fastq.validate_reads",
        "fastq.profile_overrepresented_sequences",
    ] {
        let output_ids = bijux_dna_domain_fastq::stage_output_ids(stage_id)
            .unwrap_or_else(|| panic!("stage outputs must exist for governed benchmark stage"));
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
