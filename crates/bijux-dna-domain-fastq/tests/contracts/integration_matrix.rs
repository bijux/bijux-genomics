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
    let from_stage_api = bijux_dna_domain_fastq::FASTQ_STAGE_ID_CATALOG
        .iter()
        .map(|stage_id| StageId::from_static(stage_id))
        .flat_map(|stage_id| bijux_dna_domain_fastq::stage_tool_bindings_for_stage(&stage_id))
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

fn assert_benchmark_scenario(
    stage_name: &str,
    scenario_id: &str,
    fairness_rule: &str,
    comparison_artifact_id: Option<&str>,
    normalization_artifact_id: Option<&str>,
    cohort_artifact_id: Option<&str>,
) {
    let stage_id = StageId::new(stage_name.to_string());
    let scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&stage_id);
    assert_eq!(scenarios.len(), 1);
    assert_eq!(scenarios[0].scenario_id, scenario_id);
    assert!(scenarios[0].fairness_rules.iter().any(|rule| rule == fairness_rule));
    if let Some(expected_comparison_artifact_id) = comparison_artifact_id {
        assert_eq!(scenarios[0].comparison_artifact_id, expected_comparison_artifact_id);
    }
    if let Some(expected_normalization_artifact_id) = normalization_artifact_id {
        assert_eq!(scenarios[0].normalization_artifact_id, expected_normalization_artifact_id);
    }
    if let Some(expected_cohort_artifact_id) = cohort_artifact_id {
        assert_eq!(scenarios[0].cohort_artifact_id, expected_cohort_artifact_id);
    }
}

#[test]
fn preprocessing_benchmark_scenarios_attach_to_governed_stages() {
    assert_benchmark_scenario(
        "fastq.trim_reads",
        "trim_fairness",
        "same_input_hash",
        Some("trim_tool_comparison_json"),
        None,
        None,
    );
    assert_benchmark_scenario(
        "fastq.trim_polyg_tails",
        "polyg_trim_fairness",
        "same_polyg_trim_policy",
        None,
        Some("polyg_trim_tool_normalization_json"),
        None,
    );
    assert_benchmark_scenario(
        "fastq.screen_taxonomy",
        "screen_fairness",
        "same_contamination_db_hash",
        None,
        None,
        Some("taxonomy_tool_benchmark_cohort_json"),
    );
    assert_benchmark_scenario(
        "fastq.filter_reads",
        "filter_fairness",
        "same_filter_contract_hash",
        None,
        None,
        None,
    );
    assert_benchmark_scenario(
        "fastq.merge_pairs",
        "merge_fairness",
        "same_merge_policy",
        None,
        None,
        None,
    );
    assert_benchmark_scenario(
        "fastq.filter_low_complexity",
        "low_complexity_fairness",
        "same_complexity_policy",
        None,
        None,
        None,
    );
    assert_benchmark_scenario(
        "fastq.profile_read_lengths",
        "read_length_fairness",
        "same_length_profile_contract",
        None,
        None,
        None,
    );
    assert_benchmark_scenario(
        "fastq.correct_errors",
        "correction_fairness",
        "same_correction_policy",
        None,
        None,
        None,
    );
    assert_benchmark_scenario(
        "fastq.trim_terminal_damage",
        "terminal_damage_fairness",
        "same_damage_trim_policy",
        None,
        None,
        None,
    );
    assert_benchmark_scenario(
        "fastq.validate_reads",
        "validation_fairness",
        "same_validation_contract",
        Some("validation_tool_comparison_json"),
        None,
        None,
    );
}

#[test]
fn specialized_benchmark_scenarios_attach_to_governed_stages() {
    assert_benchmark_scenario(
        "fastq.screen_taxonomy",
        "screen_fairness",
        "same_contamination_db_hash",
        None,
        None,
        Some("taxonomy_tool_benchmark_cohort_json"),
    );
    assert_benchmark_scenario(
        "fastq.normalize_primers",
        "primer_normalization_fairness",
        "same_primer_contract",
        None,
        None,
        None,
    );
    assert_benchmark_scenario(
        "fastq.profile_overrepresented_sequences",
        "overrepresented_sequence_fairness",
        "same_overrepresented_sequence_contract",
        None,
        None,
        None,
    );
    let dedup_stage = StageId::from_static("fastq.remove_duplicates");
    let dedup_scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&dedup_stage);
    assert_eq!(dedup_scenarios.len(), 1);
    assert_eq!(dedup_scenarios[0].scenario_id, "dedup_fairness");
    assert!(dedup_scenarios[0].fairness_rules.iter().any(|rule| rule == "same_dedup_policy"));
    assert!(dedup_scenarios[0].fairness_rules.iter().any(|rule| rule == "same_keep_order_policy"));
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
fn stage_tool_registration_queries_keep_planned_tools_visible_but_not_runnable() {
    let trim_stage = StageId::from_static("fastq.trim_reads");

    assert!(bijux_dna_domain_fastq::registered_tool_ids_for_stage(&trim_stage)
        .contains(&ToolId::from_static("seqpurge")));
    assert!(bijux_dna_domain_fastq::planned_tool_ids_for_stage(&trim_stage)
        .contains(&ToolId::from_static("seqpurge")));
    assert!(!bijux_dna_domain_fastq::governed_tool_ids_for_stage(&trim_stage)
        .contains(&ToolId::from_static("seqpurge")));
    assert!(!bijux_dna_domain_fastq::admitted_execution_tools_for_stage(&trim_stage)
        .contains(&ToolId::from_static("seqpurge")));

    let detect_duplicates_stage = StageId::from_static("fastq.detect_duplicates_premerge");
    assert_eq!(
        bijux_dna_domain_fastq::registered_tool_ids_for_stage(&detect_duplicates_stage),
        vec![ToolId::from_static("bijux_dna")]
    );
    assert!(bijux_dna_domain_fastq::governed_tool_ids_for_stage(&detect_duplicates_stage)
        .contains(&ToolId::from_static("bijux_dna")));
    assert!(bijux_dna_domain_fastq::admitted_execution_tools_for_stage(&detect_duplicates_stage)
        .contains(&ToolId::from_static("bijux_dna")));

    let declared_stage = StageId::from_static("fastq.build_contaminant_db");
    assert_eq!(
        bijux_dna_domain_fastq::registered_tool_ids_for_stage(&declared_stage),
        vec![ToolId::from_static("bijux_dna")]
    );
    assert!(bijux_dna_domain_fastq::governed_tool_ids_for_stage(&declared_stage).is_empty());
    assert!(bijux_dna_domain_fastq::admitted_execution_tools_for_stage(&declared_stage).is_empty());
}

