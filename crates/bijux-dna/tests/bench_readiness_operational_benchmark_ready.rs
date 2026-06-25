#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn read_committed_json() -> serde_json::Value {
    let repo_root = support::repo_root().expect("repo root");
    let payload = std::fs::read(
        repo_root.join("benchmarks/readiness/FASTQ_BAM_VCF_OPERATIONAL_BENCHMARK_READY.json"),
    )
    .expect("read committed operational benchmark gate");
    serde_json::from_slice(&payload).expect("parse committed operational benchmark gate")
}

#[test]
fn bench_readiness_operational_benchmark_ready_reports_green_surface() {
    let payload = read_committed_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.operational_benchmark_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/FASTQ_BAM_VCF_OPERATIONAL_BENCHMARK_READY.json")
    );
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(141)
    );
    assert_eq!(payload.get("blocker_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("missing_result_row_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(
        payload.get("insufficient_data_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("unsupported_pair_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks array");
    let blockers =
        payload.get("blockers").and_then(serde_json::Value::as_array).expect("blockers array");
    assert!(checks.iter().all(|check| check.get("ok") == Some(&serde_json::Value::Bool(true))));
    assert!(blockers.is_empty(), "green operational gate must not emit blockers");

    let surface_ids = checks
        .iter()
        .filter_map(|check| check.get("surface_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    for required_surface in [
        "fastq_bam_benchmark_binding_coverage",
        "vcf_stage_catalog_ready",
        "vcf_smoke_suite_ready",
        "stage_tool_resources",
        "full_benchmark_result_collector",
        "full_benchmark_report",
        "full_benchmark_dashboard",
    ] {
        assert!(
            surface_ids.contains(required_surface),
            "operational gate must keep `{required_surface}` explicit"
        );
    }
}
