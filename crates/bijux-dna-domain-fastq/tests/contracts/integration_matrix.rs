use std::collections::BTreeSet;

use bijux_dna_core::ids::{StageId, ToolId};

#[test]
fn integration_matrix_covers_indexed_stage_tool_bindings() {
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
}

fn stage_tool_binding(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> bijux_dna_domain_fastq::StageToolBinding {
    bijux_dna_domain_fastq::stage_tool_binding(stage_id, tool_id)
        .unwrap_or_else(|| panic!("governed binding missing for {stage_id} / {tool_id}"))
}

fn governance_profile(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> bijux_dna_domain_fastq::StageToolGovernanceProfile {
    bijux_dna_domain_fastq::stage_tool_governance_profile(stage_id, tool_id).unwrap_or_else(|| {
        panic!("governance profile missing for stage {stage_id} and tool {tool_id}")
    })
}

fn capability_contract(
    stage_id: &StageId,
    tool_id: &ToolId,
    level: bijux_dna_domain_fastq::RuntimeNormalizationLevel,
) -> bijux_dna_domain_fastq::StageToolCapabilityContract {
    bijux_dna_domain_fastq::stage_tool_capability_contract(stage_id, tool_id, level).unwrap_or_else(
        || panic!("capability contract missing for stage {stage_id} and tool {tool_id}"),
    )
}

fn benchmark_governance(stage_id: &StageId) -> bijux_dna_domain_fastq::StageBenchmarkGovernance {
    bijux_dna_domain_fastq::stage_benchmark_governance(stage_id)
        .unwrap_or_else(|| panic!("benchmark governance missing for {stage_id}"))
}

#[test]
fn benchmark_scenarios_attach_to_governed_stages() {
    let trim_stage = StageId::from_static("fastq.trim_reads");
    let trim_scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&trim_stage);
    assert_eq!(trim_scenarios.len(), 1);
    assert_eq!(trim_scenarios[0].scenario_id, "trim_fairness");
    assert!(trim_scenarios[0]
        .fairness_rules
        .iter()
        .any(|rule| rule == "same_input_hash"));
    assert_eq!(
        trim_scenarios[0].comparison_artifact_id,
        "trim_tool_comparison_json"
    );

    let polyg_stage = StageId::from_static("fastq.trim_polyg_tails");
    let polyg_scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&polyg_stage);
    assert_eq!(polyg_scenarios.len(), 1);
    assert_eq!(polyg_scenarios[0].scenario_id, "polyg_trim_fairness");
    assert!(polyg_scenarios[0]
        .fairness_rules
        .iter()
        .any(|rule| rule == "same_polyg_trim_policy"));
    assert_eq!(
        polyg_scenarios[0].normalization_artifact_id,
        "polyg_trim_tool_normalization_json"
    );

    let screen_stage = StageId::from_static("fastq.screen_taxonomy");
    let screen_scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&screen_stage);
    assert_eq!(screen_scenarios.len(), 1);
    assert_eq!(screen_scenarios[0].scenario_id, "screen_fairness");
    assert!(screen_scenarios[0]
        .fairness_rules
        .iter()
        .any(|rule| rule == "same_contamination_db_hash"));
    assert_eq!(
        screen_scenarios[0].cohort_artifact_id,
        "taxonomy_tool_benchmark_cohort_json"
    );

    let filter_stage = StageId::from_static("fastq.filter_reads");
    let filter_scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&filter_stage);
    assert_eq!(filter_scenarios.len(), 1);
    assert_eq!(filter_scenarios[0].scenario_id, "filter_fairness");
    assert!(filter_scenarios[0]
        .fairness_rules
        .iter()
        .any(|rule| rule == "same_filter_contract_hash"));

    let merge_stage = StageId::from_static("fastq.merge_pairs");
    let merge_scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&merge_stage);
    assert_eq!(merge_scenarios.len(), 1);
    assert_eq!(merge_scenarios[0].scenario_id, "merge_fairness");
    assert!(merge_scenarios[0]
        .fairness_rules
        .iter()
        .any(|rule| rule == "same_merge_policy"));

    let low_complexity_stage = StageId::from_static("fastq.filter_low_complexity");
    let low_complexity_scenarios =
        bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&low_complexity_stage);
    assert_eq!(low_complexity_scenarios.len(), 1);
    assert_eq!(
        low_complexity_scenarios[0].scenario_id,
        "low_complexity_fairness"
    );
    assert!(low_complexity_scenarios[0]
        .fairness_rules
        .iter()
        .any(|rule| rule == "same_complexity_policy"));

    let dedup_stage = StageId::from_static("fastq.remove_duplicates");
    let dedup_scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&dedup_stage);
    assert_eq!(dedup_scenarios.len(), 1);
    assert_eq!(dedup_scenarios[0].scenario_id, "dedup_fairness");
    assert!(dedup_scenarios[0]
        .fairness_rules
        .iter()
        .any(|rule| rule == "same_dedup_policy"));
    assert!(dedup_scenarios[0]
        .fairness_rules
        .iter()
        .any(|rule| rule == "same_keep_order_policy"));

    let read_length_stage = StageId::from_static("fastq.profile_read_lengths");
    let read_length_scenarios =
        bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&read_length_stage);
    assert_eq!(read_length_scenarios.len(), 1);
    assert_eq!(read_length_scenarios[0].scenario_id, "read_length_fairness");
    assert!(read_length_scenarios[0]
        .fairness_rules
        .iter()
        .any(|rule| rule == "same_length_profile_contract"));

    let correction_stage = StageId::from_static("fastq.correct_errors");
    let correction_scenarios =
        bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&correction_stage);
    assert_eq!(correction_scenarios.len(), 1);
    assert_eq!(correction_scenarios[0].scenario_id, "correction_fairness");
    assert!(correction_scenarios[0]
        .fairness_rules
        .iter()
        .any(|rule| rule == "same_correction_policy"));

    let normalize_primers_stage = StageId::from_static("fastq.normalize_primers");
    let primer_scenarios =
        bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&normalize_primers_stage);
    assert_eq!(primer_scenarios.len(), 1);
    assert_eq!(
        primer_scenarios[0].scenario_id,
        "primer_normalization_fairness"
    );
    assert!(primer_scenarios[0]
        .fairness_rules
        .iter()
        .any(|rule| rule == "same_primer_contract"));

    let terminal_damage_stage = StageId::from_static("fastq.trim_terminal_damage");
    let terminal_damage_scenarios =
        bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&terminal_damage_stage);
    assert_eq!(terminal_damage_scenarios.len(), 1);
    assert_eq!(
        terminal_damage_scenarios[0].scenario_id,
        "terminal_damage_fairness"
    );
    assert!(terminal_damage_scenarios[0]
        .fairness_rules
        .iter()
        .any(|rule| rule == "same_damage_trim_policy"));

    let overrepresented_stage = StageId::from_static("fastq.profile_overrepresented_sequences");
    let overrepresented_scenarios =
        bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&overrepresented_stage);
    assert_eq!(overrepresented_scenarios.len(), 1);
    assert_eq!(
        overrepresented_scenarios[0].scenario_id,
        "overrepresented_sequence_fairness"
    );
    assert!(overrepresented_scenarios[0]
        .fairness_rules
        .iter()
        .any(|rule| rule == "same_overrepresented_sequence_contract"));

    let validation_stage = StageId::from_static("fastq.validate_reads");
    let validation_scenarios =
        bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&validation_stage);
    assert_eq!(validation_scenarios.len(), 1);
    assert_eq!(validation_scenarios[0].scenario_id, "validation_fairness");
    assert!(validation_scenarios[0]
        .fairness_rules
        .iter()
        .any(|rule| rule == "same_validation_contract"));
    assert_eq!(
        validation_scenarios[0].comparison_artifact_id,
        "validation_tool_comparison_json"
    );
}

