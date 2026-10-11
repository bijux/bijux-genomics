#![cfg(feature = "bam_downstream")]
#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_readiness_parser_completeness_gate_reports_parser_complete_benchmark_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-parser-completeness-gate", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.parser_completeness_gate.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/gate-parser-complete.json")
    );
    assert_eq!(payload.get("passes_gate"), Some(&serde_json::Value::Bool(true)));
    let row_count =
        payload.get("row_count").and_then(serde_json::Value::as_u64).expect("row count");
    assert!(row_count > 0);
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    let gate_row_count =
        payload.get("gate_row_count").and_then(serde_json::Value::as_u64).expect("gate row count");
    assert_eq!(
        payload.get("gate_passed_row_count").and_then(serde_json::Value::as_u64),
        Some(gate_row_count)
    );
    assert_eq!(payload.get("gate_failed_row_count").and_then(serde_json::Value::as_u64), Some(0));
    let excluded_row_count = payload
        .get("excluded_row_count")
        .and_then(serde_json::Value::as_u64)
        .expect("excluded row count");
    assert!(excluded_row_count > 0);
    assert_eq!(gate_row_count + excluded_row_count, row_count);
    let domain_row_counts = payload["domain_row_counts"].as_object().expect("domain row counts");
    assert!(domain_row_counts
        .get("fastq")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count > 0));
    assert!(domain_row_counts
        .get("bam")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count > 0));
    assert_eq!(
        domain_row_counts.values().filter_map(serde_json::Value::as_u64).sum::<u64>(),
        row_count
    );
    let gate_domain_row_counts =
        payload["gate_domain_row_counts"].as_object().expect("gate domain row counts");
    assert_eq!(
        gate_domain_row_counts.values().filter_map(serde_json::Value::as_u64).sum::<u64>(),
        gate_row_count
    );
    assert_eq!(
        payload.get("excluded_readiness_gap_counts").and_then(serde_json::Value::as_object),
        Some(&serde_json::Map::new())
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), usize::try_from(row_count).expect("row count fits usize"));
    assert!(rows
        .iter()
        .all(|row| { row.get("gate_status").and_then(serde_json::Value::as_str) != Some("fail") }));

    let bwa_align = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bwa")
        })
        .expect("bam align bwa row");
    assert_eq!(
        bwa_align.get("gate_scope").and_then(serde_json::Value::as_str),
        Some("benchmark_reporting")
    );
    assert_eq!(bwa_align.get("gate_status").and_then(serde_json::Value::as_str), Some("pass"));
    assert_eq!(
        bwa_align.get("parser_status").and_then(serde_json::Value::as_str),
        Some("parser_fixture_validated")
    );

    let bowtie2_align = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2")
        })
        .expect("bam align bowtie2 row");
    assert_eq!(
        bowtie2_align.get("gate_scope").and_then(serde_json::Value::as_str),
        Some("benchmark_reporting")
    );
    assert_eq!(bowtie2_align.get("gate_status").and_then(serde_json::Value::as_str), Some("pass"));
    assert_eq!(
        bowtie2_align.get("parser_status").and_then(serde_json::Value::as_str),
        Some("parser_fixture_validated")
    );

    let excluded_fastq = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.index_reference")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2_build")
        })
        .expect("excluded fastq index-reference row");
    assert_eq!(
        excluded_fastq.get("gate_scope").and_then(serde_json::Value::as_str),
        Some("excluded")
    );
    assert_eq!(
        excluded_fastq.get("readiness_gap").and_then(serde_json::Value::as_str),
        Some("none")
    );
}
