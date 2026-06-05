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
fn bench_readiness_parser_failure_tests_report_structures_parser_errors() {
    let payload = run_cli_json(&["bench", "readiness", "render-parser-failure-tests", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.parser_failure_tests.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/parser-failure-tests.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(96));
    assert_eq!(payload.get("passed_row_count").and_then(serde_json::Value::as_u64), Some(96));
    assert_eq!(payload.get("failed_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload
            .get("domain_row_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(45)
    );
    assert_eq!(
        payload
            .get("domain_row_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(51)
    );
    assert_eq!(
        payload
            .get("expected_failure_class_counts")
            .and_then(|value| value.get("missing_raw_output"))
            .and_then(serde_json::Value::as_u64),
        Some(32)
    );
    assert_eq!(
        payload
            .get("expected_failure_class_counts")
            .and_then(|value| value.get("empty_raw_output"))
            .and_then(serde_json::Value::as_u64),
        Some(32)
    );
    assert_eq!(
        payload
            .get("expected_failure_class_counts")
            .and_then(|value| value.get("malformed_raw_output"))
            .and_then(serde_json::Value::as_u64),
        Some(32)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 96);
    assert!(rows.iter().all(|row| {
        row.get("passed") == Some(&serde_json::Value::Bool(true))
            && row
                .get("observed_error")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
    }));

    let fastq_row = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.detect_adapters")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastqc")
                && row.get("expected_failure_class").and_then(serde_json::Value::as_str)
                    == Some("malformed_raw_output")
        })
        .expect("fastq malformed parser row");
    assert!(fastq_row
        .get("observed_error")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value.contains("fastqc total sequences missing")));

    let bam_row = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.qc_pre")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
                && row.get("expected_failure_class").and_then(serde_json::Value::as_str)
                    == Some("malformed_raw_output")
        })
        .expect("bam malformed parser row");
    assert!(bam_row
        .get("observed_error")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value.contains("flagstat summary missing `in total` line")));
}
