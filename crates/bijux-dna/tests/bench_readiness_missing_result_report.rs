#![cfg(feature = "bam_downstream")]
#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
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
fn bench_readiness_missing_result_report_tracks_governed_missing_row() {
    let payload = run_cli_json(&["bench", "readiness", "render-missing-result-report", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.missing_result_report.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/missing-result-report-test.json")
    );
    assert_eq!(
        payload.get("fake_result_root").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/missing-result-report-fixture")
    );
    assert_eq!(payload.get("expected_row_count").and_then(serde_json::Value::as_u64), Some(115));
    assert_eq!(
        payload.get("present_result_row_count").and_then(serde_json::Value::as_u64),
        Some(114)
    );
    assert_eq!(
        payload.get("missing_result_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(payload.get("passes_behavior_test"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(66)
    );
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(49)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 115);

    let removed_result_row_id = payload
        .get("removed_result_row_id")
        .and_then(serde_json::Value::as_str)
        .expect("removed_result_row_id");
    let removed_row = rows
        .iter()
        .find(|row| {
            row.get("result_row_id").and_then(serde_json::Value::as_str)
                == Some(removed_result_row_id)
        })
        .expect("removed result row");
    assert_eq!(removed_row.get("domain").and_then(serde_json::Value::as_str), Some("fastq"));
    assert_eq!(
        removed_row.get("stage_id").and_then(serde_json::Value::as_str),
        Some("fastq.screen_taxonomy")
    );
    assert_eq!(removed_row.get("tool_id").and_then(serde_json::Value::as_str), Some("kraken2"));
    assert_eq!(
        removed_row.get("result_status").and_then(serde_json::Value::as_str),
        Some("missing_result")
    );
    assert_eq!(
        removed_row
            .get("observed_output_artifact_ids")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(0)
    );

    let bam_present = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.kinship")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("king")
        })
        .expect("bam kinship result row");
    assert_eq!(
        bam_present.get("result_status").and_then(serde_json::Value::as_str),
        Some("present")
    );
    assert!(
        bam_present
            .get("observed_output_artifact_ids")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|outputs| outputs.iter().any(|value| value.as_str() == Some("summary"))),
        "present BAM rows must retain their observed output ids"
    );
}
