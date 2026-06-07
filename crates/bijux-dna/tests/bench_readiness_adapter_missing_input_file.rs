#![cfg(feature = "bam_downstream")]
#![allow(clippy::expect_used)]

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
fn bench_readiness_adapter_missing_input_file_writes_self_describing_report() {
    let output = run_cli(&["bench", "readiness", "render-adapter-missing-input-tests"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/adapter-missing-input-tests.json");

    let repo_root = support::repo_root().expect("repo root");
    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read adapter missing-input report");
    let report: serde_json::Value = serde_json::from_str(&payload).expect("parse report JSON");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.adapter_missing_input_tests.v1")
    );
    assert_eq!(report.get("row_count").and_then(serde_json::Value::as_u64), Some(33));
    assert_eq!(report.get("passed_row_count").and_then(serde_json::Value::as_u64), Some(33));
    assert_eq!(report.get("rows").and_then(serde_json::Value::as_array).map(Vec::len), Some(33));
}
