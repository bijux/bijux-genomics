use std::collections::BTreeMap;

use bijux_dna_bench::{
    build_full_benchmark_failures_section, build_full_benchmark_missing_result_rows,
    AllDomainFailureClassificationReport, FullBenchmarkReportRow, FullBenchmarkReportRowStatus,
    FullBenchmarkResultCollectorReport, ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_V1,
    FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_V1,
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
