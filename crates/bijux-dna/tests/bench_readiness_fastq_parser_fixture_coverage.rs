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
fn bench_readiness_fastq_parser_fixture_coverage_reports_governed_rows() {
    let payload = run_cli_json(&[
        "bench",
        "readiness",
        "render-fastq-parser-fixture-coverage",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_parser_fixture_coverage.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq/fastq-parser-fixture-coverage.tsv")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(26));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(41));
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(69));
    assert_eq!(payload.get("covered_row_count").and_then(serde_json::Value::as_u64), Some(69));
    assert_eq!(payload.get("missing_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("parser_fixture_coverage_percent").and_then(serde_json::Value::as_f64),
        Some(100.0)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 69);
    assert!(rows.iter().all(|row| {
        row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("covered")
            && row.get("parser_fixture_reference_kind").and_then(serde_json::Value::as_str)
                == Some("fixture_case")
            && row.get("parser_fixture_reference").and_then(serde_json::Value::as_str).is_some()
            && row.get("parser_fixture_parser_id").and_then(serde_json::Value::as_str).is_some()
            && row.get("parser_fixture_schema_id").and_then(serde_json::Value::as_str).is_some()
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("trimmomatic")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_reads")
            && row.get("parser_fixture_parser_id").and_then(serde_json::Value::as_str)
                == Some("parse_trim_reads_report")
            && row.get("parser_fixture_reference").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_reads.report_json")
            && row.get("parser_fixture_canonical_tool_id").and_then(serde_json::Value::as_str)
                == Some("fastp")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.detect_duplicates_premerge")
            && row.get("parser_fixture_parser_id").and_then(serde_json::Value::as_str)
                == Some("parse_detect_duplicates_premerge_report")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("kraken2")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.screen_taxonomy")
            && row.get("parser_fixture_schema_id").and_then(serde_json::Value::as_str)
                == Some("bijux.fastq.screen_taxonomy.report.v2")
    }));
}
