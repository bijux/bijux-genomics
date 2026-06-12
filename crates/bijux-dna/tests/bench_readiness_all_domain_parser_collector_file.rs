#![allow(clippy::expect_used)]

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
fn bench_readiness_all_domain_parser_collector_writes_governed_report_and_fixture_paths() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "readiness",
        "render-all-domain-parser-collector",
        "--json",
    ]);

    let report_path = repo_root.join("benchmarks/readiness/parser-collector-all-domains.json");
    assert!(report_path.is_file(), "collector report must exist");

    let persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read parser collector report"))
            .expect("parse parser collector report");
    assert_eq!(
        persisted.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/parser-collector-all-domains.json")
    );
    assert_eq!(persisted.get("row_count").and_then(serde_json::Value::as_u64), Some(140));

    let fixture_root = repo_root.join("runs/bench/readiness-probes/all-domains/parser-collector");
    assert!(fixture_root.is_dir(), "collector fixture root must exist");
    let fake_run_root = fixture_root.join("fake-runs");
    assert!(fake_run_root.is_dir(), "collector fake-run root must exist");
    assert!(
        fake_run_root.join("manifest.json").is_file(),
        "collector fake-run manifest must exist"
    );

    let fake_metrics = repo_root.join(
        payload
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .expect("rows array")
            .iter()
            .find(|row| {
                row.get("result_id").and_then(serde_json::Value::as_str)
                    == Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
            })
            .and_then(|row| row.get("parsed_path"))
            .and_then(serde_json::Value::as_str)
            .expect("fake VCF parsed path"),
    );
    assert!(fake_metrics.is_file(), "fake-run metrics path must exist");

    let fastq_smoke_path = repo_root.join(
        payload
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .expect("rows array")
            .iter()
            .find(|row| {
                row.get("record_id").and_then(serde_json::Value::as_str)
                    == Some("real-smoke:fastq.validate_reads")
            })
            .and_then(|row| row.get("parsed_path"))
            .and_then(serde_json::Value::as_str)
            .expect("fastq smoke path"),
    );
    assert!(fastq_smoke_path.is_file(), "FASTQ smoke report path must exist");

    let vcf_manifest_path = repo_root.join(
        payload
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .expect("rows array")
            .iter()
            .find(|row| {
                row.get("record_id").and_then(serde_json::Value::as_str)
                    == Some("real-smoke:vcf.stats")
            })
            .and_then(|row| row.get("manifest_path"))
            .and_then(serde_json::Value::as_str)
            .expect("vcf manifest path"),
    );
    assert!(vcf_manifest_path.is_file(), "VCF smoke manifest path must exist");
}
