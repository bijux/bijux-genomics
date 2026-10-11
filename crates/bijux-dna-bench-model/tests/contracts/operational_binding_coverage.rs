use super::*;

#[test]
fn binding_blocker_keeps_exact_blocker_tuple_fields() {
    let binding = BindingKey {
        domain: "vcf".to_string(),
        stage_id: "vcf.stats".to_string(),
        tool_id: "bcftools".to_string(),
        corpus_id: "vcf_production_regression".to_string(),
        asset_profile_id: "vcf_cohort".to_string(),
    };

    let blocker = binding_blocker(
        &binding,
        "resource_hint_missing",
        "configs/bench/local/stage-tool-resources.toml",
        "canonical benchmark result is missing governed resource hints",
    );

    assert_eq!(blocker.domain, "vcf");
    assert_eq!(blocker.stage_id, "vcf.stats");
    assert_eq!(blocker.tool_id, "bcftools");
    assert_eq!(blocker.corpus_id, "vcf_production_regression");
    assert_eq!(blocker.asset_profile_id, "vcf_cohort");
    assert_eq!(blocker.blocker_type, "resource_hint_missing");
    assert_eq!(blocker.blocker_path, "configs/bench/local/stage-tool-resources.toml");
}

#[test]
fn operational_binding_coverage_requires_matching_corpus_assets_and_prerequisites() {
    let expected_row = crate::AllDomainExpectedBenchmarkResultRow {
        result_id: "result".to_string(),
        domain: "bam".to_string(),
        stage_id: "bam.validate".to_string(),
        tool_id: "declared-tool".to_string(),
        corpus_id: "truth".to_string(),
        asset_profile_id: "diploid".to_string(),
        expected_outputs: vec!["report".to_string()],
        expected_metrics: vec![],
        report_section: "validation".to_string(),
    };
    let expected = AllDomainExpectedBenchmarkResultsReport {
        schema_version: crate::ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_SCHEMA_V1.to_string(),
        output_path: "expected.tsv".to_string(),
        row_count: 1,
        result_id_count: 1,
        stage_count: 1,
        tool_count: 1,
        corpus_count: 1,
        asset_profile_count: 1,
        domain_counts: BTreeMap::new(),
        report_section_counts: BTreeMap::new(),
        rows: vec![expected_row],
    };
    let row = crate::AllDomainStageToolTableRow {
        domain: "bam".to_string(),
        stage_id: "bam.validate".to_string(),
        tool_id: "declared-tool".to_string(),
        corpus_id: "truth".to_string(),
        asset_profile_id: "diploid".to_string(),
        adapter_id: "adapter".to_string(),
        parser_id: "parser".to_string(),
        benchmark_status: "benchmark_ready".to_string(),
    };
    let table = AllDomainStageToolTableReport {
        schema_version: crate::ALL_DOMAIN_STAGE_TOOL_TABLE_SCHEMA_VERSION.to_string(),
        output_path: "table.tsv".to_string(),
        row_count: 1,
        benchmark_ready_row_count: 1,
        benchmark_ready_unique_binding_count: 1,
        domain_counts: BTreeMap::new(),
        benchmark_ready_domain_counts: BTreeMap::new(),
        rows: vec![row],
    };
    let mut blockers = BTreeSet::new();
    check_expected_bindings(Some(&expected), Some(&table), &mut blockers);
    check_binding_metadata(Some(&expected), Some(&table), &mut blockers);
    assert!(blockers.is_empty());
    for field in ["corpus", "asset"] {
        let mut changed = table.clone();
        if field == "corpus" {
            changed.rows[0].corpus_id = "other".to_string();
        } else {
            changed.rows[0].asset_profile_id = "haploid".to_string();
        }
        let mut blockers = BTreeSet::new();
        check_expected_bindings(Some(&expected), Some(&changed), &mut blockers);
        check_binding_metadata(Some(&expected), Some(&changed), &mut blockers);
        assert_eq!(blockers.len(), 2);
        assert!(blockers.iter().any(|b| b.blocker_type == "stage_tool_binding_missing"
            && b.corpus_id == "truth"
            && b.asset_profile_id == "diploid"));
    }
    let check = evaluate_operational_binding_coverage(
        OperationalBindingEvidence {
            expected_results: Some(&expected),
            stage_tool_table: Some(&table),
            ..Default::default()
        },
        &mut blockers,
        "missing.tsv",
    );
    assert!(!check.ok);
    assert!(check.detail.contains("all_domain_rendered_commands"));
    let check = evaluate_operational_binding_coverage(
        OperationalBindingEvidence::default(),
        &mut blockers,
        "missing.tsv",
    );
    assert!(!check.ok);
    assert_eq!(check.output_path, "missing.tsv");
}
