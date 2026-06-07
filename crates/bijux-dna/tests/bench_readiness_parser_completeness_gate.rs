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
fn bench_readiness_parser_completeness_gate_reports_parser_complete_benchmark_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-parser-completeness-gate", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.parser_completeness_gate.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/gate-parser-complete.json")
    );
    assert_eq!(payload.get("passes_gate"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(123));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(112)
    );
    assert_eq!(payload.get("gate_row_count").and_then(serde_json::Value::as_u64), Some(112));
    assert_eq!(payload.get("gate_passed_row_count").and_then(serde_json::Value::as_u64), Some(112));
    assert_eq!(payload.get("gate_failed_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("excluded_row_count").and_then(serde_json::Value::as_u64), Some(11));
    assert_eq!(
        payload
            .get("domain_row_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(74)
    );
    assert_eq!(
        payload
            .get("domain_row_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(49)
    );
    assert_eq!(
        payload
            .get("gate_domain_row_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(63)
    );
    assert_eq!(
        payload
            .get("gate_domain_row_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(49)
    );
    assert_eq!(
        payload
            .get("excluded_readiness_gap_counts")
            .and_then(|value| value.get("corpus"))
            .and_then(serde_json::Value::as_u64),
        Some(6)
    );
    assert_eq!(
        payload
            .get("excluded_readiness_gap_counts")
            .and_then(|value| value.get("support"))
            .and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert!(
        payload
            .get("excluded_readiness_gap_counts")
            .and_then(|value| value.get("parser"))
            .is_none(),
        "parser completeness gate must not retain excluded parser blockers once benchmark rows are fixture-validated"
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 123);
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
        Some("corpus")
    );
}
