use std::collections::{BTreeMap, BTreeSet};

use bijux_dna_bench_model::{
    full_benchmark_result_collector_schema, validate_full_benchmark_result_collector,
    FullBenchmarkResultBinding, FullBenchmarkResultCollectorAdmissionContext,
    FullBenchmarkResultCollectorReport, FullBenchmarkResultCollectorRow, FullBenchmarkResultStatus,
    FullBenchmarkResultSurfaceKind, FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_V1,
};

#[test]
fn full_benchmark_result_collector_schema_matches_checked_in_contract() -> anyhow::Result<()> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let path =
        root.join("benchmarks/contracts/schemas/full-benchmark-result-collector.schema.json");
    let generated = serde_json::to_value(full_benchmark_result_collector_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&std::fs::read(path)?)?, generated);
    Ok(())
}

#[test]
fn full_benchmark_result_collector_admits_exact_source_and_status_closure() -> anyhow::Result<()> {
    let mut report = valid_report();
    validate_full_benchmark_result_collector(&mut report, &context())?;
    assert!(report.passes_behavior_test);
    Ok(())
}

#[test]
fn full_benchmark_result_collector_rejects_source_surface_drift() {
    let mut report = valid_report();
    report.rows[0].source_surface = "wrong_source".to_string();
    assert!(validate_full_benchmark_result_collector(&mut report, &context()).is_err());
    assert!(!report.passes_behavior_test);
}

fn valid_report() -> FullBenchmarkResultCollectorReport {
    let rows = vec![
        row(
            "expected",
            "all_domain_expected_benchmark_results",
            FullBenchmarkResultSurfaceKind::BenchmarkExpected,
            FullBenchmarkResultStatus::Expected,
            "vcf.expected",
            "bcftools",
        ),
        row(
            "pipeline",
            "essential_pipeline_fake_runs",
            FullBenchmarkResultSurfaceKind::PipelineFakeRun,
            FullBenchmarkResultStatus::Succeeded,
            "vcf.pipeline",
            "bcftools",
        ),
        row(
            "fake-run",
            "all_domain_fake_runs",
            FullBenchmarkResultSurfaceKind::FakeRun,
            FullBenchmarkResultStatus::Succeeded,
            "vcf.fake_run",
            "bcftools",
        ),
        row(
            "fake-failure",
            "all_domain_fake_failures",
            FullBenchmarkResultSurfaceKind::FakeFailure,
            FullBenchmarkResultStatus::Failed,
            "vcf.fake_failure",
            "bcftools",
        ),
        row(
            "audit",
            "all_domain_missing_result_audit",
            FullBenchmarkResultSurfaceKind::MissingResultAudit,
            FullBenchmarkResultStatus::Present,
            "vcf.audit",
            "bcftools",
        ),
        row(
            "real-smoke",
            "real_smoke_core_subset",
            FullBenchmarkResultSurfaceKind::RealSmoke,
            FullBenchmarkResultStatus::Succeeded,
            "vcf.smoke",
            "bcftools",
        ),
        row(
            "insufficient",
            "all_domain_failure_classification",
            FullBenchmarkResultSurfaceKind::FailureClassification,
            FullBenchmarkResultStatus::InsufficientData,
            "vcf.demography",
            "fastsimcoal2",
        ),
        row(
            "unsupported",
            "all_domain_failure_classification",
            FullBenchmarkResultSurfaceKind::UnsupportedPair,
            FullBenchmarkResultStatus::UnsupportedPair,
            "vcf.filter",
            "samtools",
        ),
    ];
    FullBenchmarkResultCollectorReport {
        schema_version: FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_V1.to_string(),
        output_path: "artifacts/collector.json".to_string(),
        row_count: rows.len(),
        benchmark_expected_row_count: 1,
        pipeline_fake_run_row_count: 1,
        fake_run_row_count: 1,
        fake_failure_row_count: 1,
        missing_result_audit_row_count: 1,
        real_smoke_row_count: 1,
        insufficient_data_row_count: 1,
        unsupported_pair_row_count: 1,
        missing_result_status_count: 0,
        insufficient_data_status_count: 1,
        unsupported_pair_status_count: 1,
        passes_behavior_test: false,
        surface_kind_counts: counts(rows.iter().map(|row| row.surface_kind.as_str())),
        result_status_counts: counts(rows.iter().map(|row| row.result_status.as_str())),
        domain_counts: BTreeMap::from([("vcf".to_string(), rows.len())]),
        rows,
    }
}

fn context() -> FullBenchmarkResultCollectorAdmissionContext {
    FullBenchmarkResultCollectorAdmissionContext {
        expected_pipeline_fake_run_row_count: 1,
        expected_real_smoke_row_count: 1,
        expected_insufficient_data_row_count: 1,
        expected_missing_result_row_count: 0,
        expected_domains: BTreeSet::from(["vcf".to_string()]),
        expected_unsupported_bindings: BTreeSet::from([FullBenchmarkResultBinding {
            domain: "vcf".to_string(),
            stage_id: "vcf.filter".to_string(),
            tool_id: "samtools".to_string(),
        }]),
    }
}

fn row(
    record_id: &str,
    source_surface: &str,
    surface_kind: FullBenchmarkResultSurfaceKind,
    result_status: FullBenchmarkResultStatus,
    stage_id: &str,
    tool_id: &str,
) -> FullBenchmarkResultCollectorRow {
    FullBenchmarkResultCollectorRow {
        record_id: record_id.to_string(),
        source_surface: source_surface.to_string(),
        surface_kind,
        result_status,
        domain: "vcf".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        pipeline_id: None,
        node_id: None,
        result_id: None,
        execution_id: None,
        corpus_id: None,
        asset_profile_id: None,
        report_section: None,
        evidence_path: format!("artifacts/{record_id}.json"),
        manifest_path: None,
        declared_output_count: 0,
        normalized_metric_count: 0,
        detail: "test evidence".to_string(),
    }
}

fn counts<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    values.fold(BTreeMap::new(), |mut counts, value| {
        *counts.entry(value.to_string()).or_default() += 1;
        counts
    })
}

#[test]
fn full_benchmark_result_collector_rejects_each_overflowing_source_count() {
    for source in 0..8 {
        let mut report = valid_report();
        match source {
            0 => report.benchmark_expected_row_count = usize::MAX,
            1 => report.pipeline_fake_run_row_count = usize::MAX,
            2 => report.fake_run_row_count = usize::MAX,
            3 => report.fake_failure_row_count = usize::MAX,
            4 => report.missing_result_audit_row_count = usize::MAX,
            5 => report.real_smoke_row_count = usize::MAX,
            6 => report.insufficient_data_row_count = usize::MAX,
            _ => report.unsupported_pair_row_count = usize::MAX,
        }
        assert!(validate_full_benchmark_result_collector(&mut report, &context()).is_err());
        assert!(!report.passes_behavior_test);
    }
}

#[test]
fn full_benchmark_result_collector_rejects_missing_count_beyond_expected_results() {
    let mut report = valid_report();
    let mut context = context();
    context.expected_missing_result_row_count = 2;
    assert!(validate_full_benchmark_result_collector(&mut report, &context).is_err());
    assert!(!report.passes_behavior_test);
}
