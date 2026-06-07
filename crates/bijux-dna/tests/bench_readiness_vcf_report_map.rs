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
fn bench_readiness_vcf_report_map_reports_expected_result_sections() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-report-map", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_report_map.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf-report-map.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("section_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("summary_table_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(
        payload
            .get("section_counts")
            .and_then(|value| value.get("variant_calling"))
            .and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        payload
            .get("section_counts")
            .and_then(|value| value.get("quality_control"))
            .and_then(serde_json::Value::as_u64),
        Some(2)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 9);

    let call = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call"))
        .expect("vcf.call row");
    assert_eq!(call.get("tool_id").and_then(serde_json::Value::as_str), Some("bcftools"));
    assert_eq!(call.get("section_id").and_then(serde_json::Value::as_str), Some("variant_calling"));
    assert_eq!(
        call.get("summary_table").and_then(serde_json::Value::as_str),
        Some("variant_calling_metrics")
    );

    let damage_filter = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.damage_filter")
        })
        .expect("vcf.damage_filter row");
    assert_eq!(
        damage_filter.get("section_id").and_then(serde_json::Value::as_str),
        Some("damage_aware_filtering")
    );
    assert_eq!(
        damage_filter.get("summary_table").and_then(serde_json::Value::as_str),
        Some("damage_filtering_metrics")
    );

    let filter = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.filter"))
        .expect("vcf.filter row");
    assert_eq!(
        filter.get("section_id").and_then(serde_json::Value::as_str),
        Some("quality_control")
    );
    assert!(filter
        .get("metric_columns")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("filter_ids"))));

    let gl_propagation = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.gl_propagation")
        })
        .expect("vcf.gl_propagation row");
    assert_eq!(
        gl_propagation.get("section_id").and_then(serde_json::Value::as_str),
        Some("likelihood_postprocess")
    );
    assert!(gl_propagation
        .get("failure_columns")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("observed_error"))));

    let postprocess = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.postprocess")
        })
        .expect("vcf.postprocess row");
    assert_eq!(
        postprocess.get("section_id").and_then(serde_json::Value::as_str),
        Some("normalization")
    );
    assert_eq!(
        postprocess.get("summary_table").and_then(serde_json::Value::as_str),
        Some("normalization_metrics")
    );
}
