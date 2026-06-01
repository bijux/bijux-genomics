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
fn bench_local_collect_runtime_metrics_json_reports_governed_51_stage_slice() {
    let fake_run_root = "target/local-fake-runs/stages-runtime-metrics-cli";
    let report_output = "target/local-ready/runtime-metrics.cli.json";

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
        "collect-runtime-metrics",
        "--fake-run-root",
        fake_run_root,
        "--output",
        report_output,
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_runtime_metrics.v1")
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
    assert!(payload.get("stages").and_then(serde_json::Value::as_array).is_some_and(
        |stages| stages.len() == 51
            && stages.iter().all(|stage| {
                stage.get("started_at").and_then(serde_json::Value::as_str).is_some()
                    && stage.get("finished_at").and_then(serde_json::Value::as_str).is_some()
                    && stage
                        .get("elapsed_seconds")
                        .and_then(serde_json::Value::as_f64)
                        .is_some_and(|elapsed| elapsed >= 0.0)
                    && stage.get("exit_code").and_then(serde_json::Value::as_i64) == Some(0)
                    && stage.get("status").and_then(serde_json::Value::as_str) == Some("succeeded")
            })
    ));
}
