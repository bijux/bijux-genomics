#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_imputation_family_adapter_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-vcf-imputation-family-adapter"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report_path = repo_root.join("benchmarks/readiness/adapters/imputation-family.vcf.json");
    assert!(report_path.is_file(), "VCF imputation-family adapter JSON must exist");

    let payload = serde_json::from_slice::<serde_json::Value>(
        &std::fs::read(&report_path).expect("read VCF imputation-family adapter JSON"),
    )
    .expect("parse VCF imputation-family adapter JSON");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_imputation_family_adapter.v1")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(8));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let benchmark_ready_rows = rows
        .iter()
        .filter(|row| {
            row.get("benchmark_status").and_then(serde_json::Value::as_str)
                == Some("benchmark_ready")
        })
        .count();
    assert_eq!(benchmark_ready_rows, 2, "beagle must remain the only benchmark-ready backend");

    let minimac_row = rows
        .iter()
        .find(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("minimac4")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.imputation")
        })
        .expect("minimac4 imputation row");
    let panel_m3vcf = minimac_row
        .get("panel_m3vcf_path")
        .and_then(serde_json::Value::as_str)
        .expect("minimac4 panel_m3vcf_path");
    assert!(repo_root.join(panel_m3vcf).is_file(), "materialized m3vcf panel must exist");

    let beagle_impute = rows
        .iter()
        .find(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.impute")
        })
        .expect("beagle impute row");
    assert!(
        beagle_impute.get("declared_outputs").and_then(serde_json::Value::as_array).is_some_and(
            |items| {
                items.iter().any(|item| {
                    item.get("artifact_id").and_then(serde_json::Value::as_str)
                        == Some("panel_mismatch_diagnostics_json")
                }) && items.iter().any(|item| {
                    item.get("artifact_id").and_then(serde_json::Value::as_str)
                        == Some("imputation_qc_tsv")
                })
            }
        ),
        "vcf.impute rows must retain heavy-stage diagnostics and TSV quality outputs"
    );
}
