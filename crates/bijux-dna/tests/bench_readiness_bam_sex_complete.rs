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
        .args(["bench", "readiness", "render-bam-sex-complete", "--json"])
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
fn bench_readiness_bam_sex_complete_reports_governed_rows() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_sex_complete.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/stages/bam.sex.complete.json")
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(payload.get("local_smoke_case_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("toolset_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        payload.get("local_smoke_ready_sample_id").and_then(serde_json::Value::as_str),
        Some("adna_xy_autosome_coverage")
    );
    assert_eq!(
        payload.get("local_smoke_insufficient_sample_id").and_then(serde_json::Value::as_str),
        Some("adna_y_haplogroup_panel")
    );

    assert!(payload.get("expected_tool_ids").and_then(serde_json::Value::as_array).is_some_and(
        |tools| {
            tools.iter().any(|tool| tool.as_str() == Some("angsd"))
                && tools.iter().any(|tool| tool.as_str() == Some("rxy"))
                && tools.iter().any(|tool| tool.as_str() == Some("yleaf"))
        }
    ));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.sex")
            && row.get("active_scope_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("command_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("output_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("parser_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("expected_result_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("report_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("schema_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("local_smoke_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("ready_case_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("parser_contract_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("ready_summary_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("ready_stage_metrics_ready").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("insufficient_summary_ready").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("insufficient_stage_metrics_ready").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("insufficiency_behavior_ready").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("tool_specific_artifact_ready").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("complete")
    }));

    let rxy_row = rows
        .iter()
        .find(|row| row.get("tool_id").and_then(serde_json::Value::as_str) == Some("rxy"))
        .expect("rxy row");
    assert_eq!(
        rxy_row.get("ready_case_summary_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.sex_summary.v1")
    );
    assert_eq!(
        rxy_row.get("insufficient_case_summary_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.sex_summary.v1")
    );
    assert_eq!(
        rxy_row.get("ready_case_stage_metrics_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.sex.stage_metrics.v1")
    );
    assert_eq!(
        rxy_row
            .get("insufficient_case_stage_metrics_schema_version")
            .and_then(serde_json::Value::as_str),
        Some("bijux.bam.sex.stage_metrics.v1")
    );
    assert_eq!(rxy_row.get("ready_case_call").and_then(serde_json::Value::as_str), Some("male"));
    assert_eq!(
        rxy_row.get("insufficient_case_call").and_then(serde_json::Value::as_str),
        Some("insufficient")
    );
    assert_eq!(
        rxy_row.get("insufficient_case_insufficiency_reason").and_then(serde_json::Value::as_str),
        Some("insufficient_chromosomes")
    );
}
