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
fn fixtures_validate_vcf_mini_reports_governed_manifest_assets() {
    let payload = run_cli_json(&["fixtures", "validate", "--corpus", "vcf-mini", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.vcf_corpus_fixture_validation.v1")
    );
    assert_eq!(payload.get("corpus_id").and_then(serde_json::Value::as_str), Some("vcf-mini"));
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/corpora/vcf-mini/manifest.toml")
    );
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("population_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("target_interval_count").and_then(serde_json::Value::as_u64), Some(4));

    let variant_sets = payload
        .get("variant_sets")
        .and_then(serde_json::Value::as_array)
        .expect("variant_sets array");
    assert_eq!(variant_sets.len(), 5);

    let phased = variant_sets
        .iter()
        .find(|row| row.get("variant_role").and_then(serde_json::Value::as_str) == Some("phased"))
        .expect("phased row");
    assert_eq!(
        phased
            .get("observed_sample_ids")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["sample_a", "sample_b", "sample_c", "sample_d"])
    );
    assert_eq!(
        phased.get("phased_genotypes_only").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let panel = variant_sets
        .iter()
        .find(|row| row.get("variant_role").and_then(serde_json::Value::as_str) == Some("panel"))
        .expect("panel row");
    assert_eq!(
        panel
            .get("observed_sample_ids")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["panel_ref_1", "panel_ref_2"])
    );
}
