#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_host_tool_smoke_writes_manifest_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .args(["bench", "readiness", "run-host-tool-smoke"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_root = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_root.trim(), "runs/bench/tool-smoke/host");

    let manifest_path = repo_root.join("runs/bench/tool-smoke/host/bijux_dna/manifest.json");
    assert!(manifest_path.is_file(), "host smoke manifest must exist");

    let payload = std::fs::read_to_string(&manifest_path).expect("read host smoke manifest");
    let parsed: serde_json::Value = serde_json::from_str(&payload).expect("parse manifest json");

    assert_eq!(
        parsed.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.host_tool_smoke_manifest.v1")
    );
    assert_eq!(parsed.get("tool_id").and_then(serde_json::Value::as_str), Some("bijux_dna"));
    assert_eq!(parsed.get("status").and_then(serde_json::Value::as_str), Some("ok"));
    assert_eq!(
        parsed.get("declared_command").and_then(serde_json::Value::as_str),
        Some("bijux-dna --version")
    );
    assert_eq!(parsed.get("working_directory").and_then(serde_json::Value::as_str), Some("."));
    assert_eq!(parsed.get("exit_code").and_then(serde_json::Value::as_i64), Some(0));
    assert_eq!(parsed.get("version").and_then(serde_json::Value::as_str), Some("0.1.0"));
    assert_eq!(
        parsed.get("version_matches_regex").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert!(parsed.get("applied_command").and_then(serde_json::Value::as_array).is_some_and(
        |argv| {
            argv.len() == 2
                && argv
                    .get(0)
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|value| value.ends_with("/debug/bijux-dna"))
                && argv.get(1).and_then(serde_json::Value::as_str) == Some("--version")
        }
    ));
    assert_eq!(parsed.get("stdout").and_then(serde_json::Value::as_str), Some("bijux-dna 0.1.0\n"));
    assert_eq!(parsed.get("stderr").and_then(serde_json::Value::as_str), Some(""));
}
