#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

use crate::support;

fn run_cli_report(args: &[&str], report_path: &str) -> serde_json::Value {
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
        "structural imputation-metrics readiness failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&std::fs::read(repo_root.join(report_path)).expect("read report"))
        .expect("parse report as json")
}

#[test]
fn bench_readiness_vcf_imputation_metrics_ready_reports_all_supported_backends() {
    let payload = run_cli_report(
        &["bench", "readiness", "render-vcf-imputation-metrics-ready", "--json"],
        "artifacts/benchmark-smoke/readiness/vcf/imputation-metrics-ready.json",
    );

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_imputation_metrics_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/vcf/imputation-metrics-ready.json")
    );
    assert_eq!(payload.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(4));
    let complete_count = payload
        .get("complete_row_count")
        .and_then(serde_json::Value::as_u64)
        .expect("complete row count");
    let incomplete_count = payload
        .get("incomplete_row_count")
        .and_then(serde_json::Value::as_u64)
        .expect("incomplete row count");
    assert_eq!(complete_count + incomplete_count, 4);
    assert_eq!(complete_count, 4);
    assert_eq!(incomplete_count, 0);
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let required_metrics = payload
        .get("required_metric_names")
        .and_then(serde_json::Value::as_array)
        .expect("required metrics");
    assert!(required_metrics.iter().any(|value| value.as_str() == Some("concordance")));
    assert!(required_metrics.iter().any(|value| value.as_str() == Some("dosage_r2")));
    assert!(required_metrics.iter().any(|value| value.as_str() == Some("sample_count")));
    assert!(required_metrics.iter().any(|value| value.as_str() == Some("sample_ids")));
    assert!(required_metrics.iter().any(|value| value.as_str() == Some("variant_count")));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 4);
    let tool_ids = rows
        .iter()
        .filter_map(|row| row.get("tool_id").and_then(serde_json::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        tool_ids,
        std::collections::BTreeSet::from(["beagle", "glimpse", "impute5", "minimac4"])
    );
    let row = rows
        .iter()
        .find(|row| row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle"))
        .expect("beagle row");
    assert_eq!(
        row.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.imputation_metrics")
    );
    assert_eq!(row.get("tool_id").and_then(serde_json::Value::as_str), Some("beagle"));
    for field in
        ["command_ready", "output_ready", "parser_ready", "expected_result_ready", "report_ready"]
    {
        assert_eq!(row.get(field).and_then(serde_json::Value::as_bool), Some(true));
    }
    if row.get("smoke_ready").and_then(serde_json::Value::as_bool) == Some(true) {
        assert_eq!(
            row.get("coverage_status").and_then(serde_json::Value::as_str),
            Some("complete")
        );
        assert_eq!(row.get("smoke_concordance").and_then(serde_json::Value::as_f64), Some(1.0));
        assert_eq!(row.get("smoke_r2_available").and_then(serde_json::Value::as_bool), Some(true));
        let dosage_r2 = row
            .get("smoke_dosage_r2")
            .and_then(serde_json::Value::as_f64)
            .expect("smoke dosage_r2");
        assert!((dosage_r2 - 0.775).abs() < 1e-9, "unexpected smoke_dosage_r2: {dosage_r2}");
    } else {
        assert!(row.get("missing_surfaces").and_then(serde_json::Value::as_array).is_some_and(
            |values| values
                .iter()
                .any(|value| { value.as_str() == Some("local_vcf_imputation_metrics_smoke") })
        ));
    }

    let expected_metrics = row
        .get("expected_metrics")
        .and_then(serde_json::Value::as_array)
        .expect("expected metrics");
    assert!(expected_metrics.iter().any(|value| value.as_str() == Some("concordance")));
    assert!(expected_metrics.iter().any(|value| value.as_str() == Some("dosage_r2")));
    assert!(expected_metrics.iter().any(|value| value.as_str() == Some("sample_count")));
    assert!(expected_metrics.iter().any(|value| value.as_str() == Some("sample_ids")));
    assert!(expected_metrics.iter().any(|value| value.as_str() == Some("variant_count")));

    let report_metric_columns = row
        .get("report_metric_columns")
        .and_then(serde_json::Value::as_array)
        .expect("report metric columns");
    assert!(report_metric_columns.iter().any(|value| value.as_str() == Some("concordance")));
    assert!(report_metric_columns.iter().any(|value| value.as_str() == Some("dosage_r2")));
    assert!(report_metric_columns.iter().any(|value| value.as_str() == Some("sample_count")));
    assert!(report_metric_columns.iter().any(|value| value.as_str() == Some("sample_ids")));
    assert!(report_metric_columns.iter().any(|value| value.as_str() == Some("variant_count")));

    let impute5 = rows
        .iter()
        .find(|row| row.get("tool_id").and_then(serde_json::Value::as_str) == Some("impute5"))
        .expect("IMPUTE5 row");
    assert_eq!(impute5.get("tool_status").and_then(serde_json::Value::as_str), Some("supported"));
    for field in
        ["command_ready", "output_ready", "parser_ready", "expected_result_ready", "report_ready"]
    {
        assert_eq!(impute5.get(field).and_then(serde_json::Value::as_bool), Some(true));
    }
    assert_eq!(impute5.get("smoke_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        impute5.get("coverage_status").and_then(serde_json::Value::as_str),
        Some("complete")
    );
    assert!(impute5
        .get("missing_surfaces")
        .and_then(serde_json::Value::as_array)
        .is_some_and(Vec::is_empty));
}
