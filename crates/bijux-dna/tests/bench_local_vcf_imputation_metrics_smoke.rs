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
fn bench_local_vcf_imputation_metrics_smoke_reports_quality_contract() {
    let payload = run_cli_json(&["bench", "local", "run-vcf-imputation-metrics-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_imputation_metrics_smoke.v1")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-imputation-metrics-smoke --tool-id beagle")
    );
    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.imputation_metrics")
    );
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("beagle"));
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        payload.get("input_fixture_id").and_then(serde_json::Value::as_str),
        Some("masked_truth_two_sample")
    );
    assert_eq!(
        payload.get("panel_id").and_then(serde_json::Value::as_str),
        Some("hsapiens_grch38_mini")
    );
    assert_eq!(
        payload.get("map_id").and_then(serde_json::Value::as_str),
        Some("hsapiens_grch38_chr_map")
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.imputation_metrics/beagle")
    );
    assert_eq!(
        payload.get("imputation_metrics_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.imputation_metrics/beagle/imputation_metrics.json")
    );
    assert_eq!(
        payload.get("source_imputation_qc_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.imputation_metrics/beagle/source_imputation_qc.json")
    );
    assert_eq!(
        payload.get("source_impute_smoke_metrics_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.imputation_metrics/beagle/source_impute_smoke_metrics.json")
    );
    assert_eq!(
        payload.get("source_imputation_manifest_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.imputation_metrics/beagle/source_imputation_manifest.json")
    );
    assert_eq!(
        payload.get("stage_result_manifest_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.imputation_metrics/beagle/stage-result.json")
    );
    assert_eq!(payload.get("exit_code").and_then(serde_json::Value::as_i64), Some(0));
    assert_eq!(payload.get("concordance").and_then(serde_json::Value::as_f64), Some(1.0));
    let mean_info_score = payload
        .get("mean_info_score")
        .and_then(serde_json::Value::as_f64)
        .expect("mean_info_score");
    assert!(
        (mean_info_score - 0.825).abs() < 1e-9,
        "unexpected mean_info_score: {mean_info_score}"
    );
    assert_eq!(payload.get("r2_available").and_then(serde_json::Value::as_bool), Some(true));
    let dosage_r2 =
        payload.get("dosage_r2").and_then(serde_json::Value::as_f64).expect("dosage_r2");
    assert!((dosage_r2 - 0.775).abs() < 1e-9, "unexpected dosage_r2: {dosage_r2}");
    assert_eq!(payload.get("low_confidence_sites").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("masked_truth_sites").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload
            .get("missing_quality_fields")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len()),
        Some(0)
    );
    assert_eq!(payload.get("status").and_then(serde_json::Value::as_str), Some("complete"));

    let availability = payload
        .get("quality_field_availability")
        .and_then(serde_json::Value::as_object)
        .expect("quality_field_availability object");
    assert_eq!(availability.get("concordance").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(availability.get("dosage_r2").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(availability.get("maf_strata").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        availability.get("masked_truth_sites").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let repo_root = support::repo_root().expect("repo root");
    let metrics_path =
        repo_root.join("target/local-smoke/vcf.imputation_metrics/beagle/imputation_metrics.json");
    let raw = std::fs::read_to_string(&metrics_path).expect("read metrics");
    let metrics: serde_json::Value = serde_json::from_str(&raw).expect("parse metrics");
    assert_eq!(
        metrics.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_imputation_metrics_smoke.v1")
    );
    assert_eq!(metrics.get("concordance").and_then(serde_json::Value::as_f64), Some(1.0));
    assert_eq!(metrics.get("r2_available").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(metrics.get("low_confidence_sites").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(metrics.get("masked_truth_sites").and_then(serde_json::Value::as_u64), Some(1));
}
