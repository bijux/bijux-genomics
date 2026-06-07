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
fn bench_local_check_manifest_completion_reports_incomplete_stage_when_manifest_is_missing() {
    let repo_root = support::repo_root().expect("repo root");

    let fake_run_root = "target/local-fake-runs/stages-manifest-completion-missing-cli";
    let report_output =
        "benchmarks/readiness/local-ready/manifest-completion-report.missing-cli.json";
    let fake_run_manifest = run_cli_json(&[
        "bench",
        "local",
        "fake-run-stages",
        "--output-root",
        fake_run_root,
        "--json",
    ]);

    let missing_manifest_path = fake_run_manifest
        .get("stages")
        .and_then(serde_json::Value::as_array)
        .expect("stages array")
        .iter()
        .find(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
        })
        .and_then(|stage| stage.get("stage_manifest_path").and_then(serde_json::Value::as_str))
        .expect("report_qc stage manifest path");
    std::fs::remove_file(repo_root.join(missing_manifest_path))
        .expect("remove fake stage manifest");

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

    assert_eq!(payload.get("complete").and_then(serde_json::Value::as_bool), Some(false));
    assert!(payload
        .get("incomplete_stage_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 1));
    let stage = payload
        .get("stages")
        .and_then(serde_json::Value::as_array)
        .expect("stages array")
        .iter()
        .find(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
        })
        .expect("fastq.report_qc stage");
    assert_eq!(stage.get("complete").and_then(serde_json::Value::as_bool), Some(false));
    assert_eq!(stage.get("manifest_exists").and_then(serde_json::Value::as_bool), Some(false));
    assert_eq!(
        stage.get("present_output_count").and_then(serde_json::Value::as_u64),
        stage.get("declared_output_count").and_then(serde_json::Value::as_u64)
    );
    assert!(stage
        .get("stage_manifest_path")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|path| path.ends_with("fastq.report_qc/stage-result.json")));
}
