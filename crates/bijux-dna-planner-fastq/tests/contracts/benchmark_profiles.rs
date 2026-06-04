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
        bijux_dna_planner_fastq::stage_api::RuntimeInterpretationLevel::ObserverSpecialized
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
    .expect("governed profile");
    assert_eq!(
        infer_profile.readiness,
        bijux_dna_planner_fastq::stage_api::BenchmarkReadinessLevel::GovernedExecution
    );
    assert_eq!(
        infer_profile.runtime_interpretation,
        bijux_dna_planner_fastq::stage_api::RuntimeInterpretationLevel::ObserverSpecialized
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
        bijux_dna_planner_fastq::stage_api::BenchmarkReadinessLevel::ObserverSpecializedBenchmark
    );

    let overrepresented_stage = StageId::from_static("fastq.profile_overrepresented_sequences");
    let seqkit_profile = bijux_dna_planner_fastq::stage_api::benchmark_profile_for_stage_tool(
        &overrepresented_stage,
        &ToolId::from_static("seqkit"),
    )
    .expect("seqkit profile");
    assert_eq!(
        seqkit_profile.runtime_interpretation,
        bijux_dna_planner_fastq::stage_api::RuntimeInterpretationLevel::ObserverSpecialized
    );
    assert_eq!(
        seqkit_profile.readiness,
        bijux_dna_planner_fastq::stage_api::BenchmarkReadinessLevel::ObserverSpecializedBenchmark
    );

    let screen_stage = StageId::from_static("fastq.screen_taxonomy");
    let profiles = bijux_dna_planner_fastq::stage_api::benchmark_profiles_for_stage(&screen_stage);
    let mut screen_tool_ids =
        profiles.iter().map(|profile| profile.tool_id.as_str().to_string()).collect::<Vec<_>>();
    screen_tool_ids.sort();
    assert_eq!(screen_tool_ids, vec!["centrifuge", "kaiju", "kraken2", "krakenuniq"]);
    assert!(
        profiles.iter().all(|profile| {
            profile.integration_level
                == bijux_dna_planner_fastq::stage_api::ToolIntegrationLevel::GovernedContract
                && profile.readiness
                    == bijux_dna_planner_fastq::stage_api::BenchmarkReadinessLevel::GovernedBenchmarkCohort
                && profile.runtime_interpretation
                    == bijux_dna_planner_fastq::stage_api::RuntimeInterpretationLevel::ObserverSpecialized
        }),
        "governed taxonomy screening backends must stay observer-specialized and cohort-benchmarkable",
    );
    assert!(
        profiles.iter().all(|profile| {
            profile.readiness
                == bijux_dna_planner_fastq::stage_api::BenchmarkReadinessLevel::GovernedBenchmarkCohort
                && profile.benchmark_scenarios == vec!["screen_fairness"]
        }),
        "taxonomy screening benchmark profiles must stay attached to the governed fairness cohort",
    );
}

#[test]
fn stage_tool_capabilities_distinguish_declared_runnable_and_comparable_bindings() {
    let infer_capability = bijux_dna_planner_fastq::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.infer_asvs"),
        &ToolId::from_static("dada2"),
    )
    .expect("infer_asvs capability");
    assert!(infer_capability.execution.declared);
    assert!(infer_capability.execution.plannable);
    assert!(infer_capability.execution.runnable);
    assert!(infer_capability.normalization.parse_normalized);
    assert!(!infer_capability.normalization.benchmark_normalized);
    assert!(!infer_capability.normalization.comparable);

    let trim_capability = bijux_dna_planner_fastq::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.trim_reads"),
        &ToolId::from_static("fastp"),
    )
    .expect("trim capability");
    assert!(trim_capability.execution.runnable);
    assert!(trim_capability.normalization.parse_normalized);
    assert!(trim_capability.normalization.benchmark_normalized);
    assert!(!trim_capability.normalization.comparable);

    let detect_capability = bijux_dna_planner_fastq::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.detect_adapters"),
        &ToolId::from_static("fastqc"),
    )
    .expect("detect capability");
    assert!(detect_capability.execution.runnable);
    assert!(detect_capability.normalization.parse_normalized);
    assert!(detect_capability.normalization.benchmark_normalized);
    assert!(detect_capability.normalization.comparable);
}

