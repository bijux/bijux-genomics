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
fn bench_readiness_fastq_duplicate_stages_ready_reports_complete_duplicate_bindings() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-fastq-duplicate-stages-ready", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_duplicate_stages_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq/duplicate-stages-ready.json")
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let report_targets_by_stage = payload
        .get("report_targets_by_stage")
        .and_then(serde_json::Value::as_object)
        .expect("report targets by stage");
    assert!(report_targets_by_stage
        .get("fastq.detect_duplicates_premerge")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|targets| {
            targets.iter().any(|target| target.as_str() == Some("quality_profiling"))
                && targets.iter().any(|target| target.as_str() == Some("premerge_complexity"))
        }));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 3);
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

    let detect_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.detect_duplicates_premerge")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna")
        })
        .expect("detect_duplicates_premerge row");
    assert_eq!(
        detect_row.get("normalized_metrics_output_id").and_then(serde_json::Value::as_str),
        Some("duplicate_signal_report")
    );
    assert!(detect_row
        .get("schema_required_fields")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|fields| {
            fields.iter().any(|field| field.as_str() == Some("duplicate_count"))
                && fields.iter().any(|field| field.as_str() == Some("duplicate_fraction"))
                && fields.iter().any(|field| field.as_str() == Some("inspected_pair_count"))
        }));

    let remove_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.remove_duplicates")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("clumpify")
        })
        .expect("remove_duplicates clumpify row");
    assert_eq!(
        remove_row.get("normalized_metrics_output_id").and_then(serde_json::Value::as_str),
        Some("report_json")
    );
    assert!(remove_row
        .get("required_metric_fields")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|fields| {
            fields.iter().any(|field| field.as_str() == Some("unique_reads"))
                && fields.iter().any(|field| field.as_str() == Some("output_reads"))
                && fields.iter().any(|field| field.as_str() == Some("duplicates_removed"))
        }));
}
