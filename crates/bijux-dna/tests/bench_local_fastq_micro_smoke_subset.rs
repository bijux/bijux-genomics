#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
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
        .args(args)
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
fn bench_local_fastq_micro_smoke_subset_reports_one_governed_row_per_family() {
    let payload = run_cli_json(&["bench", "local", "run-fastq-micro-smoke-subset", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_fastq_micro_smoke_subset.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/micro/fastq/MICRO_FASTQ_SUMMARY.json")
    );
    assert_eq!(payload.get("family_count").and_then(serde_json::Value::as_u64), Some(13));
    assert_eq!(payload.get("local_smoke_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("container_needed_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("unavailable_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("passes_behavior_test").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 13);

    let family_ids = rows
        .iter()
        .filter_map(|row| row.get("family_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        family_ids,
        BTreeSet::from([
            "fastq.adapter_detection",
            "fastq.amplicon",
            "fastq.complexity_correction",
            "fastq.depletion",
            "fastq.duplicate_handling",
            "fastq.filtering",
            "fastq.index_reference",
            "fastq.merge_umi",
            "fastq.qc_reporting",
            "fastq.read_profiling",
            "fastq.taxonomy",
            "fastq.trimming",
            "fastq.validate_reads",
        ])
    );

    let validate_reads = rows
        .iter()
        .find(|row| {
            row.get("family_id").and_then(serde_json::Value::as_str) == Some("fastq.validate_reads")
        })
        .expect("fastq.validate_reads family row");
    assert_eq!(
        validate_reads.get("execution_status").and_then(serde_json::Value::as_str),
        Some("local_smoke")
    );
    assert_eq!(
        validate_reads.get("representative_stage_id").and_then(serde_json::Value::as_str),
        Some("fastq.validate_reads")
    );
    assert_eq!(
        validate_reads.get("representative_tool_id").and_then(serde_json::Value::as_str),
        Some("fastqvalidator")
    );
    assert_eq!(
        validate_reads.get("evidence_format").and_then(serde_json::Value::as_str),
        Some("json")
    );
    assert_eq!(
        validate_reads.get("parsed_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.fastq.validate.local_smoke.report.v1")
    );

    let depletion = rows
        .iter()
        .find(|row| {
            row.get("family_id").and_then(serde_json::Value::as_str) == Some("fastq.depletion")
        })
        .expect("fastq.depletion family row");
    assert_eq!(
        depletion.get("execution_status").and_then(serde_json::Value::as_str),
        Some("container_needed")
    );
    assert_eq!(
        depletion.get("representative_stage_id").and_then(serde_json::Value::as_str),
        Some("fastq.deplete_host")
    );

    let amplicon = rows
        .iter()
        .find(|row| {
            row.get("family_id").and_then(serde_json::Value::as_str) == Some("fastq.amplicon")
        })
        .expect("fastq.amplicon family row");
    assert_eq!(
        amplicon.get("execution_status").and_then(serde_json::Value::as_str),
        Some("local_smoke")
    );
    assert_eq!(
        amplicon.get("representative_stage_id").and_then(serde_json::Value::as_str),
        Some("fastq.normalize_primers")
    );

    let read_profiling = rows
        .iter()
        .find(|row| {
            row.get("family_id").and_then(serde_json::Value::as_str)
                == Some("fastq.read_profiling")
        })
        .expect("fastq.read_profiling family row");
    assert_eq!(
        read_profiling.get("representative_stage_id").and_then(serde_json::Value::as_str),
        Some("fastq.profile_overrepresented_sequences")
    );

    let reporting = rows
        .iter()
        .find(|row| {
            row.get("family_id").and_then(serde_json::Value::as_str) == Some("fastq.qc_reporting")
        })
        .expect("fastq.qc_reporting family row");
    assert_eq!(
        reporting.get("execution_status").and_then(serde_json::Value::as_str),
        Some("local_smoke")
    );
    assert_eq!(
        reporting.get("representative_stage_id").and_then(serde_json::Value::as_str),
        Some("fastq.report_qc")
    );
    assert_eq!(
        reporting.get("parsed_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.fastq.report_qc.report.v2")
    );
}
