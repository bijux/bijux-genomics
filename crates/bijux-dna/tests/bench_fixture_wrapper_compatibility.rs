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
fn bench_local_validate_taxonomy_database_fixture_accepts_root_fixture_wrapper() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-taxonomy-database-fixture",
        "--manifest",
        "tests/fixtures/databases/taxonomy-mini/manifest.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("tests/fixtures/databases/taxonomy-mini/manifest.toml")
    );
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));
}

#[test]
fn bench_local_validate_corpus_fixture_accepts_root_fixture_wrapper() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-corpus-fixture",
        "--manifest",
        "tests/fixtures/corpora/corpus-01-mini/manifest.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("tests/fixtures/corpora/corpus-01-mini/manifest.toml")
    );
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));
}
