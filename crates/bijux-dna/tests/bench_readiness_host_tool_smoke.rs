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

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
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
fn bench_readiness_host_tool_smoke_reports_executed_host_commands() {
    let payload = run_cli_json(&["bench", "readiness", "run-host-tool-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.host_tool_smoke_report.v1")
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/tool-smoke/host")
    );
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("success_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("failure_count").and_then(serde_json::Value::as_u64), Some(0));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 1);
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna")
            && row.get("status").and_then(serde_json::Value::as_str) == Some("ok")
            && row.get("command").and_then(serde_json::Value::as_str) == Some("bijux-dna --version")
            && row.get("exit_code").and_then(serde_json::Value::as_i64) == Some(0)
            && row.get("manifest_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/tool-smoke/host/bijux_dna/manifest.json")
    }));
}
