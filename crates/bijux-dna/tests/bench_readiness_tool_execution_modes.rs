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

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
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
fn bench_readiness_tool_execution_modes_report_governs_all_benchmark_tools() {
    let payload = run_cli_json(&["bench", "readiness", "validate-tool-execution-modes", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.tool_execution_modes_validation.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/bench/local/tool-execution-modes.toml")
    );
    assert_eq!(
        payload.get("classification_scope").and_then(serde_json::Value::as_str),
        Some("primary_operator_runtime")
    );
    assert_eq!(payload.get("mode_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(69));
    assert_eq!(payload.get("multidomain_tool_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let multiqc = rows
        .iter()
        .find(|row| row.get("tool_id").and_then(serde_json::Value::as_str) == Some("multiqc"))
        .expect("multiqc row");
    assert_eq!(multiqc.get("execution_mode").and_then(serde_json::Value::as_str), Some("python"));
}