#[test]
fn integration_matrix_distinguishes_governed_and_planned_bindings() {
    let infer_asvs_stage = StageId::from_static("fastq.infer_asvs");
    let dada2 = ToolId::from_static("dada2");
    let infer_binding = stage_tool_binding(&infer_asvs_stage, &dada2);
    assert_eq!(
        infer_binding.integration_level,
        bijux_dna_domain_fastq::ToolIntegrationLevel::GovernedContract
    );

    let trim_stage = StageId::from_static("fastq.trim_reads");
    let fastp = ToolId::from_static("fastp");
    let trim_binding = stage_tool_binding(&trim_stage, &fastp);
    assert_eq!(
        trim_binding.integration_level,
        bijux_dna_domain_fastq::ToolIntegrationLevel::GovernedContract
    );
}

#[test]
fn stage_tool_governance_profile_centralizes_benchmark_contract_truth() {
    let validation_profile = governance_profile(
        &StageId::from_static("fastq.validate_reads"),
        &ToolId::from_static("fastqvalidator"),
    );
    assert!(validation_profile.default_tool);
    assert!(validation_profile.admitted_runtime_tool);
    assert!(validation_profile.is_plannable());
    assert!(validation_profile.is_runnable());
    assert_eq!(
        validation_profile.benchmark_scenario_ids,
        vec!["validation_fairness"]
    );
    assert_eq!(
        validation_profile.comparison_input_artifact_ids,
        vec!["validation_report", "validated_reads_manifest"]
    );
    assert!(validation_profile.has_governed_benchmark_contract());
    assert_eq!(
        validation_profile.normalization_maturity(),
        bijux_dna_domain_fastq::StageToolNormalizationMaturity::ObserverSpecialized
    );
    assert_eq!(
        validation_profile.benchmark_contract_maturity(),
        bijux_dna_domain_fastq::StageToolBenchmarkContractMaturity::BenchmarkComparable
    );

    let infer_profile = governance_profile(
        &StageId::from_static("fastq.infer_asvs"),
        &ToolId::from_static("dada2"),
    );
    assert!(infer_profile.default_tool);
    assert!(infer_profile.admitted_runtime_tool);
    assert!(infer_profile.is_plannable());
    assert!(infer_profile.is_runnable());
    assert!(!infer_profile.has_governed_benchmark_contract());
    assert_eq!(
        infer_profile.normalization_maturity(),
        bijux_dna_domain_fastq::StageToolNormalizationMaturity::ObserverSpecialized
    );
    assert_eq!(
        infer_profile.benchmark_contract_maturity(),
        bijux_dna_domain_fastq::StageToolBenchmarkContractMaturity::None
    );

    let trim_profile = governance_profile(
        &StageId::from_static("fastq.trim_reads"),
        &ToolId::from_static("fastp"),
    );
    assert_eq!(
        trim_profile.normalization_maturity(),
        bijux_dna_domain_fastq::StageToolNormalizationMaturity::ObserverSpecialized
    );
    assert_eq!(
        trim_profile.benchmark_contract_maturity(),
        bijux_dna_domain_fastq::StageToolBenchmarkContractMaturity::GovernedBenchmarkCohort
    );
}

