#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_validate_damage_fixture_json_reports_governed_corpus_01_adna_metadata() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new("cargo")
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["run", "-q", "-p", "bijux-dna", "--"])
        .args([
            "bench",
            "local",
            "validate-corpus-fixture",
            "--manifest",
            "tests/fixtures/corpora/corpus-01-adna-damage-mini/manifest.toml",
            "--json",
        ])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse stdout as json");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.bam_damage_fixture_validation.v1")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("tests/fixtures/corpora/corpus-01-adna-damage-mini/manifest.toml")
    );
    assert_eq!(
        payload.get("fixture_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-adna-damage-mini")
    );
    assert_eq!(
        payload.get("sample_id").and_then(serde_json::Value::as_str),
        Some("adna_damage_non_udg")
    );
    assert_eq!(
        payload.get("udg_model").and_then(serde_json::Value::as_str),
        Some("non_udg")
    );
    assert_eq!(
        payload
            .get("expected_terminal_pattern_class")
            .and_then(serde_json::Value::as_str),
        Some("ct5p_dominant")
    );
    assert!(payload
        .get("limitations")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|limitations| limitations.len() == 2));
    assert_eq!(
        payload
            .get("expected_damage")
            .and_then(|value| value.get("damage_signal"))
            .and_then(serde_json::Value::as_str),
        Some("moderate")
    );
    assert!(payload.get("valid").and_then(serde_json::Value::as_bool) == Some(true));
}
