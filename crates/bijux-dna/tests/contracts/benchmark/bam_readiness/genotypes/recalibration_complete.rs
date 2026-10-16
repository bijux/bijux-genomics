#![allow(clippy::expect_used, clippy::too_many_lines)]

use crate::support;

fn run_cli_json() -> serde_json::Value {
    let sandbox = support::RepoSandbox::new_bam_genotype("bam-recalibration-complete-")
        .expect("repo sandbox");
    let runtime_root = sandbox.path().join("artifacts/test-runtime");
    std::fs::create_dir_all(&runtime_root).expect("private runtime root");
    let home = tempfile::Builder::new()
        .prefix("bam-genotype-runtime-")
        .tempdir_in(&runtime_root)
        .expect("private runtime directory");

    let output = sandbox
        .bijux_dna_command()
        .envs([
            ("HOME", home.path()),
            ("TMPDIR", home.path()),
            ("TMP", home.path()),
            ("TEMP", home.path()),
        ])
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-bam-recalibration-complete", "--json"])
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
fn bench_readiness_bam_recalibration_complete_reports_governed_row() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_recalibration_complete.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/bam/stages/bam.recalibration.complete.json")
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(13));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("toolset_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        payload.get("local_smoke_sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_recalibration_low_coverage")
    );
    assert_eq!(
        payload.get("expected_tool_ids").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::json!("gatk")])
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];

    assert_eq!(row.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.recalibration"));
    assert_eq!(row.get("tool_id").and_then(serde_json::Value::as_str), Some("gatk"));
    assert_eq!(
        row.get("local_smoke_summary_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.recalibration.v1")
    );
    assert_eq!(
        row.get("local_smoke_stage_metrics_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.recalibration.local_smoke.metrics.v1")
    );
    assert_eq!(
        row.get("local_smoke_known_sites_asset_ids").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::json!("human_like_recalibration_known_sites")])
    );
    assert_eq!(
        row.get("local_smoke_requested_mode").and_then(serde_json::Value::as_str),
        Some("standard")
    );
    assert_eq!(
        row.get("local_smoke_effective_mode").and_then(serde_json::Value::as_str),
        Some("skip")
    );
    assert_eq!(row.get("local_smoke_status").and_then(serde_json::Value::as_str), Some("skipped"));
    assert_eq!(
        row.get("local_smoke_reason").and_then(serde_json::Value::as_str),
        Some("coverage_below_gate")
    );
    assert_eq!(
        row.get("local_smoke_min_mean_coverage").and_then(serde_json::Value::as_f64),
        Some(0.2)
    );
    assert_eq!(
        row.get("local_smoke_min_breadth_1x").and_then(serde_json::Value::as_f64),
        Some(0.2)
    );
    assert_eq!(
        row.get("local_smoke_observed_mean_coverage").and_then(serde_json::Value::as_f64),
        Some(0.192)
    );
    assert_eq!(
        row.get("local_smoke_observed_breadth_1x").and_then(serde_json::Value::as_f64),
        Some(0.192)
    );
    assert_eq!(row.get("active_scope_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("command_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("output_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("parser_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("expected_result_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("report_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("schema_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("local_smoke_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("summary_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("stage_metrics_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("skip_behavior_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        row.get("known_sites_identity_ready").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        row.get("recalibration_report_ready").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(row.get("coverage_status").and_then(serde_json::Value::as_str), Some("complete"));
}