#[test]
fn multi_tool_comparable_stages_publish_shared_sanity_metrics() {
    let comparable_multi_tool_stages = bijux_dna_domain_fastq::comparable_benchmark_stage_ids()
        .into_iter()
        .filter(|stage_id| {
            bijux_dna_domain_fastq::admitted_execution_tools_for_stage(stage_id).len() >= 2
        })
        .collect::<Vec<_>>();

    assert_eq!(
        comparable_multi_tool_stages
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "fastq.index_reference".to_string(),
            "fastq.profile_overrepresented_sequences".to_string(),
            "fastq.validate_reads".to_string(),
        ]),
        "the governed multi-tool FASTQ comparable slice must stay explicit"
    );

    for stage_id in &comparable_multi_tool_stages {
        let metrics = bijux_dna_domain_fastq::stage_sanity_metrics_for_stage(stage_id);
        assert!(
            !metrics.is_empty(),
            "multi-tool comparable stage `{stage_id}` must publish shared sanity metrics"
        );
    }

    assert_eq!(
        bijux_dna_domain_fastq::stage_sanity_metrics_for_stage(&StageId::from_static(
            "fastq.index_reference",
        )),
        vec!["index_build_exit_code".to_string()]
    );
    assert_eq!(
        bijux_dna_domain_fastq::stage_sanity_metrics_for_stage(&StageId::from_static(
            "fastq.validate_reads",
        )),
        vec!["format_validation_pass_rate".to_string()]
    );
    assert_eq!(
        bijux_dna_domain_fastq::stage_sanity_metrics_for_stage(&StageId::from_static(
            "fastq.profile_overrepresented_sequences",
        )),
        vec![
            "sequence_count".to_string(),
            "flagged_sequences".to_string(),
            "top_fraction".to_string(),
        ]
    );
    assert_eq!(
        bijux_dna_domain_fastq::stage_sanity_metrics_for_stage(&StageId::from_static(
            "fastq.screen_taxonomy",
        )),
        vec![
            "taxonomy_database_id".to_string(),
            "classified_reads".to_string(),
            "unclassified_reads".to_string(),
            "classified_fraction".to_string(),
            "unclassified_fraction".to_string(),
            "top_taxa".to_string(),
        ]
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
    assert_eq!(validation_profile.benchmark_scenario_ids, vec!["validation_fairness"]);
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

    let detect_duplicates_profile = governance_profile(
        &StageId::from_static("fastq.detect_duplicates_premerge"),
        &ToolId::from_static("bijux_dna"),
    );
    assert!(detect_duplicates_profile.default_tool);
    assert!(detect_duplicates_profile.admitted_runtime_tool);
    assert!(detect_duplicates_profile.is_plannable());
    assert!(detect_duplicates_profile.is_runnable());
    assert!(!detect_duplicates_profile.has_governed_benchmark_contract());
    assert_eq!(
        detect_duplicates_profile.normalization_maturity(),
        bijux_dna_domain_fastq::StageToolNormalizationMaturity::GenericEnvelope
    );
    assert_eq!(
        detect_duplicates_profile.benchmark_contract_maturity(),
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
        vec!["validation_report".to_string(), "validated_reads_manifest".to_string()]
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

    let detect_duplicates_stage = StageId::from_static("fastq.detect_duplicates_premerge");
    let bijux_dna = ToolId::from_static("bijux_dna");
    let detect_duplicates_capability = capability_contract(
        &detect_duplicates_stage,
        &bijux_dna,
        bijux_dna_domain_fastq::RuntimeNormalizationLevel::GenericEnvelope,
    );
    assert!(detect_duplicates_capability.runnable);
    assert!(detect_duplicates_capability.parse_normalized);
    assert!(!detect_duplicates_capability.benchmark_normalized);
    assert!(!detect_duplicates_capability.comparable);

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
    assert_eq!(
        bijux_dna_domain_fastq::benchmark_readiness_for_stage_tool(
            &detect_duplicates_stage,
            &bijux_dna,
            bijux_dna_domain_fastq::RuntimeNormalizationLevel::GenericEnvelope,
        ),
        Some(bijux_dna_domain_fastq::BenchmarkReadinessLevel::GovernedExecution)
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
    assert_eq!(report_qc.scenarios[0].scenario_id, "qc_aggregation_fairness");
    assert_eq!(
        report_qc.comparison_input_artifact_ids,
        vec!["report_json", "governed_qc_inputs_manifest", "multiqc_report", "multiqc_data"]
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
    assert!(bijux_dna_domain_fastq::is_reference_index_backend_compatible(
        &bowtie2,
        &ToolId::from_static("bowtie2_build"),
    ));
    assert!(!bijux_dna_domain_fastq::is_reference_index_backend_compatible(
        &bowtie2,
        &ToolId::from_static("star"),
    ));
}
