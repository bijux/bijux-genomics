#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_judge_taxonomy_output_json_reports_governed_corpus_02_match() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    let output_path = home.path().join("taxonomy-judgment.json");

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
            "judge-taxonomy-output",
            "--manifest",
            "benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/manifest.toml",
            "--report",
            "mock_community_sample_a=benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/observed_taxonomy/mock_community_sample_a.classification_report.json",
            "--report",
            "mock_community_sample_b=benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/observed_taxonomy/mock_community_sample_b.classification_report.json",
            "--output",
            output_path.to_str().expect("utf8 output path"),
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
        Some("bijux.bench.local_taxonomy_output_judgment.v1")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/manifest.toml")
    );
    assert_eq!(
        payload.get("expected_taxa_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/expected_taxa.tsv")
    );
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("expectation_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(
        payload.get("matched_expectation_count").and_then(serde_json::Value::as_u64),
        Some(6)
    );
    assert_eq!(
        payload.get("mismatched_expectation_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert!(payload.get("valid").and_then(serde_json::Value::as_bool) == Some(true));
    assert!(payload.get("samples").and_then(serde_json::Value::as_array).is_some_and(|samples| {
        samples.len() == 2
            && samples.iter().all(|sample| {
                sample.get("valid").and_then(serde_json::Value::as_bool) == Some(true)
                    && sample
                        .get("mismatched_expectation_count")
                        .and_then(serde_json::Value::as_u64)
                        == Some(0)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("mock_community_sample_b")
                    && sample
                        .get("observed_taxa")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|taxa| {
                            taxa.len() == 1
                                && taxa[0].get("name").and_then(serde_json::Value::as_str)
                                    == Some("Halobacterium salinarum")
                        })
            })
    }));

    let written = std::fs::read_to_string(&output_path).expect("read written report");
    let written_payload: serde_json::Value =
        serde_json::from_str(&written).expect("parse written report");
    assert_eq!(written_payload, payload);
}
