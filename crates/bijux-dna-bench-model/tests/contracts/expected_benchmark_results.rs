use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    all_domain_expected_benchmark_results_schema, validate_all_domain_expected_benchmark_results,
    AllDomainExpectedBenchmarkResultRow, AllDomainExpectedBenchmarkResultsReport,
    ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_SCHEMA_V1,
};

#[test]
fn expected_benchmark_results_schema_matches_checked_in_contract() -> anyhow::Result<()> {
    let generated = serde_json::to_value(all_domain_expected_benchmark_results_schema())?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let path =
        root.join("benchmarks/contracts/schemas/all-domain-expected-benchmark-results.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&std::fs::read(path)?)?, generated);
    Ok(())
}

#[test]
fn expected_benchmark_results_admit_exact_aggregate_closure() -> anyhow::Result<()> {
    validate_all_domain_expected_benchmark_results(&valid_report())?;

    let mut legacy_protocol = valid_report();
    legacy_protocol.rows[0].asset_profile_id = "reference_fasta+reference_panel".to_string();
    validate_all_domain_expected_benchmark_results(&legacy_protocol)?;
    Ok(())
}

#[test]
fn expected_benchmark_results_reject_identity_and_aggregate_drift() {
    let mut report = valid_report();
    report.rows.push(report.rows[0].clone());
    report.row_count += 1;
    assert!(validate_all_domain_expected_benchmark_results(&report).is_err());

    let mut report = valid_report();
    report.stage_count += 1;
    assert!(validate_all_domain_expected_benchmark_results(&report).is_err());

    let mut report = valid_report();
    report.output_path = "../outside.tsv".to_string();
    assert!(validate_all_domain_expected_benchmark_results(&report).is_err());

    let mut report = valid_report();
    report.rows[0].domain = "protein".to_string();
    assert!(validate_all_domain_expected_benchmark_results(&report).is_err());

    let mut report = valid_report();
    report.rows[0].result_id = "drifted-result-id".to_string();
    assert!(validate_all_domain_expected_benchmark_results(&report).is_err());

    let mut report = valid_report();
    report.rows[0].expected_metrics.clear();
    assert!(validate_all_domain_expected_benchmark_results(&report).is_err());
}

fn valid_report() -> AllDomainExpectedBenchmarkResultsReport {
    AllDomainExpectedBenchmarkResultsReport {
        schema_version: ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_SCHEMA_V1.to_string(),
        output_path: "artifacts/readiness/expected.tsv".to_string(),
        row_count: 1,
        result_id_count: 1,
        stage_count: 1,
        tool_count: 1,
        corpus_count: 1,
        asset_profile_count: 1,
        domain_counts: BTreeMap::from([("fastq".to_string(), 1)]),
        report_section_counts: BTreeMap::from([("read_validation".to_string(), 1)]),
        rows: vec![AllDomainExpectedBenchmarkResultRow {
            result_id: "fastq:corpus:validate_reads:sample-set:seqkit".to_string(),
            domain: "fastq".to_string(),
            stage_id: "validate_reads".to_string(),
            tool_id: "seqkit".to_string(),
            corpus_id: "corpus".to_string(),
            asset_profile_id: "sample-set".to_string(),
            expected_outputs: vec!["validated_fastq".to_string()],
            expected_metrics: vec!["valid_read_fraction".to_string()],
            report_section: "read_validation".to_string(),
        }],
    }
}

#[test]
fn collector_projection_preserves_expected_evidence_without_claiming_execution(
) -> anyhow::Result<()> {
    use bijux_dna_bench_model::{
        project_full_benchmark_observation_rows as project, FullBenchmarkResultStatus,
        FullBenchmarkResultSurfaceKind, FullBenchmarkSourceObservation as Observation,
    };
    let mut report = valid_report();
    let rows = project(Observation::Expected(&report))?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].result_status, FullBenchmarkResultStatus::Expected);
    assert_eq!(rows[0].surface_kind, FullBenchmarkResultSurfaceKind::BenchmarkExpected);
    assert_eq!(rows[0].record_id, format!("expected:{}", report.rows[0].result_id));
    assert_eq!(rows[0].result_id.as_ref(), Some(&report.rows[0].result_id));
    assert_eq!(rows[0].evidence_path, report.output_path);
    assert!(rows[0].execution_id.is_none() && rows[0].manifest_path.is_none());
    report.row_count += 1;
    assert!(project(Observation::Expected(&report)).is_err());
    report.row_count -= 1;
    report.rows[0].expected_metrics.clear();
    assert!(project(Observation::Expected(&report)).is_err());
    Ok(())
}
