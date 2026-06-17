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
        .args(["bench", "readiness", "render-bam-overlap-correction-complete", "--json"])
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
fn bench_readiness_bam_overlap_correction_complete_reports_governed_metrics() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_overlap_correction_complete.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/stages/bam.overlap_correction.complete.json")
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(11));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 1);
    let row = rows.first().expect("overlap row");

    assert_eq!(
        row.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.overlap_correction")
    );
    assert_eq!(row.get("tool_id").and_then(serde_json::Value::as_str), Some("bamutil"));
    assert_eq!(
        row.get("local_smoke_proof_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/bam.overlap_correction/overlap_correction.json")
    );
    assert_eq!(
        row.get("overlap_corrected_bam_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/bam.overlap_correction/overlap_corrected.bam")
    );
    assert_eq!(
        row.get("overlap_corrected_bai_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/bam.overlap_correction/overlap_corrected.bam.bai")
    );
    assert_eq!(
        row.get("summary_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.overlap_correction.v1")
    );
    assert_eq!(
        row.get("normalized_metrics_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.overlap_correction.local_smoke.metrics.v1")
    );
    assert_eq!(row.get("summary_pair_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(row.get("summary_corrected_pairs").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        row.get("summary_corrected_overlap_bases").and_then(serde_json::Value::as_u64),
        Some(7)
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
        row.get("corrected_overlap_metrics_ready").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(row.get("coverage_status").and_then(serde_json::Value::as_str), Some("complete"));

    let normalized_metrics = row
        .get("normalized_metrics")
        .and_then(serde_json::Value::as_object)
        .expect("normalized metrics");
    assert_eq!(
        normalized_metrics.get("expected_pair_count").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(normalized_metrics.get("pair_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        normalized_metrics.get("expected_corrected_pairs").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        normalized_metrics.get("corrected_pairs").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        normalized_metrics
            .get("expected_corrected_overlap_bases")
            .and_then(serde_json::Value::as_u64),
        Some(7)
    );
    assert_eq!(
        normalized_metrics.get("corrected_overlap_bases").and_then(serde_json::Value::as_u64),
        Some(7)
    );
    assert_eq!(
        normalized_metrics.get("corrected_pair_delta").and_then(serde_json::Value::as_i64),
        Some(0)
    );
    assert_eq!(
        normalized_metrics.get("corrected_overlap_base_delta").and_then(serde_json::Value::as_i64),
        Some(0)
    );
    assert_eq!(
        normalized_metrics.get("expectation_matched").and_then(serde_json::Value::as_bool),
        Some(true)
    );
}
