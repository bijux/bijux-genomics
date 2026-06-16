#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_essential_pipelines_local_complete_writes_gate_file() {
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
        .args(["bench", "readiness", "render-essential-pipelines-local-complete"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        rendered_path.trim(),
        "benchmarks/readiness/pipelines/ESSENTIAL_PIPELINES_LOCAL_COMPLETE.json"
    );

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read essential pipelines local-complete gate");
    let report: serde_json::Value = serde_json::from_str(&payload).expect("parse gate");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.essential_pipelines_local_complete.v1")
    );
    assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("failing_node_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(report.get("node_count").and_then(serde_json::Value::as_u64), Some(93));
}