#[test]
fn benchmark_cohorts_surface_governed_toolsets_per_fairness_scenario() {
    let trim_stage = StageId::from_static("fastq.trim_reads");
    let trim_cohorts = bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&trim_stage);
    assert_eq!(trim_cohorts.len(), 1);
    assert_eq!(trim_cohorts[0].scenario_id, "trim_fairness");
    assert_eq!(
        trim_cohorts[0].tool_ids.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        vec![
            "adapterremoval",
            "alientrimmer",
            "atropos",
            "bbduk",
            "cutadapt",
            "fastp",
            "fastx_clipper",
            "leehom",
            "prinseq",
            "seqkit",
            "skewer",
            "trim_galore",
            "trimmomatic",
        ]
    );
    assert_eq!(
        trim_cohorts[0].observer_specialized_tools.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        vec![
            "adapterremoval",
            "alientrimmer",
            "atropos",
            "bbduk",
            "cutadapt",
            "fastp",
            "fastx_clipper",
            "leehom",
            "prinseq",
            "seqkit",
            "skewer",
            "trim_galore",
            "trimmomatic",
        ]
    );

    let polyg_stage = StageId::from_static("fastq.trim_polyg_tails");
    let polyg_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&polyg_stage);
    assert_eq!(polyg_cohorts.len(), 1);
    assert_eq!(polyg_cohorts[0].scenario_id, "polyg_trim_fairness");
    assert_eq!(
        polyg_cohorts[0].tool_ids.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        vec!["bbduk", "fastp"]
    );
    assert_eq!(
        polyg_cohorts[0].observer_specialized_tools.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        vec!["bbduk", "fastp"]
    );

    let screen_stage = StageId::from_static("fastq.screen_taxonomy");
    let screen_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&screen_stage);
    assert_eq!(screen_cohorts.len(), 1);
    assert_eq!(screen_cohorts[0].scenario_id, "screen_fairness");
    assert!(!screen_cohorts[0].tool_ids.is_empty());
    assert_eq!(
        screen_cohorts[0].tool_ids.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        screen_cohorts[0].observer_specialized_tools.iter().map(ToolId::as_str).collect::<Vec<_>>()
    );

    let filter_stage = StageId::from_static("fastq.filter_reads");
    let filter_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&filter_stage);
    assert_eq!(filter_cohorts.len(), 1);
    assert_eq!(filter_cohorts[0].scenario_id, "filter_fairness");
    assert!(!filter_cohorts[0].tool_ids.is_empty());
    assert_eq!(
        filter_cohorts[0].tool_ids.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        filter_cohorts[0].observer_specialized_tools.iter().map(ToolId::as_str).collect::<Vec<_>>()
    );

    let merge_stage = StageId::from_static("fastq.merge_pairs");
    let merge_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&merge_stage);
    assert_eq!(merge_cohorts.len(), 1);
    assert_eq!(merge_cohorts[0].scenario_id, "merge_fairness");
    assert_eq!(
        merge_cohorts[0].tool_ids.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        vec!["adapterremoval", "bbmerge", "flash2", "leehom", "pear", "vsearch",]
    );
    assert_eq!(
        merge_cohorts[0].observer_specialized_tools.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        vec!["adapterremoval", "bbmerge", "flash2", "leehom", "pear", "vsearch",]
    );

    let low_complexity_stage = StageId::from_static("fastq.filter_low_complexity");
    let low_complexity_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&low_complexity_stage);
    assert_eq!(low_complexity_cohorts.len(), 1);
    assert_eq!(low_complexity_cohorts[0].scenario_id, "low_complexity_fairness");
    assert!(!low_complexity_cohorts[0].tool_ids.is_empty());
    assert_eq!(
        low_complexity_cohorts[0].tool_ids.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        low_complexity_cohorts[0]
            .observer_specialized_tools
            .iter()
            .map(ToolId::as_str)
            .collect::<Vec<_>>()
    );

    let dedup_stage = StageId::from_static("fastq.remove_duplicates");
    let dedup_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&dedup_stage);
    assert_eq!(dedup_cohorts.len(), 1);
    assert_eq!(dedup_cohorts[0].scenario_id, "dedup_fairness");
    assert_eq!(
        dedup_cohorts[0].tool_ids.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        vec!["clumpify", "fastuniq"]
    );
    assert_eq!(
        dedup_cohorts[0].observer_specialized_tools.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        vec!["clumpify", "fastuniq"]
    );

    let read_length_stage = StageId::from_static("fastq.profile_read_lengths");
    let read_length_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&read_length_stage);
    assert_eq!(read_length_cohorts.len(), 1);
    assert_eq!(read_length_cohorts[0].scenario_id, "read_length_fairness");
    assert_eq!(
        read_length_cohorts[0].tool_ids.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        vec!["fastp", "prinseq", "seqfu", "seqkit_stats"]
    );
    assert_eq!(
        read_length_cohorts[0]
            .observer_specialized_tools
            .iter()
            .map(ToolId::as_str)
            .collect::<Vec<_>>(),
        vec!["fastp", "prinseq", "seqfu", "seqkit_stats"]
    );

    let correction_stage = StageId::from_static("fastq.correct_errors");
    let correction_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&correction_stage);
    assert_eq!(correction_cohorts.len(), 1);
    assert_eq!(correction_cohorts[0].scenario_id, "correction_fairness");
    assert_eq!(
        correction_cohorts[0].tool_ids.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        vec!["bayeshammer", "lighter", "musket", "rcorrector"]
    );
    assert_eq!(
        correction_cohorts[0]
            .observer_specialized_tools
            .iter()
            .map(ToolId::as_str)
            .collect::<Vec<_>>(),
        vec!["bayeshammer", "lighter", "musket", "rcorrector"]
    );

    let otu_stage = StageId::from_static("fastq.cluster_otus");
    let otu_cohorts = bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&otu_stage);
    assert_eq!(otu_cohorts.len(), 1);
    assert_eq!(otu_cohorts[0].scenario_id, "otu_clustering_fairness");
    assert_eq!(
        otu_cohorts[0].tool_ids.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        vec!["vsearch"]
    );
    assert_eq!(
        otu_cohorts[0].observer_specialized_tools.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        vec!["vsearch"]
    );

    let normalize_primers_stage = StageId::from_static("fastq.normalize_primers");
    let primer_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&normalize_primers_stage);
    assert_eq!(primer_cohorts.len(), 1);
    assert_eq!(primer_cohorts[0].scenario_id, "primer_normalization_fairness");
    assert!(!primer_cohorts[0].tool_ids.is_empty());
    assert_eq!(
        primer_cohorts[0].tool_ids.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        primer_cohorts[0].observer_specialized_tools.iter().map(ToolId::as_str).collect::<Vec<_>>()
    );

    let terminal_damage_stage = StageId::from_static("fastq.trim_terminal_damage");
    let terminal_damage_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&terminal_damage_stage);
    assert_eq!(terminal_damage_cohorts.len(), 1);
    assert_eq!(terminal_damage_cohorts[0].scenario_id, "terminal_damage_fairness");
    assert_eq!(
        terminal_damage_cohorts[0].tool_ids.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        vec!["adapterremoval", "cutadapt", "seqkit"]
    );
    assert_eq!(
        terminal_damage_cohorts[0]
            .observer_specialized_tools
            .iter()
            .map(ToolId::as_str)
            .collect::<Vec<_>>(),
        vec!["adapterremoval", "cutadapt", "seqkit"]
    );

    let overrepresented_stage = StageId::from_static("fastq.profile_overrepresented_sequences");
    let overrepresented_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&overrepresented_stage);
    assert_eq!(overrepresented_cohorts.len(), 1);
    assert_eq!(overrepresented_cohorts[0].scenario_id, "overrepresented_sequence_fairness");
    assert!(!overrepresented_cohorts[0].tool_ids.is_empty());
    assert_eq!(
        overrepresented_cohorts[0].tool_ids.iter().map(ToolId::as_str).collect::<Vec<_>>(),
        overrepresented_cohorts[0]
            .observer_specialized_tools
            .iter()
            .map(ToolId::as_str)
            .collect::<Vec<_>>()
    );

    let validation_stage = StageId::from_static("fastq.validate_reads");
    let validation_cohorts =
        bijux_dna_planner_fastq::stage_api::benchmark_cohorts_for_stage(&validation_stage);
    assert_eq!(validation_cohorts.len(), 1);
    assert_eq!(validation_cohorts[0].scenario_id, "validation_fairness");
    assert!(validation_cohorts[0]
        .tool_ids
        .iter()
        .any(|tool_id| tool_id.as_str() == "fastqvalidator"));
    for tool_id in ["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"] {
        assert!(
            validation_cohorts[0].tool_ids.iter().any(|candidate| candidate.as_str() == tool_id),
            "validation cohort must include {tool_id}"
        );
        assert!(
            validation_cohorts[0]
                .observer_specialized_tools
                .iter()
                .any(|candidate| candidate.as_str() == tool_id),
            "validation observer-specialized cohort must include {tool_id}"
        );
    }
}
