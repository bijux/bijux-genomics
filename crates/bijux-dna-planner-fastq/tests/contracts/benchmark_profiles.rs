use bijux_dna_core::ids::{StageId, ToolId};

#[test]
fn benchmark_profiles_distinguish_planned_governed_and_benchmarkable_bindings() {
    let trim_stage = StageId::from_static("fastq.trim_reads");
    let trim_profile = bijux_dna_planner_fastq::stage_api::benchmark_profile_for_stage_tool(
        &trim_stage,
        &ToolId::from_static("fastp"),
    )
    .expect("trim benchmark profile");
    assert_eq!(
        trim_profile.integration_level,
        bijux_dna_planner_fastq::stage_api::ToolIntegrationLevel::GovernedContract
    );
    assert_eq!(
        trim_profile.runtime_interpretation,
        bijux_dna_planner_fastq::stage_api::RuntimeInterpretationLevel::GenericEnvelope
    );
    assert_eq!(
        trim_profile.readiness,
        bijux_dna_planner_fastq::stage_api::BenchmarkReadinessLevel::GovernedBenchmarkCohort
    );
    assert_eq!(trim_profile.benchmark_scenarios, vec!["trim_fairness"]);

    let infer_stage = StageId::from_static("fastq.infer_asvs");
    let infer_profile = bijux_dna_planner_fastq::stage_api::benchmark_profile_for_stage_tool(
        &infer_stage,
        &ToolId::from_static("dada2"),
    )
    .expect("planned profile");
    assert_eq!(
        infer_profile.readiness,
        bijux_dna_planner_fastq::stage_api::BenchmarkReadinessLevel::PlannedContract
    );
    assert!(infer_profile.benchmark_scenarios.is_empty());
}

#[test]
fn benchmark_profiles_keep_observer_coverage_visible() {
    let detect_stage = StageId::from_static("fastq.detect_adapters");
    let detect_profile = bijux_dna_planner_fastq::stage_api::benchmark_profile_for_stage_tool(
        &detect_stage,
        &ToolId::from_static("fastqc"),
    )
    .expect("detect profile");
    assert_eq!(
        detect_profile.runtime_interpretation,
        bijux_dna_planner_fastq::stage_api::RuntimeInterpretationLevel::ObserverSpecialized
    );
    assert_eq!(
        detect_profile.readiness,
        bijux_dna_planner_fastq::stage_api::BenchmarkReadinessLevel::GovernedExecution
    );

    let overrepresented_stage = StageId::from_static("fastq.profile_overrepresented_sequences");
    let seqkit_profile = bijux_dna_planner_fastq::stage_api::benchmark_profile_for_stage_tool(
        &overrepresented_stage,
        &ToolId::from_static("seqkit"),
    )
    .expect("seqkit profile");
    assert_eq!(
        seqkit_profile.runtime_interpretation,
        bijux_dna_planner_fastq::stage_api::RuntimeInterpretationLevel::GenericEnvelope
    );
    assert_eq!(
        seqkit_profile.readiness,
        bijux_dna_planner_fastq::stage_api::BenchmarkReadinessLevel::GovernedExecution
    );

    let screen_stage = StageId::from_static("fastq.screen_taxonomy");
    let profiles = bijux_dna_planner_fastq::stage_api::benchmark_profiles_for_stage(&screen_stage);
    assert!(
        profiles.iter().any(|profile| {
            profile.tool_id.as_str() == "diamond"
                && profile.readiness
                    == bijux_dna_planner_fastq::stage_api::BenchmarkReadinessLevel::PlannedContract
        }),
        "planned taxonomy bindings must remain visible as planned-only profiles",
    );
    assert!(
        profiles.iter().filter(|profile| profile.integration_level
            == bijux_dna_planner_fastq::stage_api::ToolIntegrationLevel::GovernedContract)
            .all(|profile| {
            profile.readiness
                == bijux_dna_planner_fastq::stage_api::BenchmarkReadinessLevel::GovernedBenchmarkCohort
        }),
        "closed taxonomy screening backends should surface the shared benchmark cohort",
    );
}

#[test]
fn stage_tool_capabilities_distinguish_declared_runnable_and_comparable_bindings() {
    let infer_capability = bijux_dna_planner_fastq::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.infer_asvs"),
        &ToolId::from_static("dada2"),
    )
    .expect("infer_asvs capability");
    assert!(infer_capability.declared);
    assert!(infer_capability.plannable);
    assert!(!infer_capability.runnable);
    assert!(!infer_capability.benchmark_normalized);
    assert!(!infer_capability.comparable);

    let trim_capability = bijux_dna_planner_fastq::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.trim_reads"),
        &ToolId::from_static("fastp"),
    )
    .expect("trim capability");
    assert!(trim_capability.runnable);
    assert!(trim_capability.parse_normalized);
    assert!(trim_capability.benchmark_normalized);
    assert!(!trim_capability.comparable);

    let detect_capability = bijux_dna_planner_fastq::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.detect_adapters"),
        &ToolId::from_static("fastqc"),
    )
    .expect("detect capability");
    assert!(detect_capability.runnable);
    assert!(detect_capability.parse_normalized);
    assert!(!detect_capability.benchmark_normalized);
    assert!(!detect_capability.comparable);
}

