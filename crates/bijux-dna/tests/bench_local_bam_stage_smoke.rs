#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
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

    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn bench_local_run_bam_stage_smoke_json_for_validate_forwards_governed_report() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "run-bam-stage-smoke",
        "--stage-id",
        "bam.validate",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.validate.local_smoke.report.v1")
    );
    assert_eq!(payload.get("case_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("all_cases_matched").and_then(serde_json::Value::as_bool), Some(true));
}

#[test]
fn bench_local_run_bam_stage_smoke_json_for_coverage_reports_tsv_artifact() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "run-bam-stage-smoke",
        "--stage-id",
        "bam.coverage",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_bam_stage_smoke.v1")
    );
    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.coverage"));
    assert_eq!(
        payload.get("artifact_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/bam.coverage/coverage.tsv")
    );
    assert_eq!(payload.get("artifact_format").and_then(serde_json::Value::as_str), Some("tsv"));
}
