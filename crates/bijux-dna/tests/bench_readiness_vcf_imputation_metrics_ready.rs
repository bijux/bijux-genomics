#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_readiness_vcf_imputation_metrics_ready_reports_active_quality_gate() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-vcf-imputation-metrics-ready", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_imputation_metrics_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf/imputation-metrics-ready.json")
    );
    assert_eq!(payload.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let required_metrics = payload
        .get("required_metric_names")
        .and_then(serde_json::Value::as_array)
        .expect("required metrics");
    assert!(required_metrics.iter().any(|value| value.as_str() == Some("concordance")));
    assert!(required_metrics.iter().any(|value| value.as_str() == Some("dosage_r2")));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(
        row.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.imputation_metrics")
    );
    assert_eq!(row.get("tool_id").and_then(serde_json::Value::as_str), Some("beagle"));
    assert_eq!(row.get("coverage_status").and_then(serde_json::Value::as_str), Some("complete"));
    assert_eq!(row.get("smoke_concordance").and_then(serde_json::Value::as_f64), Some(1.0));
    assert_eq!(row.get("smoke_r2_available").and_then(serde_json::Value::as_bool), Some(true));
    let dosage_r2 =
        row.get("smoke_dosage_r2").and_then(serde_json::Value::as_f64).expect("smoke dosage_r2");
    assert!((dosage_r2 - 0.775).abs() < 1e-9, "unexpected smoke_dosage_r2: {dosage_r2}");

    let expected_metrics = row
        .get("expected_metrics")
        .and_then(serde_json::Value::as_array)
        .expect("expected metrics");
    assert!(expected_metrics.iter().any(|value| value.as_str() == Some("concordance")));
    assert!(expected_metrics.iter().any(|value| value.as_str() == Some("dosage_r2")));

    let report_metric_columns = row
        .get("report_metric_columns")
        .and_then(serde_json::Value::as_array)
        .expect("report metric columns");
    assert!(report_metric_columns.iter().any(|value| value.as_str() == Some("concordance")));
    assert!(report_metric_columns.iter().any(|value| value.as_str() == Some("dosage_r2")));
}
