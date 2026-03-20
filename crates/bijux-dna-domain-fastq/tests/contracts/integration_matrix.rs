use std::collections::BTreeSet;

use anyhow::Result;
use bijux_dna_core::ids::{StageId, ToolId};

#[test]
fn integration_matrix_covers_indexed_stage_tool_bindings() -> Result<()> {
    let indexed = bijux_dna_domain_fastq::stage_tool_bindings()
        .into_iter()
        .map(|binding| {
            (
                binding.stage_id.as_str().to_string(),
                binding.tool_id.as_str().to_string(),
                format!("{:?}", binding.integration_level),
            )
        })
        .collect::<BTreeSet<_>>();
    let from_stage_api = bijux_dna_domain_fastq::STAGES
        .iter()
        .flat_map(bijux_dna_domain_fastq::stage_tool_bindings_for_stage)
        .map(|binding| {
            (
                binding.stage_id.as_str().to_string(),
                binding.tool_id.as_str().to_string(),
                format!("{:?}", binding.integration_level),
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        indexed, from_stage_api,
        "stage_tool_bindings_for_stage must partition the full integration matrix"
    );
    Ok(())
}

#[test]
fn benchmark_scenarios_attach_to_governed_stages() {
    let trim_stage = StageId::from_static("fastq.trim_reads");
    let trim_scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&trim_stage);
    assert_eq!(trim_scenarios.len(), 1);
    assert_eq!(trim_scenarios[0].scenario_id, "trim_fairness");
    assert!(
        trim_scenarios[0]
            .fairness_rules
            .iter()
            .any(|rule| rule == "same_input_hash")
    );

    let screen_stage = StageId::from_static("fastq.screen_taxonomy");
    let screen_scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&screen_stage);
    assert_eq!(screen_scenarios.len(), 1);
    assert_eq!(screen_scenarios[0].scenario_id, "screen_fairness");
    assert!(
        screen_scenarios[0]
            .fairness_rules
            .iter()
            .any(|rule| rule == "same_contamination_db_hash")
    );

    let filter_stage = StageId::from_static("fastq.filter_reads");
    let filter_scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&filter_stage);
    assert_eq!(filter_scenarios.len(), 1);
    assert_eq!(filter_scenarios[0].scenario_id, "filter_fairness");
    assert!(
        filter_scenarios[0]
            .fairness_rules
            .iter()
            .any(|rule| rule == "same_filter_contract_hash")
    );

    let merge_stage = StageId::from_static("fastq.merge_pairs");
    let merge_scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&merge_stage);
    assert_eq!(merge_scenarios.len(), 1);
    assert_eq!(merge_scenarios[0].scenario_id, "merge_fairness");
    assert!(
        merge_scenarios[0]
            .fairness_rules
            .iter()
            .any(|rule| rule == "same_merge_policy")
    );

    let low_complexity_stage = StageId::from_static("fastq.filter_low_complexity");
    let low_complexity_scenarios =
        bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&low_complexity_stage);
    assert_eq!(low_complexity_scenarios.len(), 1);
    assert_eq!(low_complexity_scenarios[0].scenario_id, "low_complexity_fairness");
    assert!(
        low_complexity_scenarios[0]
            .fairness_rules
            .iter()
            .any(|rule| rule == "same_complexity_policy")
    );

    let dedup_stage = StageId::from_static("fastq.remove_duplicates");
    let dedup_scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&dedup_stage);
    assert_eq!(dedup_scenarios.len(), 1);
    assert_eq!(dedup_scenarios[0].scenario_id, "dedup_fairness");
    assert!(
        dedup_scenarios[0]
            .fairness_rules
            .iter()
            .any(|rule| rule == "same_dedup_policy")
    );

    let read_length_stage = StageId::from_static("fastq.profile_read_lengths");
    let read_length_scenarios =
        bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&read_length_stage);
    assert_eq!(read_length_scenarios.len(), 1);
    assert_eq!(read_length_scenarios[0].scenario_id, "read_length_fairness");
    assert!(
        read_length_scenarios[0]
            .fairness_rules
            .iter()
            .any(|rule| rule == "same_length_profile_contract")
    );

    let correction_stage = StageId::from_static("fastq.correct_errors");
    let correction_scenarios =
        bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&correction_stage);
    assert_eq!(correction_scenarios.len(), 1);
    assert_eq!(correction_scenarios[0].scenario_id, "correction_fairness");
    assert!(
        correction_scenarios[0]
            .fairness_rules
            .iter()
            .any(|rule| rule == "same_correction_policy")
    );

    let normalize_primers_stage = StageId::from_static("fastq.normalize_primers");
    let primer_scenarios =
        bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&normalize_primers_stage);
    assert_eq!(primer_scenarios.len(), 1);
    assert_eq!(primer_scenarios[0].scenario_id, "primer_normalization_fairness");
    assert!(
        primer_scenarios[0]
            .fairness_rules
            .iter()
            .any(|rule| rule == "same_primer_contract")
    );
}

#[test]
fn integration_matrix_distinguishes_governed_and_planned_bindings() {
    let infer_asvs_stage = StageId::from_static("fastq.infer_asvs");
    let dada2 = ToolId::from_static("dada2");
    let infer_binding = bijux_dna_domain_fastq::stage_tool_binding(&infer_asvs_stage, &dada2)
        .expect("planned binding");
    assert_eq!(
        infer_binding.integration_level,
        bijux_dna_domain_fastq::ToolIntegrationLevel::PlannedContract
    );

    let trim_stage = StageId::from_static("fastq.trim_reads");
    let fastp = ToolId::from_static("fastp");
    let trim_binding = bijux_dna_domain_fastq::stage_tool_binding(&trim_stage, &fastp)
        .expect("governed binding");
    assert_eq!(
        trim_binding.integration_level,
        bijux_dna_domain_fastq::ToolIntegrationLevel::GovernedContract
    );
}

#[test]
fn reference_index_compatibility_is_queryable_from_domain_api() {
    let bowtie2 = ToolId::from_static("bowtie2");
    let backends = bijux_dna_domain_fastq::reference_index_backends_for_tool(&bowtie2);
    assert_eq!(backends, vec![ToolId::from_static("bowtie2_build")]);
    assert!(bijux_dna_domain_fastq::is_reference_index_backend_compatible(
        &bowtie2,
        &ToolId::from_static("bowtie2_build"),
    ));
    assert!(!bijux_dna_domain_fastq::is_reference_index_backend_compatible(
        &bowtie2,
        &ToolId::from_static("star"),
    ));
}
