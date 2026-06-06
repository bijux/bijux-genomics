#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json() -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-vcf-adapter-output-coverage", "--json"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn bench_readiness_vcf_adapter_output_coverage_reports_governed_rows() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_adapter_output_coverage.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/vcf-adapter-output-coverage.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(38));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(8)
    );
    assert_eq!(
        payload.get("benchmark_ready_complete_row_count").and_then(serde_json::Value::as_u64),
        Some(8)
    );
    assert_eq!(
        payload.get("benchmark_ready_incomplete_row_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(35));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(3));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 38);

    let call = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
        })
        .expect("bcftools call row");
    assert_eq!(
        call.get("benchmark_status").and_then(serde_json::Value::as_str),
        Some("benchmark_ready")
    );
    assert_eq!(call.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert!(
        call.get("normalized_metrics")
            .and_then(serde_json::Value::as_array)
            .expect("call normalized metrics")
            .iter()
            .filter_map(serde_json::Value::as_str)
            .any(|entry| entry.starts_with("called_vcf=")),
        "call row must keep the normalized VCF output explicit"
    );
    assert!(
        call.get("index_outputs")
            .and_then(serde_json::Value::as_array)
            .expect("call index outputs")
            .iter()
            .filter_map(serde_json::Value::as_str)
            .any(|entry| entry.contains(".tbi")),
        "call row must keep the index output explicit"
    );

    let stats = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.stats")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
        })
        .expect("bcftools stats row");
    assert_eq!(stats.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert_eq!(
        stats.get("index_outputs").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(0)
    );
    assert!(
        stats
            .get("normalized_metrics")
            .and_then(serde_json::Value::as_array)
            .expect("stats normalized metrics")
            .iter()
            .filter_map(serde_json::Value::as_str)
            .any(|entry| entry.starts_with("stats_json=")),
        "stats row must keep the normalized stats artifact explicit"
    );

    let phasing = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("shapeit5")
        })
        .expect("shapeit5 row");
    assert_eq!(
        phasing.get("benchmark_status").and_then(serde_json::Value::as_str),
        Some("not_benchmark_ready")
    );
    assert_eq!(phasing.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert!(
        phasing
            .get("index_outputs")
            .and_then(serde_json::Value::as_array)
            .expect("phasing index outputs")
            .iter()
            .filter_map(serde_json::Value::as_str)
            .any(|entry| entry.contains(".tbi")),
        "shapeit5 row must keep the phased VCF index explicit"
    );

    let demography = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.demography")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("ibdne")
        })
        .expect("ibdne row");
    assert_eq!(demography.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert!(
        demography
            .get("manifest")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|path| path.ends_with("/stage-result.json")),
        "demography row must keep the deterministic stage-result path template"
    );

    let roh_count = rows
        .iter()
        .filter(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.roh")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
        })
        .count();
    assert_eq!(roh_count, 1, "VCF ROH coverage must keep one canonical plink2 row");

    let angsd_gl = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call_gl")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("angsd")
        })
        .expect("angsd call_gl row");
    assert_eq!(angsd_gl.get("status").and_then(serde_json::Value::as_str), Some("incomplete"));
    assert_eq!(
        angsd_gl
            .get("missing_declarations")
            .and_then(serde_json::Value::as_array)
            .expect("angsd call_gl missing declarations")
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>(),
        vec!["index_outputs"]
    );
}
