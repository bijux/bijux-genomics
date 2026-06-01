#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_validate_edna_corpus_fixture_json_reports_governed_corpus_02_contract() {
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
            "tests/fixtures/corpora/corpus-02-edna-mini/manifest.toml",
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
        Some("bijux.bench.edna_corpus_fixture_validation.v1")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("tests/fixtures/corpora/corpus-02-edna-mini/manifest.toml")
    );
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-02-edna-mini")
    );
    assert_eq!(
        payload.get("community_id").and_then(serde_json::Value::as_str),
        Some("mock_community_taxonomy")
    );
    assert_eq!(payload.get("expected_taxa_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(2));
    assert!(payload.get("valid").and_then(serde_json::Value::as_bool) == Some(true));
    assert!(payload.get("expected_taxa").and_then(serde_json::Value::as_array).is_some_and(
        |taxa| {
            taxa.len() == 3
                && taxa.iter().any(|taxon| {
                    taxon.get("taxon_id").and_then(serde_json::Value::as_u64) == Some(561)
                        && taxon.get("name").and_then(serde_json::Value::as_str)
                            == Some("Escherichia coli")
                        && taxon.get("rank").and_then(serde_json::Value::as_str) == Some("species")
                })
        }
    ));
    assert!(payload.get("samples").and_then(serde_json::Value::as_array).is_some_and(|samples| {
        samples.len() == 2
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("mock_community_sample_a")
                    && sample.get("community_label").and_then(serde_json::Value::as_str)
                        == Some("mixed_microbiome")
                    && sample.get("observed_read_count").and_then(serde_json::Value::as_u64)
                        == Some(2)
            })
    }));
}
