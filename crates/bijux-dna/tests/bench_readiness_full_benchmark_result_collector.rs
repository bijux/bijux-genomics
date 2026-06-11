#![allow(clippy::expect_used)]

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
fn bench_readiness_full_benchmark_result_collector_merges_all_governed_surfaces() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-full-benchmark-result-collector", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.full_benchmark_result_collector.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/full-result-collector-test.json")
    );
    let row_count = support::json_u64(&payload, "row_count").expect("row_count");
    let benchmark_expected_row_count =
        support::json_u64(&payload, "benchmark_expected_row_count").expect("benchmark_expected_row_count");
    assert_eq!(
        payload.get("pipeline_fake_run_row_count").and_then(serde_json::Value::as_u64),
        Some(93)
    );
    assert_eq!(
        payload.get("fake_run_row_count").and_then(serde_json::Value::as_u64),
        Some(benchmark_expected_row_count)
    );
    assert_eq!(
        payload.get("fake_failure_row_count").and_then(serde_json::Value::as_u64),
        Some(benchmark_expected_row_count)
    );
    assert_eq!(
        payload.get("missing_result_audit_row_count").and_then(serde_json::Value::as_u64),
        Some(benchmark_expected_row_count)
    );
    assert_eq!(payload.get("real_smoke_row_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(
        payload.get("insufficient_data_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("unsupported_pair_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("missing_result_status_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(
        payload.get("insufficient_data_status_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("unsupported_pair_status_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("passes_behavior_test").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let surface_kind_counts = support::json_object(&payload, "surface_kind_counts");
    assert_eq!(
        surface_kind_counts.get("benchmark_expected").and_then(serde_json::Value::as_u64),
        Some(benchmark_expected_row_count)
    );
    assert_eq!(
        surface_kind_counts.get("pipeline_fake_run").and_then(serde_json::Value::as_u64),
        Some(93)
    );
    assert_eq!(
        surface_kind_counts.get("fake_run").and_then(serde_json::Value::as_u64),
        Some(benchmark_expected_row_count)
    );
    assert_eq!(
        surface_kind_counts.get("fake_failure").and_then(serde_json::Value::as_u64),
        Some(benchmark_expected_row_count)
    );
    assert_eq!(
        surface_kind_counts.get("missing_result_audit").and_then(serde_json::Value::as_u64),
        Some(benchmark_expected_row_count)
    );
    assert_eq!(surface_kind_counts.get("real_smoke").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(
        surface_kind_counts.get("failure_classification").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        surface_kind_counts.get("unsupported_pair").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(support::object_u64_sum(surface_kind_counts), row_count);

    let result_status_counts = support::json_object(&payload, "result_status_counts");
    assert_eq!(
        result_status_counts.get("expected").and_then(serde_json::Value::as_u64),
        Some(benchmark_expected_row_count)
    );
    assert_eq!(
        result_status_counts.get("failed").and_then(serde_json::Value::as_u64),
        Some(benchmark_expected_row_count)
    );
    assert_eq!(
        result_status_counts.get("present").and_then(serde_json::Value::as_u64),
        Some(benchmark_expected_row_count - 3)
    );
    assert_eq!(
        result_status_counts.get("missing_result").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(
        result_status_counts.get("insufficient_data").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        result_status_counts.get("unsupported_pair").and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let domain_counts = support::json_object(&payload, "domain_counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(282));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(228));
    assert_eq!(support::object_u64_sum(domain_counts), row_count);

    let rows = support::json_array(&payload, "rows");
    assert_eq!(rows.len() as u64, row_count);

    let record_ids = rows
        .iter()
        .filter_map(|row| row.get("record_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(record_ids.len() as u64, row_count);

    let missing_result_ids = rows
        .iter()
        .filter(|row| {
            row.get("result_status").and_then(serde_json::Value::as_str) == Some("missing_result")
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

    let unsupported_pair = rows
        .iter()
        .find(|row| {
            row.get("result_status").and_then(serde_json::Value::as_str) == Some("unsupported_pair")
        })
        .expect("unsupported pair row");
    assert_eq!(
        unsupported_pair.get("surface_kind").and_then(serde_json::Value::as_str),
        Some("unsupported_pair")
    );
    assert_eq!(
        unsupported_pair.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.filter")
    );
    assert_eq!(
        unsupported_pair.get("tool_id").and_then(serde_json::Value::as_str),
        Some("samtools")
    );
    assert_eq!(unsupported_pair.get("result_id").and_then(serde_json::Value::as_str), None);

    let insufficient_data = rows
        .iter()
        .find(|row| {
            row.get("result_status").and_then(serde_json::Value::as_str)
                == Some("insufficient_data")
        })
        .expect("insufficient-data row");
    assert_eq!(
        insufficient_data.get("surface_kind").and_then(serde_json::Value::as_str),
        Some("failure_classification")
    );
    assert_eq!(
        insufficient_data.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.demography")
    );
    assert_eq!(insufficient_data.get("tool_id").and_then(serde_json::Value::as_str), Some("ibdne"));

    let pipeline_row = rows
        .iter()
        .find(|row| {
            row.get("record_id").and_then(serde_json::Value::as_str)
                == Some("pipeline:core-germline-fastq-bam-vcf:vcf.stats")
        })
        .expect("pipeline row");
    assert_eq!(
        pipeline_row.get("surface_kind").and_then(serde_json::Value::as_str),
        Some("pipeline_fake_run")
    );
    assert_eq!(
        pipeline_row.get("result_status").and_then(serde_json::Value::as_str),
        Some("succeeded")
    );

    let real_smoke_bridge = rows
        .iter()
        .find(|row| {
            row.get("record_id").and_then(serde_json::Value::as_str)
                == Some("real-smoke:bridge:bam-to-vcf.call")
        })
        .expect("real smoke bridge row");
    assert_eq!(
        real_smoke_bridge.get("surface_kind").and_then(serde_json::Value::as_str),
        Some("real_smoke")
    );
    assert_eq!(
        real_smoke_bridge.get("detail").and_then(serde_json::Value::as_str),
        Some("governed real-smoke pipeline bridge execution")
    );
}
