#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn read_committed_json() -> serde_json::Value {
    let repo_root = support::repo_root().expect("repo root");
    let payload = std::fs::read(repo_root.join(
        "benchmarks/readiness/all-domains/FASTQ_BAM_VCF_LOCAL_OPERATIONAL_BENCHMARK_COMPLETE.json",
    ))
    .expect("read committed all-domain operational benchmark gate");
    serde_json::from_slice(&payload).expect("parse committed all-domain operational benchmark gate")
}

#[test]
fn bench_readiness_all_domain_local_operational_benchmark_complete_reports_green_surface() {
    let payload = read_committed_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_local_operational_benchmark_complete.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some(
            "benchmarks/readiness/all-domains/FASTQ_BAM_VCF_LOCAL_OPERATIONAL_BENCHMARK_COMPLETE.json"
        )
    );
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("failed_surface_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(141));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(141)
    );
    assert_eq!(payload.get("blocker_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks array");
    assert!(checks.iter().all(|check| check.get("ok") == Some(&serde_json::Value::Bool(true))));

    let surface_ids = checks
        .iter()
        .filter_map(|check| check.get("surface_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    for required_surface in [
        "benchmark_paths_cleanup_proof",
        "all_domain_active_scope_complete",
        "operational_benchmark_ready",
    ] {
        assert!(
            surface_ids.contains(required_surface),
            "final Goal 400 gate must keep `{required_surface}` explicit"
        );
    }
}
