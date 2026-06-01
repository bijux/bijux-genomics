#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_validate_taxonomy_database_fixture_json_reports_governed_taxonomy_mini_contract() {
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
            "validate-taxonomy-database-fixture",
            "--manifest",
            "tests/fixtures/databases/taxonomy-mini/manifest.toml",
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
        Some("bijux.bench.taxonomy_database_fixture_validation.v1")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("tests/fixtures/databases/taxonomy-mini/manifest.toml")
    );
    assert_eq!(
        payload.get("database_id").and_then(serde_json::Value::as_str),
        Some("taxonomy-mini")
    );
    assert_eq!(
        payload.get("sequence_index_path").and_then(serde_json::Value::as_str),
        Some("tests/fixtures/databases/taxonomy-mini/kraken2/hash.k2d")
    );
    assert_eq!(
        payload.get("source_record_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(payload.get("taxa_count").and_then(serde_json::Value::as_u64), Some(3));
    assert!(payload.get("valid").and_then(serde_json::Value::as_bool) == Some(true));
    assert!(payload
        .get("expected_classifier_compatibility")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|classifiers| {
            classifiers.len() == 1
                && classifiers.first().and_then(serde_json::Value::as_str) == Some("kraken2")
        }));
    assert!(payload.get("taxa").and_then(serde_json::Value::as_array).is_some_and(|taxa| {
        taxa.len() == 3
            && taxa.iter().any(|taxon| {
                taxon.get("taxon_id").and_then(serde_json::Value::as_u64) == Some(28901)
                    && taxon.get("name").and_then(serde_json::Value::as_str)
                        == Some("Salmonella enterica")
                    && taxon.get("rank").and_then(serde_json::Value::as_str) == Some("species")
            })
    }));
}
