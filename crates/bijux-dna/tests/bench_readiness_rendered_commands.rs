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
fn bench_readiness_render_commands_reports_governed_benchmark_ready_row_slice() {
    let payload = run_cli_json(&["bench", "readiness", "render-commands", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.rendered_commands.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/rendered-commands.sh")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(112));
    assert_eq!(payload.get("rows").and_then(serde_json::Value::as_array).map(Vec::len), Some(112));
    assert!(payload
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .expect("rows array")
        .iter()
        .all(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str).is_some()
                && row.get("tool_id").and_then(serde_json::Value::as_str).is_some()
                && row
                    .get("argv")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|argv| argv.first().and_then(serde_json::Value::as_str).is_some())
                && row
                    .get("command")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|command| !command.is_empty())
        }));
}
