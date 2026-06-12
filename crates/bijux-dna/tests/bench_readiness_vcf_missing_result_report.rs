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
fn bench_readiness_vcf_missing_result_report_tracks_one_removed_row() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-vcf-missing-result-report", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_missing_result_report.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf-missing-result-report-test.json")
    );
    assert_eq!(
        payload.get("fake_result_root").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf-missing-result-report-fixture")
    );
    assert_eq!(payload.get("expected_row_count").and_then(serde_json::Value::as_u64), Some(19));
    assert_eq!(
        payload.get("present_result_row_count").and_then(serde_json::Value::as_u64),
        Some(18)
    );
    assert_eq!(
        payload.get("missing_result_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("passes_behavior_test").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 19);
    let missing_rows = rows
        .iter()
        .filter(|row| {
            row.get("result_status").and_then(serde_json::Value::as_str) == Some("missing_result")
        })
        .collect::<Vec<_>>();
    assert_eq!(missing_rows.len(), 1);

    let removed_row = missing_rows[0];
    assert_eq!(removed_row.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.stats"));
    assert_eq!(removed_row.get("tool_id").and_then(serde_json::Value::as_str), Some("bcftools"));
    assert_eq!(
        removed_row.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        removed_row.get("asset_profile_id").and_then(serde_json::Value::as_str),
        Some("vcf_cohort")
    );
    assert_eq!(
        removed_row.get("report_section").and_then(serde_json::Value::as_str),
        Some("quality_control")
    );
    assert!(removed_row
        .get("expected_output_artifact_ids")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("stats_json"))));
    assert!(removed_row
        .get("observed_output_artifact_ids")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|items| items.is_empty()));
}
