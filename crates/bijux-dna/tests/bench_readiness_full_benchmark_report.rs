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
fn bench_readiness_full_benchmark_report_tracks_governed_report_sections() {
    let payload = run_cli_json(&["bench", "readiness", "render-full-benchmark-report", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.full_benchmark_report.v1")
    );
    assert_eq!(
        payload.get("markdown_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_REPORT.md")
    );
    assert_eq!(
        payload.get("json_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_REPORT.json")
    );
    let row_count = support::json_u64(&payload, "row_count").expect("row_count");
    let expected_result_row_count = support::json_u64(&payload, "expected_result_row_count")
        .expect("expected_result_row_count");
    assert_eq!(
        payload.get("explicit_unsupported_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    let present_row_count =
        support::json_u64(&payload, "present_row_count").expect("present_row_count");
    let missing_result_row_count =
        support::json_u64(&payload, "missing_result_row_count").expect("missing_result_row_count");
    let unsupported_pair_row_count = support::json_u64(&payload, "unsupported_pair_row_count")
        .expect("unsupported_pair_row_count");
    assert_eq!(missing_result_row_count, 3);
    assert_eq!(unsupported_pair_row_count, 1);
    assert_eq!(present_row_count + missing_result_row_count, expected_result_row_count);
    assert_eq!(row_count, expected_result_row_count + unsupported_pair_row_count);
    assert_eq!(
        payload.get("failure_row_count").and_then(serde_json::Value::as_u64),
        Some(expected_result_row_count)
    );
    assert_eq!(payload.get("failure_class_row_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(
        payload.get("passes_behavior_test").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let status_counts = payload
        .get("row_status_counts")
        .and_then(serde_json::Value::as_object)
        .expect("row status counts");
    assert_eq!(
        status_counts.get("present").and_then(serde_json::Value::as_u64),
        Some(present_row_count)
    );
    assert_eq!(
        status_counts.get("missing_result").and_then(serde_json::Value::as_u64),
        Some(missing_result_row_count)
    );
    assert_eq!(
        status_counts.get("unsupported_pair").and_then(serde_json::Value::as_u64),
        Some(unsupported_pair_row_count)
    );

    let rows = support::json_array(&payload, "rows");
    assert_eq!(rows.len() as u64, row_count);

    let result_ids = rows
        .iter()
        .filter_map(|row| row.get("result_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(result_ids.len() as u64, expected_result_row_count);

    let missing_result_ids = rows
        .iter()
        .filter(|row| {
            row.get("row_status").and_then(serde_json::Value::as_str) == Some("missing_result")
        })
        .filter_map(|row| row.get("result_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(missing_result_ids.len(), 3);
    assert!(missing_result_ids
        .contains("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2"));
    assert!(missing_result_ids.contains("bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools"));
    assert!(
        missing_result_ids.contains("vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools")
    );

    let unsupported_row = rows
        .iter()
        .find(|row| {
            row.get("row_status").and_then(serde_json::Value::as_str) == Some("unsupported_pair")
        })
        .expect("unsupported row");
    assert_eq!(
        unsupported_row.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.filter")
    );
    assert_eq!(
        unsupported_row.get("tool_id").and_then(serde_json::Value::as_str),
        Some("samtools")
    );
    assert_eq!(
        unsupported_row.get("report_section").and_then(serde_json::Value::as_str),
        Some("unsupported_pairs")
    );

    let runtime = support::json_array(&payload, "runtime");
    assert_eq!(runtime.len() as u64, row_count);
    let memory = support::json_array(&payload, "memory");
    assert_eq!(memory.len() as u64, row_count);

    let failures = payload.get("failures").expect("failures section");
    assert_eq!(
        failures.get("simulated_failure_row_count").and_then(serde_json::Value::as_u64),
        Some(expected_result_row_count)
    );
    assert_eq!(
        failures.get("failure_class_row_count").and_then(serde_json::Value::as_u64),
        Some(7)
    );

    let missing_results = payload
        .get("missing_results")
        .and_then(serde_json::Value::as_array)
        .expect("missing results");
    assert_eq!(missing_results.len(), 3);

    let unsupported_pairs = payload
        .get("unsupported_pairs")
        .and_then(serde_json::Value::as_array)
        .expect("unsupported pairs");
    assert_eq!(unsupported_pairs.len(), 1);
}
