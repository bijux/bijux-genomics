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
}
