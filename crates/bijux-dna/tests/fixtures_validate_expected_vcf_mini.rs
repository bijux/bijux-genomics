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
fn fixtures_validate_expected_vcf_mini_reports_governed_truth_bundle() {
    let payload = run_cli_json(&["fixtures", "validate-expected", "--corpus", "vcf-mini", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.vcf_expected_truth_validation.v1")
    );
    assert_eq!(payload.get("corpus_id").and_then(serde_json::Value::as_str), Some("vcf-mini"));
    assert_eq!(
        payload.get("expected_dir").and_then(serde_json::Value::as_str),
        Some("tests/fixtures/corpora/vcf-mini/expected")
    );
    assert_eq!(payload.get("truth_file_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("cohort_sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("pair_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));

    let checked_truth_files = payload
        .get("checked_truth_files")
        .and_then(serde_json::Value::as_array)
        .expect("checked_truth_files array")
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(checked_truth_files.len(), 8);
    assert!(checked_truth_files.contains(&"tests/fixtures/corpora/vcf-mini/expected/variant_counts.json"));
    assert!(checked_truth_files.contains(&"tests/fixtures/corpora/vcf-mini/expected/pca_expected.json"));
    assert!(checked_truth_files.contains(&"tests/fixtures/corpora/vcf-mini/expected/ibd_expected.json"));
}
