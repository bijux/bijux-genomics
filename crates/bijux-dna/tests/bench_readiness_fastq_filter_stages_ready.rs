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
fn bench_readiness_fastq_filter_stages_ready_reports_complete_filter_bindings() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-fastq-filter-stages-ready", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_filter_stages_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq/filter-stages-ready.json")
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let required_output_ids_by_stage = payload
        .get("required_output_ids_by_stage")
        .and_then(serde_json::Value::as_object)
        .expect("required output ids by stage");
    assert!(required_output_ids_by_stage
        .get("fastq.filter_low_complexity")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| {
            outputs.iter().any(|output| output.as_str() == Some("filtered_fastq_r1"))
        }));

    let required_metric_fields_by_stage = payload
        .get("required_metric_fields_by_stage")
        .and_then(serde_json::Value::as_object)
        .expect("required metric fields by stage");
    assert!(required_metric_fields_by_stage
        .get("fastq.filter_reads")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|fields| {
            fields.iter().any(|field| field.as_str() == Some("reads_removed_by_entropy"))
                && fields.iter().any(|field| field.as_str() == Some("reads_removed_low_complexity"))
        }));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 6);
    assert!(rows.iter().all(|row| {
        row.get("active_scope_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("command_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("output_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("parser_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("expected_result_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("report_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("schema_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("complete")
    }));

    let low_complexity_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.filter_low_complexity")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bbduk")
        })
        .expect("bbduk filter_low_complexity row");
    assert_eq!(
        low_complexity_row.get("normalized_metrics_output_id").and_then(serde_json::Value::as_str),
        Some("filter_report_json")
    );
    assert_eq!(
        low_complexity_row
            .get("expected_normalized_metrics_output_id")
            .and_then(serde_json::Value::as_str),
        Some("filter_report_json")
    );
    assert!(low_complexity_row
        .get("schema_required_fields")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|fields| {
            fields.iter().any(|field| field.as_str() == Some("reads_retained"))
                && fields.iter().any(|field| field.as_str() == Some("reads_removed"))
                && fields.iter().any(|field| field.as_str() == Some("reads_removed_low_complexity"))
        }));
}
