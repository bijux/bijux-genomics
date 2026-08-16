#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_container_tool_smoke_writes_manifest_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    let runtime_root = tempfile::tempdir().expect("runtime tempdir");

    let mut command = Command::new(env!("CARGO_BIN_EXE_bijux-dna"));
    support::configure_isolated_runtime_resolution(&mut command, runtime_root.path());
    let output = command
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .args(["bench", "readiness", "run-container-tool-smoke", "--tools", "shapeit5"])
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
    assert_eq!(rendered_root.trim(), "runs/bench/tool-smoke/container");

    let manifest_path = repo_root.join("runs/bench/tool-smoke/container/shapeit5/manifest.json");
    assert!(manifest_path.is_file(), "container smoke manifest must exist");

    let payload = std::fs::read_to_string(&manifest_path).expect("read container smoke manifest");
    let parsed: serde_json::Value = serde_json::from_str(&payload).expect("parse manifest json");

    assert_eq!(
        parsed.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.container_tool_smoke_manifest.v1")
    );
    assert_eq!(parsed.get("tool_id").and_then(serde_json::Value::as_str), Some("shapeit5"));
    assert_eq!(
        parsed.get("status").and_then(serde_json::Value::as_str),
        Some("unavailable_with_reason")
    );
    assert_eq!(parsed.get("smoke_runtime").and_then(serde_json::Value::as_str), Some("apptainer"));
    assert_eq!(
        parsed.get("declared_command").and_then(serde_json::Value::as_str),
        Some("bijux-dna env smoke apptainer shapeit5")
    );
    let applied_command = parsed
        .get("applied_command")
        .and_then(serde_json::Value::as_array)
        .expect("applied command array");
    assert_eq!(applied_command.len(), 4);
    assert!(applied_command[0].as_str().is_some_and(|value| value.ends_with("bijux-dna-dev")));
    assert_eq!(applied_command[1].as_str(), Some("containers"));
    assert_eq!(applied_command[2].as_str(), Some("run"));
    assert_eq!(applied_command[3].as_str(), Some("smoke-containers-apptainer"));
    assert!(parsed.get("exit_code").is_none_or(serde_json::Value::is_null));
    assert_eq!(parsed.get("stdout").and_then(serde_json::Value::as_str), Some(""));
    assert_eq!(parsed.get("stderr").and_then(serde_json::Value::as_str), Some(""));
    assert!(parsed
        .get("unavailable_reason")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value.contains("runtime `apptainer` is not available on PATH")));
}
