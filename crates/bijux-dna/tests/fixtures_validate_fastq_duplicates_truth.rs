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
fn fixtures_validate_fastq_duplicates_truth_reports_governed_duplicate_bundle() {
    let payload =
        run_cli_json(&["fixtures", "validate", "--corpus", "fastq-duplicates-truth", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.fastq_duplicates_truth.validation.v1")
    );
    assert_eq!(
        payload.get("fixture_id").and_then(serde_json::Value::as_str),
        Some("fastq-duplicates-truth")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/fastq-duplicates-truth/manifest.toml")
    );
    assert_eq!(
        payload.get("expected_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/fastq-duplicates-truth/expected.json")
    );
    assert_eq!(payload.get("validated_stage_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("validated_case_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));
}
