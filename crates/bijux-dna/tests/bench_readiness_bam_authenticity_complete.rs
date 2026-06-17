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
        .args(["bench", "readiness", "render-bam-authenticity-complete", "--json"])
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
fn bench_readiness_bam_authenticity_complete_reports_governed_rows() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_authenticity_complete.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/stages/bam.authenticity.complete.json")
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("toolset_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    assert!(payload.get("expected_tool_ids").and_then(serde_json::Value::as_array).is_some_and(
        |tools| {
            tools.iter().any(|tool| tool.as_str() == Some("authenticct"))
                && tools.iter().any(|tool| tool.as_str() == Some("damageprofiler"))
                && tools.iter().any(|tool| tool.as_str() == Some("pmdtools"))
        }
    ));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.authenticity")
            && row.get("active_scope_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("command_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("output_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("parser_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("expected_result_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("report_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("schema_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("local_smoke_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("authenticity_report_contract_ready").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("summary_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("composition_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("normalized_metrics_ready").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("evidence_consumption_ready").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("complete")
    }));

    let authenticct_row = rows
        .iter()
        .find(|row| row.get("tool_id").and_then(serde_json::Value::as_str) == Some("authenticct"))
        .expect("authenticct row");
    assert_eq!(
        authenticct_row.get("summary_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.authenticity_advisory.v1")
    );
    assert_eq!(
        authenticct_row.get("composition_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.authenticity.composition.v1")
    );
    assert_eq!(
        authenticct_row
            .get("normalized_metrics_schema_version")
            .and_then(serde_json::Value::as_str),
        Some("bijux.bam.authenticity.local_smoke.metrics.v1")
    );
    assert_eq!(
        authenticct_row
            .get("local_smoke_report")
            .and_then(|value| value.get("status"))
            .and_then(serde_json::Value::as_str),
        Some("pass")
    );
    assert_eq!(
        authenticct_row
            .get("summary")
            .and_then(|value| value.get("status"))
            .and_then(serde_json::Value::as_str),
        Some("pass")
    );
    assert_eq!(
        authenticct_row
            .get("normalized_metrics")
            .and_then(|value| value.get("expected_status"))
            .and_then(serde_json::Value::as_str),
        Some("pass")
    );
}