#[test]
fn governed_qc_contract_is_owned_by_domain() {
    let validation_stage = StageId::from_static("fastq.validate_reads");
    let validation_artifacts =
        bijux_dna_domain_fastq::governed_qc_output_ids_for_stage(&validation_stage);
    assert_eq!(
        validation_artifacts,
        vec![
            "validation_report".to_string(),
            "validated_reads_manifest".to_string()
        ]
    );

    let report_qc_stage = StageId::from_static("fastq.report_qc");
    assert!(bijux_dna_domain_fastq::governed_qc_output_ids_for_stage(&report_qc_stage).is_empty());

    let producers = bijux_dna_domain_fastq::governed_qc_producer_stage_ids();
    assert!(producers.contains(&validation_stage));
    assert!(!producers.contains(&report_qc_stage));
}

#[test]
fn stage_tool_capability_contract_is_owned_by_domain() {
    let trim_stage = StageId::from_static("fastq.trim_reads");
    let fastp = ToolId::from_static("fastp");
    let trim_capability = capability_contract(
        &trim_stage,
        &fastp,
        bijux_dna_domain_fastq::RuntimeNormalizationLevel::GenericEnvelope,
    );
    assert!(trim_capability.runnable);
    assert!(!trim_capability.parse_normalized);
    assert!(!trim_capability.benchmark_normalized);
    assert!(!trim_capability.comparable);

    let trim_observer_capability = capability_contract(
        &trim_stage,
        &fastp,
        bijux_dna_domain_fastq::RuntimeNormalizationLevel::ObserverSpecialized,
    );
    assert!(trim_observer_capability.runnable);
    assert!(trim_observer_capability.parse_normalized);
    assert!(trim_observer_capability.benchmark_normalized);
    assert!(!trim_observer_capability.comparable);

    let detect_stage = StageId::from_static("fastq.detect_adapters");
    let fastqc = ToolId::from_static("fastqc");
    let detect_capability = capability_contract(
        &detect_stage,
        &fastqc,
        bijux_dna_domain_fastq::RuntimeNormalizationLevel::ObserverSpecialized,
    );
    assert!(detect_capability.benchmark_normalized);
    assert!(detect_capability.comparable);

    let infer_stage = StageId::from_static("fastq.infer_asvs");
    let dada2 = ToolId::from_static("dada2");
    let infer_capability = capability_contract(
        &infer_stage,
        &dada2,
        bijux_dna_domain_fastq::RuntimeNormalizationLevel::GenericEnvelope,
    );
    assert!(infer_capability.runnable);
    assert!(!infer_capability.parse_normalized);

    let infer_observer_capability = capability_contract(
        &infer_stage,
        &dada2,
        bijux_dna_domain_fastq::RuntimeNormalizationLevel::ObserverSpecialized,
    );
    assert!(infer_observer_capability.runnable);
    assert!(infer_observer_capability.parse_normalized);
    assert!(!infer_observer_capability.benchmark_normalized);
    assert!(!infer_observer_capability.comparable);

    assert_eq!(
        bijux_dna_domain_fastq::benchmark_readiness_for_stage_tool(
            &trim_stage,
            &fastp,
            bijux_dna_domain_fastq::RuntimeNormalizationLevel::ObserverSpecialized,
        ),
        Some(bijux_dna_domain_fastq::BenchmarkReadinessLevel::GovernedBenchmarkCohort)
    );
}

