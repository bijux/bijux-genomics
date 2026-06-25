#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn render_path(repo_root: &Path, label: &str) -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::Builder::new()
        .prefix(label)
        .tempdir_in(repo_root.join("runs/bench/hpc-dry-run"))
        .expect("temporary HPC dry-run directory");
    let report_path = temp_dir.path().join("result-collection-simulation.json");
    (temp_dir, report_path)
}

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

fn row_by_id<'a>(rows: &'a [serde_json::Value], record_id: &str) -> &'a serde_json::Value {
    rows.iter()
        .find(|row| row.get("record_id").and_then(serde_json::Value::as_str) == Some(record_id))
        .expect("governed row")
}

#[test]
fn bench_local_render_hpc_result_collection_simulation_distinguishes_governed_statuses() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, report_path) =
        render_path(&repo_root, "render-hpc-result-collection-simulation-");
    let report_arg = report_path.to_string_lossy().into_owned();

    let output = run_cli(&[
        "bench",
        "local",
        "render-hpc-result-collection-simulation",
        "--output",
        &report_arg,
    ]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let printed_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        printed_path.trim(),
        report_path
            .strip_prefix(&repo_root)
            .expect("report path relative to repo root")
            .to_string_lossy()
    );

    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).expect("read report"))
            .expect("parse report");
    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_hpc_result_collection_simulation.v1")
    );
    assert_eq!(
        report.get("output_path").and_then(serde_json::Value::as_str),
        Some(
            report_path
                .strip_prefix(&repo_root)
                .expect("report path relative to repo root")
                .to_string_lossy()
                .as_ref()
        )
    );
    assert_eq!(report.get("row_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(report.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("failed_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("missing_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("insufficient_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("unavailable_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        report
            .get("behavior")
            .and_then(|value| value.get("proven"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let rows = report.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(
        row_by_id(
            rows,
            "result:benchmark:vcf:vcf_production_regression:vcf.qc:vcf_cohort:bcftools"
        )
        .get("collection_status")
        .and_then(serde_json::Value::as_str),
        Some("complete")
    );
    assert_eq!(
        row_by_id(
            rows,
            "result:benchmark:vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools"
        )
        .get("collection_status")
        .and_then(serde_json::Value::as_str),
        Some("failed")
    );
    assert_eq!(
        row_by_id(
            rows,
            "result:benchmark:bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:contammix"
        )
        .get("collection_status")
        .and_then(serde_json::Value::as_str),
        Some("missing")
    );
    assert_eq!(
        row_by_id(rows, "result:pipeline:relatedness-segments-vcf:vcf.demography")
            .get("collection_status")
            .and_then(serde_json::Value::as_str),
        Some("insufficient")
    );
    assert_eq!(
        row_by_id(rows, "result:pipeline:relatedness-segments-vcf:vcf.demography")
            .get("insufficient_data_reason")
            .and_then(serde_json::Value::as_str),
        Some("not_enough_ibd_segments")
    );
    assert_eq!(
        row_by_id(rows, "unavailable:pipeline:relatedness-segments-vcf:vcf.ibd")
            .get("collection_status")
            .and_then(serde_json::Value::as_str),
        Some("unavailable")
    );
    assert!(
        row_by_id(rows, "unavailable:pipeline:relatedness-segments-vcf:vcf.ibd")
            .get("unavailable_reason")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|reason| reason.contains("governed local image")),
        "unavailable row must preserve the governed resolver reason"
    );
}
