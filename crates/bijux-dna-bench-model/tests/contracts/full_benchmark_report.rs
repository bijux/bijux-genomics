use bijux_dna_bench_model::full_benchmark_report_schema;

#[test]
fn full_benchmark_report_schema_matches_checked_in_contract() -> anyhow::Result<()> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let path = root.join("benchmarks/contracts/schemas/full-benchmark-report.schema.json");
    let generated = serde_json::to_value(full_benchmark_report_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&std::fs::read(path)?)?, generated);
    Ok(())
}

#[test]
fn full_benchmark_report_rejects_overflowing_source_totals() -> anyhow::Result<()> {
    use bijux_dna_bench_model::{
        validate_full_benchmark_report, FullBenchmarkReport, FullBenchmarkReportAdmissionContext,
        FULL_BENCHMARK_REPORT_SCHEMA_V1,
    };
    use std::collections::BTreeSet;
    let mut report: FullBenchmarkReport = serde_json::from_value(serde_json::json!({
        "schema_version": FULL_BENCHMARK_REPORT_SCHEMA_V1,
        "markdown_output_path": "artifacts/report.md", "json_output_path": "artifacts/report.json",
        "row_count": 0, "expected_result_row_count": usize::MAX, "explicit_unsupported_row_count": 1,
        "present_row_count": 0, "missing_result_row_count": 0, "unsupported_pair_row_count": 0,
        "stage_centric_row_count": 0, "tool_centric_row_count": 0, "corpus_centric_row_count": 0,
        "pipeline_centric_row_count": 0, "runtime_row_count": 0, "memory_row_count": 0,
        "comparable_metric_row_count": 0, "failure_row_count": 0, "failure_class_row_count": 0,
        "passes_behavior_test": false, "domain_counts": {}, "row_status_counts": {}, "rows": [],
        "stage_centric": [], "tool_centric": [], "corpus_centric": [],
        "pipeline_centric": {"row_count": 0, "unmapped_row_count": 0, "rows": []},
        "runtime": [], "memory": [],
        "failures": {"simulated_failure_row_count": 0, "failure_class_row_count": 0,
            "simulated_failure_domain_counts": {}, "rows": [], "classification_rows": []},
        "missing_results": [], "comparable_metrics": [], "unsupported_pairs": []
    }))?;
    let context = FullBenchmarkReportAdmissionContext {
        expected_result_ids: BTreeSet::new(),
        collector_missing_result_count: 0,
        collector_failure_count: 0,
        collector_unsupported_pair_count: 1,
        triggered_failure_class_count: 0,
    };
    assert!(validate_full_benchmark_report(&mut report, &context).is_err());
    assert!(!report.passes_behavior_test);
    Ok(())
}
