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
fn bench_readiness_bam_parser_coverage_reports_governed_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-bam-parser-coverage", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_parser_coverage.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam-parser-coverage.tsv")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(25));
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(
        payload.get("parser_covered_row_count").and_then(serde_json::Value::as_u64),
        Some(49)
    );
    assert_eq!(
        payload.get("parser_missing_row_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("excluded_non_benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload
            .get("excluded_readiness_gap_counts")
            .and_then(serde_json::Value::as_object)
            .map(serde_json::Map::len),
        Some(0)
    );
    assert_eq!(
        payload.get("parser_coverage_percent").and_then(serde_json::Value::as_f64),
        Some(100.0)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 49);
    assert!(rows.iter().all(|row| {
        row.get("parser_coverage").and_then(serde_json::Value::as_str) == Some("covered")
            && row.get("parser_status").and_then(serde_json::Value::as_str)
                == Some("parser_fixture_validated")
            && row
                .get("corpus_status")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value.starts_with("fixture:"))
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.validate")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("mapdamage2")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.damage")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("angsd")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.genotyping")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bwa")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align")
            && row.get("corpus_status").and_then(serde_json::Value::as_str)
                == Some("fixture:corpus-01-mini")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align")
            && row.get("corpus_status").and_then(serde_json::Value::as_str)
                == Some("fixture:corpus-01-mini")
    }));
}
