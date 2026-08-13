use std::collections::BTreeMap;

use bijux_dna_bench::{
    build_full_benchmark_comparable_metric_rows, build_full_benchmark_failures_section,
    build_full_benchmark_missing_result_rows, AllDomainFailureClassificationReport,
    BamComparableMetricsReport, ComparableMetricContractStatus, FastqComparableMetricsReport,
    FastqComparableMetricsRow, FullBenchmarkReportRow, FullBenchmarkReportRowStatus,
    FullBenchmarkResultCollectorReport, ScientificAcceptanceDirection,
    ScientificAcceptanceInsufficiencyBehavior, ScientificAcceptancePassRule,
    ScientificAcceptanceThresholdRow, ScientificAcceptanceToleranceKind,
    VcfComparableMetricsReport, ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_V1,
    BAM_COMPARABLE_METRICS_SCHEMA_V1, FASTQ_COMPARABLE_METRICS_SCHEMA_V1,
    FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_V1, VCF_COMPARABLE_METRICS_SCHEMA_V1,
};

#[test]
fn aggregate_failure_section_refuses_collector_count_drift() {
    let collector = FullBenchmarkResultCollectorReport {
        schema_version: FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_V1.to_string(),
        output_path: "artifacts/collector.json".to_string(),
        row_count: 0,
        benchmark_expected_row_count: 0,
        pipeline_fake_run_row_count: 0,
        fake_run_row_count: 0,
        fake_failure_row_count: 1,
        missing_result_audit_row_count: 0,
        real_smoke_row_count: 0,
        insufficient_data_row_count: 0,
        unsupported_pair_row_count: 0,
        missing_result_status_count: 0,
        insufficient_data_status_count: 0,
        unsupported_pair_status_count: 0,
        passes_behavior_test: false,
        surface_kind_counts: BTreeMap::new(),
        result_status_counts: BTreeMap::new(),
        domain_counts: BTreeMap::new(),
        rows: Vec::new(),
    };
    let classifications = AllDomainFailureClassificationReport {
        schema_version: ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_V1.to_string(),
        output_path: "artifacts/classifications.json".to_string(),
        fixture_root: "benchmarks/tests/fixtures".to_string(),
        row_count: 0,
        triggered_row_count: 0,
        required_class_count: 0,
        triggered_class_count: 0,
        missing_class_count: 0,
        class_counts: BTreeMap::new(),
        missing_class_ids: Vec::new(),
        passes_behavior_test: false,
        rows: Vec::new(),
    };

    let error = build_full_benchmark_failures_section(&collector, &classifications)
        .expect_err("collector count drift must be rejected");
    assert!(error.to_string().contains("drifted from governed fake-failure coverage"));
}

#[test]
fn aggregate_missing_result_section_refuses_absent_result_identity() {
    let row = FullBenchmarkReportRow {
        report_row_id: "missing-row".to_string(),
        row_status: FullBenchmarkReportRowStatus::MissingResult,
        result_id: None,
        domain: "fastq".to_string(),
        stage_id: "validate_reads".to_string(),
        tool_id: "seqkit".to_string(),
        corpus_id: "corpus".to_string(),
        asset_profile_id: "sample-set".to_string(),
        report_section: "validation".to_string(),
        pipeline_ids: Vec::new(),
        expected_output_count: 1,
        expected_metric_count: 1,
        comparable_metric_count: 0,
        simulated_elapsed_seconds: None,
        real_smoke_execution_id: None,
        real_smoke_elapsed_seconds: None,
        declared_memory_mb: None,
        declared_cpu_threads: None,
        real_smoke_memory_mb: None,
        real_smoke_cpu_threads: None,
        evidence_path: "artifacts/missing.json".to_string(),
        manifest_path: None,
        detail: "missing".to_string(),
    };

    let error = build_full_benchmark_missing_result_rows(&[row])
        .expect_err("missing result identity must be rejected");
    assert!(error.to_string().contains("has no result identity"));
}

#[test]
fn comparable_metric_aggregation_requires_exact_acceptance_closure() {
    let fastq = FastqComparableMetricsReport {
        schema_version: FASTQ_COMPARABLE_METRICS_SCHEMA_V1.to_string(),
        output_path: "artifacts/fastq.tsv".to_string(),
        comparable_stage_count: 1,
        multi_tool_stage_count: 1,
        comparable_tool_row_count: 2,
        row_count: 1,
        declared_stage_count: 1,
        missing_shared_metric_stage_count: 0,
        shared_metric_field_count: 1,
        rows: vec![FastqComparableMetricsRow {
            stage_id: "trim".to_string(),
            comparison_contract_status: ComparableMetricContractStatus::Declared,
            tool_count: 2,
            tool_ids: vec!["cutadapt".to_string(), "fastp".to_string()],
            default_tool_id: "fastp".to_string(),
            corpus_status: "admitted".to_string(),
            shared_metric_field_count: 1,
            shared_metric_fields: vec!["reads_retained".to_string()],
            reason: "shared normalized output".to_string(),
        }],
    };
    let bam = BamComparableMetricsReport {
        schema_version: BAM_COMPARABLE_METRICS_SCHEMA_V1.to_string(),
        output_path: "artifacts/bam.tsv".to_string(),
        comparable_stage_count: 0,
        multi_tool_stage_count: 0,
        comparable_tool_row_count: 0,
        row_count: 0,
        declared_stage_count: 0,
        missing_shared_metric_stage_count: 0,
        shared_metric_field_count: 0,
        rows: Vec::new(),
    };
    let vcf = VcfComparableMetricsReport {
        schema_version: VCF_COMPARABLE_METRICS_SCHEMA_V1.to_string(),
        output_path: "artifacts/vcf.tsv".to_string(),
        stage_count: 0,
        multi_tool_stage_count: 0,
        retained_tool_row_count: 0,
        row_count: 0,
        rows: Vec::new(),
    };
    let acceptance = ScientificAcceptanceThresholdRow {
        domain: "fastq".to_string(),
        stage_id: "trim".to_string(),
        metric_id: "reads_retained".to_string(),
        metric_name: "Reads retained".to_string(),
        unit: Some("reads".to_string()),
        direction: ScientificAcceptanceDirection::HigherIsBetter,
        tolerance_kind: ScientificAcceptanceToleranceKind::RelativeFraction,
        tolerance_value: 0.01,
        pass_rule: ScientificAcceptancePassRule::MeetOrExceedReference,
        insufficiency_behavior: ScientificAcceptanceInsufficiencyBehavior::RefuseStageComparison,
        required: true,
        declaration_origin: "fastq_comparable_metric_policy".to_string(),
    };

    let rows = build_full_benchmark_comparable_metric_rows(
        &fastq,
        &bam,
        &vcf,
        std::slice::from_ref(&acceptance),
    )
    .expect("accept exact comparable-metric closure");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].contract_status, "declared");
    assert_eq!(rows[0].direction, "higher_is_better");
    assert_eq!(rows[0].tool_ids, vec!["cutadapt".to_string(), "fastp".to_string()]);

    let missing = build_full_benchmark_comparable_metric_rows(&fastq, &bam, &vcf, &[])
        .expect_err("missing acceptance semantics must be rejected");
    assert!(missing.to_string().contains("missing scientific acceptance semantics"));

    let duplicate = build_full_benchmark_comparable_metric_rows(
        &fastq,
        &bam,
        &vcf,
        &[acceptance.clone(), acceptance],
    )
    .expect_err("duplicate acceptance semantics must be rejected");
    assert!(duplicate.to_string().contains("duplicate scientific acceptance threshold"));
}
