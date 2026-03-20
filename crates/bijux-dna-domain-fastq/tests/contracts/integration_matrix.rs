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
