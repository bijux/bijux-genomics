#![allow(clippy::expect_used)]

use std::fs;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn plan_validate_all_local_pipelines_writes_governed_report_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    let output_path =
        repo_root.join("target/local-ready/pipeline-dag/all-benchmark-pipelines.json");

    if output_path.exists() {
        fs::remove_file(&output_path).expect("remove stale output");
    }

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["pipeline", "validate", "--benchmark-root", "benchmarks", "--all", "--strict"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout utf8").trim(),
        "target/local-ready/pipeline-dag/all-benchmark-pipelines.json"
    );

    let payload = fs::read_to_string(&output_path).expect("read output file");
    let json: serde_json::Value = serde_json::from_str(&payload).expect("parse output json");
    assert_eq!(json.get("pipeline_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(json.get("all_valid").and_then(serde_json::Value::as_bool), Some(true));
}
