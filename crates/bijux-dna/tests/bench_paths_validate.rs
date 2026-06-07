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

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
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
fn bench_paths_validate_reports_tracked_benchmark_roots() {
    let payload = run_cli_json(&["bench", "paths", "validate", "--strict", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.paths_validate.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/benchmark-paths-validation.json")
    );
    assert_eq!(payload.get("strict").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("root_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("existing_root_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("tracked_marker_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("ignored_root_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("root_tests_regular_file_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("root_tests_readme_tracked_by_git").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    let legacy_wrapper = payload
        .get("legacy_fixture_wrapper")
        .and_then(serde_json::Value::as_object)
        .expect("legacy fixture wrapper object");
    assert_eq!(
        legacy_wrapper.get("wrapper_path").and_then(serde_json::Value::as_str),
        Some("tests/fixtures")
    );
    assert_eq!(
        legacy_wrapper.get("actual_target").and_then(serde_json::Value::as_str),
        Some("../benchmarks/tests/fixtures")
    );
    assert_eq!(legacy_wrapper.get("is_symlink").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        payload.get("roots").and_then(serde_json::Value::as_array).map(std::vec::Vec::len),
        Some(5)
    );
}
