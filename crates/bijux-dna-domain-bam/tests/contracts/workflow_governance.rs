use bijux_dna_domain_bam::{
    bam_adna_workflow_contract, bam_alignment_strategies, bam_alignment_strategy_for_tool,
    bam_bench_corpus_manifest, bam_contamination_workflow_contract,
    bam_scientific_report_contract_for_stage, bam_scientific_report_contracts,
    classify_bam_coverage_regime, compare_bam_duplicate_methods, estimate_bam_stage_resources,
    evaluate_bam_merge_compatibility, required_bam_bench_corpus_scenarios,
    BamBenchDatasetScenarioV1, BamDuplicateMethodMetricsV1,
    BamScientificReportIdV1,
};

#[test]
fn alignment_strategies_are_explicit_and_post_aligned() {
    let strategies = bam_alignment_strategies();
    assert!(strategies.len() >= 3, "expected multiple governed alignment strategies");
    for strategy in &strategies {
        assert!(
            !strategy.hidden_default_allowed,
            "strategy {} must not rely on hidden defaults",
            strategy.strategy_id
        );
        assert_eq!(strategy.post_alignment_chain.chain_id, "samtools_coordinate_validate");
        assert!(strategy.post_alignment_chain.validate_before_downstream);
    }
    let adna = bam_alignment_strategy_for_tool("bwa", Some("adna_short"))
        .expect("ancient bwa strategy");
    assert_eq!(adna.strategy_id, "bwa_aln_adna_short");
    let bowtie2 = bam_alignment_strategy_for_tool("bowtie2", Some("default"))
        .expect("bowtie2 strategy");
    assert_eq!(bowtie2.mode, "local");
}

#[test]
fn merge_compatibility_requires_sample_reference_library_and_read_groups() {
    let compatible = evaluate_bam_merge_compatibility(&[
        bijux_dna_domain_bam::BamMergeInputIdentityV1 {
            sample_id: "sample-1".to_string(),
            library_id: Some("lib-a".to_string()),
            lane_id: Some("L001".to_string()),
            read_group_ids: vec!["rg1".to_string()],
            reference_digest: Some("sha256:ref".to_string()),
            sequencing_platform: Some("ILLUMINA".to_string()),
            platform_unit: Some("sample-1.pu1".to_string()),
        },
        bijux_dna_domain_bam::BamMergeInputIdentityV1 {
            sample_id: "sample-1".to_string(),
            library_id: Some("lib-a".to_string()),
            lane_id: Some("L002".to_string()),
            read_group_ids: vec!["rg2".to_string()],
            reference_digest: Some("sha256:ref".to_string()),
            sequencing_platform: Some("ILLUMINA".to_string()),
            platform_unit: Some("sample-1.pu2".to_string()),
        },
    ]);
    assert!(compatible.compatible);

    let incompatible = evaluate_bam_merge_compatibility(&[
        bijux_dna_domain_bam::BamMergeInputIdentityV1 {
            sample_id: "sample-1".to_string(),
            library_id: Some("lib-a".to_string()),
            lane_id: Some("L001".to_string()),
            read_group_ids: vec!["rg1".to_string()],
            reference_digest: Some("sha256:ref-a".to_string()),
            sequencing_platform: Some("ILLUMINA".to_string()),
            platform_unit: Some("sample-1.pu1".to_string()),
        },
        bijux_dna_domain_bam::BamMergeInputIdentityV1 {
            sample_id: "sample-2".to_string(),
            library_id: Some("lib-b".to_string()),
            lane_id: Some("L002".to_string()),
            read_group_ids: Vec::new(),
            reference_digest: Some("sha256:ref-b".to_string()),
            sequencing_platform: Some("ONT".to_string()),
            platform_unit: Some("sample-2.pu2".to_string()),
        },
    ]);
    assert!(!incompatible.compatible);
    assert!(incompatible.refusal_codes.contains(&"merge_sample_id_conflict".to_string()));
    assert!(incompatible.refusal_codes.contains(&"merge_reference_digest_conflict".to_string()));
    assert!(incompatible.refusal_codes.contains(&"merge_platform_conflict".to_string()));
    assert!(incompatible.refusal_codes.contains(&"merge_library_conflict".to_string()));
    assert!(incompatible.refusal_codes.contains(&"merge_read_group_missing".to_string()));
}