#[test]
fn benchmark_cohorts_surface_governed_toolsets_per_fairness_scenario() {
    let trim_stage = StageId::from_static("fastq.trim_reads");
    let trim_cohorts = bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&trim_stage);
    assert_eq!(trim_cohorts.len(), 1);
    assert_eq!(trim_cohorts[0].scenario_id, "trim_fairness");
    assert!(
        trim_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "fastp")
    );
    assert!(
        trim_cohorts[0]
            .tool_ids
            .iter()
            .all(|tool_id| tool_id.as_str() != "seqpurge")
    );
    assert!(trim_cohorts[0].observer_specialized_tools.is_empty());
    assert!(!trim_cohorts[0].generic_envelope_tools.is_empty());

    let screen_stage = StageId::from_static("fastq.screen_taxonomy");
    let screen_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&screen_stage);
    assert_eq!(screen_cohorts.len(), 1);
    assert_eq!(screen_cohorts[0].scenario_id, "screen_fairness");
    assert!(
        screen_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "kraken2")
    );
    assert!(
        screen_cohorts[0]
            .tool_ids
            .iter()
            .all(|tool_id| tool_id.as_str() != "diamond")
    );

    let filter_stage = StageId::from_static("fastq.filter_reads");
    let filter_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&filter_stage);
    assert_eq!(filter_cohorts.len(), 1);
    assert_eq!(filter_cohorts[0].scenario_id, "filter_fairness");
    assert!(
        filter_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "fastp")
    );
    assert!(
        filter_cohorts[0]
            .tool_ids
            .iter()
            .all(|tool_id| tool_id.as_str() != "cutadapt")
    );

    let merge_stage = StageId::from_static("fastq.merge_pairs");
    let merge_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&merge_stage);
    assert_eq!(merge_cohorts.len(), 1);
    assert_eq!(merge_cohorts[0].scenario_id, "merge_fairness");
    assert!(
        merge_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "pear")
    );
    assert!(
        merge_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "vsearch")
    );

    let low_complexity_stage = StageId::from_static("fastq.filter_low_complexity");
    let low_complexity_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&low_complexity_stage);
    assert_eq!(low_complexity_cohorts.len(), 1);
    assert_eq!(low_complexity_cohorts[0].scenario_id, "low_complexity_fairness");
    assert!(
        low_complexity_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "bbduk")
    );
    assert!(
        low_complexity_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "prinseq")
    );
    assert!(
        low_complexity_cohorts[0]
            .tool_ids
            .iter()
            .all(|tool_id| tool_id.as_str() != "dustmasker")
    );

    let dedup_stage = StageId::from_static("fastq.remove_duplicates");
    let dedup_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&dedup_stage);
    assert_eq!(dedup_cohorts.len(), 1);
    assert_eq!(dedup_cohorts[0].scenario_id, "dedup_fairness");
    assert!(
        dedup_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "fastuniq")
    );
    assert!(
        dedup_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "clumpify")
    );

    let read_length_stage = StageId::from_static("fastq.profile_read_lengths");
    let read_length_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&read_length_stage);
    assert_eq!(read_length_cohorts.len(), 1);
    assert_eq!(read_length_cohorts[0].scenario_id, "read_length_fairness");
    assert!(
        read_length_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "seqkit_stats")
    );
    assert!(
        read_length_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "fastp")
    );
    assert!(
        read_length_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "prinseq")
    );
    assert!(
        read_length_cohorts[0]
            .tool_ids
            .iter()
            .all(|tool_id| tool_id.as_str() != "seqfu")
    );

    let correction_stage = StageId::from_static("fastq.correct_errors");
    let correction_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&correction_stage);
    assert_eq!(correction_cohorts.len(), 1);
    assert_eq!(correction_cohorts[0].scenario_id, "correction_fairness");
    assert!(
        correction_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "rcorrector")
    );
    assert!(
        correction_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "musket")
    );
    assert!(
        correction_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "lighter")
    );
    assert!(
        correction_cohorts[0]
            .tool_ids
            .iter()
            .any(|tool_id| tool_id.as_str() == "bayeshammer")
    );
}
