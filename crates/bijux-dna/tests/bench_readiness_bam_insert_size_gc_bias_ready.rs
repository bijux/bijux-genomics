#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json() -> serde_json::Value {
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
        .args(["bench", "readiness", "render-bam-insert-size-gc-bias-ready", "--json"])
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
fn bench_readiness_bam_insert_size_gc_bias_ready_reports_complete_governed_rows() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_insert_size_gc_bias_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/insert-size-gc-bias-ready.json")
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| {
        row.get("active_scope_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("command_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("output_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("parser_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("expected_result_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("report_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("schema_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("local_smoke_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("complete")
    }));

    let gc_bias_row = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.gc_bias"))
        .expect("bam.gc_bias row");
    assert_eq!(
        gc_bias_row
            .get("expected_normalized_metrics_output_id")
            .and_then(serde_json::Value::as_str),
        Some("gc_bias_report")
    );
    assert_eq!(
        gc_bias_row.get("report_section_id").and_then(serde_json::Value::as_str),
        Some("coverage_quality")
    );
    assert!(gc_bias_row
        .get("required_local_smoke_fields")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|fields| {
            fields.iter().any(|field| field.as_str() == Some("gc_bin"))
                && fields.iter().any(|field| field.as_str() == Some("gc_bias_score"))
        }));

    let insert_size_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.insert_size")
        })
        .expect("bam.insert_size row");
    assert_eq!(
        insert_size_row
            .get("expected_normalized_metrics_output_id")
            .and_then(serde_json::Value::as_str),
        Some("insert_size_report")
    );
    assert!(insert_size_row
        .get("local_smoke_artifact_paths")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|paths| {
            paths.iter().any(|path| {
                path.as_str() == Some("runs/bench/local-smoke/bam.insert_size/insert_size.json")
            })
        }));
}
