#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

use crate::support;

#[test]
fn bench_readiness_vcf_call_gl_ready_writes_governed_json_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(support::cli_binary_path())
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-vcf-call-gl-ready"])
        .output()
        .expect("run cli");

    assert!(!output.status.success(), "unadmitted ANGSD evidence must keep the gate open");

    let payload = std::fs::read_to_string(
        repo_root.join("artifacts/benchmark-smoke/readiness/vcf/call-gl-ready.json"),
    )
    .expect("read VCF call_gl readiness report");
    let report: serde_json::Value = serde_json::from_str(&payload).expect("parse report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_call_gl_ready.v1")
    );
    assert_eq!(report.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(2));
    let complete_row_count = report
        .get("complete_row_count")
        .and_then(serde_json::Value::as_u64)
        .expect("complete row count");
    let incomplete_row_count = report
        .get("incomplete_row_count")
        .and_then(serde_json::Value::as_u64)
        .expect("incomplete row count");
    assert_eq!(complete_row_count + incomplete_row_count, 2);
    assert!(incomplete_row_count >= 1);
    assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(false));

    let row = report
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .and_then(|rows| {
            rows.iter().find(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            })
        })
        .expect("bcftools row");
    assert_eq!(row.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.call_gl"));
    assert_eq!(row.get("tool_id").and_then(serde_json::Value::as_str), Some("bcftools"));
    assert_eq!(
        row.get("expected_outputs").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::Value::String("gl_sites_vcf".to_string())])
    );
    assert!(row.get("report_metric_columns").and_then(serde_json::Value::as_array).is_some_and(
        |columns| {
            columns.iter().any(|value| value.as_str() == Some("likelihood_field"))
                && columns.iter().any(|value| value.as_str() == Some("sites_with_likelihoods"))
                && columns.iter().any(|value| value.as_str() == Some("sample_count"))
        }
    ));
    assert!(
        row.get("raw_outputs")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|outputs| outputs.iter().any(|value| {
                value.as_str()
                    == Some(
                        "gl_sites_vcf=artifacts/benchmark-smoke/readiness/adapters/bcftools/vcf.call_gl/gl_sites_vcf.vcf.gz",
                    )
            }))
    );
    assert!(
        row.get("index_outputs")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|outputs| outputs.iter().any(|value| {
                value.as_str()
                    == Some(
                        "gl_sites_vcf_tbi=artifacts/benchmark-smoke/readiness/adapters/bcftools/vcf.call_gl/gl_sites_vcf.vcf.gz.tbi",
                    )
            }))
    );
    let reason = row.get("reason").and_then(serde_json::Value::as_str).expect("bcftools reason");
    if row.get("smoke_ready").and_then(serde_json::Value::as_bool) == Some(true) {
        assert!(reason.contains("likelihood-bearing smoke proof"));
    } else {
        assert!(reason.contains("missing: local_vcf_call_gl_smoke"));
        assert_eq!(
            row.get("missing_surfaces").and_then(serde_json::Value::as_array),
            Some(&vec![serde_json::Value::String("local_vcf_call_gl_smoke".to_string())])
        );
    }
}
