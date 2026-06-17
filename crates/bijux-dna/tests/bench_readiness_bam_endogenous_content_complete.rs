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
        .args(["bench", "readiness", "render-bam-endogenous-content-complete", "--json"])
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
fn bench_readiness_bam_endogenous_content_complete_reports_governed_metrics() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_endogenous_content_complete.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/stages/bam.endogenous_content.complete.json")
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(11));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 1);
    let row = rows.first().expect("endogenous row");

    assert_eq!(
        row.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.endogenous_content")
    );
    assert_eq!(row.get("tool_id").and_then(serde_json::Value::as_str), Some("samtools"));
    assert_eq!(
        row.get("local_smoke_proof_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/bam.endogenous_content/endogenous_content.json")
    );
    assert_eq!(
        row.get("endogenous_report_path").and_then(serde_json::Value::as_str),
        Some(
            "runs/bench/local-smoke/bam.endogenous_content/human_like_endogenous_partial_mapping/samtools/endogenous.content.json"
        )
    );
    assert_eq!(
        row.get("endogenous_summary_path").and_then(serde_json::Value::as_str),
        Some(
            "runs/bench/local-smoke/bam.endogenous_content/human_like_endogenous_partial_mapping/samtools/endogenous.summary.json"
        )
    );
    assert_eq!(
        row.get("stage_metrics_path").and_then(serde_json::Value::as_str),
        Some(
            "runs/bench/local-smoke/bam.endogenous_content/human_like_endogenous_partial_mapping/samtools/stage.metrics.json"
        )
    );
    assert_eq!(
        row.get("summary_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.endogenous_content.v1")
    );
    assert_eq!(
        row.get("normalized_metrics_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.endogenous_content.local_smoke.metrics.v1")
    );
    assert_eq!(row.get("summary_mapped_reads").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(row.get("summary_endogenous_reads").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(row.get("summary_total_reads").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(
        row.get("summary_endogenous_fraction").and_then(serde_json::Value::as_f64),
        Some(0.6)
    );
    assert_eq!(
        row.get("summary_host_reference_scope").and_then(serde_json::Value::as_str),
        Some("human_host")
    );
    assert_eq!(
        row.get("contaminant_reads"),
        Some(&serde_json::Value::Null),
        "contaminant reads remain explicit as unavailable in the governed contract"
    );
    assert_eq!(
        row.get("local_smoke_expectation_matched").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(row.get("summary_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        row.get("normalized_metrics_ready").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        row.get("endogenous_metric_consistency_ready").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(row.get("coverage_status").and_then(serde_json::Value::as_str), Some("complete"));

    let normalized_metrics = row
        .get("normalized_metrics")
        .and_then(serde_json::Value::as_object)
        .expect("normalized metrics");
    assert_eq!(
        normalized_metrics.get("expected_mapped_reads").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(normalized_metrics.get("mapped_reads").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(
        normalized_metrics.get("expected_endogenous_reads").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(
        normalized_metrics.get("endogenous_reads").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(
        normalized_metrics.get("expected_total_reads").and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert_eq!(normalized_metrics.get("total_reads").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(
        normalized_metrics.get("expected_endogenous_fraction").and_then(serde_json::Value::as_f64),
        Some(0.6)
    );
    assert_eq!(
        normalized_metrics.get("endogenous_fraction").and_then(serde_json::Value::as_f64),
        Some(0.6)
    );
    assert_eq!(
        normalized_metrics.get("endogenous_fraction_delta").and_then(serde_json::Value::as_f64),
        Some(0.0)
    );
    assert_eq!(
        normalized_metrics.get("expectation_matched").and_then(serde_json::Value::as_bool),
        Some(true)
    );
}
