#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::path::PathBuf;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json_with_repo_root(args: &[&str]) -> (PathBuf, serde_json::Value) {
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
        .args(args)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    (repo_root, serde_json::from_slice(&output.stdout).expect("parse stdout as json"))
}

#[test]
fn bench_readiness_full_benchmark_result_collector_writes_report_and_keeps_status_evidence_distinct(
) {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "readiness",
        "render-full-benchmark-result-collector",
        "--json",
    ]);

    let report_path = repo_root.join("benchmarks/readiness/full-result-collector-test.json");
    assert!(report_path.is_file(), "collector report must exist");

    let persisted: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&report_path).expect("read full benchmark result collector report"),
    )
    .expect("parse full benchmark result collector report");
    assert_eq!(persisted.get("row_count").and_then(serde_json::Value::as_u64), Some(607));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");

    let fake_run_metrics_path = repo_root.join(
        rows.iter()
            .find(|row| {
                row.get("record_id").and_then(serde_json::Value::as_str)
                    == Some("fake-run:vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
            })
            .and_then(|row| row.get("evidence_path"))
            .and_then(serde_json::Value::as_str)
            .expect("fake-run evidence path"),
    );
    assert!(fake_run_metrics_path.is_file(), "fake-run evidence path must exist");

    let pipeline_manifest_path = repo_root.join(
        rows.iter()
            .find(|row| {
                row.get("record_id").and_then(serde_json::Value::as_str)
                    == Some("pipeline:core-germline-fastq-bam-vcf:vcf.stats")
            })
            .and_then(|row| row.get("manifest_path"))
            .and_then(serde_json::Value::as_str)
            .expect("pipeline manifest path"),
    );
    assert!(pipeline_manifest_path.is_file(), "pipeline stage-result path must exist");

    let unsupported_pair_evidence_path = repo_root.join(
        rows.iter()
            .find(|row| {
                row.get("result_status").and_then(serde_json::Value::as_str)
                    == Some("unsupported_pair")
            })
            .and_then(|row| row.get("evidence_path"))
            .and_then(serde_json::Value::as_str)
            .expect("unsupported pair evidence path"),
    );
    assert!(unsupported_pair_evidence_path.is_file(), "unsupported-pair evidence path must exist");

    let insufficient_data_evidence_path = repo_root.join(
        rows.iter()
            .find(|row| {
                row.get("result_status").and_then(serde_json::Value::as_str)
                    == Some("insufficient_data")
            })
            .and_then(|row| row.get("evidence_path"))
            .and_then(serde_json::Value::as_str)
            .expect("insufficient-data evidence path"),
    );
    assert!(
        insufficient_data_evidence_path.is_file(),
        "insufficient-data evidence path must exist"
    );

    let missing_manifest_path = repo_root.join(
        rows.iter()
            .find(|row| {
                row.get("record_id").and_then(serde_json::Value::as_str)
                    == Some(
                        "missing-audit:vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools",
                    )
            })
            .and_then(|row| row.get("manifest_path"))
            .and_then(serde_json::Value::as_str)
            .expect("missing-result manifest path"),
    );
    assert!(!missing_manifest_path.exists(), "missing-result manifest path must stay absent");

    let real_smoke_manifest_path = repo_root.join(
        rows.iter()
            .find(|row| {
                row.get("record_id").and_then(serde_json::Value::as_str)
                    == Some("real-smoke:bridge:bam-to-vcf.call")
            })
            .and_then(|row| row.get("manifest_path"))
            .and_then(serde_json::Value::as_str)
            .expect("real-smoke manifest path"),
    );
    assert!(real_smoke_manifest_path.is_file(), "real-smoke manifest path must exist");
}
