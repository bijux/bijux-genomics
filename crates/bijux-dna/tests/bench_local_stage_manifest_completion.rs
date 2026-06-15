#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new("cargo")
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["run", "-q", "-p", "bijux-dna", "--features", "bam_downstream", "--"])
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

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_check_manifest_completion_json_reports_governed_51_stage_slice_complete() {
    let fake_run_root = "runs/bench/local-fake-runs/stages-manifest-completion-cli";
    let report_output = "benchmarks/readiness/local-ready/manifest-completion-report.cli.json";

    let _fake_run_manifest = run_cli_json(&[
        "bench",
        "local",
        "fake-run-stages",
        "--output-root",
        fake_run_root,
        "--json",
    ]);
    let payload = run_cli_json(&[
        "bench",
        "local",
        "check-manifest-completion",
        "--fake-run-root",
        fake_run_root,
        "--output",
        report_output,
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_stage_manifest_completion.v1")
    );
    assert_eq!(
        payload.get("fake_run_root").and_then(serde_json::Value::as_str),
        Some(fake_run_root)
    );
    assert_eq!(
        payload.get("report_output_path").and_then(serde_json::Value::as_str),
        Some(report_output)
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(payload.get("complete_stage_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(payload.get("incomplete_stage_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("complete").and_then(serde_json::Value::as_bool), Some(true));
}
