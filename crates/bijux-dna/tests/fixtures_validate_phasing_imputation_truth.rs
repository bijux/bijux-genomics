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
fn fixtures_validate_phasing_imputation_truth_reports_governed_bundle() {
    let payload =
        run_cli_json(&["fixtures", "validate", "--corpus", "phasing-imputation-truth", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.phasing_imputation_truth.validation.v1")
    );
    assert_eq!(
        payload.get("fixture_id").and_then(serde_json::Value::as_str),
        Some("phasing-imputation-truth")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/phasing-imputation-truth/manifest.toml")
    );
    assert_eq!(
        payload.get("expected_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/phasing-imputation-truth/expected.json")
    );
    assert_eq!(payload.get("validated_case_count").and_then(serde_json::Value::as_u64), Some(11));
    assert_eq!(
        payload.get("validated_stage_ids").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::json!("vcf.imputation_metrics"),
            serde_json::json!("vcf.impute"),
            serde_json::json!("vcf.phasing"),
        ])
    );
    assert_eq!(
        payload.get("validated_tool_ids").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::json!("beagle"),
            serde_json::json!("eagle"),
            serde_json::json!("glimpse"),
            serde_json::json!("impute5"),
            serde_json::json!("minimac4"),
            serde_json::json!("shapeit5"),
        ])
    );
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));
}
