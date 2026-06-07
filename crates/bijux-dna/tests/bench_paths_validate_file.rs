#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_paths_validate_writes_governed_validation_report() {
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
        .args(["bench", "paths", "validate", "--strict"])
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
    assert_eq!(rendered_path.trim(), "target/bench-readiness/benchmark-paths-validation.json");

    let payload: serde_json::Value = serde_json::from_slice(
        &std::fs::read(repo_root.join(rendered_path.trim()))
            .expect("read benchmark paths validation report"),
    )
    .expect("parse benchmark paths validation report");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.paths_validate.v1")
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload
            .get("legacy_fixture_wrapper")
            .and_then(|value| value.get("wrapper_path"))
            .and_then(serde_json::Value::as_str),
        Some("tests/fixtures")
    );
    assert_eq!(
        payload
            .get("legacy_fixture_wrapper")
            .and_then(|value| value.get("actual_target"))
            .and_then(serde_json::Value::as_str),
        Some("../benchmarks/tests/fixtures")
    );
}
