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
fn bench_readiness_fastq_trim_stages_ready_reports_complete_trim_bindings() {
    let payload = run_cli_json(&["bench", "readiness", "render-fastq-trim-stages-ready", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_trim_stages_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq/trim-stages-ready.json")
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(18));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(18));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let expected_tool_ids_by_stage = payload
        .get("expected_tool_ids_by_stage")
        .and_then(serde_json::Value::as_object)
        .expect("expected tool ids by stage");
    assert_eq!(
        expected_tool_ids_by_stage
            .get("fastq.trim_terminal_damage")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(3)
    );

    let required_metric_fields_by_stage = payload
        .get("required_metric_fields_by_stage")
        .and_then(serde_json::Value::as_object)
        .expect("required metric fields by stage");
    assert!(required_metric_fields_by_stage
        .get("fastq.trim_polyg_tails")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|fields| {
            fields.iter().any(|field| field.as_str() == Some("trimmed_tail_count"))
                && fields.iter().any(|field| field.as_str() == Some("bases_trimmed_polyg"))
        }));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 18);
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

    let fastp_polyg = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_polyg_tails")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastp")
        })
        .expect("fastp trim-polyg row");
    assert_eq!(
        fastp_polyg.get("report_section_id").and_then(serde_json::Value::as_str),
        Some("read_cleanup")
    );
    assert_eq!(
        fastp_polyg.get("summary_table_id").and_then(serde_json::Value::as_str),
        Some("cleanup_retention")
    );
    assert_eq!(
        fastp_polyg.get("command_proof_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq-command-adapter-coverage.tsv")
    );
    assert_eq!(
        fastp_polyg.get("schema_proof_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/schemas/fastq-normalized-metrics.v1.json")
    );
    assert_eq!(
        fastp_polyg.get("normalized_metrics_output_id").and_then(serde_json::Value::as_str),
        Some("report_json")
    );
    assert!(fastp_polyg
        .get("schema_required_fields")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|fields| {
            fields.iter().any(|field| field.as_str() == Some("reads_retained"))
                && fields.iter().any(|field| field.as_str() == Some("trimmed_tail_count"))
                && fields.iter().any(|field| field.as_str() == Some("bases_trimmed_polyg"))
        }));
}
