#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_qc_ready_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-vcf-qc-ready"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/vcf/qc-ready.json");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read VCF QC readiness report");
    let report: serde_json::Value = serde_json::from_str(&payload).expect("parse report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_qc_ready.v1")
    );
    assert_eq!(report.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(report.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let rows = report.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 3);

    let bcftools = rows
        .iter()
        .find(|row| row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools"))
        .expect("bcftools row");
    assert_eq!(bcftools.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.qc"));
    assert!(
        bcftools
            .get("normalized_metrics_outputs")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|outputs| outputs.iter().any(|value| {
                value.as_str().is_some_and(|entry| entry.starts_with("qc_report="))
            }))
    );
    assert!(
        bcftools
            .get("report_metric_columns")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|columns| {
                columns.iter().any(|value| value.as_str() == Some("sample_missingness"))
                    && columns.iter().any(|value| value.as_str() == Some("hwe_summary"))
                    && columns.iter().any(|value| value.as_str() == Some("excluded_variants"))
            })
    );
    assert_eq!(
        bcftools.get("reason").and_then(serde_json::Value::as_str),
        Some(
            "active retained VCF QC caller `bcftools` keeps active scope, command, output, parser, expected-result, report, and normalized qc smoke proof for `vcf.qc`",
        )
    );
}
