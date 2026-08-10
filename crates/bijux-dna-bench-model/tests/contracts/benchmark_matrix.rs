use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    validate_benchmark_matrix_report, BenchmarkMatrixReport, BenchmarkMatrixRow,
    BenchmarkMatrixSummary, BenchmarkReadiness, BenchmarkSurfaceMatch,
    BENCHMARK_MATRIX_REPORT_SCHEMA_V1,
};

#[test]
fn benchmark_matrix_report_validates_identity_rows_and_summary() {
    assert!(validate_benchmark_matrix_report(&report()).is_ok());

    let mut duplicate = report();
    duplicate.rows.push(duplicate.rows[0].clone());
    duplicate.summary.total_rows += 1;
    *duplicate.summary.readiness_counts.entry("ready".to_string()).or_insert(0) += 1;
    *duplicate.summary.domain_counts.entry("fastq".to_string()).or_insert(0) += 1;
    assert!(validate_benchmark_matrix_report(&duplicate)
        .is_err_and(|error| error.to_string().contains("row identity is invalid")));

    let mut drifted = report();
    drifted.summary.total_rows = 0;
    assert!(validate_benchmark_matrix_report(&drifted)
        .is_err_and(|error| error.to_string().contains("summary drifted")));

    let mut inconsistent = report();
    inconsistent.rows[0].image_match.ready = false;
    assert!(validate_benchmark_matrix_report(&inconsistent)
        .is_err_and(|error| error.to_string().contains("disagrees with surface matches")));
}

#[test]
fn benchmark_matrix_report_schema_matches_model_and_version_fails_closed() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path =
        repository_root.join("benchmarks/contracts/schemas/benchmark-matrix-report.schema.json");
    let generated = serde_json::to_value(bijux_dna_bench_model::benchmark_matrix_report_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "benchmark matrix report schema drifted");

    let mut value = serde_json::to_value(report())?;
    value["schema_version"] = serde_json::json!("bijux.hpc.benchmark_matrix.v2");
    assert!(serde_json::from_value::<BenchmarkMatrixReport>(value).is_err());
    Ok(())
}

fn report() -> BenchmarkMatrixReport {
    BenchmarkMatrixReport {
        schema_version: BENCHMARK_MATRIX_REPORT_SCHEMA_V1.to_string(),
        campaign_id: "campaign-fastq".to_string(),
        domain: "fastq".to_string(),
        domains: vec!["fastq".to_string()],
        generated_at: "1786393800".to_string(),
        summary: BenchmarkMatrixSummary {
            total_rows: 1,
            readiness_counts: BTreeMap::from([("ready".to_string(), 1)]),
            domain_counts: BTreeMap::from([("fastq".to_string(), 1)]),
        },
        rows: vec![BenchmarkMatrixRow {
            row_id: "fastq.trim_reads::adapterremoval".to_string(),
            matrix_domain: "fastq".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            tool_id: "adapterremoval".to_string(),
            corpus_match: surface("general"),
            database_match: surface("not-required"),
            image_match: surface("tool-images"),
            readiness: BenchmarkReadiness { class: "ready".to_string(), reasons: Vec::new() },
            repetitions: 3,
        }],
    }
}

fn surface(profile: &str) -> BenchmarkSurfaceMatch {
    BenchmarkSurfaceMatch {
        required_profile: profile.to_string(),
        matched_profile: profile.to_string(),
        ready: true,
    }
}