#[test]
fn infer_asvs_governance_profile_exposes_closed_runtime_and_observer_surface() {
    let infer_stage = StageId::from_static("fastq.infer_asvs");
    let dada2 = ToolId::from_static("dada2");
    let profile = governance_profile(&infer_stage, &dada2);
    assert!(profile.default_tool);
    assert!(profile.admitted_runtime_tool);
    assert!(profile.is_plannable());
    assert!(profile.is_runnable());
    assert_eq!(
        profile.normalization_maturity(),
        bijux_dna_domain_fastq::StageToolNormalizationMaturity::ObserverSpecialized
    );
    assert_eq!(
        profile.benchmark_contract_maturity(),
        bijux_dna_domain_fastq::StageToolBenchmarkContractMaturity::None
    );
    assert!(!profile.has_governed_benchmark_contract());

    let generic_capability = capability_contract(
        &infer_stage,
        &dada2,
        bijux_dna_domain_fastq::RuntimeNormalizationLevel::GenericEnvelope,
    );
    assert!(generic_capability.runnable);
    assert!(!generic_capability.parse_normalized);

    let observer_capability = capability_contract(
        &infer_stage,
        &dada2,
        bijux_dna_domain_fastq::RuntimeNormalizationLevel::ObserverSpecialized,
    );
    assert!(observer_capability.runnable);
    assert!(observer_capability.parse_normalized);
    assert_eq!(
        bijux_dna_domain_fastq::benchmark_readiness_for_stage_tool(
            &infer_stage,
            &dada2,
            bijux_dna_domain_fastq::RuntimeNormalizationLevel::ObserverSpecialized,
        ),
        Some(bijux_dna_domain_fastq::BenchmarkReadinessLevel::GovernedExecution)
    );
}

#[test]
fn stage_benchmark_governance_centralizes_stage_fairness_contracts() {
    let report_qc = benchmark_governance(&StageId::from_static("fastq.report_qc"));
    assert!(report_qc.has_governed_benchmark_contract());
    assert_eq!(report_qc.scenarios.len(), 1);
    assert_eq!(
        report_qc.scenarios[0].scenario_id,
        "qc_aggregation_fairness"
    );
    assert_eq!(
        report_qc.comparison_input_artifact_ids,
        vec![
            "report_json",
            "governed_qc_inputs_manifest",
            "multiqc_report",
            "multiqc_data"
        ]
    );

    let polyg = benchmark_governance(&StageId::from_static("fastq.trim_polyg_tails"));
    assert!(polyg.has_governed_benchmark_contract());
    assert_eq!(polyg.scenarios[0].scenario_id, "polyg_trim_fairness");
}

#[test]
fn reference_index_compatibility_is_queryable_from_domain_api() {
    let bowtie2 = ToolId::from_static("bowtie2");
    let backends = bijux_dna_domain_fastq::reference_index_backends_for_tool(&bowtie2);
    assert_eq!(backends, vec![ToolId::from_static("bowtie2_build")]);
    assert!(
        bijux_dna_domain_fastq::is_reference_index_backend_compatible(
            &bowtie2,
            &ToolId::from_static("bowtie2_build"),
        )
    );
    assert!(
        !bijux_dna_domain_fastq::is_reference_index_backend_compatible(
            &bowtie2,
            &ToolId::from_static("star"),
        )
    );
}
