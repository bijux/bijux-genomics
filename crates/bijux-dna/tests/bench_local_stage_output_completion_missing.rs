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
fn bench_local_check_output_completion_reports_incomplete_stage_when_fake_output_is_missing() {
    let repo_root = support::repo_root().expect("repo root");

    let fake_run_root = "runs/bench/local-fake-runs/stages-output-completion-missing-cli";
    let report_output =
        "benchmarks/readiness/local-ready/output-completion-report.missing-cli.json";
    let fake_run_manifest = run_cli_json(&[
        "bench",
        "local",
        "fake-run-stages",
        "--output-root",
        fake_run_root,
        "--json",
    ]);

    let missing_path = fake_run_manifest
        .get("stages")
        .and_then(serde_json::Value::as_array)
        .expect("stages array")
        .iter()
        .find(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
        })
        .and_then(|stage| stage.get("outputs").and_then(serde_json::Value::as_array))
        .and_then(|outputs| {
            outputs.iter().find(|artifact| {
                artifact.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("report_json")
                    && artifact
                        .get("fake_run_path")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|path| path.ends_with("report_qc_report.json"))
            })
        })
        .and_then(|artifact| artifact.get("fake_run_path").and_then(serde_json::Value::as_str))
        .expect("report_qc report json path");
    std::fs::remove_file(repo_root.join(missing_path)).expect("remove fake output");

    let payload = run_cli_json(&[
        "bench",
        "local",
        "check-output-completion",
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
    assert!(stage
        .get("missing_output_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 1));
    assert!(stage.get("missing_outputs").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| outputs.iter().any(|artifact| {
            artifact
                .get("expected_fake_run_path")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|path| path.ends_with("report_qc_report.json"))
        })
    ));
}