#[test]
fn duplicate_and_coverage_contracts_classify_behavior_explicitly() {
    let duplicate = compare_bam_duplicate_methods(
        "bam.markdup",
        BamDuplicateMethodMetricsV1 {
            method: "samtools".to_string(),
            duplicate_reads: Some(120),
            duplicate_fraction: Some(0.12),
            optical_duplicates: Some("mark_only".to_string()),
            duplicate_action: Some("mark".to_string()),
        },
        BamDuplicateMethodMetricsV1 {
            method: "picard".to_string(),
            duplicate_reads: Some(150),
            duplicate_fraction: Some(0.15),
            optical_duplicates: Some("mark_only".to_string()),
            duplicate_action: Some("remove".to_string()),
        },
    );
    assert!(duplicate.comparable);
    assert!(duplicate.policy_explicit);
    assert_eq!(duplicate.duplicate_reads_delta, Some(-30));

    let sparse = classify_bam_coverage_regime(0.5, 0.2);
    assert_eq!(sparse.regime_id, "sparse");
    let low_pass = classify_bam_coverage_regime(3.0, 0.5);
    assert_eq!(low_pass.regime_id, "low_pass");
    let whole_genome_like = classify_bam_coverage_regime(20.0, 0.9);
    assert_eq!(whole_genome_like.regime_id, "whole_genome_like");
}

#[test]
fn adna_contamination_and_scientific_reports_stay_context_bound() {
    let adna = bam_adna_workflow_contract();
    assert!(adna.evidence_only);
    assert!(adna
        .authenticity_caveats
        .iter()
        .any(|note| note.contains("must not be reported as authenticity certification")));

    let contamination = bam_contamination_workflow_contract();
    let verifybamid2 = contamination
        .tools
        .iter()
        .find(|tool| tool.tool_id == "verifybamid2")
        .expect("verifybamid2 contract");
    assert!(verifybamid2
        .required_inputs
        .contains(&"sex_or_chromosome_context".to_string()));
    assert!(verifybamid2
        .required_inputs
        .contains(&"minimum_coverage_context".to_string()));

    let reports = bam_scientific_report_contracts();
    assert_eq!(reports.len(), 4);
    let kinship = bam_scientific_report_contract_for_stage("bam.kinship")
        .expect("kinship report contract");
    assert_eq!(kinship.report_id, BamScientificReportIdV1::Kinship);
    assert!(kinship.optional);
    assert!(kinship
        .required_population_or_reference_context
        .contains(&"cohort_context".to_string()));
}

#[test]
fn bam_resource_plans_and_corpus_manifest_cover_large_and_edge_cases() {
    let markdup = estimate_bam_stage_resources("bam.markdup", 50_u64 * 1024 * 1024 * 1024);
    assert!(markdup.memory_gb >= 50);
    assert!(markdup.disk_gb >= 100 / 2); // durability check that large BAMs scale above floor
    assert!(markdup.requires_index);

    let endogenous = estimate_bam_stage_resources("bam.endogenous_content", 4_u64 * 1024 * 1024);
    assert!(endogenous.requires_index);
    assert!(endogenous.notes.iter().any(|note| note.contains("indexed depth calculations")));

    let required = required_bam_bench_corpus_scenarios();
    let manifest = bam_bench_corpus_manifest();
    for scenario in required {
        assert!(manifest.scenarios_covered.contains(&scenario));
    }
    assert!(manifest
        .datasets
        .iter()
        .any(|dataset| dataset.scenarios.contains(&BamBenchDatasetScenarioV1::MissingIndex)));
}
