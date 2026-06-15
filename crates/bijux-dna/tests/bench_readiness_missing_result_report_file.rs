#![cfg(feature = "bam_downstream")]
#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

#[test]
fn bench_readiness_missing_result_report_writes_self_describing_report() {
    let output = run_cli(&["bench", "readiness", "render-missing-result-report"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/missing-result-report-test.json");

    let repo_root = support::repo_root().expect("repo root");
    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read missing-result report");
    let report: serde_json::Value = serde_json::from_str(&payload).expect("parse report JSON");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.missing_result_report.v1")
    );
    assert_eq!(report.get("passes_behavior_test"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(report.get("expected_row_count").and_then(serde_json::Value::as_u64), Some(118));
    assert_eq!(
        report.get("present_result_row_count").and_then(serde_json::Value::as_u64),
        Some(117)
    );
    assert_eq!(report.get("missing_result_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("rows").and_then(serde_json::Value::as_array).map(Vec::len), Some(118));
}
