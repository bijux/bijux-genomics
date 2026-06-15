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
fn bench_readiness_pair_readiness_reports_governed_gap_columns() {
    let payload = run_cli_json(&["bench", "readiness", "render-pair-readiness", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.pair_readiness.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/pair-readiness.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(122));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(118)
    );
    assert_eq!(
        payload.get("not_benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(73)
    );
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(49)
    );
    assert_eq!(
        payload
            .get("asset_status_counts")
            .and_then(|value| value.get("assigned"))
            .and_then(serde_json::Value::as_u64),
        Some(20)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 122);

    let taxonomy = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.screen_taxonomy")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("kraken2")
        })
        .expect("taxonomy readiness row");
    assert_eq!(
        taxonomy.get("benchmark_status").and_then(serde_json::Value::as_str),
        Some("benchmark_ready")
    );
    assert_eq!(
        taxonomy.get("adapter_status").and_then(serde_json::Value::as_str),
        Some("runnable")
    );
    assert_eq!(
        taxonomy.get("parser_status").and_then(serde_json::Value::as_str),
        Some("benchmark_normalized")
    );
    assert_eq!(
        taxonomy.get("corpus_status").and_then(serde_json::Value::as_str),
        Some("fixture:corpus-02-edna-mini")
    );
    assert_eq!(taxonomy.get("asset_status").and_then(serde_json::Value::as_str), Some("assigned"));

    let index_reference = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.index_reference")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2_build")
        })
        .expect("index-reference readiness row");
    assert_eq!(
        index_reference.get("benchmark_status").and_then(serde_json::Value::as_str),
        Some("benchmark_ready")
    );
    assert_eq!(
        index_reference.get("readiness_gap").and_then(serde_json::Value::as_str),
        Some("none")
    );
    assert_eq!(
        index_reference.get("corpus_status").and_then(serde_json::Value::as_str),
        Some("asset:reference-index-assets")
    );
    assert_eq!(
        index_reference.get("asset_status").and_then(serde_json::Value::as_str),
        Some("assigned")
    );
}
